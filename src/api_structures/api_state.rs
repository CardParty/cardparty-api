// src/api_structures/api_state.rs
use std::sync::{Arc, Mutex};
use crate::api_structures::managers::session_manager::SessionManager;

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