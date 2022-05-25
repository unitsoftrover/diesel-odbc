//! Actix web Diesel integration example
//!
//! Diesel does not support tokio, so we have to run it in separate threads using the web::block
//! function which offloads blocking code (like Diesel's) in order to not block the server's thread.
#![allow(dead_code)]
#![recursion_limit="512"]
#![warn(warnings)]

#[macro_use]
extern crate diesel;
extern crate log;

mod actions;
mod models;
mod schema;
mod sales;
mod sales_test;
mod company_test;

use diesel::prelude::*;
use diesel::sql_types::{SingleValue, SqlType};
use diesel::sql_types::*;
use diesel_odbc::connection::RawConnection;
use diesel_odbc::{OdbcQueryBuilder,Odbc};
use odbc_safe as safe;

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
            (ClauseWithSelect),
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

simple_clause!(
    NoReturningClause1,
    ReturningClause1,
    " RETURNING "
);


fn main(){
    println!("test odbc");
    use self::schema::company::*;
    let select = ReturningClauseWithSelect(CompanyName);

    let mut query_builder = OdbcQueryBuilder::new();    
    let ast_pass = AstPass::<Odbc>::to_sql(&mut query_builder);    
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
    let mut conn = RawConnection::<safe::AutocommitOn>::establish(connspec).unwrap();

    sales_test::test(&mut conn);
    // println!("test odbc finish");
    // company_test::test(&conn);
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


