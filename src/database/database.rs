use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use uuid::Uuid;
use crate::models::{NewUser, User};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
pub fn create_user(conn: &mut PgConnection, username: String, password: String) -> User {
    use crate::schema::users;

    let new_user= NewUser {user_id: &Uuid::new_v4(), username: &username ,password: &password };

    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Error saving new user")
}
pub fn delete_user(conn: &mut PgConnection, username: String, password: String) -> User {
    use crate::schema::users;


    diesel::delete(users::table)
        .filter(users::username.eq(&username))
        .filter(users::password.eq(&password))
        .returning(User::as_returning())
        .get_result(conn)
        .expect("Error deleting user")
}