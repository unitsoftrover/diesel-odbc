#![allow(dead_code)]
#![recursion_limit="512"]
mod odbc;
pub use odbc::*;

extern crate diesel;
extern crate log;
