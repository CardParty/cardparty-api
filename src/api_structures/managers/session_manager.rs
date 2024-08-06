use actix::fut::ok;
use actix::{Actor, Addr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api_structures::id::*;
use crate::api_structures::messages::{GetHostId, VerifyExistance};
use crate::api_structures::session::Session;

pub struct SessionManager {
    pub sessions: Vec<Addr<Session>>,
}

enum SessionManagerError {
    UserSessionInstanceAlreadyExists,
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
    ) -> Result<(), SessionManagerError> {
        for session in &self.sessions {
            let session_host_id = session
                .send(GetHostId())
                .await
                .expect("getting host id failed");
            if Uuid::parse_str(&session_host_id).expect("parsing uuid failed") == host_id {
                return Err(SessionManagerError::UserSessionInstanceAlreadyExists);
            }
        }

        &mut self.sessions.push(Session::init(host_id, username));
        Ok(())
    }
}
