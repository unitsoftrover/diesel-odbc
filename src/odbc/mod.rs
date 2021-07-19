//! Provides types and functions related to working with MySQL
//!
//! Much of this module is re-exported from database agnostic locations.
//! However, if you are writing code specifically to extend Diesel on
//! MySQL, you may need to work with this module directly.

mod backend;
pub mod connection;
mod value;
pub mod utility;

mod query_builder;
pub mod types;

pub use self::backend::{Mysql, MysqlType};
pub use self::query_builder::MysqlQueryBuilder;
pub use self::value::{MysqlValue, NumericRepresentation};

pub fn odbc_test()
{
    println!("odbc_test");
}