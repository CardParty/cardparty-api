use crate::api_structures::session::Session;

// Globalny stan api
// dodawac tu tylko najważniejsze żeczy ktore MUSZĄ byc w globalnym stanie
pub struct ApiState {
    sessions: Vec<Session>,
}

impl ApiState {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }
    pub fn add_session(&mut self, session: Session) {
        self.sessions.push(session)
    }
}
