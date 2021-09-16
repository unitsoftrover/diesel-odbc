mod types;
mod input;
mod output;
mod prepare;
mod statement_iterator;
mod metadata;
mod bind;

pub use self::output::Output;
use super::{ffi, safe, Return, Result, Raii, Handle};
use super::RawConnection;
use ffi::SQLRETURN::*;
use ffi::Nullable;
use std::marker::PhantomData;
use crate::odbc::backend::OdbcSqlType;
pub use self::types::OdbcType;
pub use self::types::{SqlDate, SqlTime, SqlSsTime2, SqlTimestamp, EncodedValue};
use super::environment;
pub use bind::Binds;
pub use statement_iterator::StatementIterator;
use odbc_safe::{AutocommitMode, AutocommitOn};
use diesel::result::QueryResult;
use statement_iterator::OdbcRow;
pub use metadata::*;
use std::rc::Rc;

// Allocate CHUNK_LEN elements at a time
const CHUNK_LEN: usize = 64;
struct Chunks<T>(Vec<Box<[T; CHUNK_LEN]>>);

/// Heap allocator that will keep allocated element pointers valid until the allocator is dropped or cleared
impl<T: Copy + Default> Chunks<T> {
    fn new() -> Chunks<T> {
        Chunks(Vec::new())
    }

