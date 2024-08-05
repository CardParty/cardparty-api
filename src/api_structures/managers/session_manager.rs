use crate::api_structures::id::Id;
use crate::api_structures::session::Session;
pub struct SessionManager {
    pub sessions: Vec<Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }
}

impl SessionManager {
    pub fn create_session(&mut self, session: Session) {
        self.sessions.push(session)
    }
    pub fn join_session(&self, unvalid_session_id: Id) {
        if let Some(id) = unvalid_session_id.verify_session_id() {
        } else {
            log::error!("Failed To Join session")
        }
    }
    pub fn verify_session() {
        todo!()
    }

    pub fn activate_session() {}
}
