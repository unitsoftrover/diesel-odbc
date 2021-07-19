//! The Odbc backend

use byteorder::NativeEndian;
use super::query_builder::OdbcQueryBuilder;
use super::OdbcValue;
use diesel::backend::*;
use diesel::query_builder::bind_collector::RawBytesBindCollector;
use diesel::sql_types::TypeMetadata;


/// The Odbc backend
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Odbc;

#[allow(missing_debug_implementations)]
/// Represents possible types, that can be transmitted as via the
/// Odbc wire protocol
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum OdbcSqlType {
    /// A 8 bit signed integer
    Tiny,
    /// A 8 bit unsigned integer
    UnsignedTiny,
    /// A 16 bit signed integer
    Short,
    /// A 16 bit unsigned integer
    UnsignedShort,
    /// A 32 bit signed integer
    Long,
    /// A 32 bit unsigned integer
    UnsignedLong,
    /// A 64 bit signed integer
    LongLong,
    /// A 64 bit unsigned integer
    UnsignedLongLong,    
    /// A 32 bit floating point number
    Float,
    /// A 64 bit floating point number
    Double,
    /// A fixed point decimal value
    Numeric,
    /// A datatype to store a time value
    Time,
    /// A datatype to store a date value
    Date,
    /// A datatype containing timestamp values ranging from
    /// '1000-01-01 00:00:00' to '9999-12-31 23:59:59'.
    DateTime,
    /// A datatype containing timestamp values ranging from
    /// 1970-01-01 00:00:01' UTC to '2038-01-19 03:14:07' UTC.
    Timestamp,
    /// A datatype for string values
    String,
    /// A datatype containing binary large objects
    Blob,
    /// A value containing a set of bit's
    Bit,
    /// A user defined set type
    Set,
    /// A user defined enum type
    Enum,
}

impl Backend for Odbc {
    type QueryBuilder = OdbcQueryBuilder;
    type BindCollector = RawBytesBindCollector<Self>;
    type ByteOrder = NativeEndian;
}

impl<'a> HasRawValue<'a> for Odbc {
    type RawValue = OdbcValue<'a>;
}

impl TypeMetadata for Odbc {
    type TypeMetadata = OdbcSqlType;
    type MetadataLookup = ();
}

impl SupportsReturningClause for Odbc {}
impl SupportsDefaultKeyword for Odbc {}
impl UsesAnsiSavepointSyntax for Odbc {}
