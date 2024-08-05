use std::sync::{Arc, Mutex};

use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use api_structures::api_state::ApiState;
use scopes::game_session::game::game_scope;

mod api_structures;
mod auth;
mod database;
mod scopes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(Arc::new(Mutex::new(ApiState::new()))))
            .service(game_scope())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
