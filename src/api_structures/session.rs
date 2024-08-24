use crate::api_structures::id::*;
use crate::api_structures::messages::TestMessage;
use crate::api_structures::{messages::BroadcastMessage};
use actix::{Actor, Addr, Context, Handler, StreamHandler};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::messages::{
    AddConnection, AddPlayer, ConnectWithSession, GetHostId, GetSessionId, SendToClient,
    VerifyExistence,
};
#[derive(Debug)]
pub enum SessionError {
    FailedToAddPlayer,
    PlayerAlreadyInSession,
    FailedToAddHostToSession,
    BroadcastMessageFailure,
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
    id: Uuid,
}

impl SessionConnection {
    pub fn new(user_id: UserId, session: Addr<Session>) -> Self {
        Self {
            user_id,
            session,
            id: Uuid::new_v4(),
        }
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

                let split: Vec<String> = text
                    .parse::<String>()
                    .expect("failed to parse websocket string")
                    .split(' ')
                    .map(String::from)
                    .collect();


                if let Some(str) = split.first() {
                    if str.starts_with("send_all") {
                        self.session.do_send(BroadcastMessage(split[1..].join(" ")));
                    } else {
                        self.session
                            .do_send(TestMessage(text.parse::<String>().unwrap()));
                    }
                } else {
                    self.session
                        .do_send(TestMessage(text.parse::<String>().unwrap()));
                }
            }
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
        ctx.text(msg.0);
    }
}

impl Handler<ConnectWithSession> for SessionConnection {
    type Result = ();

    fn handle(&mut self, msg: ConnectWithSession, _ctx: &mut Self::Context) -> Self::Result {
        self.session.do_send(AddConnection(msg.0))
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
    pub async fn init(host_id: UserId, _username: String) -> (Addr<Self>, SessionId) {
        let id = Uuid::new_v4();
        let addr = Self {
            id,
            host_id,
            connections: Vec::new(),
            players: Vec::new(),
            admin_token: Uuid::new_v4(),
        }
        .start();

        (addr, id)
    }
}

impl Handler<TestMessage> for Session {
    type Result = ();
    fn handle(&mut self, _msg: TestMessage, _ctx: &mut Self::Context) -> Self::Result {
    }
}

impl Handler<VerifyExistence> for Session {
    type Result = bool;
    fn handle(&mut self, msg: VerifyExistence, _ctx: &mut Self::Context) -> Self::Result {
        self.id == msg.0
    }
}

impl Handler<GetHostId> for Session {
    type Result = String;
    fn handle(&mut self, _msg: GetHostId, _ctx: &mut Self::Context) -> Self::Result {
        self.host_id.to_string()
    }
}

impl Handler<AddPlayer> for Session {
    type Result = Result<SessionConnection, SessionError>;
    fn handle(&mut self, msg: AddPlayer, _ctx: &mut Self::Context) -> Self::Result {
        self.players
            .push(Player::new(msg.id, msg.username.clone(), msg.is_host));
        let conn = SessionConnection::new(msg.id, msg.session_addr);
        Ok(conn)
    }
}

impl Handler<GetSessionId> for Session {
    type Result = String;
    fn handle(&mut self, _msg: GetSessionId, _ctx: &mut Self::Context) -> Self::Result {
        self.id.to_string()
    }
}
impl Handler<BroadcastMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _ctx: &mut Self::Context) -> Self::Result {
        for conn in &self.connections {
            conn.do_send(SendToClient(msg.0.clone()))
        }
    }
}

impl Handler<AddConnection> for Session {
    type Result = ();

    fn handle(&mut self, msg: AddConnection, _ctx: &mut Self::Context) -> Self::Result {
        self.connections.push(msg.0);
    }
}
