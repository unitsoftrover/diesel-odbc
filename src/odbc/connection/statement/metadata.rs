use super::ColumnDescriptor;
use std::ffi::CStr;
use crate::odbc::connection::bind::Flags;
use super::bind::*;

pub struct StatementMetadata {
    pub result: Vec<ColumnDescriptor>,
}

impl StatementMetadata {
    pub fn new(result: Vec<ColumnDescriptor>) -> Self {
        StatementMetadata { result }
    }

    pub fn fields(&'_ self) -> &'_ [ColumnDescriptor] {
        self.result.as_ref()
    }

    pub fn len(&self)->usize{
        return self.result.len()
    }
}

impl Drop for StatementMetadata {
    fn drop(&mut self) {      
    }
}

#[repr(transparent)]
pub struct MysqlFieldMetadata<'a>(MYSQL_FIELD, std::marker::PhantomData<&'a ()>);

impl<'a> MysqlFieldMetadata<'a> {
    pub fn field_name(&self) -> Option<&str> {
        if self.0.name.is_null() {
            None
        } else {
            unsafe {
                Some(CStr::from_ptr(self.0.name).to_str().expect(
                    "Expect mysql field names to be UTF-8, because we \
                     requested UTF-8 encoding on connection setup",
                ))
            }
        }
    }

    pub fn field_type(&self) -> enum_field_types {
        self.0.type_
    }

    pub(crate) fn flags(&self) -> Flags {
        Flags::from(self.0.flags)
    }
}
