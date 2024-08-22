use actix::{Addr, Message};

use super::{
    id::{SessionId, UserId},
    session::Session,
};
use crate::api_structures::session::{SessionConnection, SessionError}; // Import the missing type SessionConnection
// TEMPLATE
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct TEMPLATE_DONT_USE();

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct TestMessage(pub String);

#[derive(Message, Debug)]
#[rtype(result = "bool")]
pub struct VerifyExistance(pub SessionId);

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
