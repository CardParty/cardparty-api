use crate::api_structures::{
    api_state::ApiState,
    id::Id,
    session::{Player, Session},
};
use actix_web::{
    post,
    web::{self, ServiceConfig},
    HttpResponse, Responder, Scope,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct CreateSession {
    host_id: String,
    username: String,
}

#[derive(Serialize, Deserialize)]
struct SessionInfo {}

#[post("/create")]
async fn create_game(
    data: web::Data<Arc<Mutex<ApiState>>>,
    context: web::Json<CreateSession>,
) -> impl Responder {
    if let Ok(host_id) = Uuid::parse_str(&context.host_id[..]) {
        let mut session = Session::new();
        let username = &context.username[..];
        session.add_player(
            Player::new(true, Id::UserId(host_id), username).expect("player creation fucked up"),
        );
        return HttpResponse::Ok().json(session);
    } else {
        return HttpResponse::BadRequest().finish();
    };
}
// #[post("/join")]
// async fn join_game() ->

pub fn game_scope() -> Scope {
    Scope::new("/game").service(create_game)
}
