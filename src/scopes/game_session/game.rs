use crate::api_structures::id::Id;
use actix_web::{post, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct CreateSession {
    host_id: String,
}

#[derive(Serialize, Deserialize)]
struct SessionInfo {}

#[post("/create")]
async fn create_game(context: web::Json<CreateSession>) -> impl Responder {
    if let Ok(host_id) = Uuid::parse_str(&context.host_id[..]) {
    } else {
        return HttpResponse::BadRequest().await.unwrap();
    }
}

fn game_scope() -> Scope {
    Scope::new("/game").service(create_game)
}
