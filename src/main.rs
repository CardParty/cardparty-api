use std::sync::{Arc, Mutex};

use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use api_structures::api_state::ApiState;
use scopes::game_session::game::game_scope;
use scopes::user::user::user_scope;
use models::*;
use self::schema::users::dsl::*;

mod schema;
mod models;
mod api_structures;
mod auth;
mod scopes;
mod database;

#[actix_web::main]
async fn main()-> std::io::Result<()> {
    // let connection = &mut database::database::establish_connection();


    // let results = users
    //     
    //     .limit(5)
    //     .select(User::as_select())
    //     .load(connection)
    //     .expect("Error loading posts");
    // 
    // println!("Displaying {} posts", results.len());
    // for post in results {
    //     println!("{}", post.user_id);
    //     println!("-----------\n");
    //     println!("{}", post.username);
    // }
    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(Arc::new(Mutex::new(ApiState::new()))))
            .service(game_scope())
            .service(user_scope())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
