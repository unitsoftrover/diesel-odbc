#[cfg(feature = "bigdecimal")]
pub mod bigdecimal {
    extern crate bigdecimal;

    use self::bigdecimal::{BigDecimal, FromPrimitive};
    use std::io::prelude::*;

    use crate::deserialize::{self, FromSql};
    use crate::odbc::{Mysql, MysqlValue};
    use crate::serialize::{self, IsNull, Output, ToSql};
    use crate::sql_types::Numeric;

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
                Decimal(bytes) => BigDecimal::parse_bytes(bytes, 10)
                    .ok_or_else(|| format!("{:?} is not valid decimal number ", bytes).into()),
            }
        }
    }
}