    fn alloc(&mut self, i: usize, value: T) -> *mut T {
        let chunk_no = i / CHUNK_LEN;
        if self.0.len() <= chunk_no {
            // Resizing Vec that holds pointers to heap allocated arrays so we don't invalidate the references
            self.0.resize(chunk_no + 1, Box::new([T::default(); CHUNK_LEN]))
        }
        let v = self.0[chunk_no].get_mut(i % CHUNK_LEN).unwrap();
        *v = value;
        v as *mut T
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

/// `Statement` state used to represent a freshly allocated connection
pub enum Allocated {}
/// `Statement` state used to represent a statement with a result set cursor. A statement is most
/// likely to enter this state after a `SELECT` query.
pub type Executed = Allocated;
/// `Statement` state used to represent a statement compiled into an access plan. A statement will
/// enter this state after a call to `Statement::prepared`
pub enum Prepared {}
/// `Statement` state used to represent a statement with a result set cursor. A statement is most
/// likely to enter this state after a `SELECT` query.
pub enum HasResult {}
/// `Statement` state used to represent a statement with no result set. A statement is likely to
/// enter this state after executing e.g. a `CREATE TABLE` statement
pub enum NoResult {}

/// Holds a `Statement` after execution of a query.Allocated
///
/// A executed statement may be in one of two states. Either the statement has yielded a result set
/// or not. Keep in mind that some ODBC drivers just yield empty result sets on e.g. `INSERT`
/// Statements
pub enum ResultSetState<S, AC: AutocommitMode> {
    Data(Statement<S, HasResult, AC>),
    NoData(Statement<S, NoResult, AC>),
}
pub use ResultSetState::*;
use std::ptr::null_mut;

impl Raii<ffi::Stmt> {
    fn affected_row_count(&self) -> Return<ffi::SQLLEN> {
        let mut count: ffi::SQLLEN = 0;
        unsafe {
            match ffi::SQLRowCount(self.handle(), &mut count as *mut ffi::SQLLEN) {
                SQL_SUCCESS => Return::Success(count),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(count),
                SQL_ERROR => Return::Error,
                r => panic!("SQLRowCount returned unexpected result: {:?}", r),
            }
        }
    }

    fn num_result_cols(&self) -> Return<i16> {
        let mut num_cols: ffi::SQLSMALLINT = 0;
        unsafe {
            match ffi::SQLNumResultCols(self.handle(), &mut num_cols as *mut ffi::SQLSMALLINT) {
                SQL_SUCCESS => Return::Success(num_cols),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(num_cols),
                SQL_ERROR => Return::Error,
                r => panic!("SQLNumResultCols returned unexpected result: {:?}", r),
            }
        }
    }

    fn describe_col(&self, idx: u16) -> Return<ColumnDescriptor> {
        let mut name_buffer: [u8; 512] = [0; 512];
        let mut name_length: ffi::SQLSMALLINT = 0;
        let mut data_type: ffi::SqlDataType = ffi::SqlDataType::SQL_UNKNOWN_TYPE;
        let mut column_size: ffi::SQLULEN = 0;
        let mut decimal_digits: ffi::SQLSMALLINT = 0;
        let mut nullable: Nullable = Nullable::SQL_NULLABLE_UNKNOWN;
        unsafe {
            match ffi::SQLDescribeCol(
                self.handle(),
                idx,
                name_buffer.as_mut_ptr(),
                name_buffer.len() as ffi::SQLSMALLINT,
                &mut name_length as *mut ffi::SQLSMALLINT,
                &mut data_type as *mut ffi::SqlDataType,
                &mut column_size as *mut ffi::SQLULEN,
                &mut decimal_digits as *mut ffi::SQLSMALLINT,
                &mut nullable as *mut ffi::Nullable,
            ) {
                SQL_SUCCESS => Return::Success(ColumnDescriptor {
                    name: environment::DB_ENCODING.decode(&name_buffer[..(name_length as usize)]).0
                        .to_string(),
                    data_type: data_type,
                    column_size: if column_size == 0 {
                        None
                    } else {
                        Some(column_size)
                    },
                    decimal_digits: if decimal_digits == 0 {
                        None
                    } else {
                        Some(decimal_digits as u16)
                    },
                    nullable: match nullable {
                        Nullable::SQL_NULLABLE_UNKNOWN => None,
                        Nullable::SQL_NULLABLE => Some(true),
                        Nullable::SQL_NO_NULLS => Some(false),
                    },
                }),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(ColumnDescriptor {
                    name: environment::DB_ENCODING.decode(&name_buffer[..(name_length as usize)]).0
                        .to_string(),
                    data_type: data_type,
                    column_size: if column_size == 0 {
                        None
                    } else {
                        Some(column_size)
                    },
                    decimal_digits: if decimal_digits == 0 {
                        None
                    } else {
                        Some(decimal_digits as u16)
                    },
                    nullable: match nullable {
                        Nullable::SQL_NULLABLE_UNKNOWN => None,
                        Nullable::SQL_NULLABLE => Some(true),
                        Nullable::SQL_NO_NULLS => Some(false),
                    },
                }),
                SQL_ERROR => Return::Error,
                r => panic!("SQLDescribeCol returned unexpected result: {:?}", r),
            }
        }

    }

    fn exec_direct(&mut self, statement_text: &str) -> Return<bool> {
        let bytes = unsafe { crate::odbc::connection::environment::DB_ENCODING }.encode(statement_text).0;

        let length = bytes.len();
        if length > ffi::SQLINTEGER::max_value() as usize {
            panic!("Statement text too long");
        }
        match unsafe {
            ffi::SQLExecDirect(
                self.handle(),
                bytes.as_ptr(),
                length as ffi::SQLINTEGER,
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NEED_DATA => panic!("SQLExecDirec returned SQL_NEED_DATA"),
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecDirect returned unexpected result: {:?}", r),
        }
    }

    fn exec_direct_bytes(&mut self, bytes: &[u8]) -> Return<bool> {
        let length = bytes.len();
        if length > ffi::SQLINTEGER::max_value() as usize {
            panic!("Statement text too long");
        }
        match unsafe {
            ffi::SQLExecDirect(
                self.handle(),
                bytes.as_ptr(),
                length as ffi::SQLINTEGER,
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NEED_DATA => panic!("SQLExecDirec returned SQL_NEED_DATA"),
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecDirect returned unexpected result: {:?}", r),
        }
    }
    

    /// Fetches the next rowset of data from the result set and returns data for all bound columns.
    fn fetch(&mut self) -> Return<bool> {
        match unsafe { ffi::SQLFetch(self.handle()) } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLFetch returned unexpected result: {:?}", r),
        }
    }

    fn tables(&mut self, catalog_name: Option<&str>, schema_name: Option<&str>, table_name: Option<&str>, table_type: &str) -> Return<()> {
        unsafe {
            let mut catalog: *const odbc_sys::SQLCHAR = null_mut();
            let mut schema: *const odbc_sys::SQLCHAR = null_mut();
            let mut table: *const odbc_sys::SQLCHAR = null_mut();

            let mut catalog_size: odbc_sys::SQLSMALLINT = 0;
            let mut schema_size: odbc_sys::SQLSMALLINT = 0;
            let mut table_size: odbc_sys::SQLSMALLINT = 0;

            if catalog_name.is_some() {
                catalog = catalog_name.unwrap().as_ptr();
                catalog_size = catalog_name.unwrap().len() as odbc_sys::SQLSMALLINT;
            }

            if schema_name.is_some() {
                schema = schema_name.unwrap().as_ptr();
                schema_size = schema_name.unwrap().len() as odbc_sys::SQLSMALLINT;
            }

            if table_name.is_some() {
                table = table_name.unwrap().as_ptr();
                table_size = table_name.unwrap().len() as odbc_sys::SQLSMALLINT;
            }

            match odbc_sys::SQLTables(
                self.handle(),
                catalog,
                catalog_size,
                schema,
                schema_size,
                table,
                table_size,
                table_type.as_ptr(),
                table_type.as_bytes().len() as odbc_sys::SQLSMALLINT,
            ) {
                SQL_SUCCESS => Return::Success(()),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
                SQL_ERROR => Return::Error,
                r => panic!("SQLTables returned: {:?}", r),
            }
        }
    }

    fn close_cursor(&mut self) -> Return<()> {
        unsafe {
            match ffi::SQLCloseCursor(self.handle()) {
                ffi::SQL_SUCCESS => Return::Success(()),
                ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
                ffi::SQL_ERROR => Return::Error,
                r => panic!("unexpected return value from SQLCloseCursor: {:?}", r),
            }
        }
    }
}

/// A `Statement` can be used to execute queries and retrieves results.
pub struct Statement<S, R, AC: AutocommitMode> {
    //raii: ffi::SQLHSTMT,
    pub raii: Rc<Raii<ffi::Stmt>>,
    state: PhantomData<S>,
    autocommit_mode: PhantomData<AC>,
    // Indicates wether there is an open result set or not associated with this statement.
    result: PhantomData<R>,
    //parameters: PhantomData<&'b [u8]>,
    param_ind_buffers: Chunks<ffi::SQLLEN>,
    // encoded values are saved to use its pointer.
    encoded_values: Vec<EncodedValue>,
    pub input_binds: Option<Binds>,
}

/// Used to retrieve data from the fields of a query result
pub struct Cursor<'s, S: 's, AC: AutocommitMode> {
    stmt: &'s mut Statement<S, HasResult, AC>,
    buffer: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnDescriptor {
    pub name: String,
    pub data_type: ffi::SqlDataType,
    pub column_size: Option<ffi::SQLULEN>,
    pub decimal_digits: Option<u16>,
    pub nullable: Option<bool>,
}

impl<S, R, AC: AutocommitMode> Handle for Statement<S, R, AC> {
    type To = ffi::Stmt;
    unsafe fn handle(&self) -> ffi::SQLHSTMT {
        self.raii.handle()
    }
}

impl<S, R, AC: AutocommitMode> Statement<S, R, AC> {
    pub fn with_raii(raii: Rc<Raii<ffi::Stmt>>) -> Self {
        Statement {            
            raii: raii,
            autocommit_mode: PhantomData,
            state: PhantomData,
            result: PhantomData,
            //parameters: PhantomData,
            param_ind_buffers: Chunks::new(),
            encoded_values: Vec::new(),
            input_binds: None,
        }
    }
    
    fn affected_row_count1(&self) -> Return<ffi::SQLLEN> {
        let mut count: ffi::SQLLEN = 0;
        unsafe {
            match ffi::SQLRowCount(self.handle(), &mut count as *mut ffi::SQLLEN) {
                SQL_SUCCESS => Return::Success(count),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(count),
                SQL_ERROR => Return::Error,
                r => panic!("SQLRowCount returned unexpected result: {:?}", r),
            }
        }
    }

    fn num_result_cols1(&self) -> Return<i16> {
        let mut num_cols: ffi::SQLSMALLINT = 0;
        unsafe {
            match ffi::SQLNumResultCols(self.handle(), &mut num_cols as *mut ffi::SQLSMALLINT) {
                SQL_SUCCESS => Return::Success(num_cols),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(num_cols),
                SQL_ERROR => Return::Error,
                r => panic!("SQLNumResultCols returned unexpected result: {:?}", r),
            }
        }
    }

    fn describe_col1(&self, idx: u16) -> Return<ColumnDescriptor> {
        let mut name_buffer: [u8; 512] = [0; 512];
        let mut name_length: ffi::SQLSMALLINT = 0;
        let mut data_type: ffi::SqlDataType = ffi::SqlDataType::SQL_UNKNOWN_TYPE;
        let mut column_size: ffi::SQLULEN = 0;
        let mut decimal_digits: ffi::SQLSMALLINT = 0;
        let mut nullable: Nullable = Nullable::SQL_NULLABLE_UNKNOWN;
        unsafe {
            match ffi::SQLDescribeCol(
                self.handle(),
                idx,
                name_buffer.as_mut_ptr(),
                name_buffer.len() as ffi::SQLSMALLINT,
                &mut name_length as *mut ffi::SQLSMALLINT,
                &mut data_type as *mut ffi::SqlDataType,
                &mut column_size as *mut ffi::SQLULEN,
                &mut decimal_digits as *mut ffi::SQLSMALLINT,
                &mut nullable as *mut ffi::Nullable,
            ) {
                SQL_SUCCESS => Return::Success(ColumnDescriptor {
                    name: environment::DB_ENCODING.decode(&name_buffer[..(name_length as usize)]).0
                        .to_string(),
                    data_type: data_type,
                    column_size: if column_size == 0 {
                        None
                    } else {
                        let column_size = match data_type{
                            ffi::SqlDataType::SQL_EXT_WCHAR | ffi::SqlDataType::SQL_EXT_WVARCHAR |ffi::SqlDataType::SQL_EXT_WLONGVARCHAR                            
                            =>(column_size+1)*2,                            
                            ffi::SqlDataType::SQL_CHAR | ffi::SqlDataType::SQL_VARCHAR |ffi::SqlDataType::SQL_EXT_LONGVARCHAR 
                            =>column_size+1,
                            ffi::SqlDataType::SQL_DECIMAL | ffi::SqlDataType::SQL_NUMERIC
                            =>column_size+1,
                            ffi::SqlDataType::SQL_DATE=>{
                                // println!("column_size:{:?}", column_size);
                                column_size
                            },
                            ffi::SqlDataType::SQL_TIME=>{
                                println!("column_size:{:?}", column_size);
                                column_size
                            },
                            _=>column_size
                        };
                        Some(column_size)
                    },
                    decimal_digits: if decimal_digits == 0 {
                        None
                    } else {
                        Some(decimal_digits as u16)
                    },
                    nullable: match nullable {
                        Nullable::SQL_NULLABLE_UNKNOWN => None,
                        Nullable::SQL_NULLABLE => Some(true),
                        Nullable::SQL_NO_NULLS => Some(false),
                    },
                }),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(ColumnDescriptor {
                    name: environment::DB_ENCODING.decode(&name_buffer[..(name_length as usize)]).0
                        .to_string(),
                    data_type: data_type,
                    column_size: if column_size == 0 {
                        None
                    } else {
                        Some(column_size)
                    },
                    decimal_digits: if decimal_digits == 0 {
                        None
                    } else {
                        Some(decimal_digits as u16)
                    },
                    nullable: match nullable {
                        Nullable::SQL_NULLABLE_UNKNOWN => None,
                        Nullable::SQL_NULLABLE => Some(true),
                        Nullable::SQL_NO_NULLS => Some(false),
                    },
                }),
                SQL_ERROR => Return::Error,
                r => panic!("SQLDescribeCol returned unexpected result: {:?}", r),
            }
        }

    }

    fn exec_direct1(&mut self, statement_text: &str) -> Return<bool> {
        let bytes = unsafe { crate::odbc::connection::environment::DB_ENCODING }.encode(statement_text).0;

        let length = bytes.len();
        if length > ffi::SQLINTEGER::max_value() as usize {
            panic!("Statement text too long");
        }
        
        match unsafe {

            ffi::SQLExecDirect(
                self.handle() as ffi::SQLHSTMT,
                bytes.as_ptr(),
                length as ffi::SQLINTEGER,
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => {
                let ret:Return<bool> = Return::Error;                 
                ret.into_result(self).unwrap();
                Return::Error
            },
            ffi::SQL_NEED_DATA => panic!("SQLExecDirec returned SQL_NEED_DATA"),
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecDirect returned unexpected result: {:?}", r),
        }
    }

    fn exec_direct_bytes1(&mut self, bytes: &[u8]) -> Return<bool> {
        let length = bytes.len();
        if length > ffi::SQLINTEGER::max_value() as usize {
            panic!("Statement text too long");
        }
        match unsafe {
            ffi::SQLExecDirect(
                self.handle(),
                bytes.as_ptr(),
                length as ffi::SQLINTEGER,
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NEED_DATA => panic!("SQLExecDirec returned SQL_NEED_DATA"),
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecDirect returned unexpected result: {:?}", r),
        }
    }

    /// Fetches the next rowset of data from the result set and returns data for all bound columns.
    fn fetch1(&mut self) -> Return<bool> {
        match unsafe { ffi::SQLFetch(self.handle()) } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLFetch returned unexpected result: {:?}", r),
        }
    }

    pub fn get_more_results(&self) -> Result<i16> {
        
        let result = 
        unsafe {
            match ffi::SQLMoreResults(self.handle()) {
                SQL_SUCCESS => Return::Success(1),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(1),
                SQL_NO_DATA=>Return::Success(0),
                SQL_ERROR => Return::Error,
                r => panic!("SQLMoreResults returned unexpected result: {:?}", r),
            }
        };
        result.into_result(self)      
    }
    
    fn tables1(&mut self, catalog_name: Option<&str>, schema_name: Option<&str>, table_name: Option<&str>, table_type: &str) -> Return<()> {
        unsafe {
            let mut catalog: *const odbc_sys::SQLCHAR = null_mut();
            let mut schema: *const odbc_sys::SQLCHAR = null_mut();
            let mut table: *const odbc_sys::SQLCHAR = null_mut();

            let mut catalog_size: odbc_sys::SQLSMALLINT = 0;
            let mut schema_size: odbc_sys::SQLSMALLINT = 0;
            let mut table_size: odbc_sys::SQLSMALLINT = 0;

            if catalog_name.is_some() {
                catalog = catalog_name.unwrap().as_ptr();
                catalog_size = catalog_name.unwrap().len() as odbc_sys::SQLSMALLINT;
            }

            if schema_name.is_some() {
                schema = schema_name.unwrap().as_ptr();
                schema_size = schema_name.unwrap().len() as odbc_sys::SQLSMALLINT;
            }

            if table_name.is_some() {
                table = table_name.unwrap().as_ptr();
                table_size = table_name.unwrap().len() as odbc_sys::SQLSMALLINT;
            }

            match odbc_sys::SQLTables(
                self.handle(),
                catalog,
                catalog_size,
                schema,
                schema_size,
                table,
                table_size,
                table_type.as_ptr(),
                table_type.as_bytes().len() as odbc_sys::SQLSMALLINT,
            ) {
                SQL_SUCCESS => Return::Success(()),
                SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
                SQL_ERROR => Return::Error,
                r => panic!("SQLTables returned: {:?}", r),
            }
        }
    }

    fn close_cursor1(&mut self) -> Return<()> {
        unsafe {
            match ffi::SQLCloseCursor(self.handle()) {
                ffi::SQL_SUCCESS => Return::Success(()),
                ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
                ffi::SQL_ERROR => Return::Error,
                r => panic!("unexpected return value from SQLCloseCursor: {:?}", r),
            }
        }
    }

    fn prepare1(&mut self, sql_text: &str) -> Return<()> {
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

    fn prepare_byte1(&mut self, bytes: &[u8]) -> Return<()> {
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

    fn execute1(&self) -> Return<bool> {
        match unsafe { ffi::SQLExecute(self.handle()) } {
            ffi::SQL_SUCCESS => Return::Success(true),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(true),
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NO_DATA => Return::Success(false),
            r => panic!("SQLExecute returned unexpected result: {:?}", r),
        }
    }
    
    pub fn bind<Iter>(&mut self, binds: Iter) -> QueryResult<()>
    where
        Iter: IntoIterator<Item = (OdbcSqlType, Option<Vec<u8>>)>,
    {
        let input_binds = Binds::from_input_data(binds)?;  
        let i = 1;
        self.bind_parameter1(1,  &i);
        self.input_binds = Some(input_binds);

        Ok(())        
    }
    
    fn bind_input_parameter1<'c, T>(
        &mut self,
        parameter_index: u16,
        value: &'c T,
        str_len_or_ind_ptr: *mut ffi::SQLLEN,
        enc_value: &EncodedValue,
    ) -> Return<()>
    where
        T: OdbcType<'c>,
        T: ?Sized,
    {
        //if encoded value exists, use it.
        let (column_size, value_ptr) = if enc_value.has_value() {
            (enc_value.column_size(), enc_value.value_ptr())
        } else {
            (value.column_size(), value.value_ptr())
        };

        match unsafe {
            ffi::SQLBindParameter(
                self.handle(),
                parameter_index,
                ffi::SQL_PARAM_INPUT,
                T::c_data_type(),
                T::sql_data_type(),
                column_size + 1,
                value.decimal_digits(),
                value_ptr,
                0,                  // buffer length
                str_len_or_ind_ptr, // Note that this ptr has to be valid until statement is executed
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("Unexpected return from SQLBindParameter: {:?}", r),
        }
    }

    fn reset_parameters1(&mut self) -> Return<()> {
        match unsafe { ffi::SQLFreeStmt(self.handle(), ffi::SQL_RESET_PARAMS) } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("SQLFreeStmt returned unexpected result: {:?}", r),
        }
    }

    fn bind_col1<'c, T>(
        &mut self,
        col_index: u16,
        value: &'c T,
        str_len_or_ind_ptr: *mut ffi::SQLLEN,
        enc_value: EncodedValue,
    ) -> Return<()>
    where
        T: OdbcType<'c>,
        T: ?Sized,
    {
        //if encoded value exists, use it.
        let (column_size, value_ptr) = if enc_value.has_value() {
            (enc_value.column_size(), enc_value.value_ptr())
        } else {
            unsafe{
                (*str_len_or_ind_ptr as u64, value.value_ptr())
            }
        };        

        match unsafe {
            ffi::SQLBindCol(
                self.handle(),
                col_index,
                T::c_data_type(),
                value_ptr,
                column_size as i64,    // buffer length
                str_len_or_ind_ptr, // Note that this ptr has to be valid until statement is executed
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("Unexpected return from SQLBindParameter: {:?}", r),
        }
    }

    
    fn bind_col2<'c, T>(
        &mut self,
        col_index: u16,
        value: &'c T,
        c_data_type:ffi::SqlCDataType,
        str_len_or_ind_ptr: *mut ffi::SQLLEN,
        enc_value: EncodedValue,
    ) -> Return<()>
    where
        T: OdbcType<'c>,
        T: ?Sized,
    {
        //if encoded value exists, use it.
        let (column_size, value_ptr) = if enc_value.has_value() {
            (enc_value.column_size(), enc_value.value_ptr())
        } else {
            unsafe{
                (*str_len_or_ind_ptr as u64, value.value_ptr())
            }
        };        

        match unsafe {
            ffi::SQLBindCol(
                self.handle(),
                col_index,
                c_data_type,
                value_ptr,
                column_size as i64,    // buffer length
                str_len_or_ind_ptr, // Note that this ptr has to be valid until statement is executed
            )
        } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("Unexpected return from SQLBindParameter: {:?}", r),
        }
    }

    fn get_data1<'a, T>(
        &mut self,
        col_or_param_num: u16,
        buffer: &'a mut Vec<u8>
    ) -> Return<Option<T>>
    where
        T: OdbcType<'a>,
    {
        self.get_partial_data1(col_or_param_num, buffer, 0)
    }

    fn get_partial_data1<'a, T>(
        &mut self,
        col_or_param_num: u16,
        buffer: &'a mut Vec<u8>,
        start_pos: usize
    ) -> Return<Option<T>>
    where
        T: OdbcType<'a>,
    {
        if buffer.len() - start_pos == 0 {
            panic!("buffer length may not be zero");
        }
        if buffer.len() - start_pos > ffi::SQLLEN::max_value() as usize {
            panic!("buffer is larger than {} bytes", ffi::SQLLEN::max_value());
        }
        let mut indicator: ffi::SQLLEN = 0;
        // Get buffer length...
        let result = unsafe { ffi::SQLGetData(
                self.handle(),
                col_or_param_num,
                T::c_data_type(),
                buffer.as_mut_ptr().offset(start_pos as isize) as ffi::SQLPOINTER,
                (buffer.len() - start_pos) as ffi::SQLLEN,
                &mut indicator as *mut ffi::SQLLEN,
            ) };
        match result {
            ffi::SQL_SUCCESS => {
                if indicator == ffi::SQL_NULL_DATA {
                    Return::Success(None)
                } else {
                    assert!(start_pos + indicator as usize <= buffer.len(), "no more data but indicatior outside of data buffer");
                    let slice = &buffer[..(start_pos + indicator as usize)];
                    Return::Success(Some(T::convert(slice)))
                }
            }
            ffi::SQL_SUCCESS_WITH_INFO => {
                let initial_len = buffer.len();
                // // As a workaround for drivers that don't include tailing null(s) check if last bytes are null
                // let null_offset = if buffer.ends_with(T::null_bytes()) { T::null_bytes().len() } else { 0 };

                // (Alexander Yekimov <a.yekimov@gmail.com>) It's a bad idea to do such workarounds
                // for buggy drivers here. They always can implement OdbcType trait and set any
                // amount of null-terminators to do the workaround.

                let null_offset = T::null_bytes_count();
                if indicator == ffi::SQL_NO_TOTAL {
                    buffer.resize(initial_len * 2, 0);
                    return self.get_partial_data1(col_or_param_num, buffer, initial_len - null_offset);
                } else {
                    // Check if string has been truncated.
                    if indicator >= initial_len as ffi::SQLLEN {
                        buffer.resize(indicator as usize + T::null_bytes_count(), 0);
                        return self.get_partial_data1(col_or_param_num, buffer, initial_len - null_offset);
                    } else {
                        let slice = &buffer[..(start_pos + indicator as usize)];
                        // No truncation. Warning may be due to some other issue.
                        Return::SuccessWithInfo(Some(T::convert(slice)))
                    }
                }
            }
            ffi::SQL_ERROR => Return::Error,
            ffi::SQL_NO_DATA => panic!("SQLGetData has already returned the colmun data"),
            r => panic!("unexpected return value from SQLGetData: {:?}", r),
        }
    }

    pub fn metadata(&self) -> QueryResult<StatementMetadata> {
        use diesel::result::Error::DeserializationError;             
        
        let mut vec = Vec::new();       
        for i in 1..=self.num_result_cols1().into_result(self).unwrap(){
            vec.push(self.describe_col1(i as u16).into_result(self).unwrap());
        }

        Some(vec).map(StatementMetadata::new)
           .ok_or_else(|| DeserializationError("No metadata exists".into()))
    }
}

impl<AC: AutocommitMode> Statement<Allocated, NoResult, AC> {
    pub fn with_parent(ds: &RawConnection<AC>) -> Result<Self> {
        let raii = Rc::new(Raii::with_parent(ds).into_result(ds)?); 
        Ok(Self::with_raii(raii))        
    }

    pub fn affected_row_count(&self) -> Result<ffi::SQLLEN> {
         self.affected_row_count1().into_result(self)
    }

    pub fn tables(self, catalog_name: &String, schema_name: &String, table_name: &String, table_type: &String) -> Result<Statement<Executed, HasResult, AC>> {
        self.tables_str(catalog_name.as_str(), schema_name.as_str(), table_name.as_str(), table_type.as_str())
    }

    pub fn tables_str(self, catalog_name: &str, schema_name: &str, table_name: &str, table_type: &str) -> Result<Statement<Executed, HasResult, AC>> {
        self.tables_opt_str(Option::Some(catalog_name), Option::Some(schema_name), Option::Some(table_name), table_type)
    }

    pub fn tables_opt_str(mut self, catalog_name: Option<&str>, schema_name: Option<&str>, table_name:Option<&str>, table_type: &str) -> Result<Statement<Executed, HasResult, AC>> {
        self.tables1(catalog_name, schema_name, table_name, table_type).into_result(&self)?;
        Ok(Statement::with_raii(self.raii))
    }
    
    /// Executes a preparable statement, using the current values of the parameter marker variables
    /// if any parameters exist in the statement.
    ///
    /// `SQLExecDirect` is the fastest way to submit an SQL statement for one-time execution.
    pub fn exec_direct(mut self, statement_text: &str) -> Result<ResultSetState<Executed, AC>> {
        if self.exec_direct1(statement_text).into_result(&self)? {
            let num_cols = self.num_result_cols1().into_result(&self)?;
            if num_cols > 0 {
                Ok(ResultSetState::Data(Statement::with_raii(self.raii)))
            } else {
                Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
            }
        } else {
            Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
        }
    }

    /// Executes a preparable statement, using the current values of the parameter marker variables
    /// if any parameters exist in the statement.
    ///
    /// `SQLExecDirect` is the fastest way to submit an SQL statement for one-time execution.
    pub fn exec_direct_bytes(mut self, bytes: &[u8]) -> Result<ResultSetState<Executed, AC>> {
        if self.exec_direct_bytes1(bytes).into_result(&self)? {
            let num_cols = self.num_result_cols1().into_result(&self)?;
            if num_cols > 0 {
                Ok(ResultSetState::Data(Statement::with_raii(self.raii)))
            } else {
                Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
            }
        } else {
            Ok(ResultSetState::NoData(Statement::with_raii(self.raii)))
        }
    }

}

impl<S, AC: AutocommitMode> Statement<S, HasResult, AC> {

    pub fn affected_row_count(&self) -> Result<ffi::SQLLEN> {
        self.affected_row_count1().into_result(self)
    }

    /// The number of columns in a result set
    ///
    /// Can be called successfully only when the statement is in the prepared, executed, or
    /// positioned state. If the statement does not return columns the result will be 0.
    pub fn num_result_cols(&self) -> Result<i16> {
        self.num_result_cols1().into_result(self)
    }


    /// Returns description struct for result set column with a given index. Note: indexing is starting from 1.
    pub fn describe_col(&self, idx: u16) -> Result<ColumnDescriptor> {
        self.describe_col1(idx).into_result(self)
    }

    /// Fetches the next rowset of data from the result set and returns data for all bound columns.
    pub fn fetch<'s>(&'s mut self) -> Result<Option<Cursor<'s, S, AC>>> {
        if self.fetch1().into_result(self)? {
            Ok(Some(Cursor {
                stmt: self,
                buffer: vec![0; 10000],
            }))
        } else {
            Ok(None)
        }
    }

