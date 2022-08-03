use super::super::{ffi, Raii, Return, Handle};
use super::{Statement, Result};     
use odbc_safe::AutocommitMode;
use diesel::result::*;

pub use super::types::{SqlDate, SqlTime, SqlSsTime2, SqlTimestamp, EncodedValue};

impl<AC: AutocommitMode> Statement<AC> {
    /// Prepares a statement for execution. Executing a prepared statement is faster than directly
    /// executing an unprepared statement, since it is already compiled into an Access Plan. This
    /// makes preparing statement a good idea if you want to repeatedly execute a query with a
    /// different set of parameters and care about performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use odbc::{*, safe};
    /// # fn doc() -> Result<()>{
    /// let env = create_environment_v3().map_err(|e| e.unwrap())?;
    /// let conn = env.connect("TestDataSource", "", "")?;
    /// let stmt = Statement::with_parent(&conn)?;
    /// let mut stmt = stmt.prepare("SELECT TITLE FROM MOVIES WHERE YEAR = ?")?;
    ///
    /// fn print_one_movie_from<'a> (year: u16, stmt: Statement<'a,'a, Prepared, NoResult, safe::AutocommitOn>) -> Result<Statement<'a, 'a, Prepared, NoResult, safe::AutocommitOn>>{
    ///    let stmt = stmt.bind_parameter(1, &year)?;
    ///    let stmt = if let Data(mut stmt) = stmt.execute()?{
    ///        if let Some(mut cursor) = stmt.fetch()?{
    ///            println!("{}", cursor.get_data::<String>(1)?.unwrap());
    ///        }
    ///        stmt.close_cursor()?
    ///    } else {
    ///       panic!("SELECT statement returned no result set");
    ///    };
    ///    stmt.reset_parameters()
    /// };
    ///
    /// for year in 1990..2010{
    ///     stmt = print_one_movie_from(year, stmt)?
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn prepare(&mut self, sql_text: &str) -> QueryResult<bool> {
        match self.raii.prepare(sql_text).into_result(self){
            Ok(_stmt) => Ok(true),
            Err(_stmt) => Err(Error::NotFound)
        }       
    }
    
}

impl<AC: AutocommitMode> Statement<AC> {

    /// Executes a prepared statement.
    pub fn execute(&mut self) -> Result<bool> {
        if self.raii.execute().into_result(self)? {
            let num_cols = self.num_result_cols()?;
            if num_cols > 0 {
                Ok(true)
            } else {
                Ok(false)
            }
        } else { Ok(false)            
        }
    }

    pub fn execute_statement(&mut self, meta: &super::StatementMetadata, binds: &mut super::Binds) -> Result<bool> {
        for i in 1..= binds.data.len() as u16 {
            let bind = &mut binds.data[i as usize - 1];
            match bind.tpe{
                odbc_sys::SqlDataType::SQL_INTEGER=>{                   
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const i32;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },
                odbc_sys::SqlDataType::SQL_SMALLINT=>{                   
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const i16;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },
                odbc_sys::SqlDataType::SQL_EXT_TINYINT=>{                   
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const i8;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },
                odbc_sys::SqlDataType::SQL_EXT_BIGINT=>{                   
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const i64;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },
                odbc_sys::SqlDataType::SQL_EXT_BIT=>{
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const bool;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },  
                odbc_sys::SqlDataType::SQL_REAL
                =>{
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const f32;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },
                odbc_sys::SqlDataType::SQL_FLOAT | odbc_sys::SqlDataType::SQL_DOUBLE
                =>{
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const f64;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },
                odbc_sys::SqlDataType::SQL_DECIMAL | odbc_sys::SqlDataType::SQL_NUMERIC
                =>{
                    let _ = self.bind_col2(i, &bind.bytes, ffi::SqlCDataType::SQL_C_NUMERIC,&mut bind.length, EncodedValue::new(None));                    
                },
                odbc_sys::SqlDataType::SQL_EXT_WVARCHAR
                | odbc_sys::SqlDataType::SQL_EXT_WCHAR
                | odbc_sys::SqlDataType::SQL_EXT_WLONGVARCHAR
                =>{
                    let _ = self.bind_col2(i, &bind.bytes, ffi::SqlCDataType::SQL_C_WCHAR,&mut bind.length, EncodedValue::new(None));                    
                },
                odbc_sys::SqlDataType::SQL_VARCHAR
                | odbc_sys::SqlDataType::SQL_CHAR
                | odbc_sys::SqlDataType::SQL_EXT_LONGVARCHAR                
                =>{                                                           
                    let _ = self.bind_col2(i, &bind.bytes, ffi::SqlCDataType::SQL_C_CHAR, &mut bind.length, EncodedValue::new(None));                    
                },
                odbc_sys::SqlDataType::SQL_DATETIME                                                
                =>{
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const ffi::SQL_TIMESTAMP_STRUCT;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },
                odbc_sys::SqlDataType::SQL_DATE                                                
                =>{
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const ffi::SQL_DATE_STRUCT;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },
                odbc_sys::SqlDataType::SQL_TIME
                =>{
                    unsafe{
                        let ptr = bind.bytes.as_ptr() as * const ffi::SQL_TIME_STRUCT;
                        let _ = self.bind_col1(i, &(*ptr), &mut bind.length, EncodedValue::new(None));
                    }
                },
                odbc_sys::SqlDataType::SQL_EXT_BINARY | odbc_sys::SqlDataType::SQL_EXT_VARBINARY | odbc_sys::SqlDataType::SQL_EXT_LONGVARBINARY
                =>{
                     let _ = self.bind_col1(i, &bind.bytes, &mut (bind.length as i64), EncodedValue::new(None));
                },
                _=>{
                    // let _ = self.bind_col1(i, &bind.bytes, &mut (bind.length as i64), EncodedValue::new(None));
                }
            };                            
        };

        if self.raii.execute().into_result(self)? {           
            let meta_cols = binds.len() as i16;
            loop{

                if self.num_result_cols()? == meta_cols{

                    let mut cols_match = true;
                    let fields = meta.fields(); 
                    for i in 1..= meta_cols{
                        let col = &fields[i as usize - 1];             
                        let col_current = self.describe_col(i as u16).unwrap();                   
                        if col.name != col_current.name{
                            cols_match = false;
                        }
                    }
                    if cols_match{
                        return Ok(true);
                    }
                }
                else{
                    if let Ok(status) = self.get_more_results(){                                    
                        if status == 0 {                        
                            return Ok(false);
                        }
                    }
                }
            }

        } else {
            Ok(false)
        }
    }
}


impl Raii<ffi::Stmt> {
    fn prepare(&self, sql_text: &str) -> Return<()> {
        let bytes = unsafe { crate::odbc::connection::environment::DB_ENCODING }.encode(sql_text).0;
        match unsafe {
            ffi::SQLPrepare(
                self.handle(),
                bytes.as_ptr(),
                bytes.len() as ffi::SQLINTEGER,
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("SQLPrepare returned unexpected result: {:?}", r),
        }
    }

    fn prepare_byte(&mut self, bytes: &[u8]) -> Return<()> {
        match unsafe {
            ffi::SQLPrepare(
                self.handle(),
                bytes.as_ptr(),
                bytes.len() as ffi::SQLINTEGER,
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("SQLPrepare returned unexpected result: {:?}", r),
        }
    }

    fn execute(&mut self) -> Return<bool> {
        match unsafe { ffi::SQLExecute(self.handle()) } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecute returned unexpected result: {:?}", r),
        }
    }
}
