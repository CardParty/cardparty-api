use actix::Message;

use super::id::{SessionId, UserId};

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct TestMessage(pub String);

#[derive(Message, Debug)]
#[rtype(result = "bool")]
pub struct VerifyExistance(pub SessionId);

#[derive(Message, Debug)]
#[rtype(result = "String")]
pub struct GetHostId();
