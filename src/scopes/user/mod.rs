use std::sync::{Arc, Mutex};

use actix_web::{
    HttpResponse,
    post,
    Responder, Scope, web::{self},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api_structures::{
    api_state::ApiState,
    id::Id,
    session::{Player, Session},
};

#[derive(Serialize, Deserialize)]
struct CreateUser {
    username: String,
    password: String
}
#[derive(Serialize, Deserialize)]
struct DeleteUser{
    username: String,
    password: String
}
#[post("/create")]
async fn create_user(
    data: web::Data<Arc<Mutex<ApiState>>>,
    context: web::Json<CreateUser>,
) -> impl Responder {
    if let Ok(username) = Uuid::parse_str(&context.username[..]) {

        let password=&context.password[..];
        return HttpResponse::Ok().json("User created");
    } else {
        return HttpResponse::BadRequest().finish();
    };
}
#[post("/delete")]
async fn delete_user(
    data: web::Data<Arc<Mutex<ApiState>>>,
    context: web::Json<DeleteUser>,
) -> impl Responder {
    return HttpResponse::BadRequest().finish();

}

pub fn user_scope() -> Scope {
    Scope::new("/user")
        .service(create_user)
        .service(delete_user)
}
