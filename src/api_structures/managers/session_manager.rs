use actix::{Addr};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::api_structures::id::*;
use crate::api_structures::messages::{AddPlayer, GetHostId, GetSessionId};
use crate::api_structures::session::{Session, SessionConnection};

pub struct SessionManager {
    pub sessions: Vec<Addr<Session>>,
}
#[derive(Debug)]
pub enum SessionManagerError {
    UserSessionInstanceAlreadyExists,
    NoActiveSessions,
}

impl SessionManager {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            sessions: Vec::new(),
        }))
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

        self.sessions.push(addr.clone());
        Ok(id)
    }

    pub async fn join_session(
        &mut self,
        session_id: SessionId,
        user_id: UserId,
        username: String,
    ) -> Option<SessionConnection> {
        for session in &self.sessions {
            let session_id_res = session
                .send(GetSessionId())
                .await
                .expect("getting session id failed");
            if Uuid::parse_str(&session_id_res).expect("parsing uuid failed") == session_id {
                let conn = session
                    .send(AddPlayer {
                        id: user_id,
                        username,
                        is_host: false,
                        session_addr: session.clone(),
                    })
                    .await
                    .unwrap()
                    .unwrap();
                return Some(conn);
            }
        }
        None
    }
}
