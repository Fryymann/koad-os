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
    /// Unique terminal/body identifier — one UUID per shell session, generated at boot.
    #[serde(default)]
    pub body_id: String,
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
        body_id: String,
    ) -> Self {
        Self {
            session_id,
            identity,
            environment,
            context,
            status: "active".to_string(),
            last_heartbeat: Utc::now(),
            metadata: HashMap::new(),
            body_id,
            hot_context: Vec::new(),
        }
    }

    pub fn is_active(&self, timeout_secs: i64) -> bool {
        let now = Utc::now();
        (now - self.last_heartbeat).num_seconds() < timeout_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Identity, Rank};
    use crate::types::EnvironmentType;
    use chrono::Duration;

    fn make_session() -> AgentSession {
        AgentSession::new(
            "sess-001".to_string(),
            Identity {
                name: "test-agent".to_string(),
                rank: Rank::Crew,
                permissions: vec![],
                access_keys: vec![],
                tier: 3,
            },
            EnvironmentType::Wsl,
            ProjectContext {
                project_name: "test-project".to_string(),
                root_path: "/home/user/.koad-os/agents/KAPVs/clyde/project".to_string(),
                allowed_paths: vec![
                    "/home/user/.koad-os/agents/KAPVs/clyde/".to_string(),
                ],
                stack: vec!["rust".to_string()],
            },
            "body-abc".to_string(),
        )
    }

    #[test]
    fn new_initializes_with_active_status() {
        let session = make_session();
        assert_eq!(session.status, "active", "New session should have 'active' status");
        assert!(session.metadata.is_empty(), "Metadata should start empty");
        assert!(session.hot_context.is_empty(), "Hot context should start empty");
        assert_eq!(session.body_id, "body-abc");
        assert_eq!(session.session_id, "sess-001");
    }

    #[test]
    fn is_active_returns_true_for_fresh_session() {
        let session = make_session();
        // Heartbeat is Utc::now() from construction — well within any reasonable timeout
        assert!(session.is_active(60), "Fresh session should be active within a 60s window");
    }

    #[test]
    fn is_active_returns_false_when_heartbeat_exceeds_timeout() {
        let mut session = make_session();
        session.last_heartbeat = Utc::now() - Duration::seconds(120);
        assert!(
            !session.is_active(60),
            "Session with heartbeat 120s ago should be inactive with 60s timeout"
        );
    }

    #[test]
    fn is_active_boundary_just_inside_timeout_is_active() {
        let mut session = make_session();
        // 59 seconds elapsed, 60s timeout — strictly less than, so still active
        session.last_heartbeat = Utc::now() - Duration::seconds(59);
        assert!(
            session.is_active(60),
            "Session 59s old with 60s timeout should still be active"
        );
    }

    #[test]
    fn is_active_boundary_at_exact_timeout_is_inactive() {
        let mut session = make_session();
        // Exactly 60s elapsed with 60s timeout — uses < (not <=), so this is inactive
        session.last_heartbeat = Utc::now() - Duration::seconds(60);
        assert!(
            !session.is_active(60),
            "Session exactly at the timeout boundary should be considered inactive (strict <)"
        );
    }

    #[test]
    fn agent_path_uses_agents_folder() {
        let session = make_session();
        // Verify path convention: agents/<name> (no dot prefix on the agents dir)
        assert!(
            session.context.root_path.contains("agents/KAPVs/clyde"),
            "Agent path should use 'agents/KAPVs/clyde'"
        );
        assert!(
            !session.context.root_path.contains("agents/KAPVs/clyde"),
            "Dotted agents/ prefix is no longer valid — use agents/ instead"
        );
    }
}
