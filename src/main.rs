use std::sync::{Arc, Mutex};

use actix_web::middleware::Logger;
use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use dotenv::dotenv;
use env_logger::Env;
use log::info;

use api_structures::api_state::ApiState;
use scopes::game_session::game::game_scope;

mod api_structures;
mod auth;
mod database;
mod scopes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("trace"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(Mutex::new(ApiState::new())))
            .service(game_scope())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