    /// Call this method to reuse the statement to execute another query.
    ///
    /// For many drivers allocating new statements is expensive. So reusing a `Statement` is usually
    /// more efficient than freeing an existing and allocating a new one. However to reuse a
    /// statement any open result sets must be closed.
    /// Only call this method if you have already read the result set returned by the previous
    /// query, or if you do no not intend to read it.
    ///
    /// # Example
    ///
    /// ```
    /// # use odbc::*;
    /// # fn reuse () -> Result<()> {
    /// let env = create_environment_v3().map_err(|e| e.unwrap())?;
    /// let conn = env.connect("TestDataSource", "", "")?;
    /// let stmt = Statement::with_parent(&conn)?;
    /// let stmt = match stmt.exec_direct("CREATE TABLE STAGE (A TEXT, B TEXT);")?{
    ///     // Some drivers will return an empty result set. We need to close it before we can use
    ///     // statement again.
    ///     Data(stmt) => stmt.close_cursor()?,
    ///     NoData(stmt) => stmt,
    /// };
    /// let stmt = stmt.exec_direct("INSERT INTO STAGE (A, B) VALUES ('Hello', 'World');")?;
    /// //...
    /// # Ok(())
    /// # };
    /// ```
    pub fn close_cursor(mut self) -> Result<Statement<S, NoResult, AC>> {
        self.close_cursor1().into_result(&self)?;
        Ok(Statement::with_raii(self.raii))
    }
    
}

