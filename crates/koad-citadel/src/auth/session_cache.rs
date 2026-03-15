use crate::state::docking::DockingState;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub agent_name: String,
    pub state: DockingState,
    pub last_heartbeat: DateTime<Utc>,
    pub body_id: String,
    pub session_token: String,
    pub level: String,
}

pub type ActiveSessions = Arc<Mutex<HashMap<String, SessionRecord>>>;
