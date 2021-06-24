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
extern crate serde_derive;
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
use models::*;
use diesel::prelude::*;
use chrono::{NaiveDateTime, NaiveDate};
use diesel::debug_query;
use diesel::sql_types::Numeric;
use odbc::types::numeric::*;

fn main(){
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::builder().target(env_logger::Target::Stdout).init();
    dotenv::dotenv().ok();

    // set up database connection pool
    let _connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    // let conn = RawConnection<safe::AutocommitOn>>::new(connspec);
    // let connspec = "driver={sql server};server=localhost;database=UnitsoftEBS;UID=main;PWD=unitsoft_main;";
    let connspec = "driver={sql server};server=192.168.1.10;database=UnitsoftEBS;uid=main;pwd=unitsoft_main;";
    let conn = RawConnection::<safe::AutocommitOn>::establish(connspec).unwrap();
    {
        let stmt = Statement::with_parent(&conn).unwrap();
        let stmt = stmt.prepare("select count(*) from company").unwrap();        
        let stmt = stmt.execute().unwrap();    

        match stmt{
            ResultSetState::Data(mut st)=>{
                let row_count = st.affected_row_count().unwrap();
                println!("row count: {}", row_count);

                if let Some(mut cursor) = st.fetch().unwrap(){
                    if let Some(count) = cursor.get_data(1).unwrap() as Option<i64>{
                        println!("count:{}", count);
                    }

                    let row_count = st.affected_row_count().unwrap();
                    println!("row count2: {}", row_count);

                }
            },
            ResultSetState::NoData(_st)=>{},
        }
    }

    {     
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
                        println!("Company Code:{}", val);
                    }
                    if let Some(val) = cursor.get_data(3).unwrap() as Option<String>{
                        println!("Company Name:{}", val);
                    }

                    {
                        let row_count = st.affected_row_count().unwrap();
                        println!("row count2: {}", row_count);
                    }                    
                }
            },
            ResultSetState::NoData(_st)=>{},
        }    
    }

    use self::schema::company::dsl::*;

    let results = company
        .filter(CompanyID.eq(1))              
        .load::<Company>(&conn)
        .expect("Error loading company");

    println!("Displaying {} company", results.len());
    for company1 in results {
        println!("CompanyID:{}", company1.CompanyID);
        println!("CompanyCode:{}", company1.CompanyCode);
        println!("CompanyName:{}", company1.CompanyName);
        println!("Create Date:{:?}", company1.DateCreated);
        println!("Credit Amount:{}", company1.CreditAmount);
        println!("Is Headquater:{}", company1.IsHeadOffice);
        println!("Test Date:{}", company1.TestDate);
    }

    use diesel::insert_into;
    use diesel::select;
    // let one_company = insert_into(company)
    //     .values((CompanyCode.eq("00000000"), CompanyName.eq("unitsoft"), DateCreated.eq(NaiveDateTime::parse_from_str("2020-1-1 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()), CreateOffice.eq("SH"), CompanyType.eq("C")))
    //     .execute(&conn);
    let query = company.select(CompanyCode).filter(CompanyID.eq(1));
    let query_str = debug_query::<odbc::Mysql, _>(&query).to_string();
    println!("query：{:?}", query_str);
    let company_code = query.load::<String>(&conn).unwrap();
    println!("company code:{}", company_code[0]);    
    
    // let average : Numeric = company.select(avg(CreditAmount)).get_result(&conn).unwrap();

    // select("select 1").execute(&conn);

    // conn.execute(query: &str)    
}

use diesel::sql_types::Foldable;

sql_function!{
 #[aggregate]
    fn sum<ST: Foldable>(expr: ST) -> ST::Sum;
}

sql_function!{
    #[aggregate]
       fn avg<ST: Foldable>(expr: ST) -> ST::Avg;
   }

sql_function!(fn getdate() -> Text);
