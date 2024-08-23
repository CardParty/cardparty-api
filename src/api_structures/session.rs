use std::{clone, ops::Add, vec};

use crate::api_structures::id::*;
use crate::api_structures::messages::TestMessage;
use crate::api_structures::{card_game::deck::Deck, messages::BrodcastMessage};
use actix::{Actor, Addr, Context, Handler, Message, StreamHandler};
use actix_web::web::{self, Data};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::messages::{AddPlayer, GetHostId, GetSessionId, SendToClient, VerifyExistance};
#[derive(Debug)]
pub enum SessionError {
    FailedToAddPlayer,
    PlayerAlreadyInSession,
    FailedToAddHostToSession,
    BrodcastMessageFailure,
}

#[derive(Deserialize, Serialize, Clone)]
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

impl SessionConnection {
    pub fn new(user_id: UserId, session: Addr<Session>) -> Self {
        Self { user_id, session }
    }
}

impl Actor for SessionConnection {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SessionConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("Received: {}", text);

                let split: Vec<String> = text
                    .parse::<String>()
                    .expect("failed to parse websocket string")
                    .split(" ")
                    .map(|str| String::from(str))
                    .collect();

                println!("Split: {:#?}", split);

                if let Some(str) = split.first() {
                    println!("got first");
                    if str.starts_with("send_all") {
                        println!("send_all detected");
                        self.session.do_send(BrodcastMessage(split[1..].join(" ")));
                    } else {
                        println!("send_all not detected");
                        self.session
                            .do_send(TestMessage(text.parse::<String>().unwrap()));
                    }
                } else {
                    println!("error with spliting the websocket string");
                    self.session
                        .do_send(TestMessage(text.parse::<String>().unwrap()));
                }
            }
            Ok(ws::Message::Binary(bin)) => println!("Received binary: {:?}", bin),
            _ => (),
        }
    }
}

impl Handler<TestMessage> for SessionConnection {
    type Result = ();

    fn handle(&mut self, msg: TestMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Handler<SendToClient> for SessionConnection {
    type Result = ();
    fn handle(&mut self, msg: SendToClient, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0)
    }
}
#[derive(Clone)]
pub struct Session {
    pub id: SessionId,
    pub connections: Vec<Addr<SessionConnection>>,
    pub host_id: Uuid,
    pub players: Vec<Player>,
    pub admin_token: Uuid,
}

impl Actor for Session {
    type Context = Context<Self>;
}

impl Session {
    pub async fn init(host_id: UserId, username: String) -> (Addr<Self>, SessionId) {
        let id = Uuid::new_v4();
        let addr = Self {
            id: id,
            host_id: host_id,
            connections: Vec::new(),
            players: Vec::new(),
            admin_token: Uuid::new_v4(),
        }
        .start();

        return (addr, id);
    }
}

impl Handler<TestMessage> for Session {
    type Result = ();
    fn handle(&mut self, msg: TestMessage, ctx: &mut Self::Context) -> Self::Result {
        println!("hander of session got: {:#?}", msg.0);
    }
}

impl Handler<VerifyExistance> for Session {
    type Result = bool;
    fn handle(&mut self, msg: VerifyExistance, ctx: &mut Self::Context) -> Self::Result {
        log::info!("verify existence ran on: {}", &self.id);
        self.id == msg.0
    }
}

impl Handler<GetHostId> for Session {
    type Result = String;
    fn handle(&mut self, msg: GetHostId, ctx: &mut Self::Context) -> Self::Result {
        self.host_id.to_string()
    }
}

impl Handler<AddPlayer> for Session {
    type Result = Result<SessionConnection, SessionError>;
    fn handle(&mut self, msg: AddPlayer, ctx: &mut Self::Context) -> Self::Result {
        self.players
            .push(Player::new(msg.id, msg.username.clone(), msg.is_host));
        log::info!(
            "Added new player with id: {} to session: {}",
            &msg.id,
            self.id
        );
        Ok(SessionConnection::new(msg.id, msg.session_addr))
    }
}

impl Handler<GetSessionId> for Session {
    type Result = String;
    fn handle(&mut self, msg: GetSessionId, ctx: &mut Self::Context) -> Self::Result {
        return self.id.to_string();
    }
}
impl Handler<BrodcastMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: BrodcastMessage, ctx: &mut Self::Context) -> Self::Result {
        for conn in &self.connections {
            conn.do_send(SendToClient(msg.0.clone()))
        }
    }
}
