use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A high-level directive within the KoadOS ecosystem.
/// Intents represent the "What" and "Who" of a system action,
/// abstracting away the underlying IPC transport (Redis, gRPC, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Intent {
    /// Execute a shell command within the KoadOS environment.
    Execute(ExecuteIntent),
    /// Dispatch a request to a KoadOS Skill.
    Skill(SkillIntent),
    /// Perform an action related to an Agent session.
    Session(SessionIntent),
    /// A system-level lifecycle event.
    System(SystemIntent),
    /// A governance or compliance action.
    Governance(GovernanceIntent),
}

/// Details for a shell command execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecuteIntent {
    /// The name of the identity executing the command (e.g., "admin", "dood").
    pub identity: String,
    /// The raw command string to be executed by the shell.
    pub command: String,
    /// Optional arguments passed to the command.
    #[serde(default)]
    pub args: Vec<String>,
    /// The directory in which the command should be executed.
    pub working_dir: Option<String>,
    /// Environment variables to inject into the command process.
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

/// Details for dispatching to a KoadOS Skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillIntent {
    /// The unique identifier of the skill.
    pub skill_id: String,
    /// The specific action the skill should perform.
    pub action: String,
    /// Optional parameters passed as a JSON object.
    pub params: serde_json::Value,
}

/// Actions related to Agent session lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionIntent {
    /// The unique session ID.
    pub session_id: String,
    /// The action to perform (e.g., heartbeat, stop).
    pub action: SessionAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionAction {
    /// Initialize a new session.
    Start,
    /// Update the last-active timestamp.
    Heartbeat,
    /// Gracefully terminate the session.
    Stop,
}

/// System-level management actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemIntent {
    /// The system action to perform.
    pub action: SystemAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SystemAction {
    /// Reboot the KoadOS Kernel (spine).
    Reboot,
    /// Trigger a full synchronization with Notion.
    SyncNotion,
    /// Clear system caches and temporary files.
    PruneCache,
}

/// Details for a governance or compliance action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GovernanceIntent {
    /// The specific governance action to perform.
    pub action: GovernanceAction,
    /// Optional target or scope for the action.
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceAction {
    /// Clean the repository of transient state files (e.g., logs, PIDs).
    Clean,
    /// Audit the repository and system health.
    Audit,
    /// Synchronize local metadata with the Project Board.
    Sync,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_intent_execute_serialization() {
        let intent = Intent::Execute(ExecuteIntent {
            identity: "admin".to_string(),
            command: "ls".to_string(),
            args: vec!["-la".to_string()],
            working_dir: Some(std::env::var("HOME").unwrap_or_else(|_| "/home/ideans".to_string())),
            env_vars: HashMap::new(),
        });

        let serialized = serde_json::to_string(&intent).unwrap();
        let deserialized: Intent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(intent, deserialized);
    }

    #[test]
    fn test_intent_legacy_compatibility() {
        // Ensure that we can still deserialize the simple format if we use a helper or untagged
        // However, with [serde(tag = "type")], we expect the new format.
        // This test confirms the NEW format works.
        let raw_json = json!({
            "type": "execute",
            "data": {
                "identity": "admin",
                "command": "ls"
            }
        });
        let deserialized: Intent = serde_json::from_value(raw_json).unwrap();
        if let Intent::Execute(exec) = deserialized {
            assert_eq!(exec.identity, "admin");
            assert_eq!(exec.command, "ls");
            assert!(exec.args.is_empty());
        } else {
            panic!("Expected Execute intent");
        }
    }
}
