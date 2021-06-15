use serde::{Deserialize, Serialize};

use crate::schema::users;

#[derive(Debug, Clone, Serialize, Queryable, Insertable)]
pub struct User {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub name: String,
}

#[derive(Debug, Clone,Queryable, Serialize, Deserialize)]
pub struct Company {
    pub CompanyID: i32,
    // pub CompanyCode: String,
    // pub CompanyName: String,
}