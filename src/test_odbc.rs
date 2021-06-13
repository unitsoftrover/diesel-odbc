//! Actix web Diesel integration example
//!
//! Diesel does not support tokio, so we have to run it in separate threads using the web::block
//! function which offloads blocking code (like Diesel's) in order to not block the server's thread.
#![allow(dead_code)]

#[macro_use]
extern crate diesel;
extern crate odbc_sys;
extern crate log;
extern crate lazy_static;

use odbc::connection::environment;
pub use self::odbc_sys::*;

mod actions;
mod models;
mod schema;
mod odbc;
use diesel::connection::*;
use odbc::connection::raw_conn::*;
use odbc_safe as safe;
use odbc::connection::statement::{Statement, ResultSetState};
use odbc::utility::Utility;

fn main(){
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::builder().target(env_logger::Target::Stdout).init();
    dotenv::dotenv().ok();

    // set up database connection pool
    let _connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    // let conn = RawConnection<safe::AutocommitOn>>::new(connspec);
    let connspec = "driver={sql server};server=localhost;database=UnitsoftEBS;UID=main;PWD=unitsoft_main;";
    let conn = RawConnection::<safe::AutocommitOn>::establish(connspec).unwrap();
    
    let stmt = Statement::with_parent(&conn).unwrap();
    let stmt = stmt.prepare("select count(*) from company").unwrap();
    let stmt = stmt.execute().unwrap();

    match stmt{
        ResultSetState::Data(mut st)=>{
            if let Some(mut cursor) = st.fetch().unwrap(){
                if let Some(count) = cursor.get_data(1).unwrap() as Option<i64>{
                    println!("count:{}", count);
                }

            }
        },
        ResultSetState::NoData(_st)=>{},
    }


    let stmt = Statement::with_parent(&conn).unwrap();
    let stmt = stmt.prepare("select CompanyID,CompanyCode,CompanyName from company").unwrap();
    let stmt = stmt.execute().unwrap();
    match stmt{
        ResultSetState::Data(mut st)=>{
            while let Some(mut cursor) = st.fetch().unwrap(){
                if let Some(val) = cursor.get_data(1).unwrap() as Option<i64>{
                    println!("CompanyID:{}", val);
                }
                if let Some(val) = cursor.get_data(2).unwrap() as Option<String>{                    
                    println!("Company Code:{}", Utility::utf8_bytes_to_gbk(val.as_bytes()));
                }
                if let Some(val) = cursor.get_data(3).unwrap() as Option<String>{
                    println!("Company Name:{}", Utility::utf8_bytes_to_gbk(val.as_bytes()));
                }
            }
        },
        ResultSetState::NoData(_st)=>{},
    }
    
    
}