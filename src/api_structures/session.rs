use crate::api_structures::id::*;
use crate::api_structures::messages::BroadcastMessage;
use crate::api_structures::messages::TestMessage;
use actix::{Actor, Addr, Context, Handler, StreamHandler};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use uuid::Uuid;

use super::messages::{
    AddConnection, AddPlayer, ConnectWithSession, GetHostId, GetSessionId, SendToClient,
    VerifyExistence,
};
#[derive(Debug)]
pub enum SessionError {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Player {
    username: String,
    pub id: UserId,
    is_host: bool,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SessionFlag {
    AwatingHost,
    Lobby,
    Game,
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
    is_admin: bool,
}

impl SessionConnection {
    pub fn new(user_id: UserId, session: Addr<Session>, is_admin: bool) -> Self {
        Self {
            user_id,
            session,
            id: Uuid::new_v4(),
            is_admin,
        }
    }
}

impl Actor for SessionConnection {
    type Context = ws::WebsocketContext<Self>;
}
#[derive(Deserialize, Debug)]
struct Meta {
    packet_type: String,
}

#[derive(Deserialize, Debug)]
struct Packet {
    meta: Meta,
    data: Value, // `data` is deserialized as `serde_json::Value`
}
#[derive(Deserialize, Debug)]
struct UpdateData {
    id: u32,
    status: String,
}

#[derive(Deserialize, Debug)]
struct CreateData {
    name: String,
    value: u64,
}
pub fn deserialize_json(json: &str) {
    let packet: Packet = serde_json::from_str(json).unwrap();
    print!("Deserialized packet: {:?}", packet);
    match packet.meta.packet_type.as_str() {
        "update" => {
            let update_data: UpdateData = from_value(packet.data).unwrap();
            println!("Deserialized as UpdateData: {:?}", update_data);
        }
        "create" => {
            let create_data: CreateData = from_value(packet.data).unwrap();
            println!("Deserialized as CreateData: {:?}", create_data);
        }
        _ => {
            println!("Unknown packet type");
        }
    }
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
                        print!(
                            "Deserialized packet: {:?}",
                            deserialize_json(split[1..].join(" ").as_str())
                        );
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
    pub session_flag: SessionFlag,
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
            session_flag: SessionFlag::AwatingHost,
        }
        .start();

        (addr, id)
    }
}

impl Handler<TestMessage> for Session {
    type Result = ();
    fn handle(&mut self, _msg: TestMessage, _ctx: &mut Self::Context) -> Self::Result {}
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
        if self.players.len() == 1 {
            let conn = SessionConnection::new(msg.id, msg.session_addr, true);
            self.session_flag = SessionFlag::Lobby;
            Ok(conn)
        } else {
            let conn = SessionConnection::new(msg.id, msg.session_addr, false);
            self.session_flag = SessionFlag::Lobby;
            Ok(conn)
        }
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