impl<'c, S, AC: AutocommitMode> Cursor<'c, S, AC> {
    /// Retrieves data for a single column in the result set
    ///
    /// ## Panics
    ///
    /// If you try to convert to `&str` but the data can't be converted
    /// without allocating an intermediate buffer.
    pub fn get_data<'d, T>(&'d mut self, col_or_param_num: u16) -> Result<Option<T>>
    where T: Output<'d>,
    {
        unsafe{
            T::get_data(self.stmt.raii.handle(), col_or_param_num, &mut self.buffer).into_result(self.stmt)
        }
    }
}


unsafe impl<S, R, AC: AutocommitMode> safe::Handle for Statement<S, R, AC> {

    const HANDLE_TYPE : ffi::HandleType = ffi::SQL_HANDLE_STMT;

    fn handle(&self) -> ffi::SQLHANDLE {
        
        safe::Handle::handle(self.raii.as_ref()) as ffi::SQLHANDLE      
    }
}

pub struct StatementUse<'b, S> {
    statement: &'b mut Statement<S, HasResult, AutocommitOn>,
    output_binds: &'b mut Binds,
    metadata: &'b StatementMetadata,
}

impl<'b, S> StatementUse<'b, S> {

    pub fn new(statement: &'b mut Statement<S, HasResult, AutocommitOn>, output_binds: &'b mut Binds, metadata:&'b StatementMetadata) -> Self {
        // let metadata = statement.metadata().unwrap();
        // let output_binds = Binds::from_output_types(types, &metadata);
        // statement.execute_statement(&mut output_binds).unwrap();        

        StatementUse {
            statement,
            output_binds,
            metadata,
        }
    }

