use crate::identity::Identity;
use crate::types::{EnvironmentType, HotContextChunk};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an active agent session within the KoadOS ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub session_id: String,
    pub identity: Identity,
    pub environment: EnvironmentType,
    pub context: ProjectContext,
    pub status: String,
    pub last_heartbeat: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    /// Dynamic context chunks currently loaded in the agent's memory.
    #[serde(default)]
    pub hot_context: Vec<HotContextChunk>,
}

/// Defines the project-level boundaries and resources available to a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_name: String,
    pub root_path: String,
    pub allowed_paths: Vec<String>,
    pub stack: Vec<String>,
}

impl AgentSession {
    pub fn new(
        session_id: String,
        identity: Identity,
        environment: EnvironmentType,
        context: ProjectContext,
    ) -> Self {
        Self {
            session_id,
            identity,
            environment,
            context,
            status: "active".to_string(),
            last_heartbeat: Utc::now(),
            metadata: HashMap::new(),
            hot_context: Vec::new(),
        }
    }

    pub fn is_active(&self, timeout_secs: i64) -> bool {
        let now = Utc::now();
        (now - self.last_heartbeat).num_seconds() < timeout_secs
    }
}
