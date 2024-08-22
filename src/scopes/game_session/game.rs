use crate::api_structures::{
    api_state::ApiState,
    id::*,
    managers::session_manager,
    session::{Player, Session, SessionConnection},
};
use actix::fut::stream;
use actix_web::{
    post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder, Scope,
};
use actix_web_actors::ws;
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

#[derive(Deserialize, Serialize)]
struct JoinSession {
    session_id: SessionId,
    user_id: UserId,
    username: String,
}

// #[derive(Serialize, Deserialize)]
// struct SessionInfo {} // deprectaed

#[post("/create")]
async fn create_game(
    data: web::Data<Arc<Mutex<ApiState>>>,
    context: web::Json<CreateSession>,
) -> impl Responder {
    if let Ok(host_id) = Uuid::parse_str(&context.host_id) {
        let username = context.username.clone();
        let state = data.lock().expect("failed to lock state");
        let mut session_manager = state.session_manager.lock().expect("failed to lock session manager");
        match session_manager.init_session(host_id, username).await {
            Ok(id) => HttpResponse::Ok().json(id),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid host_id")
    }
}

async fn join_game(
    data: web::Data<Arc<Mutex<ApiState>>>,
    stream: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let state = data.lock().expect("failed to lock state");
    let mut session_manager = state.session_manager.lock().expect("failed to lock session manager");
    let query = web::Query::<JoinSession>::from_query(req.query_string()).unwrap();
    let session_id: SessionId = query.session_id;
    let user_id: UserId = query.user_id;
    let username: String = query.username.clone();

    log::info!(
        "Attempting to join session {} for user {}",
        session_id,
        user_id
    );

    match session_manager.join_session(session_id, user_id, username).await {
        Some(session_connection) => {
            log::info!("Successfully joined session {}", session_id);
            let resp = ws::start(session_connection, &req, stream);
            match resp {
                Ok(response) => Ok(response),
                Err(e) => {
                    log::error!("Failed to start WebSocket: {:?}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        }
        None => {
            log::warn!("Failed to join session {} for user {}", session_id, user_id);
            Ok(HttpResponse::BadRequest().body("Failed to join session"))
        }
    }
}

pub fn game_scope() -> Scope {
    Scope::new("/game")
        .service(create_game)
        .route("/join", web::get().to(join_game))
}
