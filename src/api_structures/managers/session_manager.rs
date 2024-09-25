use crate::api_structures::messages::{AddPlayer, CloseSession, GetHostId, GetSessionId};
use crate::api_structures::session::Session;
use crate::api_structures::session_connection::SessionConnection;
use crate::api_structures::{id::*};
use actix::{Actor, Addr, Context, Handler};
use futures::executor::block_on;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionManager {
    pub sessions: Arc<Mutex<Vec<Addr<Session>>>>,
}

#[derive(Debug)]
pub enum SessionManagerError {
    UserSessionInstanceAlreadyExists,
    NoActiveSessions,
}

impl Actor for SessionManager {
    type Context = Context<Self>;
}

impl SessionManager {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            sessions: Arc::new(Mutex::new(Vec::new())),
        }))
    }

    pub async fn init_session(
        &mut self,
        host_id: UserId,
        username: String,
    ) -> Result<SessionId, SessionManagerError> {
        let sessions = self.sessions.lock().expect("aaa");
        for session in sessions.iter() {
            let session_host_id = session
                .send(GetHostId())
                .await
                .expect("Failed to get host id");
            if Uuid::parse_str(&session_host_id).expect("Failed to parse UUID") == host_id {
                return Err(SessionManagerError::UserSessionInstanceAlreadyExists);
            }
        }

        let man_addr = self.clone().start();

        let (addr, id) = Session::init(host_id, username, man_addr).await;
        drop(sessions);

        self.sessions.lock().expect("sigma").push(addr.clone());
        Ok(id)
    }

    pub async fn join_session(
        &mut self,
        session_id: SessionId,
        user_id: UserId,
        username: String,
    ) -> Option<SessionConnection> {
        let sessions = self.sessions.lock().expect("Failed to lock sessions");
        for session in sessions.iter() {
            let session_id_res = session
                .send(GetSessionId())
                .await
                .expect("Failed to get session id");
            if Uuid::parse_str(&session_id_res).expect("Failed to parse UUID") == session_id {
                let conn = session
                    .send(AddPlayer {
                        id: user_id,
                        username,
                        is_host: false,
                        session_addr: session.clone(),
                    })
                    .await
                    .expect("Failed to add player")
                    .expect("Failed to add player");

                return Some(conn);
            }
        }
        None
    }
}

impl Handler<CloseSession> for SessionManager {
    type Result = ();

    fn handle(&mut self, msg: CloseSession, _ctx: &mut Self::Context) -> Self::Result {
        println!("Closing session: {:?}", msg.0);
        let sessions = Arc::clone(&self.sessions);
        actix::spawn(async move {
            let mut sessions = sessions.lock().expect("Failed to lock sessions");
            sessions.retain(|session| {
                let session_id = block_on(async {
                    session
                        .send(GetSessionId())
                        .await
                        .expect("Failed to get session ID")
                });

                let parsed_id = Uuid::parse_str(&session_id).expect("Failed to parse UUID");
                parsed_id != msg.0
            });
        });
        println!("Session closed: {:?}", msg.0);
    }

}
