use super::types::OdbcType;
use odbc_safe::AutocommitMode;
use super::types::EncodedValue;
use super::super::{ffi, Handle, Raii, Result, Return};
use super::Statement;

impl<'b, AC: AutocommitMode> Statement<AC> {
    /// Binds a parameter to a parameter marker in an SQL statement.
    ///
    /// # Result
    /// This method will destroy the statement and create a new one which may not outlive the bound
    /// parameter. This is to ensure that the statement will not dereference an invalid pointer
    /// during execution.
    ///
    /// # Arguments
    /// * `parameter_index` - Index of the marker to bind to the parameter. Starting at `1`
    /// * `value` - Reference to bind to the marker
    ///
    /// # Example
    /// ```
    /// # use odbc::*;
    /// # fn do_odbc_stuff() -> std::result::Result<(), Box<std::error::Error>> {
    /// let env = create_environment_v3().map_err(|e| e.unwrap())?;
    /// let conn = env.connect("TestDataSource", "", "")?;
    /// let stmt = Statement::with_parent(&conn)?;
    /// let param = 1968;
    /// let stmt = stmt.bind_parameter(1, &param)?;
    /// let sql_text = "SELECT TITLE FROM MOVIES WHERE YEAR = ?";
    /// if let Data(mut stmt) = stmt.exec_direct(sql_text)? {
    ///     // ...
    /// }
    /// #   Ok(())
    /// # }
    /// ```
    pub fn bind_parameter<'c, T>(
        &mut self,
        parameter_index: u16,
        value: &'c T,
    )
    where
        T: OdbcType<'c>,
        T: ?Sized,
        'b: 'c,
    {
        let ind = if value.value_ptr() == 0 as *const Self as ffi::SQLPOINTER {
            ffi::SQL_NULL_DATA
        } else {
            value.column_size() as ffi::SQLLEN
        };

        let ind_ptr = self.param_ind_buffers.alloc(parameter_index as usize, ind);

        //the result of value_ptr is changed per calling.
        //binding and saving must have the same value.
        let enc_value = value.encoded_value();

        self.bind_input_parameter(parameter_index, value, ind_ptr, &enc_value)
            .into_result(self).unwrap();

        // save encoded value to avoid memory reuse.
        if enc_value.has_value() {
            self.encoded_values.push(enc_value);
        };
    }

    /// Releasing all parameter buffers set by `bind_parameter`. This method consumes the statement
    /// and returns a new one those lifetime is no longer limited by the buffers bound.
    pub fn reset_parameters(mut self) -> Result<bool> {
        self.param_ind_buffers.clear();
        self.encoded_values.clear();
        self.raii.reset_parameters().into_result(&mut self)?;
        Ok(true)
    }

    /// Binds a buffer and an indicator to a column.
    ///
    /// See [SQLBindCol][1]:
    /// [1]: [https://docs.microsoft.com/en-us/sql/odbc/reference/syntax/sqlbindcol-function]
    pub fn bind_col<'c, T>(
        mut self,
        column_number: u16,
        value: &'c mut T
    ) -> Result<Statement<AC>>
    where
        T: OdbcType<'c> + ?Sized,
        'b: 'c,
    {
        let ind = if value.value_ptr() == 0 as *const Self as ffi::SQLPOINTER {
            ffi::SQL_NULL_DATA
        } else {
            value.column_size() as ffi::SQLLEN
        };
        let ind_ptr = self.param_ind_buffers.alloc(column_number as usize, ind);

        //the result of value_ptr is changed per calling.
        //binding and saving must have the same value.
        let enc_value = value.encoded_value();        
        let _ = self.bind_col1(column_number, value, ind_ptr, enc_value);

        Ok(self)
    }
}

impl Raii<ffi::Stmt> {
    pub fn bind_input_parameter<'c, T>(
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

    pub fn reset_parameters(&mut self) -> Return<()> {
        match unsafe { ffi::SQLFreeStmt(self.handle(), ffi::SQL_RESET_PARAMS) } {
            ffi::SQL_SUCCESS => Return::Success(()),
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(()),
            ffi::SQL_ERROR => Return::Error,
            r => panic!("SQLFreeStmt returned unexpected result: {:?}", r),
        }
    }

    pub fn bind_col<'c, T>(
        &mut self,
        col_index: u16,
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
                col_index,
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
}
