use actix_web::{get, web, HttpResponse, Scope};

fn user_auth_token() -> HttpResponse {
    todo!();
}

fn auth_scope() -> Scope {
    web::scope("/auth").route("/userauthtoken", web::get().to(user_auth_token))
}
