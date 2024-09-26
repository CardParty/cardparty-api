use actix::{Addr, Message};
use uuid::Uuid;

use super::{
    id::{self, SessionId, UserId},
    packet_parser::{Packet, PacketResponse},
    session::Session,
};
use crate::api_structures::packet_parser::PacketError;
use crate::api_structures::session::SessionError;
use crate::api_structures::session_connection::SessionConnection;
#[derive(Message, Debug)]
#[rtype(result = "()")]
#[allow(dead_code)]
pub struct TemplateDontUse();

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct TestMessage(pub String);

#[derive(Message, Debug)]
#[rtype(result = "bool")]
pub struct VerifyExistence(pub SessionId);

#[derive(Message, Debug)]
#[rtype(result = "String")]
pub struct GetHostId();

#[derive(Message, Debug)]
#[rtype(result = "Result<SessionConnection, SessionError>")]
pub struct AddPlayer {
    pub id: UserId,
    pub username: String,
    pub is_host: bool,
    pub session_addr: Addr<Session>,
}

#[derive(Message, Debug)]
#[rtype(result = "String")]
pub struct GetSessionId();

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct BroadcastMessage(pub String);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct AddConnection(pub Addr<SessionConnection>);

// session connection fucking yk TRRRTETRTRTRTRTRTRTRTRTRTRTRRTRTRTRTTR ( im losing my mind :) )

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct SendToClient(pub String);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ConnectWithSession(pub Addr<SessionConnection>);

#[derive(Message, Debug)]
#[rtype(result = "Result<PacketResponse, PacketError>")]
pub struct SendPacket(pub Packet);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct CloseSessionConnection;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct CloseSession(pub Uuid);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct PlayerUpdate(pub Vec<String>);
