use super::OdbcSqlType;
use diesel::deserialize;
use crate::odbc::connection::ffi::{SQL_TIMESTAMP_STRUCT, SQL_TIME_STRUCT, SQL_DATE_STRUCT};
use std::error::Error;

/// Raw Odbc value as received from the database
#[derive(Copy, Clone, Debug)]
pub struct OdbcValue<'a> {
    raw: &'a [u8],
    tpe: OdbcSqlType,
}

impl<'a> OdbcValue<'a> {
    pub(crate) fn new(raw: &'a [u8], tpe: OdbcSqlType) -> Self {
        Self { raw, tpe }
    }

    /// Get the underlying raw byte representation
    pub fn as_bytes(&self) -> &[u8] {
        self.raw
    }

    /// Get the Odbc type of the current value
    pub fn value_type(&self) -> OdbcSqlType {
        self.tpe
    }

    /// Checks that the type code is valid, and interprets the data as a
    /// `Odbc_TIME` pointer
    // We use `ptr.read_unaligned()` to read the potential unaligned ptr,
    // so clippy is clearly wrong here
    // https://github.com/rust-lang/rust-clippy/issues/2881
    #[allow(dead_code, clippy::cast_ptr_alignment)]
    pub(crate) fn datetime_value(&self) -> deserialize::Result<SQL_TIMESTAMP_STRUCT> {
        match self.tpe {
            OdbcSqlType::Time | OdbcSqlType::Date | OdbcSqlType::DateTime | OdbcSqlType::Timestamp => {
                let ptr = self.raw.as_ptr() as *const SQL_TIMESTAMP_STRUCT;
                let result = unsafe { ptr.read_unaligned() };                
                Ok(result)                
            }
            _ => Err(self.invalid_type_code("timestamp")),
        }
    }

    #[allow(dead_code, clippy::cast_ptr_alignment)]
    pub(crate) fn time_value(&self) -> deserialize::Result<SQL_TIME_STRUCT> {
        match self.tpe {
            OdbcSqlType::Time => {
                let ptr = self.raw.as_ptr() as *const SQL_TIME_STRUCT;
                let result = unsafe { ptr.read_unaligned() };                
                Ok(result)                
            }
            _ => Err(self.invalid_type_code("time")),
        }
    }

    #[allow(dead_code, clippy::cast_ptr_alignment)]
    pub(crate) fn date_value(&self) -> deserialize::Result<SQL_DATE_STRUCT> {
        match self.tpe {
            OdbcSqlType::Date => {
                let ptr = self.raw.as_ptr() as *const SQL_DATE_STRUCT;
                let result = unsafe { ptr.read_unaligned() };                
                Ok(result)                
            }
            _ => Err(self.invalid_type_code("date")),
        }
    }

    /// Returns the numeric representation of this value, based on the type code.
    /// Returns an error if the type code is not numeric.
    pub(crate) fn numeric_value(&self) -> deserialize::Result<NumericRepresentation> {
        use self::NumericRepresentation::*;
        use std::convert::TryInto;

        Ok(match self.tpe {
            OdbcSqlType::UnsignedTiny | OdbcSqlType::Tiny => Tiny(self.raw[0] as i8),
            OdbcSqlType::UnsignedShort | OdbcSqlType::Short => {
                Small(i16::from_ne_bytes(self.raw.try_into()?))
            }
            OdbcSqlType::UnsignedLong | OdbcSqlType::Long => {
                Medium(i32::from_ne_bytes(self.raw.try_into()?))
            }
            OdbcSqlType::UnsignedLongLong | OdbcSqlType::LongLong => {
                Big(i64::from_ne_bytes(self.raw.try_into()?))
            }
            OdbcSqlType::Float => Float(f32::from_ne_bytes(self.raw.try_into()?)),
            OdbcSqlType::Double => Double(f64::from_ne_bytes(self.raw.try_into()?)),

            OdbcSqlType::Numeric => Decimal(self.raw),
            _ => return Err(self.invalid_type_code("number")),
        })
    }

    fn invalid_type_code(&self, expected: &str) -> Box<dyn Error + Send + Sync> {
        format!(
            "Invalid representation received for {}: {:?}",
            expected, self.tpe
        )
        .into()
    }
}

/// Represents all possible forms Odbc transmits integers
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum NumericRepresentation<'a> {
    /// Correponds to `Odbc_TYPE_TINY`
    Tiny(i8),
    /// Correponds to `Odbc_TYPE_SHORT`
    Small(i16),
    /// Correponds to `Odbc_TYPE_INT24` and `Odbc_TYPE_LONG`
    Medium(i32),
    /// Correponds to `Odbc_TYPE_LONGLONG`
    Big(i64),
    /// Correponds to `Odbc_TYPE_FLOAT`
    Float(f32),
    /// Correponds to `Odbc_TYPE_DOUBLE`
    Double(f64),
    /// Correponds to `Odbc_TYPE_DECIMAL` and `Odbc_TYPE_NEWDECIMAL`
    Decimal(&'a [u8]),
}
