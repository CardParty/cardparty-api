use crate::api_structures::messages::{AddPlayer, CloseSession, GetHostId, GetSessionId};
use crate::api_structures::session::Session;
use crate::api_structures::session_connection::SessionConnection;
use crate::api_structures::{id::*, session};
use actix::{Actor, Addr, Context, Handler};
use futures::executor::block_on;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionManager {
    pub sessions: Vec<Addr<Session>>,
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

        let man_addr = self.clone().start();

        let (addr, id) = Session::init(host_id, username, man_addr).await;

        self.sessions.push(addr.clone());
        Ok(id)
    }

    pub async fn join_session(
        &mut self,
        session_id: SessionId,
        user_id: UserId,
        username: String,
    ) -> Option<SessionConnection> {
        println!("Joining session: {:?}", session_id);
        println!("Sessions: {:?}", self.sessions);
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

impl Handler<CloseSession> for SessionManager {
    type Result = ();

    fn handle(&mut self, msg: CloseSession, ctx: &mut Self::Context) -> Self::Result {
        println!("Closing session: {:?}", msg.0);
        println!("Sessions pointer address {:p}", &self.sessions);
        self.sessions.retain(|session| {
            let id = block_on(async {
                let s = session
                    .send(GetSessionId())
                    .await
                    .expect("jaja mnie swędzą");
                Uuid::parse_str(&s).expect("uuid wykurwiło sie XDDD")
            });
            println!("{}", msg.0 == id);
            msg.0 == id
        });
        println!("{}", self.sessions.len());
        println!("{:?}", self.sessions);

    }
}
