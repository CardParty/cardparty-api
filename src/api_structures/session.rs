use super::errors::FactoryError;
use crate::{api_structures::id::Id, user};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    is_host: bool,
    id: Id,
    username: String,
}

impl Player {
    pub fn new(is_host: bool, id: Id, username: &str) -> Result<Self, FactoryError> {
        if let Some(valid_id) = id.verify_user_id() {
            return Ok(Self {
                is_host: is_host,
                id: valid_id,
                username: String::from(username),
            });
        } else {
            return Err(FactoryError::InvalidIdVarient);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Session {
    id: Id,
    players: Vec<Player>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: Id::SessionId(Uuid::new_v4()),
            players: Vec::new(),
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.push(player);
    }
}
