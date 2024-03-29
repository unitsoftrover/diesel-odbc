use std::mem;
use std::ops::Index;
use std::os::raw as libc;

use crate::odbc::connection::statement::StatementMetadata;
use crate::odbc::{OdbcSqlType, OdbcValue};
use diesel::result::QueryResult;
extern crate bitflags;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum enum_field_types {
    ODBC_TYPE_DECIMAL = 0,
    ODBC_TYPE_TINY = 1,
    ODBC_TYPE_SHORT = 2,
    ODBC_TYPE_LONG = 3,
    ODBC_TYPE_FLOAT = 4,
    ODBC_TYPE_DOUBLE = 5,
    ODBC_TYPE_NULL = 6,
    ODBC_TYPE_TIMESTAMP = 7,
    ODBC_TYPE_LONGLONG = 8,
    ODBC_TYPE_INT24 = 9,
    ODBC_TYPE_DATE = 10,
    ODBC_TYPE_TIME = 11,
    ODBC_TYPE_DATETIME = 12,
    ODBC_TYPE_YEAR = 13,
    ODBC_TYPE_NEWDATE = 14,
    ODBC_TYPE_VARCHAR = 15,
    ODBC_TYPE_BIT = 16,
    ODBC_TYPE_TIMESTAMP2 = 17,
    ODBC_TYPE_DATETIME2 = 18,
    ODBC_TYPE_TIME2 = 19,
    ODBC_TYPE_JSON = 245,
    ODBC_TYPE_NEWDECIMAL = 246,
    ODBC_TYPE_ENUM = 247,
    ODBC_TYPE_SET = 248,
    ODBC_TYPE_TINY_BLOB = 249,
    ODBC_TYPE_MEDIUM_BLOB = 250,
    ODBC_TYPE_LONG_BLOB = 251,
    ODBC_TYPE_BLOB = 252,
    ODBC_TYPE_VAR_STRING = 253,
    ODBC_TYPE_STRING = 254,
    ODBC_TYPE_GEOMETRY = 255,
}

#[allow(non_camel_case_types)]
pub type my_bool = ::std::os::raw::c_char;

#[allow(non_camel_case_types)]
pub struct ODBC_BIND {
    pub length: *mut ::std::os::raw::c_ulong,
    pub is_null: *mut my_bool,
    pub buffer: *mut ::std::os::raw::c_void,
    pub error: *mut my_bool,
    pub row_ptr: *mut ::std::os::raw::c_uchar,    
    pub buffer_length: ::std::os::raw::c_ulong,
    pub offset: ::std::os::raw::c_ulong,
    pub length_value: ::std::os::raw::c_ulong,
    pub param_number: ::std::os::raw::c_uint,
    pub pack_length: ::std::os::raw::c_uint,
    pub buffer_type: odbc_sys::SqlDataType,
    pub error_value: my_bool,
    pub is_unsigned: my_bool,
    pub long_data_used: my_bool,
    pub is_null_value: my_bool,
    pub extension: *mut ::std::os::raw::c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ODBC_FIELD {
    pub name: *mut ::std::os::raw::c_char,
    pub org_name: *mut ::std::os::raw::c_char,
    pub table: *mut ::std::os::raw::c_char,
    pub org_table: *mut ::std::os::raw::c_char,
    pub db: *mut ::std::os::raw::c_char,
    pub catalog: *mut ::std::os::raw::c_char,
    pub def: *mut ::std::os::raw::c_char,
    pub length: ::std::os::raw::c_ulong,
    pub max_length: ::std::os::raw::c_ulong,
    pub name_length: ::std::os::raw::c_uint,
    pub org_name_length: ::std::os::raw::c_uint,
    pub table_length: ::std::os::raw::c_uint,
    pub org_table_length: ::std::os::raw::c_uint,
    pub db_length: ::std::os::raw::c_uint,
    pub catalog_length: ::std::os::raw::c_uint,
    pub def_length: ::std::os::raw::c_uint,
    pub flags: ::std::os::raw::c_uint,
    pub decimals: ::std::os::raw::c_uint,
    pub charsetnr: ::std::os::raw::c_uint,
    pub type_: enum_field_types,
    pub extension: *mut ::std::os::raw::c_void,
}

pub struct Binds {
    pub data: Vec<BindData>,
}

impl Binds {
    pub fn from_input_data<Iter>(input: Iter) -> QueryResult<Self>
    where
        Iter: IntoIterator<Item = (OdbcSqlType, Option<Vec<u8>>)>,
    {
        let data = input
            .into_iter()
            .map(BindData::for_input)
            .collect::<Vec<_>>();

        Ok(Binds { data })
    }

    pub fn from_output_types(types: Vec<Option<OdbcSqlType>>, metadata: &StatementMetadata) -> Self {
        let data = metadata
            .fields()
            .iter()
            .zip(types.into_iter().chain(std::iter::repeat(None)))
            .map(|(field, tpe)| {              
                if let Some(tpe) = tpe {
                    BindData::for_output(tpe.into(), field)
                } else {
                    BindData::for_output((field.data_type, Flags::NOT_NULL_FLAG), field)
                }
            })
            .collect();

        Binds { data }
    }

    pub fn with_odbc_binds<F, T>(&mut self, mut f: F)
    where
        F: FnMut(&mut BindData) -> T,
    {
        let _binds = self
            .data
            .iter_mut()
            .map(|x| unsafe { 
                f(x);
                x.odbc_bind()
            })
            .collect::<Vec<_>>();

    }

    // pub fn populate_dynamic_buffers(&mut self, stmt: &Statement) -> QueryResult<()> {
    //     for (i, data) in self.data.iter_mut().enumerate() {
    //         data.did_numeric_overflow_occur()?;
    //         // This is safe because we are re-binding the invalidated buffers
    //         // at the end of this function
    //         unsafe {
    //             if let Some((mut bind, offset)) = data.bind_for_truncated_data() {
    //                 stmt.fetch_column(&mut bind, i, offset)?
    //             } else {
    //                 data.update_buffer_length()
    //             }
    //         }
    //     }

    //     unsafe { self.with_odbc_binds(|bind_ptr| stmt.bind_result(bind_ptr)) }
    // }

