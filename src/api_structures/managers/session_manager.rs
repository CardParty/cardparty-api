use crate::api_structures::id::*;
use crate::api_structures::messages::{AddPlayer, CloseSession, GetHostId, GetSessionId};
use crate::api_structures::session::{Session, SessionCode};
use crate::api_structures::session_connection::SessionConnection;
use actix::{ spawn, Actor, Addr, Context, Handler};
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionManager {
    pub sessions: Arc<Mutex<Vec<Addr<Session>>>>,
    pub session_codes: Arc<Mutex<HashMap<SessionCode, SessionId>>>,
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
            session_codes: Arc::new(Mutex::new(HashMap::new())),
        }))
    }

    pub async fn init_session(
        &mut self,
        host_id: UserId,
        username: String,
    ) -> Result<(SessionId, SessionCode), SessionManagerError> {
        let sessions = self.sessions.lock().expect("Failed to lock sessions");
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

        drop(sessions);

        let mut code = SessionCode::gen();
        let mut codes = self.session_codes.lock().expect("Failed to lock sessions");
        while codes
            .keys()
            .into_iter()
            .collect::<Vec<&SessionCode>>()
            .contains(&&code)
        {
            code.regen();
        }

        let (addr, id) = Session::init(host_id, username, man_addr, code.clone()).await;

        codes.insert(code.clone(), id);

        self.sessions.lock().expect("Failed to lock sessions").push(addr.clone());
        Ok((id, code.clone()))
    }
    pub fn unwrap_code(&self, code: SessionCode) -> Option<String> {
        let session_codes = self.session_codes.lock().unwrap();

        if let Some(code) = session_codes.get(&code) {
            Some(code.to_string())
        } else {
            None
        }
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
    pub async fn get_games(&self) -> Vec<Uuid> {
        let sessions = self.sessions.clone();

        // Spawning an async block to run the tasks
        let ids = spawn(async move {
            let session_ids: Vec<Option<Uuid>> = {
                let sessions = sessions.lock().unwrap();
                let futures = sessions.iter().map(|session| {
                    let session = session.clone();
                    async move {
                        let session_id_res = session.send(GetSessionId()).await.ok();
                        session_id_res.and_then(|id| Uuid::parse_str(&id).ok())
                    }
                });
                join_all(futures).await
            };

            session_ids
                .into_iter()
                .filter_map(|id| id)
                .collect::<Vec<Uuid>>()
        })
        .await
        .unwrap();

        let ret_ids = ids.clone();
        ret_ids
    }
}

impl Handler<CloseSession> for SessionManager {
    type Result = ();

    fn handle(&mut self, msg: CloseSession, _ctx: &mut Self::Context) -> Self::Result {
        let sessions = self.sessions.clone();
        let msg_id = msg.0;

        spawn(async move {
            let session_ids: Vec<Option<Uuid>> = {
                let sessions = sessions.lock().unwrap();
                let futures = sessions.iter().map(|session| {
                    let session = session.clone();
                    async move {
                        let session_id_res = session.send(GetSessionId()).await.ok();

                        session_id_res.and_then(|id| Uuid::parse_str(&id).ok())
                    }
                });
                join_all(futures).await
            };

            let mut sessions = sessions.lock().unwrap();
            sessions.retain(|_session| {
                if let Some(session_id) = session_ids.iter().find(|id| id.is_some()) {
                    *session_id.as_ref().unwrap() != msg_id
                } else {
                    true
                }
            });
        });
    }
}
