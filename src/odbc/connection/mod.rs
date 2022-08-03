extern crate log;
mod url;
pub mod ffi;
pub mod environment;
mod result;
mod diagnostics;
pub mod statement;
mod raii;
mod odbc_object;

extern crate odbc_safe;
use odbc_safe as safe;
use result::*;
use diagnostics::*;
use raii::Raii;
use odbc_object::OdbcObject;

extern crate odbc_sys;
use diesel::connection::*;
use diesel::deserialize::FromSqlRow;
use diesel::expression::QueryMetadata;
use diesel::query_builder::*;
use diesel::result::*;
use diesel::r2d2::R2D2Connection;


use result::{into_result, into_result_with};
use odbc_safe::{AutocommitMode, AutocommitOn, AutocommitOff};
use statement::*;
use statement::StatementIterator;
use super::backend::Odbc;
use diesel::query_builder::bind_collector::RawBytesBindCollector;
use diesel::query_dsl::load_dsl::CompatibleType;

/// Represents a connection to an ODBC data source
//#[derive(Debug)]
pub struct RawConnection<'env, AC: AutocommitMode> {
    // environment : Environment<Version3>,    
    statement_cache: StatementCache<crate::odbc::Odbc, Statement<AC>>,
    safe : safe::Connection<'env, AC>,    
    transaction_manager: AnsiTransactionManager,
}


impl<'env, AC: AutocommitMode> Handle for RawConnection<'env, AC> {
    type To = ffi::Dbc;
    unsafe fn handle(&self) -> ffi::SQLHDBC {
        self.safe.as_raw()
    }
}

impl<'env> R2D2Connection for RawConnection<'env, safe::AutocommitOn> {
    fn ping(&mut self) -> QueryResult<()> {       
        self.execute("SELECT 1").map(|_| ())        
    }
}

unsafe impl<'env, AC: AutocommitMode> Send for RawConnection<'env, AC> {}


impl<'env, AC: AutocommitMode> SimpleConnection for RawConnection<'env, AC> {
    fn batch_execute(&mut self, _query: &str) -> QueryResult<()> {
        // self.raw_connection
        //     .enable_multi_statements(|| self.raw_connection.execute(query))
        Ok(())
    }
}


impl<'env> Connection for RawConnection<'env, safe::AutocommitOn> {
    type Backend = super::Odbc;
    type TransactionManager = AnsiTransactionManager;

    fn establish(database_url: &str) -> ConnectionResult<Self> {
        // use diesel::result::ConnectionError::CouldntSetupConfiguration;    
        //let env_box: Box<Environment<Version3>> = Box::new(create_environment_v3().expect("Can't create ODBC environment"));        
        
        //let env = create_environment_v3().expect("Can't create ODBC environment");
        //let conn = env.connect("PostgreSQL", "postgres", "postgres").unwrap();
        // let conn = RawConnection::<AutocommitOn>::new("localhost", "main", "unitsoft_main");
        environment::set_environment_os_db_encoding("GB18030","GB18030");
        let conn = RawConnection::<AutocommitOn>::new(database_url);        
        Ok(conn)    
    }

    #[doc(hidden)]
    fn execute(&mut self, query: &str) -> QueryResult<usize> {        

        let stmt = Statement::with_parent(self).unwrap();
        let stmt = stmt.exec_direct(query).unwrap();
        match stmt{
            Data(stmt)=>Ok(stmt.affected_row_count().into_result(self).unwrap() as usize),
            NoData(stmt)=>Ok(stmt.affected_row_count().into_result(self).unwrap() as usize)
        }
    }