    pub fn update_buffer_lengths(&mut self) {
        for data in &mut self.data {
            data.update_buffer_length();
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Index<usize> for Binds {
    type Output = BindData;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

bitflags::bitflags! {
    pub(crate) struct Flags: u32 {
        const NOT_NULL_FLAG = 1;
        const PRI_KEY_FAG = 2;
        const UNIQUE_KEY_FLAG = 4;
        const MULTIPLE_KEY_FLAG = 8;
        const BLOB_FLAG = 16;
        const UNSIGNED_FLAG = 32;
        const ZEROFILL_FLAG = 64;
        const BINARY_FLAG = 128;
        const ENUM_FLAG = 256;
        const AUTO_INCREMENT_FLAG = 512;
        const TIMESTAMP_FLAG = 1024;
        const SET_FLAG = 2048;
        const NO_DEFAULT_VALUE_FLAG = 4096;
        const ON_UPDATE_NOW_FLAG = 8192;
        const NUM_FLAG = 32768;
        const PART_KEY_FLAG = 16384;
        const GROUP_FLAG = 32768;
        const UNIQUE_FLAG = 65536;
        const BINCMP_FLAG = 130_172;
        const GET_FIXED_FIELDS_FLAG = (1<<18);
        const FIELD_IN_PART_FUNC_FLAG = (1 << 19);
    }
}

impl From<u32> for Flags {
    fn from(flags: u32) -> Self {
        Flags::from_bits(flags).expect(
            "We encountered a unknown type flag while parsing \
             Odbc's type information. If you see this error message \
             please open an issue at diesels github page.",
        )
    }
}

#[derive(Debug)]
pub struct BindData {
    pub tpe: odbc_sys::SqlDataType,
    pub bytes: Vec<u8>,
    pub length: super::ffi::SQLLEN,
    flags: Flags,
    is_null: my_bool,
    pub is_truncated: Option<my_bool>,
    pub field_name : String
}

impl BindData {
    fn for_input((tpe, data): (OdbcSqlType, Option<Vec<u8>>)) -> Self {
        let is_null = if data.is_none() { 1 } else { 0 };
        let bytes = data.unwrap_or_default();
        let length = bytes.len() as super::ffi::SQLLEN;
        let (tpe, flags) = tpe.into();
        BindData {
            tpe,
            bytes,
            length,
            is_null,
            is_truncated: None,
            flags,
            field_name : "".to_string(),
        }
    }

    fn for_output((tpe, flags): (odbc_sys::SqlDataType, Flags), field: &super::ColumnDescriptor) -> Self {
        let mut length = field.column_size.unwrap();
        if length > 1000{
            length = 1000;
        }
        let bytes = known_buffer_size_for_ffi_type(tpe)
            .map(|len| vec![0; len])
            .unwrap_or(vec![0; length as usize]);
        let length = bytes.len() as super::ffi::SQLLEN;

        BindData {
            tpe,
            bytes,
            length,
            is_null: 0,
            is_truncated: Some(0),
            flags,
            field_name : field.name.clone(),
        }
    }

    fn is_truncated(&self) -> bool {
        self.is_truncated.unwrap_or(0) != 0
    }

    fn is_fixed_size_buffer(&self) -> bool {
        known_buffer_size_for_ffi_type(self.tpe).is_some()
    }

    pub fn value(&'_ self) -> Option<OdbcValue<'_>> {
        if self.is_null(){
            None
        } else {
            let tpe = (self.tpe, self.flags).into();
            let val = OdbcValue::new(&self.bytes, tpe);
            Some(val)
        }
    }

    pub fn raw_value(&self)->Option<&'_ [u8]> {
        return Some(&self.bytes)
    }
    
    pub fn is_null(&self) -> bool {       
        self.length == -1        
    }

    fn update_buffer_length(&mut self) {
        use std::cmp::min;

        let actual_bytes_in_buffer = min(self.bytes.capacity(), self.length as usize);
        unsafe { self.bytes.set_len(actual_bytes_in_buffer) }
    }

    unsafe fn odbc_bind(&mut self) -> ODBC_BIND {
        let mut bind: ODBC_BIND = mem::zeroed();
        bind.buffer_type = self.tpe;
        bind.buffer = self.bytes.as_mut_ptr() as *mut libc::c_void;
        bind.buffer_length = self.bytes.capacity() as libc::c_ulong;
        bind.length = &mut (self.length as u32);
        bind.is_null = &mut self.is_null;
        bind.is_unsigned = self.flags.contains(Flags::UNSIGNED_FLAG) as my_bool;

        if let Some(ref mut is_truncated) = self.is_truncated {
            bind.error = is_truncated;
        }
        bind
    }

    /// Resizes the byte buffer to fit the value of `self.length`, and returns
    /// a tuple of a bind pointing at the truncated data, and the offset to use
    /// in order to read the truncated data into it.
    ///
    /// This invalidates the bind previously returned by `odbc_bind`. Calling
    /// this function is unsafe unless the binds are immediately rebound.
    // unsafe fn bind_for_truncated_data(&mut self) -> Option<(ffi::odbc_BIND, usize)> {
    //     if self.is_truncated() {
    //         let offset = self.bytes.capacity();
    //         let truncated_amount = self.length as usize - offset;

    //         debug_assert!(
    //             truncated_amount > 0,
    //             "output buffers were invalidated \
    //              without calling `odbc_stmt_bind_result`"
    //         );
    //         self.bytes.set_len(offset);
    //         self.bytes.reserve(truncated_amount);
    //         self.bytes.set_len(self.length as usize);

    //         let mut bind = self.odbc_bind();
    //         bind.buffer = self.bytes[offset..].as_mut_ptr() as *mut libc::c_void;
    //         bind.buffer_length = truncated_amount as libc::c_ulong;
    //         Some((bind, offset))
    //     } else {
    //         None
    //     }
    // }

    fn did_numeric_overflow_occur(&self) -> QueryResult<()> {
        use diesel::result::Error::DeserializationError;

        if self.is_truncated() && self.is_fixed_size_buffer() {
            Err(DeserializationError(
                "Numeric overflow/underflow occurred".into(),
            ))
        } else {
            Ok(())
        }
    }
}

impl From<OdbcSqlType> for (odbc_sys::SqlDataType, Flags) {
    fn from(tpe: OdbcSqlType) -> Self {        
        let mut flags = Flags::empty();
        let tpe = match tpe {
            OdbcSqlType::Tiny => odbc_sys::SqlDataType::SQL_EXT_TINYINT,
            OdbcSqlType::Short => odbc_sys::SqlDataType::SQL_SMALLINT,
            OdbcSqlType::Long => odbc_sys::SqlDataType::SQL_INTEGER,
            OdbcSqlType::LongLong => odbc_sys::SqlDataType::SQL_INTEGER,
            OdbcSqlType::Float => odbc_sys::SqlDataType::SQL_REAL,
            OdbcSqlType::Double => odbc_sys::SqlDataType::SQL_FLOAT,
            OdbcSqlType::Time => odbc_sys::SqlDataType::SQL_TIME,
            OdbcSqlType::Date => odbc_sys::SqlDataType::SQL_DATE,
            OdbcSqlType::DateTime => odbc_sys::SqlDataType::SQL_DATETIME,
            OdbcSqlType::Timestamp => odbc_sys::SqlDataType::SQL_DATETIME,
            OdbcSqlType::String => odbc_sys::SqlDataType::SQL_EXT_WCHAR,
            OdbcSqlType::Blob => odbc_sys::SqlDataType::SQL_EXT_WLONGVARCHAR,
            OdbcSqlType::Numeric => odbc_sys::SqlDataType::SQL_NUMERIC,
            OdbcSqlType::Bit => odbc_sys::SqlDataType::SQL_EXT_BIT,
            OdbcSqlType::UnsignedTiny => {
                flags = Flags::UNSIGNED_FLAG;
                odbc_sys::SqlDataType::SQL_SMALLINT
            }
            OdbcSqlType::UnsignedShort => {
                flags = Flags::UNSIGNED_FLAG;
                odbc_sys::SqlDataType::SQL_SMALLINT
            }
            OdbcSqlType::UnsignedLong => {
                flags = Flags::UNSIGNED_FLAG;
                odbc_sys::SqlDataType::SQL_INTEGER
            }
            OdbcSqlType::UnsignedLongLong => {
                flags = Flags::UNSIGNED_FLAG;
                odbc_sys::SqlDataType::SQL_INTEGER
            }
            OdbcSqlType::Set => {
                flags = Flags::SET_FLAG;
                odbc_sys::SqlDataType::SQL_EXT_WCHAR
            }
            OdbcSqlType::Enum => {
                flags = Flags::ENUM_FLAG;
                odbc_sys::SqlDataType::SQL_EXT_WCHAR
            }
        };
        (tpe, flags)
    }
}

impl From<(odbc_sys::SqlDataType, Flags)> for OdbcSqlType {
    fn from((tpe, flags): (odbc_sys::SqlDataType, Flags)) -> Self {        

        let _is_unsigned = flags.contains(Flags::UNSIGNED_FLAG);

        // https://docs.oracle.com/cd/E17952_01/Odbc-8.0-en/c-api-data-structures.html
        // https://dev.Odbc.com/doc/dev/Odbc-server/8.0.12/binary__log__types_8h.html
        // https://dev.Odbc.com/doc/internals/en/binary-protocol-value.html
        // https://mariadb.com/kb/en/packet_bindata/
        match tpe {
            odbc_sys::SqlDataType::SQL_EXT_TINYINT=>OdbcSqlType::Tiny,
            odbc_sys::SqlDataType::SQL_SMALLINT => OdbcSqlType::Short,
            odbc_sys::SqlDataType::SQL_INTEGER => OdbcSqlType::Long,
            odbc_sys::SqlDataType::SQL_EXT_BIGINT=>OdbcSqlType::LongLong,
            odbc_sys::SqlDataType::SQL_REAL => OdbcSqlType::Float,            
            odbc_sys::SqlDataType::SQL_FLOAT => OdbcSqlType::Double,            
            odbc_sys::SqlDataType::SQL_DOUBLE => OdbcSqlType::Double,            
            odbc_sys::SqlDataType::SQL_DECIMAL |  odbc_sys::SqlDataType::SQL_NUMERIC => OdbcSqlType::Numeric,
            odbc_sys::SqlDataType::SQL_EXT_BIT => OdbcSqlType::Bit,

            odbc_sys::SqlDataType::SQL_DATETIME => OdbcSqlType::DateTime,
            odbc_sys::SqlDataType::SQL_DATE => OdbcSqlType::Date,           
            odbc_sys::SqlDataType::SQL_TIME | odbc_sys::SqlDataType::SQL_EXT_TIME_OR_INTERVAL => OdbcSqlType::Time,
            odbc_sys::SqlDataType::SQL_TIMESTAMP |  odbc_sys::SqlDataType::SQL_EXT_TIMESTAMP => OdbcSqlType::Timestamp,
            
            odbc_sys::SqlDataType::SQL_EXT_BINARY
            | odbc_sys::SqlDataType::SQL_EXT_VARBINARY       
            | odbc_sys::SqlDataType::SQL_EXT_LONGVARBINARY    
            =>            
            {
                OdbcSqlType::Blob
            },

            // If the binary flag is not set consider everything as string            
            odbc_sys::SqlDataType::SQL_CHAR
            | odbc_sys::SqlDataType::SQL_VARCHAR
            | odbc_sys::SqlDataType::SQL_EXT_WCHAR
            | odbc_sys::SqlDataType::SQL_EXT_WVARCHAR
            | odbc_sys::SqlDataType::SQL_EXT_LONGVARCHAR
            | odbc_sys::SqlDataType::SQL_EXT_WLONGVARCHAR            
            | odbc_sys::SqlDataType::SQL_EXT_GUID
            => OdbcSqlType::String,            

            odbc_sys::SqlDataType::SQL_UNKNOWN_TYPE 
            |  odbc_sys::SqlDataType::SQL_SS_VARIANT
            |  odbc_sys::SqlDataType::SQL_SS_UDT
            |  odbc_sys::SqlDataType::SQL_SS_XML
            |  odbc_sys::SqlDataType::SQL_SS_TABLE
            |  odbc_sys::SqlDataType::SQL_SS_TIME2
            |  odbc_sys::SqlDataType::SQL_SS_TIMESTAMPOFFSET
            => unimplemented!(
                "没有实现。"
            )
                      
           
        }
    }
}

fn known_buffer_size_for_ffi_type(tpe: odbc_sys::SqlDataType) -> Option<usize> {
    use odbc_sys::SqlDataType as t;
    use std::mem::size_of;

    match tpe {        
        t::SQL_INTEGER => Some(4),
        t::SQL_SMALLINT=> Some(2),
        t::SQL_EXT_TINYINT=> Some(1),
        t::SQL_EXT_BIGINT=> Some(8),
        t::SQL_EXT_BIT => Some(1),
        t::SQL_REAL => Some(4),
        t::SQL_FLOAT => Some(8),
        t::SQL_DOUBLE => Some(8),
        t::SQL_DATETIME => Some(size_of::<super::ffi::SQL_TIMESTAMP_STRUCT>()),
        t::SQL_DATE => Some(size_of::<super::ffi::SQL_DATE_STRUCT>()),
        t::SQL_TIME => Some(size_of::<super::ffi::SQL_TIME_STRUCT>()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "bigdecimal")]
    use bigdecimal::FromPrimitive;

    use super::OdbcValue;
    use super::*;
    use crate::deserialize::FromSql;
    use crate::prelude::*;
    use crate::sql_types::*;

    fn to_value<ST, T>(
        bind: &BindData,
    ) -> Result<T, Box<(dyn std::error::Error + Send + Sync + 'static)>>
    where
        T: FromSql<ST, crate::odbc::Odbc> + std::fmt::Debug,
    {
        let meta = (bind.tpe, bind.flags).into();
        dbg!(meta);
        let value = OdbcValue::new(&bind.bytes, meta);
        dbg!(T::from_sql(value))
    }

    #[cfg(feature = "extras")]
    #[test]
    fn check_all_the_types() {
        let conn = crate::test_helpers::connection();

        conn.execute("DROP TABLE IF EXISTS all_odbc_types CASCADE")
            .unwrap();
        conn.execute(
            "CREATE TABLE all_odbc_types (
                    tiny_int TINYINT NOT NULL,
                    small_int SMALLINT NOT NULL,
                    medium_int MEDIUMINT NOT NULL,
                    int_col INTEGER NOT NULL,
                    big_int BIGINT NOT NULL,
                    unsigned_int INTEGER UNSIGNED NOT NULL,
                    zero_fill_int INTEGER ZEROFILL NOT NULL,
                    numeric_col NUMERIC(20,5) NOT NULL,
                    decimal_col DECIMAL(20,5) NOT NULL,
                    float_col FLOAT NOT NULL,
                    double_col DOUBLE NOT NULL,
                    bit_col BIT(8) NOT NULL,
                    date_col DATE NOT NULL,
                    date_time DATETIME NOT NULL,
                    timestamp_col TIMESTAMP NOT NULL,
                    time_col TIME NOT NULL,
                    year_col YEAR NOT NULL,
                    char_col CHAR(30) NOT NULL,
                    varchar_col VARCHAR(30) NOT NULL,
                    binary_col BINARY(30) NOT NULL,
                    varbinary_col VARBINARY(30) NOT NULL,
                    blob_col BLOB NOT NULL,
                    text_col TEXT NOT NULL,
                    enum_col ENUM('red', 'green', 'blue') NOT NULL,
                    set_col SET('one', 'two') NOT NULL,
                    geom GEOMETRY NOT NULL,
                    point_col POINT NOT NULL,
                    linestring_col LINESTRING NOT NULL,
                    polygon_col POLYGON NOT NULL,
                    multipoint_col MULTIPOINT NOT NULL,
                    multilinestring_col MULTILINESTRING NOT NULL,
                    multipolygon_col MULTIPOLYGON NOT NULL,
                    geometry_collection GEOMETRYCOLLECTION NOT NULL,
                    json_col JSON NOT NULL
            )",
        )
        .unwrap();
        conn
            .execute(
                "INSERT INTO all_odbc_types VALUES (
                    0, -- tiny_int
                    1, -- small_int
                    2, -- medium_int
                    3, -- int_col
                    -5, -- big_int
                    42, -- unsigned_int
                    1, -- zero_fill_int
                    -999.999, -- numeric_col,
                    3.14, -- decimal_col,
                    1.23, -- float_col
                    4.5678, -- double_col
                    b'10101010', -- bit_col
                    '1000-01-01', -- date_col
                    '9999-12-31 12:34:45.012345', -- date_time
                    '2020-01-01 10:10:10', -- timestamp_col
                    '23:01:01', -- time_col
                    2020, -- year_col
                    'abc', -- char_col
                    'foo', -- varchar_col
                    'a ', -- binary_col
                    'a ', -- varbinary_col
                    'binary', -- blob_col
                    'some text whatever', -- text_col
                    'red', -- enum_col
                    'one', -- set_col
                    ST_GeomFromText('POINT(1 1)'), -- geom
                    ST_PointFromText('POINT(1 1)'), -- point_col
                    ST_LineStringFromText('LINESTRING(0 0,1 1,2 2)'), -- linestring_col
                    ST_PolygonFromText('POLYGON((0 0,10 0,10 10,0 10,0 0),(5 5,7 5,7 7,5 7, 5 5))'), -- polygon_col
                    ST_MultiPointFromText('MULTIPOINT(0 0,10 10,10 20,20 20)'), -- multipoint_col
                    ST_MultiLineStringFromText('MULTILINESTRING((10 48,10 21,10 0),(16 0,16 23,16 48))'), -- multilinestring_col
                    ST_MultiPolygonFromText('MULTIPOLYGON(((28 26,28 0,84 0,84 42,28 26),(52 18,66 23,73 9,48 6,52 18)),((59 18,67 18,67 13,59 13,59 18)))'), -- multipolygon_col
                    ST_GeomCollFromText('GEOMETRYCOLLECTION(POINT(1 1),LINESTRING(0 0,1 1,2 2,3 3,4 4))'), -- geometry_collection
                    '{\"key1\": \"value1\", \"key2\": \"value2\"}' -- json_col
)",
            )
            .unwrap();

        let mut stmt = conn
            .prepare_query(&crate::sql_query(
                "SELECT
                    tiny_int, small_int, medium_int, int_col,
                    big_int, unsigned_int, zero_fill_int,
                    numeric_col, decimal_col, float_col, double_col, bit_col,
                    date_col, date_time, timestamp_col, time_col, year_col,
                    char_col, varchar_col, binary_col, varbinary_col, blob_col,
                    text_col, enum_col, set_col, ST_AsText(geom), ST_AsText(point_col), ST_AsText(linestring_col),
                    ST_AsText(polygon_col), ST_AsText(multipoint_col), ST_AsText(multilinestring_col),
                    ST_AsText(multipolygon_col), ST_AsText(geometry_collection), json_col
                 FROM all_odbc_types",
            ))
            .unwrap();

        let metadata = stmt.metadata().unwrap();
        let mut output_binds =
            Binds::from_output_types(vec![None; metadata.fields().len()], &metadata);
        stmt.execute_statement(&mut output_binds).unwrap();
        stmt.populate_row_buffers(&mut output_binds).unwrap();

        let results: Vec<(BindData, &_)> = output_binds
            .data
            .into_iter()
            .zip(metadata.fields())
            .collect::<Vec<_>>();

        macro_rules! matches {
            ($expression:expr, $( $pattern:pat )|+ $( if $guard: expr )?) => {
                match $expression {
                    $( $pattern )|+ $( if $guard )? => true,
                    _ => false
                }
            }
        }

        let tiny_int_col = &results[0].0;
        assert_eq!(tiny_int_col.tpe, ffi::enum_field_types::ODBC_TYPE_TINY);
        assert!(tiny_int_col.flags.contains(Flags::NUM_FLAG));
        assert!(!tiny_int_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert!(matches!(to_value::<TinyInt, i8>(tiny_int_col), Ok(0)));

        let small_int_col = &results[1].0;
        assert_eq!(small_int_col.tpe, ffi::enum_field_types::ODBC_TYPE_SHORT);
        assert!(small_int_col.flags.contains(Flags::NUM_FLAG));
        assert!(!small_int_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert!(matches!(to_value::<SmallInt, i16>(small_int_col), Ok(1)));

        let medium_int_col = &results[2].0;
        assert_eq!(medium_int_col.tpe, ffi::enum_field_types::ODBC_TYPE_INT24);
        assert!(medium_int_col.flags.contains(Flags::NUM_FLAG));
        assert!(!medium_int_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert!(matches!(to_value::<Integer, i32>(medium_int_col), Ok(2)));

        let int_col = &results[3].0;
        assert_eq!(int_col.tpe, ffi::enum_field_types::ODBC_TYPE_LONG);
        assert!(int_col.flags.contains(Flags::NUM_FLAG));
        assert!(!int_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert!(matches!(to_value::<Integer, i32>(int_col), Ok(3)));

        let big_int_col = &results[4].0;
        assert_eq!(big_int_col.tpe, ffi::enum_field_types::ODBC_TYPE_LONGLONG);
        assert!(big_int_col.flags.contains(Flags::NUM_FLAG));
        assert!(!big_int_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert!(matches!(to_value::<TinyInt, i8>(big_int_col), Ok(-5)));

        let unsigned_int_col = &results[5].0;
        assert_eq!(unsigned_int_col.tpe, ffi::enum_field_types::ODBC_TYPE_LONG);
        assert!(unsigned_int_col.flags.contains(Flags::NUM_FLAG));
        assert!(unsigned_int_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert!(matches!(
            to_value::<Unsigned<Integer>, u32>(unsigned_int_col),
            Ok(42)
        ));

        let zero_fill_int_col = &results[6].0;
        assert_eq!(
            zero_fill_int_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_LONG
        );
        assert!(zero_fill_int_col.flags.contains(Flags::NUM_FLAG));
        assert!(zero_fill_int_col.flags.contains(Flags::ZEROFILL_FLAG));
        assert!(matches!(to_value::<Integer, i32>(zero_fill_int_col), Ok(1)));

        let numeric_col = &results[7].0;
        assert_eq!(
            numeric_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_NEWDECIMAL
        );
        assert!(numeric_col.flags.contains(Flags::NUM_FLAG));
        assert!(!numeric_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert_eq!(
            to_value::<Numeric, bigdecimal::BigDecimal>(numeric_col).unwrap(),
            bigdecimal::BigDecimal::from_f32(-999.999).unwrap()
        );

        let decimal_col = &results[8].0;
        assert_eq!(
            decimal_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_NEWDECIMAL
        );
        assert!(decimal_col.flags.contains(Flags::NUM_FLAG));
        assert!(!decimal_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert_eq!(
            to_value::<Numeric, bigdecimal::BigDecimal>(decimal_col).unwrap(),
            bigdecimal::BigDecimal::from_f32(3.14).unwrap()
        );

        let float_col = &results[9].0;
        assert_eq!(float_col.tpe, ffi::enum_field_types::ODBC_TYPE_FLOAT);
        assert!(float_col.flags.contains(Flags::NUM_FLAG));
        assert!(!float_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert_eq!(to_value::<Float, f32>(float_col).unwrap(), 1.23);

        let double_col = &results[10].0;
        assert_eq!(double_col.tpe, ffi::enum_field_types::ODBC_TYPE_DOUBLE);
        assert!(double_col.flags.contains(Flags::NUM_FLAG));
        assert!(!double_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert_eq!(to_value::<Double, f64>(double_col).unwrap(), 4.5678);

        let bit_col = &results[11].0;
        assert_eq!(bit_col.tpe, ffi::enum_field_types::ODBC_TYPE_BIT);
        assert!(!bit_col.flags.contains(Flags::NUM_FLAG));
        assert!(bit_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert!(!bit_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Blob, Vec<u8>>(bit_col).unwrap(), vec![170]);

        let date_col = &results[12].0;
        assert_eq!(date_col.tpe, ffi::enum_field_types::ODBC_TYPE_DATE);
        assert!(!date_col.flags.contains(Flags::NUM_FLAG));
        assert_eq!(
            to_value::<Date, chrono::NaiveDate>(date_col).unwrap(),
            chrono::NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
        );

        let date_time_col = &results[13].0;
        assert_eq!(
            date_time_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_DATETIME
        );
        assert!(!date_time_col.flags.contains(Flags::NUM_FLAG));
        assert_eq!(
            to_value::<Datetime, chrono::NaiveDateTime>(date_time_col).unwrap(),
            chrono::NaiveDateTime::parse_from_str("9999-12-31 12:34:45", "%Y-%m-%d %H:%M:%S")
                .unwrap()
        );

        let timestamp_col = &results[14].0;
        assert_eq!(
            timestamp_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_TIMESTAMP
        );
        assert!(!timestamp_col.flags.contains(Flags::NUM_FLAG));
        assert_eq!(
            to_value::<Datetime, chrono::NaiveDateTime>(timestamp_col).unwrap(),
            chrono::NaiveDateTime::parse_from_str("2020-01-01 10:10:10", "%Y-%m-%d %H:%M:%S")
                .unwrap()
        );

        let time_col = &results[15].0;
        assert_eq!(time_col.tpe, ffi::enum_field_types::ODBC_TYPE_TIME);
        assert!(!time_col.flags.contains(Flags::NUM_FLAG));
        assert_eq!(
            to_value::<Time, chrono::NaiveTime>(time_col).unwrap(),
            chrono::NaiveTime::from_hms(23, 01, 01)
        );

        let year_col = &results[16].0;
        assert_eq!(year_col.tpe, ffi::enum_field_types::ODBC_TYPE_YEAR);
        assert!(year_col.flags.contains(Flags::NUM_FLAG));
        assert!(year_col.flags.contains(Flags::UNSIGNED_FLAG));
        assert!(matches!(to_value::<SmallInt, i16>(year_col), Ok(2020)));

        let char_col = &results[17].0;
        assert_eq!(char_col.tpe, ffi::enum_field_types::ODBC_TYPE_STRING);
        assert!(!char_col.flags.contains(Flags::NUM_FLAG));
        assert!(!char_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!char_col.flags.contains(Flags::SET_FLAG));
        assert!(!char_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!char_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(char_col).unwrap(), "abc");

        let varchar_col = &results[18].0;
        assert_eq!(
            varchar_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_VAR_STRING
        );
        assert!(!varchar_col.flags.contains(Flags::NUM_FLAG));
        assert!(!varchar_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!varchar_col.flags.contains(Flags::SET_FLAG));
        assert!(!varchar_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!varchar_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(varchar_col).unwrap(), "foo");

        let binary_col = &results[19].0;
        assert_eq!(binary_col.tpe, ffi::enum_field_types::ODBC_TYPE_STRING);
        assert!(!binary_col.flags.contains(Flags::NUM_FLAG));
        assert!(!binary_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!binary_col.flags.contains(Flags::SET_FLAG));
        assert!(!binary_col.flags.contains(Flags::ENUM_FLAG));
        assert!(binary_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Blob, Vec<u8>>(binary_col).unwrap(),
            b"a \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
        );

        let varbinary_col = &results[20].0;
        assert_eq!(
            varbinary_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_VAR_STRING
        );
        assert!(!varbinary_col.flags.contains(Flags::NUM_FLAG));
        assert!(!varbinary_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!varbinary_col.flags.contains(Flags::SET_FLAG));
        assert!(!varbinary_col.flags.contains(Flags::ENUM_FLAG));
        assert!(varbinary_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Blob, Vec<u8>>(varbinary_col).unwrap(), b"a ");

        let blob_col = &results[21].0;
        assert_eq!(blob_col.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!blob_col.flags.contains(Flags::NUM_FLAG));
        assert!(blob_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!blob_col.flags.contains(Flags::SET_FLAG));
        assert!(!blob_col.flags.contains(Flags::ENUM_FLAG));
        assert!(blob_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Blob, Vec<u8>>(blob_col).unwrap(), b"binary");

        let text_col = &results[22].0;
        assert_eq!(text_col.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!text_col.flags.contains(Flags::NUM_FLAG));
        assert!(text_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!text_col.flags.contains(Flags::SET_FLAG));
        assert!(!text_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!text_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(text_col).unwrap(),
            "some text whatever"
        );

        let enum_col = &results[23].0;
        assert_eq!(enum_col.tpe, ffi::enum_field_types::ODBC_TYPE_STRING);
        assert!(!enum_col.flags.contains(Flags::NUM_FLAG));
        assert!(!enum_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!enum_col.flags.contains(Flags::SET_FLAG));
        assert!(enum_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!enum_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(enum_col).unwrap(), "red");

        let set_col = &results[24].0;
        assert_eq!(set_col.tpe, ffi::enum_field_types::ODBC_TYPE_STRING);
        assert!(!set_col.flags.contains(Flags::NUM_FLAG));
        assert!(!set_col.flags.contains(Flags::BLOB_FLAG));
        assert!(set_col.flags.contains(Flags::SET_FLAG));
        assert!(!set_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!set_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(set_col).unwrap(), "one");

        let geom = &results[25].0;
        assert_eq!(geom.tpe, ffi::enum_field_types::ODBC_TYPE_LONG_BLOB);
        assert!(!geom.flags.contains(Flags::NUM_FLAG));
        assert!(!geom.flags.contains(Flags::BLOB_FLAG));
        assert!(!geom.flags.contains(Flags::SET_FLAG));
        assert!(!geom.flags.contains(Flags::ENUM_FLAG));
        assert!(!geom.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(geom).unwrap(), "POINT(1 1)");

        let point_col = &results[26].0;
        assert_eq!(point_col.tpe, ffi::enum_field_types::ODBC_TYPE_LONG_BLOB);
        assert!(!point_col.flags.contains(Flags::NUM_FLAG));
        assert!(!point_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!point_col.flags.contains(Flags::SET_FLAG));
        assert!(!point_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!point_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(point_col).unwrap(), "POINT(1 1)");

        let linestring_col = &results[27].0;
        assert_eq!(
            linestring_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_LONG_BLOB
        );
        assert!(!linestring_col.flags.contains(Flags::NUM_FLAG));
        assert!(!linestring_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!linestring_col.flags.contains(Flags::SET_FLAG));
        assert!(!linestring_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!linestring_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(linestring_col).unwrap(),
            "LINESTRING(0 0,1 1,2 2)"
        );

        let polygon_col = &results[28].0;
        assert_eq!(polygon_col.tpe, ffi::enum_field_types::ODBC_TYPE_LONG_BLOB);
        assert!(!polygon_col.flags.contains(Flags::NUM_FLAG));
        assert!(!polygon_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!polygon_col.flags.contains(Flags::SET_FLAG));
        assert!(!polygon_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!polygon_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(polygon_col).unwrap(),
            "POLYGON((0 0,10 0,10 10,0 10,0 0),(5 5,7 5,7 7,5 7,5 5))"
        );

        let multipoint_col = &results[29].0;
        assert_eq!(
            multipoint_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_LONG_BLOB
        );
        assert!(!multipoint_col.flags.contains(Flags::NUM_FLAG));
        assert!(!multipoint_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!multipoint_col.flags.contains(Flags::SET_FLAG));
        assert!(!multipoint_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!multipoint_col.flags.contains(Flags::BINARY_FLAG));
        // older Odbc and mariadb versions get back another encoding here
        // we test for both as there seems to be no clear pattern when one or
        // the other is returned
        let multipoint_res = to_value::<Text, String>(multipoint_col).unwrap();
        assert!(
            multipoint_res == "MULTIPOINT((0 0),(10 10),(10 20),(20 20))"
                || multipoint_res == "MULTIPOINT(0 0,10 10,10 20,20 20)"
        );

        let multilinestring_col = &results[30].0;
        assert_eq!(
            multilinestring_col.tpe,
            ffi::enum_field_types::ODBC_TYPE_LONG_BLOB
        );
        assert!(!multilinestring_col.flags.contains(Flags::NUM_FLAG));
        assert!(!multilinestring_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!multilinestring_col.flags.contains(Flags::SET_FLAG));
        assert!(!multilinestring_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!multilinestring_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(multilinestring_col).unwrap(),
            "MULTILINESTRING((10 48,10 21,10 0),(16 0,16 23,16 48))"
        );

        let polygon_col = &results[31].0;
        assert_eq!(polygon_col.tpe, ffi::enum_field_types::ODBC_TYPE_LONG_BLOB);
        assert!(!polygon_col.flags.contains(Flags::NUM_FLAG));
        assert!(!polygon_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!polygon_col.flags.contains(Flags::SET_FLAG));
        assert!(!polygon_col.flags.contains(Flags::ENUM_FLAG));
        assert!(!polygon_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(polygon_col).unwrap(),
            "MULTIPOLYGON(((28 26,28 0,84 0,84 42,28 26),(52 18,66 23,73 9,48 6,52 18)),((59 18,67 18,67 13,59 13,59 18)))"
        );

        let geometry_collection = &results[32].0;
        assert_eq!(
            geometry_collection.tpe,
            ffi::enum_field_types::ODBC_TYPE_LONG_BLOB
        );
        assert!(!geometry_collection.flags.contains(Flags::NUM_FLAG));
        assert!(!geometry_collection.flags.contains(Flags::BLOB_FLAG));
        assert!(!geometry_collection.flags.contains(Flags::SET_FLAG));
        assert!(!geometry_collection.flags.contains(Flags::ENUM_FLAG));
        assert!(!geometry_collection.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(geometry_collection).unwrap(),
            "GEOMETRYCOLLECTION(POINT(1 1),LINESTRING(0 0,1 1,2 2,3 3,4 4))"
        );

        let json_col = &results[33].0;
        // mariadb >= 10.2 and Odbc >=8.0 are supporting a json type
        // from those mariadb >= 10.3 and Odbc >= 8.0 are reporting
        // json here, so we assert that we get back json
        // mariadb 10.5 returns again blob
        assert!(
            json_col.tpe == ffi::enum_field_types::ODBC_TYPE_JSON
                || json_col.tpe == ffi::enum_field_types::ODBC_TYPE_BLOB
        );
        assert!(!json_col.flags.contains(Flags::NUM_FLAG));
        assert!(json_col.flags.contains(Flags::BLOB_FLAG));
        assert!(!json_col.flags.contains(Flags::SET_FLAG));
        assert!(!json_col.flags.contains(Flags::ENUM_FLAG));
        assert!(json_col.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(json_col).unwrap(),
            "{\"key1\": \"value1\", \"key2\": \"value2\"}"
        );
    }

    fn query_single_table(
        query: &'static str,
        conn: &OdbcConnection,
        bind_tpe: impl Into<(ffi::enum_field_types, Flags)>,
    ) -> BindData {
        let mut stmt: Statement = conn.raw_connection.prepare(query).unwrap();

        let bind = BindData::for_output(bind_tpe.into());

        let mut binds = Binds { data: vec![bind] };

        stmt.execute_statement(&mut binds).unwrap();
        stmt.populate_row_buffers(&mut binds).unwrap();

        binds.data.remove(0)
    }

    fn input_bind(
        query: &'static str,
        conn: &OdbcConnection,
        id: i32,
        (field, tpe): (Vec<u8>, impl Into<(ffi::enum_field_types, Flags)>),
    ) {
        let mut stmt = conn.raw_connection.prepare(query).unwrap();
        let length = field.len() as _;
        let (tpe, flags) = tpe.into();

        let field_bind = BindData {
            tpe,
            bytes: field,
            length,
            flags,
            is_null: 0,
            is_truncated: None,
        };

        let bytes = id.to_be_bytes().to_vec();
        let length = bytes.len() as _;

        let id_bind = BindData {
            tpe: ffi::enum_field_types::ODBC_TYPE_LONG,
            bytes,
            length,
            flags: Flags::empty(),
            is_null: 0,
            is_truncated: None,
        };

        let binds = Binds {
            data: vec![id_bind, field_bind],
        };
        stmt.input_bind(binds).unwrap();
        stmt.did_an_error_occur().unwrap();
        unsafe {
            stmt.execute().unwrap();
        }
    }

    #[test]
    fn check_json_bind() {
        let conn: OdbcConnection = crate::test_helpers::connection();

        table! {
            json_test {
                id -> Integer,
                json_field -> Text,
            }
        }

        conn.execute("DROP TABLE IF EXISTS json_test CASCADE")
            .unwrap();

        conn.execute("CREATE TABLE json_test(id INTEGER PRIMARY KEY, json_field JSON NOT NULL)")
            .unwrap();

        conn.execute("INSERT INTO json_test(id, json_field) VALUES (1, '{\"key1\": \"value1\", \"key2\": \"value2\"}')").unwrap();

        let json_col_as_json = query_single_table(
            "SELECT json_field FROM json_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_JSON, Flags::empty()),
        );

        assert_eq!(json_col_as_json.tpe, ffi::enum_field_types::ODBC_TYPE_JSON);
        assert!(!json_col_as_json.flags.contains(Flags::NUM_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::BLOB_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::SET_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::ENUM_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(&json_col_as_json).unwrap(),
            "{\"key1\": \"value1\", \"key2\": \"value2\"}"
        );

        let json_col_as_text = query_single_table(
            "SELECT json_field FROM json_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::empty()),
        );

        assert_eq!(json_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!json_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(&json_col_as_text).unwrap(),
            "{\"key1\": \"value1\", \"key2\": \"value2\"}"
        );
        assert_eq!(json_col_as_json.bytes, json_col_as_text.bytes);

        conn.execute("DELETE FROM json_test").unwrap();

        input_bind(
            "INSERT INTO json_test(id, json_field) VALUES (?, ?)",
            &conn,
            41,
            (
                b"{\"abc\": 42}".to_vec(),
                OdbcSqlType::String,
                //                (ffi::enum_field_types::ODBC_TYPE_JSON, Flags::empty()),
            ),
        );

        let json_col_as_json = query_single_table(
            "SELECT json_field FROM json_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_JSON, Flags::empty()),
        );

        assert_eq!(json_col_as_json.tpe, ffi::enum_field_types::ODBC_TYPE_JSON);
        assert!(!json_col_as_json.flags.contains(Flags::NUM_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::BLOB_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::SET_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::ENUM_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(&json_col_as_json).unwrap(),
            "{\"abc\": 42}"
        );

        let json_col_as_text = query_single_table(
            "SELECT json_field FROM json_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::empty()),
        );

        assert_eq!(json_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!json_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(&json_col_as_text).unwrap(),
            "{\"abc\": 42}"
        );
        assert_eq!(json_col_as_json.bytes, json_col_as_text.bytes);

        conn.execute("DELETE FROM json_test").unwrap();

        input_bind(
            "INSERT INTO json_test(id, json_field) VALUES (?, ?)",
            &conn,
            41,
            (b"{\"abca\": 42}".to_vec(), OdbcSqlType::String),
        );

        let json_col_as_json = query_single_table(
            "SELECT json_field FROM json_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_JSON, Flags::empty()),
        );

        assert_eq!(json_col_as_json.tpe, ffi::enum_field_types::ODBC_TYPE_JSON);
        assert!(!json_col_as_json.flags.contains(Flags::NUM_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::BLOB_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::SET_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::ENUM_FLAG));
        assert!(!json_col_as_json.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(&json_col_as_json).unwrap(),
            "{\"abca\": 42}"
        );

        let json_col_as_text = query_single_table(
            "SELECT json_field FROM json_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::empty()),
        );

        assert_eq!(json_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!json_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!json_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(&json_col_as_text).unwrap(),
            "{\"abca\": 42}"
        );
        assert_eq!(json_col_as_json.bytes, json_col_as_text.bytes);
    }

    #[test]
    fn check_enum_bind() {
        let conn: OdbcConnection = crate::test_helpers::connection();

        conn.execute("DROP TABLE IF EXISTS enum_test CASCADE")
            .unwrap();

        conn.execute("CREATE TABLE enum_test(id INTEGER PRIMARY KEY, enum_field ENUM('red', 'green', 'blue') NOT NULL)")
            .unwrap();

        conn.execute("INSERT INTO enum_test(id, enum_field) VALUES (1, 'green')")
            .unwrap();

        let enum_col_as_enum: BindData =
            query_single_table("SELECT enum_field FROM enum_test", &conn, OdbcSqlType::Enum);

        assert_eq!(
            enum_col_as_enum.tpe,
            ffi::enum_field_types::ODBC_TYPE_STRING
        );
        assert!(!enum_col_as_enum.flags.contains(Flags::NUM_FLAG));
        assert!(!enum_col_as_enum.flags.contains(Flags::BLOB_FLAG));
        assert!(!enum_col_as_enum.flags.contains(Flags::SET_FLAG));
        assert!(enum_col_as_enum.flags.contains(Flags::ENUM_FLAG));
        assert!(!enum_col_as_enum.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(&enum_col_as_enum).unwrap(),
            "green"
        );

        for tpe in &[
            ffi::enum_field_types::ODBC_TYPE_BLOB,
            ffi::enum_field_types::ODBC_TYPE_VAR_STRING,
            ffi::enum_field_types::ODBC_TYPE_TINY_BLOB,
            ffi::enum_field_types::ODBC_TYPE_MEDIUM_BLOB,
            ffi::enum_field_types::ODBC_TYPE_LONG_BLOB,
        ] {
            let enum_col_as_text = query_single_table(
                "SELECT enum_field FROM enum_test",
                &conn,
                (*tpe, Flags::ENUM_FLAG),
            );

            assert_eq!(enum_col_as_text.tpe, *tpe);
            assert!(!enum_col_as_text.flags.contains(Flags::NUM_FLAG));
            assert!(!enum_col_as_text.flags.contains(Flags::BLOB_FLAG));
            assert!(!enum_col_as_text.flags.contains(Flags::SET_FLAG));
            assert!(enum_col_as_text.flags.contains(Flags::ENUM_FLAG));
            assert!(!enum_col_as_text.flags.contains(Flags::BINARY_FLAG));
            assert_eq!(
                to_value::<Text, String>(&enum_col_as_text).unwrap(),
                "green"
            );
            assert_eq!(enum_col_as_enum.bytes, enum_col_as_text.bytes);
        }

        let enum_col_as_text = query_single_table(
            "SELECT enum_field FROM enum_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::empty()),
        );

        assert_eq!(enum_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!enum_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(
            to_value::<Text, String>(&enum_col_as_text).unwrap(),
            "green"
        );
        assert_eq!(enum_col_as_enum.bytes, enum_col_as_text.bytes);

        conn.execute("DELETE FROM enum_test").unwrap();

        input_bind(
            "INSERT INTO enum_test(id, enum_field) VALUES (?, ?)",
            &conn,
            41,
            (b"blue".to_vec(), OdbcSqlType::Enum),
        );

        let enum_col_as_enum =
            query_single_table("SELECT enum_field FROM enum_test", &conn, OdbcSqlType::Enum);

        assert_eq!(
            enum_col_as_enum.tpe,
            ffi::enum_field_types::ODBC_TYPE_STRING
        );
        assert!(!enum_col_as_enum.flags.contains(Flags::NUM_FLAG));
        assert!(!enum_col_as_enum.flags.contains(Flags::BLOB_FLAG));
        assert!(!enum_col_as_enum.flags.contains(Flags::SET_FLAG));
        assert!(enum_col_as_enum.flags.contains(Flags::ENUM_FLAG));
        assert!(!enum_col_as_enum.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&enum_col_as_enum).unwrap(), "blue");

        let enum_col_as_text = query_single_table(
            "SELECT enum_field FROM enum_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::ENUM_FLAG),
        );

        assert_eq!(enum_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!enum_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(enum_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&enum_col_as_text).unwrap(), "blue");
        assert_eq!(enum_col_as_enum.bytes, enum_col_as_text.bytes);

        let enum_col_as_text = query_single_table(
            "SELECT enum_field FROM enum_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::ENUM_FLAG),
        );

        assert_eq!(enum_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!enum_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(enum_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&enum_col_as_text).unwrap(), "blue");
        assert_eq!(enum_col_as_enum.bytes, enum_col_as_text.bytes);

        conn.execute("DELETE FROM enum_test").unwrap();

        input_bind(
            "INSERT INTO enum_test(id, enum_field) VALUES (?, ?)",
            &conn,
            41,
            (
                b"red".to_vec(),
                (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::ENUM_FLAG),
            ),
        );

        let enum_col_as_enum =
            query_single_table("SELECT enum_field FROM enum_test", &conn, OdbcSqlType::Enum);

        assert_eq!(
            enum_col_as_enum.tpe,
            ffi::enum_field_types::ODBC_TYPE_STRING
        );
        assert!(!enum_col_as_enum.flags.contains(Flags::NUM_FLAG));
        assert!(!enum_col_as_enum.flags.contains(Flags::BLOB_FLAG));
        assert!(!enum_col_as_enum.flags.contains(Flags::SET_FLAG));
        assert!(enum_col_as_enum.flags.contains(Flags::ENUM_FLAG));
        assert!(!enum_col_as_enum.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&enum_col_as_enum).unwrap(), "red");

        let enum_col_as_text = query_single_table(
            "SELECT enum_field FROM enum_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::ENUM_FLAG),
        );

        assert_eq!(enum_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!enum_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(enum_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!enum_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&enum_col_as_text).unwrap(), "red");
        assert_eq!(enum_col_as_enum.bytes, enum_col_as_text.bytes);
    }

    #[test]
    fn check_set_bind() {
        let conn: OdbcConnection = crate::test_helpers::connection();

        conn.execute("DROP TABLE IF EXISTS set_test CASCADE")
            .unwrap();

        conn.execute("CREATE TABLE set_test(id INTEGER PRIMARY KEY, set_field SET('red', 'green', 'blue') NOT NULL)")
            .unwrap();

        conn.execute("INSERT INTO set_test(id, set_field) VALUES (1, 'green')")
            .unwrap();

        let set_col_as_set: BindData =
            query_single_table("SELECT set_field FROM set_test", &conn, OdbcSqlType::Set);

        assert_eq!(set_col_as_set.tpe, ffi::enum_field_types::ODBC_TYPE_STRING);
        assert!(!set_col_as_set.flags.contains(Flags::NUM_FLAG));
        assert!(!set_col_as_set.flags.contains(Flags::BLOB_FLAG));
        assert!(set_col_as_set.flags.contains(Flags::SET_FLAG));
        assert!(!set_col_as_set.flags.contains(Flags::ENUM_FLAG));
        assert!(!set_col_as_set.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&set_col_as_set).unwrap(), "green");

        for tpe in &[
            ffi::enum_field_types::ODBC_TYPE_BLOB,
            ffi::enum_field_types::ODBC_TYPE_VAR_STRING,
            ffi::enum_field_types::ODBC_TYPE_TINY_BLOB,
            ffi::enum_field_types::ODBC_TYPE_MEDIUM_BLOB,
            ffi::enum_field_types::ODBC_TYPE_LONG_BLOB,
        ] {
            let set_col_as_text = query_single_table(
                "SELECT set_field FROM set_test",
                &conn,
                (*tpe, Flags::SET_FLAG),
            );

            assert_eq!(set_col_as_text.tpe, *tpe);
            assert!(!set_col_as_text.flags.contains(Flags::NUM_FLAG));
            assert!(!set_col_as_text.flags.contains(Flags::BLOB_FLAG));
            assert!(set_col_as_text.flags.contains(Flags::SET_FLAG));
            assert!(!set_col_as_text.flags.contains(Flags::ENUM_FLAG));
            assert!(!set_col_as_text.flags.contains(Flags::BINARY_FLAG));
            assert_eq!(to_value::<Text, String>(&set_col_as_text).unwrap(), "green");
            assert_eq!(set_col_as_set.bytes, set_col_as_text.bytes);
        }
        let set_col_as_text = query_single_table(
            "SELECT set_field FROM set_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::empty()),
        );

        assert_eq!(set_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!set_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&set_col_as_text).unwrap(), "green");
        assert_eq!(set_col_as_set.bytes, set_col_as_text.bytes);

        conn.execute("DELETE FROM set_test").unwrap();

        input_bind(
            "INSERT INTO set_test(id, set_field) VALUES (?, ?)",
            &conn,
            41,
            (b"blue".to_vec(), OdbcSqlType::Set),
        );

        let set_col_as_set =
            query_single_table("SELECT set_field FROM set_test", &conn, OdbcSqlType::Set);

        assert_eq!(set_col_as_set.tpe, ffi::enum_field_types::ODBC_TYPE_STRING);
        assert!(!set_col_as_set.flags.contains(Flags::NUM_FLAG));
        assert!(!set_col_as_set.flags.contains(Flags::BLOB_FLAG));
        assert!(set_col_as_set.flags.contains(Flags::SET_FLAG));
        assert!(!set_col_as_set.flags.contains(Flags::ENUM_FLAG));
        assert!(!set_col_as_set.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&set_col_as_set).unwrap(), "blue");

        let set_col_as_text = query_single_table(
            "SELECT set_field FROM set_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::SET_FLAG),
        );

        assert_eq!(set_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!set_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(set_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&set_col_as_text).unwrap(), "blue");
        assert_eq!(set_col_as_set.bytes, set_col_as_text.bytes);

        conn.execute("DELETE FROM set_test").unwrap();

        input_bind(
            "INSERT INTO set_test(id, set_field) VALUES (?, ?)",
            &conn,
            41,
            (b"red".to_vec(), OdbcSqlType::String),
        );

        let set_col_as_set =
            query_single_table("SELECT set_field FROM set_test", &conn, OdbcSqlType::Set);

        assert_eq!(set_col_as_set.tpe, ffi::enum_field_types::ODBC_TYPE_STRING);
        assert!(!set_col_as_set.flags.contains(Flags::NUM_FLAG));
        assert!(!set_col_as_set.flags.contains(Flags::BLOB_FLAG));
        assert!(set_col_as_set.flags.contains(Flags::SET_FLAG));
        assert!(!set_col_as_set.flags.contains(Flags::ENUM_FLAG));
        assert!(!set_col_as_set.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&set_col_as_set).unwrap(), "red");

        let set_col_as_text = query_single_table(
            "SELECT set_field FROM set_test",
            &conn,
            (ffi::enum_field_types::ODBC_TYPE_BLOB, Flags::SET_FLAG),
        );

        assert_eq!(set_col_as_text.tpe, ffi::enum_field_types::ODBC_TYPE_BLOB);
        assert!(!set_col_as_text.flags.contains(Flags::NUM_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::BLOB_FLAG));
        assert!(set_col_as_text.flags.contains(Flags::SET_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::ENUM_FLAG));
        assert!(!set_col_as_text.flags.contains(Flags::BINARY_FLAG));
        assert_eq!(to_value::<Text, String>(&set_col_as_text).unwrap(), "red");
        assert_eq!(set_col_as_set.bytes, set_col_as_text.bytes);
    }
}
