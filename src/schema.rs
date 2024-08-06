// @generated automatically by Diesel CLI.

diesel::table! {
    users (user_id) {
        user_id -> Uuid,
        username -> Varchar,
        password -> Varchar,
    }
}