    #[doc(hidden)]
    fn load<T, U, ST>(&mut self, source: T) -> QueryResult<Vec<U>>
    where
        T: AsQuery,
        T::Query: QueryFragment<Self::Backend> + QueryId,  
        T::SqlType: CompatibleType<U, Self::Backend, SqlType = ST>,      
        U: FromSqlRow<ST, Self::Backend>,
        Self::Backend: QueryMetadata<T::SqlType>,       
    {       
        let query = source.as_query();                
        let mut stmt = self.prepare_query(&query)?;   

        let mut types = Vec::new();
        Odbc::row_metadata(&mut (), &mut types);
        let metadata = stmt.metadata().unwrap();
        let mut output_binds = statement::Binds::from_output_types(types, &metadata);
        stmt.execute_statement(&metadata, &mut output_binds).unwrap();        
        let stmt : &mut Statement<_> = &mut stmt;

        let metadata = stmt.metadata().unwrap();
        let statement_use = StatementUse::new(&mut *stmt, &mut output_binds, &metadata);
        let iter = StatementIterator::new(statement_use);                
        let ret = iter.collect::<QueryResult<Vec<U>>>();
       
        while let Ok(status) = stmt.get_more_results(){
            if status == 0{
                break;
            }
        }
        return ret;

        // panic!("xxx");
        // match stmt{
        //     ResultSetState::Data(mut stmt)=>{
        //         let metadata = stmt.metadata().unwrap();
        //         let statement_use = StatementUse::new(&mut stmt, &mut output_binds, &metadata);
        //         let iter = StatementIterator::new(statement_use);                
        //         let ret = iter.collect::<QueryResult<Vec<U>>>();

        //         while let Ok(status) = stmt.get_more_results(){
        //             if status == 0{
        //                 break;
        //             }
        //         }

        //         return ret;
        //     },
        //     ResultSetState::NoData(_stmt)=>{
        //         // let statement_use = StatementUse::new(&mut stmt);
        //         // let iter = StatementIterator::new(statement_use);
        //         // iter.collect::<QueryResult<Vec<U>>>()
        //         panic!("no data.")
        //     }
        // }                    
    }

    #[doc(hidden)]
    fn execute_returning_count<T>(&mut self, source: &T) -> QueryResult<usize>
    where       
        T: QueryFragment<Self::Backend> + QueryId,
    {        
        let mut stmt = self.prepare_query(source)?;       
        let result = stmt.execute().unwrap();
        match result{
            true=>{
                Ok(stmt.affected_row_count().into_result(self).unwrap() as usize)
            }
            _ =>{Ok(0)}
        }
    }

    #[doc(hidden)]
    fn transaction_state(
        &mut self,
    ) -> &mut <Self::TransactionManager as TransactionManager<Self>>::TransactionStateData
    {
        &mut self.transaction_manager
    }
    
}

impl <'env> RawConnection<'env, AutocommitOn> {
   
    pub fn disable_autocommit(self) -> std::result::Result<RawConnection<'env, AutocommitOff>, Self> {
        
        let ret = self.safe.disable_autocommit();
        match ret {
            safe::Return::Success(value) => {
                let conn = RawConnection {                    
                    safe: value,
                    transaction_manager: AnsiTransactionManager::default(),
                    statement_cache: StatementCache::new(),
                };
                Ok(conn)
            },
            safe::Return::Info(value) => {
                let conn = RawConnection {                    
                    safe: value,
                    transaction_manager: AnsiTransactionManager::default(),
                    statement_cache: StatementCache::new(), 
                };
                Ok(conn)
            },
            safe::Return::Error(value) => {
                let conn = RawConnection {                   
                    safe: value,
                    transaction_manager: AnsiTransactionManager::default(),
                    statement_cache: StatementCache::new(),
                };
                Err(conn)
            }
        }       
    }

}

impl <'env> RawConnection<'env, AutocommitOff> {
    pub fn enable_autocommit(self) -> std::result::Result<RawConnection<'env, AutocommitOn>, Self> {

        let ret = self.safe.enable_autocommit();
        match ret {
            safe::Return::Success(value) => Ok(RawConnection {                
                safe: value,
                transaction_manager: AnsiTransactionManager::default(),
                statement_cache: StatementCache::new(), 
            }),
            safe::Return::Info(value) => Ok(RawConnection {                          
                safe: value,
                transaction_manager: AnsiTransactionManager::default(),
                statement_cache: StatementCache::new(),
            }),
            safe::Return::Error(value) => Err(RawConnection {                              
                safe: value,
                transaction_manager: AnsiTransactionManager::default(),
                statement_cache: StatementCache::new(), })
        }
        
    }

    pub fn commit(&mut self) -> Result<()> {
        let ret = self.safe.commit();
        into_result_with(&self.safe, ret)

    }

    pub fn rollback(&mut self) -> Result<()> {
        let ret = self.safe.rollback();
        into_result_with(&self.safe, ret)       
    }
}


impl<'env, AC: AutocommitMode> RawConnection<'env, AC> {

