use crate::api_structures::{
    api_state::ApiState,
    card_game::deck::Deck,
    id::*,
    managers::game_manager::{GameManager, GameState},
    messages::ConnectWithSession,
    session::SessionCode,
};
use actix_web::{
    get, post,
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

#[derive(Serialize, Deserialize)]
struct SessionInfo {
    id: SessionId,
    code: String,
}

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
            Ok((id, code)) => HttpResponse::Ok().json(SessionInfo {
                id,
                code: code.code,
            }),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid host_id")
    }
}
#[get("/games")]
async fn get_games(data: web::Data<Arc<Mutex<ApiState>>>) -> impl Responder {
    let state = data.lock().expect("failed to lock state");
    let session_manager = state
        .session_manager
        .lock()
        .expect("failed to lock session manager");

    let sessions = session_manager.get_games().await;
    if sessions.is_empty() {
        return HttpResponse::NoContent().finish();
    }
    HttpResponse::Ok().json(sessions)
}

#[post("/unwrap_session_code")]
async fn unwrap_session_code(
    data: web::Data<Arc<Mutex<ApiState>>>,
    context: web::Json<String>,
) -> impl Responder {
    let state = data.lock().expect("fialed to lock state");
    let session_manager = state
        .session_manager
        .lock()
        .expect("fialed to lock manager");
    if let Some(session_id) = session_manager.unwrap_code(SessionCode::from(context.0)) {
        HttpResponse::Ok().json(session_id)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[post("/deck")]
async fn test_deck(context: web::Json<Deck>) -> impl Responder {
    let deck = context.into_inner();
    println!("Got Deck: {:#?}", &deck);
    HttpResponse::Ok().json(deck)
}

#[post("/render_cards")]
async fn render_cards(context: web::Json<Deck>) -> impl Responder {
    let deck = context.into_inner();
    println!("Got Deck: {:#?}", &deck);
    let mut manager = GameManager::init(deck.into_bundle());
    let mut results = Vec::new();
    for _ in 0..10 {
        if let Some(result) = manager.get_next_card() {
            results.push(result);
        } else {
            break;
        }
    }
    HttpResponse::Ok().json(results)
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
    let query = web::Query::<JoinSession>::from_query(req.query_string())?;
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
        .service(unwrap_session_code)
        .service(get_games)
        .service(test_deck)
        .service(render_cards)
}
