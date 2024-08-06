use std::vec;

use crate::api_structures::card_game::deck::Deck;
use crate::api_structures::id::*;
use crate::api_structures::messages::TestMessage;
use actix::{Actor, Addr, Context, Handler, Message, StreamHandler};
use actix_web::web::{self, Data};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::messages::{GetHostId, VerifyExistance};

#[derive(Deserialize, Serialize)]
pub struct Player {
    username: String,
    id: UserId,
    is_host: bool,
}

impl Player {
    pub fn new(id: UserId, username: String, is_host: bool) -> Self {
        Self {
            id,
            username,
            is_host,
        }
    }
}

pub struct SessionConnection {
    session: Addr<Session>,
    user_id: UserId,
}

impl Actor for SessionConnection {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SessionConnection {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(websocket_message) = item {
            match websocket_message {
                ws::Message::Text(data) => {
                    let string = data.parse::<String>().expect("string parsing fucked up");
                    self.session.do_send(TestMessage(string))
                }
                ws::Message::Binary(_) => log::info!("bin"),
                ws::Message::Continuation(_) => log::info!("cont"),
                ws::Message::Ping(_) => ctx.pong(b"pong"),
                ws::Message::Pong(_) => ctx.ping(b"ping"),
                ws::Message::Close(_) => log::info!("close"),
                ws::Message::Nop => log::info!("nop"),
            }
        }
    }
}
pub struct Session {
    pub id: SessionId,
    pub connections: Vec<Addr<SessionConnection>>,
    pub host_id: Uuid,
    pub players: Vec<Player>,
}

impl Actor for Session {
    type Context = Context<Self>;
}

impl Session {
    pub fn init(host_id: UserId, username: String) -> Addr<Self> {
        Self {
            id: Uuid::new_v4(),
            host_id: host_id,
            connections: Vec::new(),
            players: vec![Player::new(host_id, username, true)],
        }
        .start()
    }
}

impl Handler<TestMessage> for Session {
    type Result = ();
    fn handle(&mut self, msg: TestMessage, ctx: &mut Self::Context) -> Self::Result {
        log::info!("WEBSOCKET KINDA WORKS:{:#?}", msg.0)
    }
}

impl Handler<VerifyExistance> for Session {
    type Result = bool;
    fn handle(&mut self, msg: VerifyExistance, ctx: &mut Self::Context) -> Self::Result {
        self.id == msg.0
    }
}

impl Handler<GetHostId> for Session {
    type Result = String;
    fn handle(&mut self, msg: GetHostId, ctx: &mut Self::Context) -> Self::Result {
        self.host_id.to_string()
    }
}
