#[macro_use]
extern crate slog;
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
mod logging;

use actix_rt;
use actix_web::{get, http, middleware, post, App, HttpServer};
use actix_web::{web, HttpResponse, Responder};
use data::*;

pub struct AppState {
    log: slog::Logger,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let log = logging::setup_logging();
    info!(log, "Starting server on localhost:8080");

    HttpServer::new(move || {
        App::new()
            .data(web::JsonConfig::default().limit(4096)) //limit size of the payload (global configuration)
            .data(AppState { log: log.clone() })
            .service(index)
            .service(create_user)
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

#[get("/{id}/{name}/index.html")]
async fn index(app_state: web::Data<AppState>, info: web::Path<(u32, String)>) -> impl Responder {
    let log: &slog::Logger = &app_state.log;
    info!(log, "index request");
    format!("Hello {}! id:{}", info.1, info.0)
}

#[post("/create_user")]
async fn create_user(
    app_state: web::Data<AppState>,
    request: web::Json<CreateUserRequest>,
) -> impl Responder {
    let log: &slog::Logger = &app_state.log;
    info!(log, "create user request {:?}", request);
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
        let mut app = test::init_service(
            App::new()
                .data(AppState {
                    log: logging::setup_logging(),
                })
                .service(create_user),
        )
        .await;

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