    /// `true` if the data source is set to READ ONLY mode, `false` otherwise.
    ///
    /// This characteristic pertains only to the data source itself; it is not characteristic of
    /// the driver that enables access to the data source. A driver that is read/write can be used
    /// with a data source that is read-only. If a driver is read-only, all of its data sources
    /// must be read-only.
    pub fn is_read_only(&mut self) -> Result<bool> {
        // The mutability on is_read_only is really an eyesore. Not only to clippy. But we would
        // have to introduce a cell around `self.safe`, and be careful not to change essential
        // state in the error path. For now the trouble does not seem worth it.
        let ret = self.safe.is_read_only();
        into_result_with(&self.safe, ret)
       
    }

    /// Closes the connection to the data source. If not called explicitly the disconnect will be
    /// invoked implicitly by `drop()`
    pub fn disconnect(self) -> Result<()> {       
        into_result(self.safe.disconnect())?;
        Ok(())        
    }
}

unsafe impl<'env, AC: AutocommitMode> safe::Handle for RawConnection<'env, AC> {
    const HANDLE_TYPE: ffi::HandleType = ffi::SQL_HANDLE_DBC;

    fn handle(&self) -> ffi::SQLHANDLE {
        self.safe.as_raw() as ffi::SQLHANDLE
    }
}

