use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub password: String,
}


#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub user_id: &'a uuid::Uuid,
    pub username: &'a str,
    pub password: &'a str,
    
}
