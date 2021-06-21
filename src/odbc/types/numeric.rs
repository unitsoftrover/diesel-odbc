#[cfg(feature = "bigdecimal")]
pub mod bigdecimal {
    extern crate bigdecimal;

    use self::bigdecimal::{BigDecimal, FromPrimitive};
    use std::io::prelude::*;

    use diesel::deserialize::{self, FromSql};
    use crate::odbc::{Mysql, MysqlValue};
    use diesel::serialize::{self, IsNull, Output, ToSql};
    use diesel::sql_types::Numeric;
    use crate::odbc::connection::ffi;

    impl ToSql<Numeric, Mysql> for BigDecimal {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Mysql>) -> serialize::Result {
            write!(out, "{}", *self)
                .map(|_| IsNull::No)
                .map_err(Into::into)
        }
    }

    impl FromSql<Numeric, Mysql> for BigDecimal {
        fn from_sql(value: MysqlValue<'_>) -> deserialize::Result<Self> {
            use crate::odbc::NumericRepresentation::*;

            match value.numeric_value()? {
                Tiny(x) => Ok(x.into()),
                Small(x) => Ok(x.into()),
                Medium(x) => Ok(x.into()),
                Big(x) => Ok(x.into()),
                Float(x) => BigDecimal::from_f32(x)
                    .ok_or_else(|| format!("{} is not valid decimal number ", x).into()),
                Double(x) => BigDecimal::from_f64(x)
                    .ok_or_else(|| format!("{} is not valid decimal number ", x).into()),
                Decimal(bytes) => {
                    
                    unsafe{
                        let ptr = bytes.as_ptr() as * const ffi::SQL_Numeric_STRUCT;
                        let mut n = (*ptr).val.len()-1;
                        while n >= 0 {
                            let current = (*ptr).val[n];
                            if current != 0u8{
                                break;
                            }
                            n -= 1;                                    
                        };                                                 

                        let mut value = 0i128;
                        let mut base = 1i128;
                        for i in 0..=n {
                            let current = (*ptr).val[i];
                            let low_half = current % 16;
                            let high_half = current / 16;
                            value += base * low_half as i128;
                            base *= 16i128;
                            value += base * high_half as i128;
                            base *= 16i128;                                                       
                        };
                        let decimal = bigdecimal::BigDecimal::new(num_bigint::BigInt::from(value), 0);
                        Ok(decimal)                                
                    }

                    // BigDecimal::parse_bytes(bytes, 10)
                    //     .ok_or_else(|| format!("{:?} is not valid decimal number ", bytes).into())
                },
            }
        }
    }



}

