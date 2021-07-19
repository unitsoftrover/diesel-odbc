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
use odbc::connection::*;
use odbc_safe as safe;
use odbc::connection::statement::{Statement, ResultSetState};
use models::*;
use diesel::prelude::*;
use chrono::{NaiveDateTime};
use diesel::debug_query;
// use diesel::sql_types::Numeric;
// use diesel::sql_types::Foldable;
use diesel::sql_types::{SingleValue, SqlType};
use diesel::dsl::*;
use bigdecimal::*;
use diesel::sql_types::*;
// use diesel::expression::functions::aggregate_folding::*;

macro_rules! add_as{
    ($a:expr)=>
    {            
        $a            
    };
    
    ($a:expr, $b:expr)=>
    {  
        {          
            $a + $b
        }
    };

    ($a:expr, $($b:tt)*)=>
    {        
        {
            $a + add_as!($($b)*)        
        }
    }
}

use diesel::backend::Backend;
use diesel::result::QueryResult;
use diesel::query_builder::QueryId;
use diesel::query_builder::{QueryFragment, AstPass};

macro_rules! simple_clause {
    (
        $(#[doc = $($no_clause_doc:tt)*])*
        $no_clause:ident,
        $(#[doc = $($clause_doc:tt)*])*
        $clause:ident,
        $sql:expr
    ) => {
        simple_clause!(
            $(#[doc = $($no_clause_doc)*])*
            $no_clause,
            $(#[doc = $($clause_doc)*])*
            $clause,
            $sql,      
            backend_bounds =
            #[doc = ""]
            (clause_with_select),
            " SELECT",
            backend_bounds_with_select =            
        );
    };

    (
        $(#[doc = $($no_clause_doc:tt)*])*
        $no_clause:ident,
        $(#[doc = $($clause_doc:tt)*])*
        $clause:ident,
        $sql:expr,
        backend_bounds = $($backend_bounds:ident),*
        $(#[doc = $($clause_with_select_doc:tt)*])*
        ($clause_with_select:ident),
        $sql_with_select:expr,
        backend_bounds_with_select = $($backend_bounds_with_select:ident),*
    ) => 
    {           


        $(#[doc = $($no_clause_doc)*])*
        #[derive(Debug, Clone, Copy, QueryId)]
        pub struct $no_clause;

        impl<DB: Backend> QueryFragment<DB> for $no_clause {
            fn walk_ast(&self, _: AstPass<DB>) -> QueryResult<()> {
                Ok(())
            }
        }

        $(#[doc = $($clause_doc)*])*
        #[derive(Debug, Clone, Copy, QueryId)]
        pub struct $clause<Expr>(pub Expr);

        impl<Expr, DB> QueryFragment<DB> for $clause<Expr> where
            DB: Backend $(+ $backend_bounds)*,
            Expr: QueryFragment<DB>,
        {
            fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
                out.push_sql($sql);
                self.0.walk_ast(out.reborrow())?;
                Ok(())
            }
        }

        $(#[doc = $($clause_with_select_doc)*])*
        #[derive(Debug, Clone, Copy, QueryId)]
        pub struct $clause_with_select<Expr>(pub Expr);

        impl<Expr, DB> QueryFragment<DB> for $clause_with_select<Expr> where
            DB: Backend $(+ $backend_bounds_with_select)*,
            Expr: QueryFragment<DB>,
        {
            fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
                out.push_sql($sql);
                self.0.walk_ast(out.reborrow())?;
                Ok(())
            }
        }
        
    }
}

use diesel::backend::SupportsReturningClause;

simple_clause!(
    NoReturningClause,
    ReturningClause,
    " RETURNING ",
    backend_bounds = SupportsReturningClause
    (ReturningClauseWithSelect),
    " SELECT ",
    backend_bounds_with_select = SupportsReturningClause
);

// simple_clause!(
//     NoReturningClause1,
//     ReturningClause1,
//     " RETURNING "
// );


fn main(){
    let select = ReturningClauseWithSelect(CompanyName);

    let mut query_builder = odbc::OdbcQueryBuilder::new();    
    let ast_pass = AstPass::<odbc::Odbc>::to_sql(&mut query_builder);    
    select.walk_ast(ast_pass).unwrap();

    let sum1 = add_as!(1,2,3);
    println!("sum:{}", sum1);

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
    // let mut query_builder = odbc::OdbcQueryBuilder::new();    
    // let ast_pass = AstPass::<odbc::Odbc>::to_sql(&mut query_builder);    
    // let primary_key = company.primary_key();    
    // primary_key.walk_ast(ast_pass).unwrap();
    
    let results = company
        .filter(CompanyCode.eq("C0000005"))              
        .load::<Company>(&conn)
        .expect("Error loading company");
    
    println!("Displaying {} company", results.len());
    for company1 in results {
        println!("CompanyID:{}", company1.CompanyID);
        println!("CompanyCode:{}", company1.CompanyCode);
        println!("CompanyName:{}", company1.CompanyName);
        println!("Create Date:{:?}", company1.DateCreated);
        println!("Credit Amount:{}", company1.CreditAmount.unwrap_or_default());
        // println!("Is Headquater:{}", company1.IsHeadOffice);
        // println!("Test Date:{}", company1.TestDate);
    }    


    let _one_company = insert_into(company)
        .values((CompanyCode.eq("00000000"), CompanyName.eq("unitsoft"), DateCreated.eq(NaiveDateTime::parse_from_str("2020-1-1 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()), CompanyType.eq("C"), CreateOffice.eq("SH"), CreditAmount.eq(BigDecimal::from_f64(100000.0))))
        .load::<Company>(&conn).unwrap();
    let mut company_id = 0;
    let _ = _one_company.iter().map(|com|{
        println!("CompanyID:{}", com.CompanyID);
        println!("inserted id:{:?}", com.CompanyID);
        let sql = format!("exec MakeCode 'CompanyCode', {}", com.CompanyID);
        let _ = conn.execute(sql.as_str());
        company_id = com.CompanyID;        
    }).collect::<()>();
    
    // let company1 = delete(company.filter(CompanyID.eq(company_id))).get_result::<Company>(&conn);
    // company1.map(|com|{
    //     println!("company code:{} company name:{}", com.CompanyCode, com.CompanyName)
    // }).unwrap();

    // replace_into(company)
    //  .values((CompanyCode.eq("00000000"), CompanyName.eq("unitsoft"), DateCreated.eq(NaiveDateTime::parse_from_str("2020-1-1 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()), CompanyType.eq("C"), CreateOffice.eq("SH")))
    //  .execute(&conn)
    //  .unwrap();

    // let stmt = conn.prepare_query1(&"select ident_current('company')".to_owned());
    // if let Ok(rs) = stmt.execute(){
    //     match rs{
    //         ResultSetState::Data(mut st)=>{
    //             let mut cur = st.fetch().unwrap().unwrap();
    //             if let Some(company_id) = cur.get_data::<i64>(1).unwrap(){
    //                 println!("inserted id:{:?}", company_id);
    //                 let sql = format!("exec MakeCode 'CompanyCode', {}", company_id);
    //                 let _ = conn.execute(sql.as_str());
    //             }
    //             else
    //             {
    //                 println!("no inserted id.")
    //             }
    //         },
    //         ResultSetState::NoData(_st)=>{

    //         }
    //     }        
    // }

    let query = company.select(CompanyCode).filter(CompanyID.eq(1));
    let query_str = debug_query::<odbc::Odbc, _>(&query).to_string();
    println!("query：{:?}", query_str);
    let company_code = query.load::<String>(&conn).unwrap();
    println!("company code:{}", company_code[0]);        

    diesel::update(company.filter(CompanyID.eq(1)))
    .set(CompanyName.eq("unitsoft_test"))
    .execute(&conn)
    .unwrap();            

    let query = company.select(CompanyName).filter(CompanyID.eq(1));
    let company_name = query.load::<String>(&conn).unwrap();
    println!("new company name:{}", company_name[0]); 

    let company1:Company = diesel::update(company.filter(CompanyID.eq(1)))
    .set(CompanyName.eq("unitsoft_new"))
    .get_result(&conn)
    .unwrap();            

    println!("new company name:{}", company1.CompanyName); 

    // let sum1 : Option<BigDecimal> = company.select(sum(CreditAmount)).get_result(&conn).unwrap();
    // println!("sum:{:?}", sum1);

    // let avg : Option<BigDecimal> = company.select(avg(CreditAmount)).get_result(&conn).unwrap();
    // println!("avg:{:?}", avg);

    // let count : i64= company.select(count(CompanyID)).get_result(&conn).unwrap();
    // println!("count:{}", count);

    // let count : Option<BigDecimal> = company.select(sum(CreditAmount)).get_result(&conn).unwrap();
    // println!("sum: {}", count.unwrap());

    // let company_list = sql_query("SELECT CompanyID,CompanyCode,CompanyType,CreateOffice,CompanyName,CompanyNameCN,DateCreated,CreditAmount,IsHeadOffice,TestSmallInt,TestTinyInt,TestDate,TestTime FROM company ORDER BY CompanyCode")
    // .load::<Company>(&conn).unwrap();
    // println!("company list:{:?}", company_list);

    let stmt = conn.prepare_query1(&"SELECT CompanyName FROM company where CompanyID=1 ORDER BY CompanyCode".to_owned());
    if let Ok(rs) = stmt.execute(){
        match rs {
            ResultSetState::Data(mut st)=>{
                let mut cur = st.fetch().unwrap().unwrap();
                let company_name:String = cur.get_data(1).unwrap().unwrap();
                println!("company name:{:?}", company_name);
            },
            ResultSetState::NoData(_st)=>{

            }
        }        
    }


}



// sql_function!{
//  #[aggregate]
//     fn sum<ST: Foldable>(expr: ST) -> ST::Sum;
// }

// sql_function!{
//  #[aggregate]
//     fn sum<T: SqlType + SingleValue>(expr: T) -> Decimal;
// }

// sql_function!{
//     #[aggregate]
//        fn avg<T: SqlType + SingleValue>(expr: T) -> Decimal;
// }

sql_function!{
#[aggregate]
    fn round<T: SqlType + SingleValue>(expr: T, precision: Integer) -> Decimal;
}

sql_function!{
    fn count<T: SqlType + SingleValue>(expr: T) -> BigInt;
}


// sql_function!(fn getdate() -> Text);

fn run_test_1() -> QueryResult<()> {


    let connspec = "driver={sql server};server=192.168.1.10;database=UnitsoftEBS;uid=main;pwd=unitsoft_main;";
    let _connection = RawConnection::<safe::AutocommitOn>::establish(connspec).unwrap();
    // let users = sql_query("SELECT CompanyID,CompanyCode,CompanyName FROM company ORDER BY CompanyCode")
    //     .load::<Company>(&connection);


    Ok(())
}


