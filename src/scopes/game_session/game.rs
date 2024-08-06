use crate::api_structures::{
    api_state::ApiState,
    id::*,
    managers::session_manager,
    session::{Player, Session},
};
use actix_web::{
    post,
    web::{self, ServiceConfig},
    HttpResponse, Responder, Scope,
};
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct CreateSession {
    host_id: String,
    username: String,
}
#[derive(Deserialize, Serialize)]
struct CreateSessionResponse {
    websocket_addr: String,
}

// #[derive(Serialize, Deserialize)]
// struct SessionInfo {} // deprectaed

#[post("/create")]
async fn create_game(
    data: web::Data<Mutex<ApiState>>,
    context: web::Json<CreateSession>,
) -> impl Responder {
    if let Ok(host_id) = Uuid::parse_str(&context.host_id[..]) {
        let username = String::from(&context.username[..]);
        let mut state = data.lock().expect("oopsie").deref();
        if let Ok(_) = state.session_manager.init_session(host_id, username) {}
    }
}

#[post("/join")]
async fn join_game(
    data: web::Data<Mutex<ApiState>>,
    context: web::Json<CreateSession>,
) -> impl Responder {
}

pub fn game_scope() -> Scope {
    Scope::new("/game").service(create_game).service(join_game)
}
