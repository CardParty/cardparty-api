use actix_web::{App, HttpServer};

mod api_structures;
mod auth;
mod database;
mod scopes;
mod user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new())
        .bind(("1.1.1.1", 8080))?
        .run()
        .await
}
