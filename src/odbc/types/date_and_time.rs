use chrono::*;
use std::io::Write;
use std::{mem, slice};

use crate::odbc::connection::ffi::{SQL_TIMESTAMP_STRUCT, SQL_DATE_STRUCT, SQL_TIME_STRUCT};
use diesel::deserialize::{self, FromSql};
use crate::odbc::{Odbc, OdbcValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::{Date, Datetime, Time, Timestamp};

macro_rules! odbc_datetime_impls {
    ($ty:ty) => {
        impl ToSql<$ty, Odbc> for SQL_TIMESTAMP_STRUCT {
            fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
                let bytes = unsafe {
                    let bytes_ptr = self as *const SQL_TIMESTAMP_STRUCT as *const u8;
                    slice::from_raw_parts(bytes_ptr, mem::size_of::<SQL_TIMESTAMP_STRUCT>())
                };
                out.write_all(bytes)?;
                Ok(IsNull::No)
            }
        }

        impl FromSql<$ty, Odbc> for SQL_TIMESTAMP_STRUCT {
            fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
                value.datetime_value()
            }
        }
    };
}

macro_rules! odbc_date_impls {
    ($ty:ty) => {
        impl ToSql<$ty, Odbc> for SQL_DATE_STRUCT {
            fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
                let bytes = unsafe {
                    let bytes_ptr = self as *const SQL_DATE_STRUCT as *const u8;
                    slice::from_raw_parts(bytes_ptr, mem::size_of::<SQL_DATE_STRUCT>())
                };
                out.write_all(bytes)?;
                Ok(IsNull::No)
            }
        }

        impl FromSql<$ty, Odbc> for SQL_DATE_STRUCT {
            fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
                value.date_value()
            }
        }
    };
}

macro_rules! odbc_time_impls {
    ($ty:ty) => {
        impl ToSql<$ty, Odbc> for SQL_TIME_STRUCT {
            fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
                let bytes = unsafe {
                    let bytes_ptr = self as *const SQL_TIME_STRUCT as *const u8;
                    slice::from_raw_parts(bytes_ptr, mem::size_of::<SQL_TIME_STRUCT>())
                };
                out.write_all(bytes)?;
                Ok(IsNull::No)
            }
        }

        impl FromSql<$ty, Odbc> for SQL_TIME_STRUCT {
            fn from_sql(value: OdbcValue<'_>) -> deserialize::Result<Self> {
                value.time_value()
            }
        }
    };
}

odbc_datetime_impls!(Datetime);
odbc_datetime_impls!(Timestamp);
odbc_time_impls!(Time);
odbc_date_impls!(Date);

impl ToSql<Datetime, Odbc> for NaiveDateTime {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
        <NaiveDateTime as ToSql<Timestamp, Odbc>>::to_sql(self, out)
    }
}

impl FromSql<Datetime, Odbc> for NaiveDateTime {
    fn from_sql(bytes: OdbcValue<'_>) -> deserialize::Result<Self> {
        <NaiveDateTime as FromSql<Timestamp, Odbc>>::from_sql(bytes)
    }
}

impl ToSql<Timestamp, Odbc> for NaiveDateTime {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {      
        let odbc_time = SQL_TIMESTAMP_STRUCT {
            year: self.year() as i16,
            month: self.month() as u16,
            day: self.day() as u16,
            hour: self.hour() as u16,
            minute: self.minute() as u16,
            second: self.second() as u16,
            fraction: self.timestamp_subsec_micros(),            
        };

        <SQL_TIMESTAMP_STRUCT as ToSql<Timestamp, Odbc>>::to_sql(&odbc_time, out)
    }
}

impl FromSql<Timestamp, Odbc> for NaiveDateTime {
    fn from_sql(bytes: OdbcValue<'_>) -> deserialize::Result<Self> {
        let odbc_time = <SQL_TIMESTAMP_STRUCT as FromSql<Timestamp, Odbc>>::from_sql(bytes)?;

        let datetime = NaiveDate::from_ymd_opt(
            odbc_time.year as i32,
            odbc_time.month as u32,
            odbc_time.day as u32,            
        )
        .and_then(|v| {
            v.and_hms_micro_opt(
                odbc_time.hour as u32,
                odbc_time.minute as u32,
                odbc_time.second as u32,
                odbc_time.fraction as u32,
            )
        })
        .unwrap_or(MIN_DATETIME.naive_utc());
        Ok(datetime)
        // .ok_or_else(|| format!("Cannot parse this date: {:?}", odbc_time).into())
    }
}

