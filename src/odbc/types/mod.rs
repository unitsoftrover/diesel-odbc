//! Odbc specific types

#[cfg(feature = "chrono")]
mod date_and_time;
#[cfg(feature = "serde_json")]
mod json;
pub mod numeric;
mod primitives;

use byteorder::WriteBytesExt;
use std::io::Write;
use diesel::deserialize::{self, FromSql};
use crate::odbc::{Odbc, OdbcSqlType, OdbcValue};
use diesel::query_builder::QueryId;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::ops::*;
use diesel::sql_types::*;


impl ToSql<TinyInt, Odbc> for i8 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
        out.write_i8(*self).map(|_| IsNull::No).map_err(Into::into)
    }
}

impl FromSql<TinyInt, Odbc> for i8 {
    fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes();
        Ok(bytes[0] as i8)
    }
}

/// Represents the Odbc unsigned type.
#[derive(Debug, Clone, Copy, Default, SqlType, QueryId)]
pub struct Unsigned<ST>(ST);

impl<T> Add for Unsigned<T>
where
    T: Add,
{
    type Rhs = Unsigned<T::Rhs>;
    type Output = Unsigned<T::Output>;
}

impl<T> Sub for Unsigned<T>
where
    T: Sub,
{
    type Rhs = Unsigned<T::Rhs>;
    type Output = Unsigned<T::Output>;
}

impl<T> Mul for Unsigned<T>
where
    T: Mul,
{
    type Rhs = Unsigned<T::Rhs>;
    type Output = Unsigned<T::Output>;
}

impl<T> Div for Unsigned<T>
where
    T: Div,
{
    type Rhs = Unsigned<T::Rhs>;
    type Output = Unsigned<T::Output>;
}

impl ToSql<Unsigned<TinyInt>, Odbc> for u8 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
        ToSql::<TinyInt, Odbc>::to_sql(&(*self as i8), out)
    }
}

impl FromSql<Unsigned<TinyInt>, Odbc> for u8 {
    fn from_sql(bytes: OdbcValue<'_>) -> deserialize::Result<Self> {
        let signed: i8 = FromSql::<TinyInt, Odbc>::from_sql(bytes)?;
        Ok(signed as u8)
    }
}

impl ToSql<Unsigned<SmallInt>, Odbc> for u16 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
        ToSql::<SmallInt, Odbc>::to_sql(&(*self as i16), out)
    }
}

impl FromSql<Unsigned<SmallInt>, Odbc> for u16 {
    fn from_sql(bytes: OdbcValue<'_>) -> deserialize::Result<Self> {
        let signed: i16 = FromSql::<SmallInt, Odbc>::from_sql(bytes)?;
        Ok(signed as u16)
    }
}

impl ToSql<Unsigned<Integer>, Odbc> for u32 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
        ToSql::<Integer, Odbc>::to_sql(&(*self as i32), out)
    }
}

impl FromSql<Unsigned<Integer>, Odbc> for u32 {
    fn from_sql(bytes: OdbcValue<'_>) -> deserialize::Result<Self> {
        let signed: i32 = FromSql::<Integer, Odbc>::from_sql(bytes)?;
        Ok(signed as u32)
    }
}

impl ToSql<Unsigned<BigInt>, Odbc> for u64 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
        ToSql::<BigInt, Odbc>::to_sql(&(*self as i64), out)
    }
}

impl FromSql<Unsigned<BigInt>, Odbc> for u64 {
    fn from_sql(bytes: OdbcValue<'_>) -> deserialize::Result<Self> {
        let signed: i64 = FromSql::<BigInt, Odbc>::from_sql(bytes)?;
        Ok(signed as u64)
    }
}


impl ToSql<Bool, Odbc> for bool {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
        let int_value = if *self { 1 } else { 0 };
        <i32 as ToSql<Integer, Odbc>>::to_sql(&int_value, out)
    }
}

impl FromSql<Bool, Odbc> for bool {
    fn from_sql(bytes: OdbcValue<'_>) -> deserialize::Result<Self> {
        Ok(bytes.as_bytes().iter().any(|x| *x != 0))
    }
}


impl HasSqlType<Unsigned<TinyInt>> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::UnsignedTiny
    }
}

impl HasSqlType<Unsigned<SmallInt>> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::UnsignedShort
    }
}

impl HasSqlType<Unsigned<Integer>> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::UnsignedLong
    }
}

impl HasSqlType<Unsigned<BigInt>> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::UnsignedLongLong
    }
}


impl HasSqlType<TinyInt> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Tiny
    }
}

impl HasSqlType<SmallInt> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Short
    }
}

impl HasSqlType<Integer> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Long
    }
}

impl HasSqlType<BigInt> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::LongLong
    }
}

impl HasSqlType<Bool> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Bit
    }
}


impl HasSqlType<Float> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Float
    }
}

impl HasSqlType<Double> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Double
    }
}

impl HasSqlType<Text> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::String
    }
}

impl HasSqlType<Binary> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Blob
    }
}

impl HasSqlType<Date> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Date
    }
}

impl HasSqlType<Time> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Time
    }
}

impl HasSqlType<Timestamp> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Timestamp
    }
}

/// Represents the Odbc datetime type.
///
/// ### [`ToSql`] impls
///
/// - [`chrono::NaiveDateTime`] with `feature = "chrono"`
///
/// ### [`FromSql`] impls
///
/// - [`chrono::NaiveDateTime`] with `feature = "chrono"`
///
/// [`ToSql`]: ../../serialize/trait.ToSql.html
/// [`FromSql`]: ../../deserialize/trait.FromSql.html
/// [`chrono::NaiveDateTime`]: ../../../chrono/naive/struct.NaiveDateTime.html
#[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
#[mysql_type = "DateTime"]
pub struct Datetime;




// impl ToSql<Numeric, Odbc> for BigDecimal {
//     fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
//         write!(out, "{}", *self)
//             .map(|_| IsNull::No)
//             .map_err(Into::into)
//     }
// }

// impl FromSql<Numeric, Odbc> for BigDecimal {
//     fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
//         use crate::odbc::NumericRepresentation::*;

//         match value.numeric_value()? {
//             Tiny(x) => Ok(x.into()),
//             Small(x) => Ok(x.into()),
//             Medium(x) => Ok(x.into()),
//             Big(x) => Ok(x.into()),
//             Float(x) => BigDecimal::from_f32(x)
//                 .ok_or_else(|| format!("{} is not valid decimal number ", x).into()),
//             Double(x) => BigDecimal::from_f64(x)
//                 .ok_or_else(|| format!("{} is not valid decimal number ", x).into()),
//             Decimal(bytes) => BigDecimal::parse_bytes(bytes, 10)
//                 .ok_or_else(|| format!("{:?} is not valid decimal number ", bytes).into()),
//         }
//     }
// }

impl HasSqlType<Numeric> for Odbc {
    fn metadata(_lookup: &()) -> OdbcSqlType {
        OdbcSqlType::Numeric
    }
}