extern crate actix_web;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate failure;
extern crate reqwest;

mod data;
mod handlers;

use actix_rt;
use actix_web::{get, http, middleware, post, App, HttpServer};
use actix_web::{web, HttpResponse, Responder};
use data::*;
use log::{info, warn};

#[inline]
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "jira_game=info,actix_web=info");
    env_logger::init();
    info!("Starting server on localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4096)) //limit size of the payload (global configuration)
            .service(index)
            .service(create_user)
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

#[get("/{id}/{name}/index.html")]
async fn index(info: web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", info.1, info.0)
}

#[post("/create_user")]
async fn create_user(request: web::Json<CreateUserRequest>) -> impl Responder {
    format!("user '{}' created!", request.name)
}

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