impl ToSql<Time, Odbc> for NaiveTime {
    fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, Odbc>) -> serialize::Result {
        let odbc_time = SQL_TIME_STRUCT {
            hour: self.hour() as u16,
            minute: self.minute() as u16,
            second: self.second() as u16,
        };

        <SQL_TIME_STRUCT as ToSql<Time, Odbc>>::to_sql(&odbc_time, out)
    }
}

impl FromSql<Time, Odbc> for NaiveTime {
    fn from_sql(bytes: OdbcValue<'_>) -> deserialize::Result<Self> {
        let odbc_time = <SQL_TIME_STRUCT as FromSql<Time, Odbc>>::from_sql(bytes)?;
        let time = NaiveTime::from_hms_opt(
            odbc_time.hour as u32,
            odbc_time.minute as u32,
            odbc_time.second as u32,
        )
        .unwrap_or(MIN_DATETIME.time());
        Ok(time)
        // .ok_or_else(|| format!("Unable to convert {:?} to chrono", odbc_time).into())
    }
}

impl ToSql<Date, Odbc> for NaiveDate {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Odbc>) -> serialize::Result {
        let odbc_time = SQL_DATE_STRUCT {
            year: self.year() as i16,
            month: self.month() as u16,
            day: self.day() as u16,
        };

        <SQL_DATE_STRUCT as ToSql<Date, Odbc>>::to_sql(&odbc_time, out)
    }
}

impl FromSql<Date, Odbc> for NaiveDate {
    fn from_sql(bytes: OdbcValue<'_>) -> deserialize::Result<Self> {
        // println!("bytes:{:?}", bytes);
        
        let odbc_time = <SQL_DATE_STRUCT as FromSql<Date, Odbc>>::from_sql(bytes)?;
        let date = NaiveDate::from_ymd_opt(
            odbc_time.year as i32,
            odbc_time.month as u32,
            odbc_time.day as u32,
        )
        .unwrap_or_else(||{
            let datetime = MIN_DATETIME.naive_utc();
            NaiveDate::from_ymd(datetime.year(), datetime.month(), datetime.day())
        });       
        // .ok_or_else(|| format!("Unable to convert {:?} to chrono", odbc_time).into());
        Ok(date)
    }
}

#[cfg(test)]
mod tests {
    extern crate chrono;
    extern crate dotenv;

    use self::chrono::{Duration, NaiveDate, NaiveTime, Utc};
    use self::dotenv::dotenv;

    use crate::dsl::{now, sql};
    use crate::prelude::*;
    use crate::select;
    use crate::sql_types::{Date, Datetime, Time, Timestamp};

    fn connection() -> OdbcConnection {
        dotenv().ok();

        let connection_url = ::std::env::var("ODBC_UNIT_TEST_DATABASE_URL")
            .or_else(|_| ::std::env::var("ODBC_DATABASE_URL"))
            .or_else(|_| ::std::env::var("DATABASE_URL"))
            .expect("DATABASE_URL must be set in order to run tests");
        OdbcConnection::establish(&connection_url).unwrap()
    }

