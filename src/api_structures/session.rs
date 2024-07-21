use crate::{api_structures::id::Id, user};

use super::errors::FactoryError;
pub struct Player {
    is_host: bool,
    id: Id,
    username: String,
}

impl Player {
    fn new(is_host: bool, id: Id, username: &str) -> Result<Self, FactoryError> {
        if let Some(valid_id) = id.verify_user_id() {
            return Ok(Self {
                is_host: is_host,
                id: valid_id,
                username: String::from(username),
            });
        } else {
            Err(FactoryError::InvalidIdVarient())
        }
    }
}
pub struct Session {
    id: Id,
    players: Vec,
}
