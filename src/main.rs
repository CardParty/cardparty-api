// nic zostawiamy

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{get, HttpResponse, Responder};
use actix_web::{web::Data, App, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use std::sync::{Arc, Mutex};

use api_structures::api_state::ApiState;
use scopes::game_session::game::game_scope;
mod api_structures;
mod auth;
mod database;
mod scopes;

#[get("health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let api_state = Arc::new(Mutex::new(ApiState::new()));

    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive()) // CHANGE BEFORE LAUNCH !!!!!!!!!!!!!!!!!!!!!!!!!!!
            .wrap(Logger::default())
            .app_data(Data::new(api_state.clone()))
            .service(game_scope())
            .service(health)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
