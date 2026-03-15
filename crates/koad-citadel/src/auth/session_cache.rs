//! Session Cache
//!
//! In-memory Redis-backed store of active agent sessions, keyed by session ID.

use crate::state::docking::DockingState;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A single active agent session record held in the in-memory cache.
#[derive(Debug, Clone)]
pub struct SessionRecord {
    /// The name of the agent that owns this session.
    pub agent_name: String,
    /// Current docking lifecycle state of the agent.
    pub state: DockingState,
    /// Timestamp of the most recent heartbeat received.
    pub last_heartbeat: DateTime<Utc>,
    /// Identifier of the body (shell process) this session is tethered to.
    pub body_id: String,
    /// Secret token used to authenticate gRPC requests from this session.
    pub session_token: String,
    /// Workspace level string (e.g. `"CITADEL"`, `"OUTPOST"`).
    pub level: String,
}

/// Thread-safe map of session ID → [`SessionRecord`] for all active sessions.
pub type ActiveSessions = Arc<Mutex<HashMap<String, SessionRecord>>>;