    #[test]
    fn unix_epoch_encodes_correctly() {
        let connection = connection();
        let time = NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);
        let query = select(sql::<Timestamp>("CAST('1970-01-01' AS DATETIME)").eq(time));
        assert!(query.get_result::<bool>(&connection).unwrap());
        let query = select(sql::<Datetime>("CAST('1970-01-01' AS DATETIME)").eq(time));
        assert!(query.get_result::<bool>(&connection).unwrap());
    }

    #[test]
    fn unix_epoch_decodes_correctly() {
        let connection = connection();
        let time = NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);
        let epoch_from_sql =
            select(sql::<Timestamp>("CAST('1970-01-01' AS DATETIME)")).get_result(&connection);
        assert_eq!(Ok(time), epoch_from_sql);
        let epoch_from_sql =
            select(sql::<Datetime>("CAST('1970-01-01' AS DATETIME)")).get_result(&connection);
        assert_eq!(Ok(time), epoch_from_sql);
    }

    #[test]
    fn times_relative_to_now_encode_correctly() {
        let connection = connection();
        let time = Utc::now().naive_utc() + Duration::days(1);
        let query = select(now.lt(time));
        assert!(query.get_result::<bool>(&connection).unwrap());

        let time = Utc::now().naive_utc() - Duration::days(1);
        let query = select(now.gt(time));
        assert!(query.get_result::<bool>(&connection).unwrap());
    }

    #[test]
    fn times_of_day_encode_correctly() {
        let connection = connection();

        let midnight = NaiveTime::from_hms(0, 0, 0);
        let query = select(sql::<Time>("CAST('00:00:00' AS TIME)").eq(midnight));
        assert!(query.get_result::<bool>(&connection).unwrap());

        let noon = NaiveTime::from_hms(12, 0, 0);
        let query = select(sql::<Time>("CAST('12:00:00' AS TIME)").eq(noon));
        assert!(query.get_result::<bool>(&connection).unwrap());

        let roughly_half_past_eleven = NaiveTime::from_hms(23, 37, 4);
        let query = select(sql::<Time>("CAST('23:37:04' AS TIME)").eq(roughly_half_past_eleven));
        assert!(query.get_result::<bool>(&connection).unwrap());
    }

    #[test]
    fn times_of_day_decode_correctly() {
        let connection = connection();
        let midnight = NaiveTime::from_hms(0, 0, 0);
        let query = select(sql::<Time>("CAST('00:00:00' AS TIME)"));
        assert_eq!(Ok(midnight), query.get_result::<NaiveTime>(&connection));

        let noon = NaiveTime::from_hms(12, 0, 0);
        let query = select(sql::<Time>("CAST('12:00:00' AS TIME)"));
        assert_eq!(Ok(noon), query.get_result::<NaiveTime>(&connection));

        let roughly_half_past_eleven = NaiveTime::from_hms(23, 37, 4);
        let query = select(sql::<Time>("CAST('23:37:04' AS TIME)"));
        assert_eq!(
            Ok(roughly_half_past_eleven),
            query.get_result::<NaiveTime>(&connection)
        );
    }

    #[test]
    fn dates_encode_correctly() {
        let connection = connection();
        let january_first_2000 = NaiveDate::from_ymd(2000, 1, 1);
        let query = select(sql::<Date>("CAST('2000-1-1' AS DATE)").eq(january_first_2000));
        assert!(query.get_result::<bool>(&connection).unwrap());

        let january_first_2018 = NaiveDate::from_ymd(2018, 1, 1);
        let query = select(sql::<Date>("CAST('2018-1-1' AS DATE)").eq(january_first_2018));
        assert!(query.get_result::<bool>(&connection).unwrap());
    }

    #[test]
    fn dates_decode_correctly() {
        let connection = connection();
        let january_first_2000 = NaiveDate::from_ymd(2000, 1, 1);
        let query = select(sql::<Date>("CAST('2000-1-1' AS DATE)"));
        assert_eq!(
            Ok(january_first_2000),
            query.get_result::<NaiveDate>(&connection)
        );

        let january_first_2018 = NaiveDate::from_ymd(2018, 1, 1);
        let query = select(sql::<Date>("CAST('2018-1-1' AS DATE)"));
        assert_eq!(
            Ok(january_first_2018),
            query.get_result::<NaiveDate>(&connection)
        );

        connection
            .execute("SET sql_mode = (SELECT REPLACE(@@sql_mode, 'NO_ZERO_DATE,', ''))")
            .unwrap();
        let query = select(sql::<Date>("CAST('0000-00-00' AS DATE)"));
        assert!(query.get_result::<NaiveDate>(&connection).is_err());
    }
}
