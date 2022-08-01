#![allow(dead_code)]
#![recursion_limit="512"]
pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;

pub fn data_model_test()->bool{
    return true;
}