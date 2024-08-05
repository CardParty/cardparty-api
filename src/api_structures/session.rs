use super::errors::FactoryError;
use crate::api_structures::card_game::deck::Deck;
use crate::{api_structures::id::Id, user};
use actix::{Actor, Message, StreamHandler};
use actix_web::web::{self, Data};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub enum SessionFlag {}

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
    deck: Deck,
    creation_date: chrono::DateTime<chrono::Utc>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: Id::SessionId(Uuid::new_v4()),
            players: Vec::new(),
            deck: Deck {},
            creation_date: chrono::Utc::now(),
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.push(player);
    }
}

// hours spent working on this bullshit:
// Lukasz - 4h

// add your wasted time too!!!

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(websocket_message) = item {
            match websocket_message {
                ws::Message::Text(data) => {
                    let string = data.parse::<String>().expect("string parsing fucked up");
                    log::info!("{}", string);
                }
                ws::Message::Binary(_) => ctx.text("ERROR! Bad Poll"),
                ws::Message::Continuation(_) => ctx.text("ERROR! Bad Poll"),
                ws::Message::Ping(_) => ctx.pong(b"pong"),
                ws::Message::Pong(_) => ctx.ping(b"ping"),
                ws::Message::Close(_) => todo!(),
                ws::Message::Nop => todo!(),
            }
        } else {
            log::error!("Failed To parse message from websocket")
        }
    }
}
