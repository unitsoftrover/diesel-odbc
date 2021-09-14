use std::error::Error;
use std::str::{self, FromStr};

use diesel::deserialize::{self, FromSql};
use crate::odbc::{Odbc, OdbcValue};
use diesel::sql_types::{BigInt, Binary, Double, Float, Integer, SmallInt, Text};

fn decimal_to_integer<T>(bytes: &[u8]) -> deserialize::Result<T>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    let string = str::from_utf8(bytes)?;
    let mut splited = string.split('.');
    let integer_portion = splited.next().unwrap_or_default();
    let decimal_portion = splited.next().unwrap_or_default();
    if splited.next().is_some() {
        Err(format!("Invalid decimal format: {:?}", string).into())
    } else if decimal_portion.chars().any(|c| c != '0') {
        Err(format!(
            "Tried to convert a decimal to an integer that contained /
             a non null decimal portion: {:?}",
            string
        )
        .into())
    } else {
        Ok(integer_portion.parse()?)
    }
}

impl FromSql<SmallInt, Odbc> for i16 {
    fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
        use crate::odbc::NumericRepresentation::*;

        match value.numeric_value()? {
            Tiny(x) => Ok(x.into()),
            Small(x) => Ok(x),
            Medium(x) => Ok(x as Self),
            Big(x) => Ok(x as Self),
            Float(x) => Ok(x as Self),
            Double(x) => Ok(x as Self),
            Decimal(bytes) => decimal_to_integer(bytes),
        }
    }
}

impl FromSql<Integer, Odbc> for i32 {
    fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
        use crate::odbc::NumericRepresentation::*;

        match value.numeric_value()? {
            Tiny(x) => Ok(x.into()),
            Small(x) => Ok(x.into()),
            Medium(x) => Ok(x),
            Big(x) => Ok(x as Self),
            Float(x) => Ok(x as Self),
            Double(x) => Ok(x as Self),
            Decimal(bytes) => decimal_to_integer(bytes),
        }
    }
}

impl FromSql<BigInt, Odbc> for i64 {
    fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
        use crate::odbc::NumericRepresentation::*;

        match value.numeric_value()? {
            Tiny(x) => Ok(x.into()),
            Small(x) => Ok(x.into()),
            Medium(x) => Ok(x.into()),
            Big(x) => Ok(x),
            Float(x) => Ok(x as Self),
            Double(x) => Ok(x as Self),
            Decimal(bytes) => decimal_to_integer(bytes),
        }
    }
}

impl FromSql<Float, Odbc> for f32 {
    fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
        use crate::odbc::NumericRepresentation::*;

        match value.numeric_value()? {
            Tiny(x) => Ok(x.into()),
            Small(x) => Ok(x.into()),
            Medium(x) => Ok(x as Self),
            Big(x) => Ok(x as Self),
            Float(x) => Ok(x),
            Double(x) => Ok(x as Self),
            Decimal(bytes) => Ok(str::from_utf8(bytes)?.parse()?),
        }
    }
}

impl FromSql<Double, Odbc> for f64 {
    fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
        use crate::odbc::NumericRepresentation::*;

        match value.numeric_value()? {
            Tiny(x) => Ok(x.into()),
            Small(x) => Ok(x.into()),
            Medium(x) => Ok(x.into()),
            Big(x) => Ok(x as Self),
            Float(x) => Ok(x.into()),
            Double(x) => Ok(x),
            Decimal(bytes) => Ok(str::from_utf8(bytes)?.parse()?),
        }
    }
}

impl FromSql<Text, Odbc> for String {
    fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
        let mut ret = String::from_utf8(value.as_bytes().into());
        if let Ok(str) = &ret{
            ret = Ok(str.trim_end_matches('\0').to_string());
        }
        ret.map_err(Into::into)
    }
}

impl FromSql<Binary, Odbc> for Vec<u8> {
    fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
        Ok(value.as_bytes().into())
    }
}
