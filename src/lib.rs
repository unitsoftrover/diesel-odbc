#![allow(dead_code)]
#![recursion_limit="512"]
mod odbc;
pub use odbc::*;

pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate log;
