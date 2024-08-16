use std::sync::{Arc, Mutex};
use crate::api_structures::managers::session_manager::SessionManager;

// Globalny stan api
// dodawac tu tylko najważniejsze żeczy ktore MUSZĄ byc w globalnym stanie
pub struct ApiState {
    pub session_manager: Arc<Mutex<SessionManager>>,
}

impl ApiState {
    pub fn new() -> Self {
        Self {
            session_manager: SessionManager::new(),
        }
    }
}
