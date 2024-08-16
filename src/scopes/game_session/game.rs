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

// #[derive(Serialize, Deserialize)]
// struct SessionInfo {} // deprectaed

#[post("/create")]
async fn create_game(
    data: web::Data<Mutex<ApiState>>, // Use Arc to wrap your Mutex
    context: web::Json<CreateSession>,
) -> impl Responder {
    if let Ok(host_id) = Uuid::parse_str(&context.host_id) {
        // You don't need to slice &str
        let username = context.username.clone(); // Cloning the username string

        let mut state = data.lock().expect("oppsie"); // Awaiting the lock
        match state.session_manager.init_session(host_id, username).await {
            // Awaiting init_session
            Ok(id) => HttpResponse::Ok().json(id),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid host_id") // Returning BadRequest if Uuid parsing fails
    }
}
#[derive(Deserialize, Serialize)]
struct JoinSession {
    session_id: SessionId,
    user_id: UserId,
    username: String,
}

async fn join_game(
    data: web::Data<Mutex<ApiState>>,
    stream: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let mut state = data.lock().expect("failed to lock state");
    let query = web::Query::<JoinSession>::from_query(req.query_string()).unwrap();
    let session_id: SessionId = query.session_id;
    let user_id: UserId = query.user_id;
    let username: String = query.username.clone();

    log::info!(
        "Attempting to join session {} for user {}",
        session_id,
        user_id
    );

    match state
        .session_manager
        .join_session(session_id, user_id, username)
        .await
    {
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
