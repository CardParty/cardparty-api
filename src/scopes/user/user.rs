
use std::sync::{Arc, Mutex};
use actix::ActorStreamExt;
use actix_web::{
    HttpResponse,
    post,
    Responder, Scope, web::{self},
};
use serde::{Deserialize, Serialize};

use crate::api_structures::{
    api_state::ApiState
};
use crate::database::database::*;

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
async fn create_user_fn(
    data: web::Data<Arc<Mutex<ApiState>>>,
    context: web::Json<CreateUser>,
) -> impl Responder {
    let username = &context.username;
    let password = &context.password;
    if !&username.is_empty() && !&password.is_empty() {
        create_user(&mut establish_connection(), username.to_string(), password.to_string());
        return HttpResponse::Ok().json("User created");
    } else {
        return HttpResponse::BadRequest().json("Wrong credentials");
    };
}
#[post("/delete")]
async fn delete_user_fn(
    data: web::Data<Arc<Mutex<ApiState>>>,
    context: web::Json<DeleteUser>,
) -> impl Responder {
    let username = &context.username;
    let password = &context.password;
    if !&username.is_empty() && !&password.is_empty() {
        delete_user(&mut establish_connection(), username.to_string(), password.to_string());
        return HttpResponse::Ok().json("User deleted");
    } else {
        return HttpResponse::BadRequest().json("Wrong credentials");
    };

}

pub fn user_scope() -> Scope {
    Scope::new("/user")
        .service(create_user_fn)
        .service(delete_user_fn)
}
