//! Actix web Diesel integration example
//!
//! Diesel does not support tokio, so we have to run it in separate threads using the web::block
//! function which offloads blocking code (like Diesel's) in order to not block the server's thread.
#![allow(dead_code)]
#![recursion_limit="512"]

extern crate diesel;
extern crate odbc_sys;
extern crate log;
extern crate lazy_static;
extern crate chrono;
mod actions;

use actix_web::{get, middleware, post, web, App, Error, HttpResponse, HttpServer};
use diesel::r2d2::{ConnectionManager};
use uuid::Uuid;

pub use self::odbc_sys::*;
use diesel_odbc::connection::RawConnection;
use odbc_safe as safe;
use data_model::models;
use data_model::schema;


type DbPool = r2d2::Pool<ConnectionManager<RawConnection<'static, safe::AutocommitOn>>>;

/// Finds user by UID.
#[get("/user/{user_id}")]
async fn get_user(
    pool: web::Data<DbPool>,
    user_uid: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    // ::environment::DB_ENCODING
    let user_uid = user_uid.into_inner();
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || actions::find_user_by_uid(user_uid, &mut conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    // let user: Option<models::User> = None;

    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let res = HttpResponse::NotFound()
            .body(format!("No user found with uid: {}", user_uid));
        Ok(res)
    }
    
}

/// Inserts new user with name defined in form.
#[post("/user")]
async fn add_user(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {

    println!("xxxx");
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || actions::insert_new_user(&form.name, &mut conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(user))
} 

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    // set up database connection pool
    // let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let connspec = "driver={sql server};server=192.168.1.8;database=UnitsoftERP_DEV;uid=main;pwd=unitsoft_main;";
    let manager = ConnectionManager::<RawConnection<safe::AutocommitOn>>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager) 
        .expect("Failed to create pool.");

    let bind = "127.0.0.1:8080";

    println!("Starting server at: {}", &bind);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(get_user)
            .service(add_user)
    })
    .bind(&bind)?
    .run()
    .await
}
