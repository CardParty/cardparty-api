use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

mod auth;
mod claim;
mod ids;
mod jwt;

#[post("/game/join")]
async fn join_lobby() -> impl Responder {
    todo!();
}
#[post("/game/create")]
async fn create_lobby() -> impl Responder {
    todo!();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new())
        .bind(("1.1.1.1", 8080))?
        .run()
        .await
}
