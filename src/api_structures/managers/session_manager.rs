use std::ops::Add;
use std::str::FromStr;

use actix::fut::ok;
use actix::{Actor, Addr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api_structures::id::*;
use crate::api_structures::messages::{AddPlayer, GetHostId, GetSessionId, VerifyExistance};
use crate::api_structures::session::{Session, SessionConnection, SessionError};

pub struct SessionManager {
    pub sessions: Vec<Addr<Session>>,
}
#[derive(Debug)]
pub enum SessionManagerError {
    UserSessionInstanceAlreadyExists,
    NoActiveSessions,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    pub async fn init_session(
        &mut self,
        host_id: UserId,
        username: String,
    ) -> Result<SessionId, SessionManagerError> {
        for session in &self.sessions {
            let session_host_id = session
                .send(GetHostId())
                .await
                .expect("getting host id failed");
            if Uuid::parse_str(&session_host_id).expect("parsing uuid failed") == host_id {
                return Err(SessionManagerError::UserSessionInstanceAlreadyExists);
            }
        }

        let (addr, id) = Session::init(host_id, username).await;

        &mut self.sessions.push(addr);
        log::info!("created session: {}", id);
        Ok(id)
    }

    pub async fn join_session(
        &mut self,
        session_id: SessionId,
        user_id: UserId,
        username: String,
    ) -> Option<SessionConnection> {
        if let Some(addr) = self.sessions.get(0) {
            let conn = addr
                .send(AddPlayer {
                    id: user_id,
                    username: username,
                    is_host: true,
                    session_addr: addr.to_owned(),
                })
                .await
                .unwrap()
                .unwrap();

            return Some(conn);
        } else {
            None
        }
    }
}