impl<'env, AC: AutocommitMode> RawConnection<'env, AC> {
    fn prepare_query<T>(&mut self, source: &T) -> QueryResult<MaybeCached<Statement<AC>>>
    where
        T: QueryFragment<crate::odbc::Odbc> + QueryId,            
    {        
        let conn = unsafe{&mut *(self as *const Self as *mut Self)};
        let stmt = conn
            .statement_cache
            .statement_is_cached(source, &[]); 
        if let Some(stmt) = stmt{
            return Ok(stmt);
        }
        else
        {                
            let mut query_builder = crate::odbc::OdbcQueryBuilder::new();
            source.to_sql(&mut query_builder)?;
            let sql = query_builder.finish();

            let mut stmt = Statement::with_parent(self).unwrap();     
            let _result = stmt.prepare(sql).unwrap(); 
            let mut bind_collector = RawBytesBindCollector::new();
            source.collect_binds(&mut bind_collector, &mut ())?;
            let binds = bind_collector
                .metadata
                .into_iter()
                .zip(bind_collector.binds);        

            let mut i = 1;
            let mut input_binds = statement::Binds::from_input_data(binds)?;  
            let _ = input_binds.data
                .iter_mut()
                .map(|bind| {                                    
                        match bind.tpe{                        
                            odbc_sys::SqlDataType::SQL_INTEGER=>{
                                unsafe {                                
                                    let para = bind.bytes.as_ptr() as *const i32;                                
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            },
                            odbc_sys::SqlDataType::SQL_SMALLINT=>{
                                unsafe {                                
                                    let para = bind.bytes.as_ptr() as *const i16;                                
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            },
                            odbc_sys::SqlDataType::SQL_EXT_TINYINT=>{
                                unsafe {                                
                                    let para = bind.bytes.as_ptr() as *const i8;                                
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            },
                            odbc_sys::SqlDataType::SQL_EXT_BIGINT=>{
                                unsafe {                                
                                    let para = bind.bytes.as_ptr() as *const i64;                                
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            },
                            odbc_sys::SqlDataType::SQL_EXT_BIT=>{
                                unsafe {                                
                                    let para = bind.bytes.as_ptr() as *const bool;                                
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            },
                            odbc_sys::SqlDataType::SQL_DECIMAL | odbc_sys::SqlDataType::SQL_NUMERIC  
                            =>{                            
                                let str = String::from_utf8(bind.bytes.to_vec()).unwrap();
                                stmt.bind_parameter(i,  &str);
                            },
                            odbc_sys::SqlDataType::SQL_FLOAT | odbc_sys::SqlDataType::SQL_DOUBLE
                            =>{
                                unsafe {
                                    let para = bind.bytes.as_ptr() as *const f64;                                
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            },
                            odbc_sys::SqlDataType::SQL_REAL
                            =>{
                                unsafe {
                                    let para = bind.bytes.as_ptr() as *const f32;                                
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            },
                            odbc_sys::SqlDataType::SQL_VARCHAR
                            | odbc_sys::SqlDataType::SQL_CHAR
                            | odbc_sys::SqlDataType::SQL_EXT_LONGVARCHAR
                            =>{                            
                                let str = String::from_utf8(bind.bytes.to_vec()).unwrap();
                                stmt.bind_parameter(i,  &str);
                            },
                            odbc_sys::SqlDataType::SQL_EXT_WVARCHAR
                            | odbc_sys::SqlDataType::SQL_EXT_WCHAR
                            | odbc_sys::SqlDataType::SQL_EXT_WLONGVARCHAR
                            =>{
                                let str = String::from_utf8(bind.bytes.to_vec()).unwrap();
                                stmt.bind_parameter(i,  &str);
                            },
                            odbc_sys::SqlDataType::SQL_DATETIME                                                
                            =>{
                                unsafe {
                                    let mut para = bind.bytes.as_ptr() as *mut ffi::SQL_TIMESTAMP_STRUCT;
                                    (*para).fraction = 0;    
                                    if (*para).year == 0 && (*para).month==1 && (*para).day==1 
                                        && (*para).hour == 0 && (*para).minute==0 && (*para).second==0 && (*para).fraction==0
                                    {                                                
                                        para = 0 as *mut ffi::SQL_TIMESTAMP_STRUCT;
                                    }
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            },
                            odbc_sys::SqlDataType::SQL_DATE                                                
                            =>{                            
                                unsafe {
                                    let para = bind.bytes.as_ptr() as *const ffi::SQL_DATE_STRUCT;                                
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            }, 
                            odbc_sys::SqlDataType::SQL_TIME                                                
                            =>{                            
                                unsafe {
                                    let para = bind.bytes.as_ptr() as *const ffi::SQL_TIME_STRUCT;                                
                                    stmt.bind_parameter(i,  &(*para));
                                }
                            },              
                            _=>{
                                // let str = String::from_utf8(bind.bytes.to_vec()).unwrap();
                                // stmt.bind_parameter1(i,  &str);
                            }
                        };             
                        i += 1;
                }) .collect::<Vec<_>>();
            stmt.input_binds = Some(input_binds);


            let this = unsafe{&mut *(self as *const Self as *mut Self)};
            let stmt1 = this.statement_cache.cached_statement1(source, &[], stmt)?;
            return Ok(stmt1);

        }
    }

    pub fn prepare_query1(&self, source: &String) -> Statement<AC>
    {        
        let mut stmt = Statement::with_parent(self).unwrap();
        stmt.prepare(source).unwrap();       
        stmt 
    }

    #[doc(hidden)]
    pub fn execute(&self, query: &str) -> QueryResult<usize> {
        let stmt = Statement::with_parent(self).unwrap();
        let stmt = stmt.exec_direct(query).unwrap();
        match stmt{
            Data(stmt)=>Ok(stmt.affected_row_count().into_result(self).unwrap() as usize),
            NoData(stmt)=>Ok(stmt.affected_row_count().into_result(self).unwrap() as usize)
        }
    }

    fn set_config_options(&self) -> QueryResult<()> {
        // self.execute("SET sql_mode=(SELECT CONCAT(@@sql_mode, ',PIPES_AS_CONCAT'))")?;
        // self.execute("SET time_zone = '+08:00';")?;
        // self.execute("SET character_set_client = 'utf8mb4'")?;
        // self.execute("SET character_set_connection = 'utf8mb4'")?;
        // self.execute("SET character_set_results = 'utf8mb4'")?;
        Ok(())
    }
    
    fn new1(dsn: &str, usr: &str, pwd: &str)-> RawConnection<'env, AutocommitOn> {
        
        // let env = safe::HEnv::allocate().unwrap();
        // let safe = safe::HDbc::allocate(&env).map(|handle| {
        //     safe::DataSource { handle:  safe::Unconnected::from_hdbc(handle) }
        // }).unwrap();

        let safe = safe::DataSource::new().unwrap();
        let safe = into_result(safe.connect(dsn, usr, pwd)).unwrap();
        RawConnection{
            safe, 
            transaction_manager: AnsiTransactionManager::default(),
            statement_cache: StatementCache::new(),
        }      
    }
    
    fn new(conn_str: &str)-> RawConnection<'env, AutocommitOn> {
        let safe = safe::DataSource::new().unwrap();
        let safe = into_result(safe.connect_with_connection_string(conn_str)).unwrap();
        RawConnection{
            safe: safe, 
            transaction_manager: AnsiTransactionManager::default(),
            statement_cache: StatementCache::new(),
        }
    }
}


/// Reflects the ability of a type to expose a valid handle
pub trait Handle {
    type To;
    /// Returns a valid handle to the odbc type.
    unsafe fn handle(&self) -> *mut Self::To;
}
