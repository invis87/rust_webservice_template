extern crate actix_web;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate failure;
extern crate reqwest;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::env;

mod data;
mod db_actions;
mod handlers;
mod models;
mod schema;

use actix_rt;
use actix_web::{get, http, middleware, post, App, Error, HttpServer};
use actix_web::{web, HttpResponse, Responder};
use data::*;
use log::{debug, error, info};

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "web_service_template=debug,actix_web=error");
    env_logger::init();
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    debug!("Trying to establish DB connection to '{}'", database_url);
    let db_connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder()
        .build(db_connection_manager)
        .expect("Failed to create db pool");

    let bind_address = "0.0.0.0:8080";
    info!("Starting server on '{}'", bind_address);
    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4096)) //limit size of the payload (global configuration)
            .service(get_user)
            .service(create_user)
            .service(create_ticket)
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind(bind_address)?
    .run()
    .await
}

#[post("/create_user")]
async fn create_user(
    pool: web::Data<DbPool>,
    request: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let user_id = web::block(move || db_actions::insert_user(&request.name, &conn))
        .await
        .map_err(|e| {
            error!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(CreateUserResponse { id: user_id }))
}

#[post("/create_ticket")]
async fn create_ticket(request: web::Json<CreateTicketRequest>) -> Result<HttpResponse, Error> {
    HttpResponse::Ok().await
}

#[get("/user/{user_id}")]
async fn get_user(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let user_id = user_id_param.into_inner();
    let user = web::block(move || db_actions::get_user(user_id, &conn))
        .await
        .map_err(|e| {
            error!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    let result = match user {
        None => HttpResponse::NotFound().body(format!("No user found with id '{}'", user_id)),
        Some(u) => HttpResponse::Ok().json(u),
    };

    Ok(result)
}

// ============================== tests ==============================
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::Error;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_index() -> Result<(), Error> {
        let mut app = test::init_service(App::new().service(create_user)).await;

        let req = test::TestRequest::post()
            .uri("/create_user")
            .set_json(&CreateUserRequest {
                name: "some-name".to_owned(),
                character: Character {
                    level: 10,
                    color: "white".to_owned(),
                    race: "ork".to_owned(),
                },
            })
            .to_request();

        let resp = app.call(req).await.unwrap();

        let body = resp.response().body();
        match body {
            actix_web::dev::ResponseBody::Body(some_b) => println!("some body"),
            actix_web::dev::ResponseBody::Other(other_b) => match other_b {
                actix_web::dev::Body::None => println!("body::none"),
                actix_web::dev::Body::Empty => println!("body::empty"),

                actix_web::dev::Body::Bytes(bytes) => {
                    let s = std::str::from_utf8(&bytes).expect("utf8 parse error)");
                    println!("html: {:?}", s)
                }
                actix_web::dev::Body::Message(msg) => println!("body::msg"),
            },
        }

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r##"user 'some-name' created!"##);

        Ok(())
    }
}
