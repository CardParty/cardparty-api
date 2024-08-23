use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{
    web::{Data},
    App, HttpServer,
};
use dotenv::dotenv;
use env_logger::Env;
use std::sync::{Arc, Mutex};

use api_structures::api_state::ApiState;
use scopes::game_session::game::game_scope;

mod api_structures;
mod auth;
mod database;
mod scopes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let api_state = Arc::new(Mutex::new(ApiState::new()));

    env_logger::init_from_env(Env::default().default_filter_or("default"));

    let api_state_clone = api_state.clone();
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive()) // CHANGE BEFORE LAUNCH !!!!!!!!!!!!!!!!!!!!!!!!!!!
            .wrap(Logger::default())
            .app_data(Data::new(api_state_clone.clone()))
            .service(game_scope())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
