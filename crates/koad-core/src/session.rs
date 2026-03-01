use serde::{Deserialize, Serialize};
use crate::identity::Identity;
use crate::types::EnvironmentType;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Represents an active agent session within the KoadOS ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub session_id: String,
    pub identity: Identity,
    pub environment: EnvironmentType,
    pub context: ProjectContext,
    pub last_heartbeat: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
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
    pub fn new(session_id: String, identity: Identity, environment: EnvironmentType, context: ProjectContext) -> Self {
        Self {
            session_id,
            identity,
            environment,
            context,
            last_heartbeat: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn is_active(&self, timeout_secs: i64) -> bool {
        let now = Utc::now();
        (now - self.last_heartbeat).num_seconds() < timeout_secs
    }
}
