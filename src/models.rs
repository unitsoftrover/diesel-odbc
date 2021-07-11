extern crate bigdecimal;
use serde::{Deserialize, Serialize};
use crate::schema::{users};
use chrono::{NaiveDateTime, NaiveDate};
use diesel::prelude::*;
use bigdecimal::BigDecimal;


#[derive(Debug, Clone, Serialize, Queryable, Insertable, PartialEq,QueryableByName)]
#[table_name = "users"]
pub struct User {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub name: String,
}

#[allow(non_snake_case)]
// #[derive(Debug, Clone, Queryable, PartialEq,QueryableByName)]
#[derive(Debug, Clone, Queryable, PartialEq)]
// #[table_name = "company"]
pub struct Company {
    pub CompanyID: i32,
    pub CompanyCode: String,
    pub CompanyType: String,
    pub CreateOffice : String,
    pub CompanyName: String,
    pub CompanyNameCN: String,
    pub DateCreated: NaiveDateTime,
    pub CreditAmount: Option<BigDecimal>,
    pub IsHeadOffice: bool,
    pub TestSmallInt : i16, 
    pub TestTinyInt : i8, 
    pub TestDate : NaiveDate, 
    pub TestTime : NaiveDateTime, 
    pub TestFloat : f64, 
    pub TestReal : f32, 
    pub TestBigInt : i64, 
    pub TestBin : Vec<u8>,
    pub CreditInstruction : String,
}


