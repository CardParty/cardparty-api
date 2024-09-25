use crate::api_structures::{api_state::ApiState, id::*, messages::ConnectWithSession};
use actix_web::{
    post,
    web::{self},
    HttpRequest, HttpResponse, Responder, Scope,
};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
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
// struct SessionInfo {} // deprecated

#[post("/create")]
async fn create_game(
    data: web::Data<Arc<Mutex<ApiState>>>,
    context: web::Json<CreateSession>,
) -> impl Responder {
    if let Ok(host_id) = Uuid::parse_str(&context.host_id) {
        let username = context.username.clone();
        let state = data.lock().expect("failed to lock state");
        let mut session_manager = state
            .session_manager
            .lock()
            .expect("failed to lock session manager");
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
    let mut session_manager = state
        .session_manager
        .lock()
        .expect("failed to lock session manager");
    let query = web::Query::<JoinSession>::from_query(req.query_string()).unwrap();
    let session_id: SessionId = query.session_id;
    let user_id: UserId = query.user_id;
    let username: String = query.username.clone();

    match session_manager
        .join_session(session_id, user_id, username)
        .await
    {
        Some(conn) => {
            let (addr, resp) = ws::WsResponseBuilder::new(conn, &req, stream)
                .start_with_addr()
                .expect("cannot create with addr");
            addr.do_send(ConnectWithSession(addr.clone()));
            Ok(resp)
        }

        None => Ok(HttpResponse::BadRequest().finish()),
    }
}

pub fn game_scope() -> Scope {
    Scope::new("/game")
        .service(create_game)
        .route("/join", web::get().to(join_game))
}