    // pub fn run(&mut self) -> QueryResult<()> {
    //     self.statement.run()
    // }

    pub fn step(&mut self) -> QueryResult<Option<OdbcRow>> {        
        let mut row_index = 0;
        match self.statement.fetch(){
            Ok(_value) => {
                let bind_datas = &mut self.output_binds.data;
                for i in 0..bind_datas.len(){
                    let bind = &mut bind_datas[i];
                    match bind.tpe{
                        ffi::SqlDataType::SQL_EXT_WCHAR | ffi::SqlDataType::SQL_EXT_WVARCHAR | ffi::SqlDataType::SQL_EXT_WLONGVARCHAR =>{
                                                        
                            if bind.is_null(){
                                bind.length = 0;
                                bind.is_truncated = None;                                
                            }
                            else{
                                let code_utf16 =  encoding_rs::UTF_16LE.decode(&bind.bytes).0;                                
                                let mut bytes = (&code_utf16).as_bytes().to_vec();
                                for pos in 0..bytes.len(){
                                    if bytes[pos] == 0u8{
                                        bytes = bytes[0..pos].to_vec();
                                        break;
                                    }
                                }

                                if bind.bytes.len() > bytes.len(){
                                    let blank_nums = bind.bytes.len() - bytes.len();
                                    bytes.append(&mut [0u8].repeat(blank_nums).to_vec());
                                }
                                else
                                {
                                    println!("byte length is not enough,row index:{} column index:{} field name:{}", row_index, i, bind.field_name);
                                }
                                
                                bind.length = bytes.len() as i64;
                                for i in 0..bind.length as usize{
                                    bind.bytes[i] = bytes[i];
                                }
                            }
                        },
                        ffi::SqlDataType::SQL_CHAR | ffi::SqlDataType::SQL_VARCHAR | ffi::SqlDataType::SQL_EXT_LONGVARCHAR =>{
                            if bind.is_null(){
                                bind.length = 0;
                                bind.is_truncated = None;                                
                            }
                        },
                        ffi::SqlDataType::SQL_EXT_BIT =>{                            
                            if bind.is_null(){
                                unsafe {                                
                                    let para = bind.bytes.as_ptr() as *mut bool;
                                    (*para) = false;
                                }                                
                                bind.length = 1;
                                bind.is_truncated = None;                                
                            }
                        },
                        // ffi::SqlDataType::SQL_DATE=>{
                        //     // println!("date: {:?}", bind.bytes);
                        // },
                        // ffi::SqlDataType::SQL_TIME=>{
                        //     println!("time: {:?}", bind.bytes);
                        // },
                        ffi::SqlDataType::SQL_DATETIME=>{
                            if bind.is_null() {
                                unsafe{
                                    let mut para = bind.bytes.as_ptr() as *mut ffi::SQL_TIMESTAMP_STRUCT;
                                    (*para).year = 0;
                                    (*para).month = 1;
                                    (*para).day = 1;
                                    (*para).hour = 0;
                                    (*para).minute = 0;
                                    (*para).second = 0;
                                    (*para).fraction = 0;
                                }
                                bind.length = 16;
                                bind.is_truncated = None;
                            }                            
                        },
                        ffi::SqlDataType::SQL_INTEGER=>{
                            // if bind.is_null(){
                            //     bind.length = 4;
                            //     bind.bytes[0] = 0;
                            //     bind.bytes[1] = 0;
                            //     bind.bytes[2] = 0;
                            //     bind.bytes[3] = 0;
                            //     bind.is_truncated = None;                                
                            // }
                        },
                        // ffi::SqlDataType::SQL_DOUBLE=>{
                        //     println!("double: {:?}", bind.bytes);
                        // },
                        // ffi::SqlDataType::SQL_FLOAT=>{
                        //     println!("float: {:?}", bind.bytes);
                        //     unsafe{
                        //         let ptr = bind.bytes.as_ptr() as * const f64;
                        //         println!("float result: {:?}", *ptr);
                        //     }
                        // },
                        _=>{}                        
                    }                  
                }

                row_index += 1;
                let _ = row_index;               

                if let Some(mut _cur) = _value{
                    Ok(Some(OdbcRow {
                        col_idx: 0,
                        binds: &mut self.output_binds,
                        metadata: &self.metadata,
                    }))
                }
                else{
                    Ok(None)
                } 
            },
            Err(_e) => Err(diesel::result::Error::NotFound)
        }
    }
}

impl<'b, S> Drop for StatementUse<'b, S> {
    fn drop(&mut self) {
        // self.statement.reset();
    }
}