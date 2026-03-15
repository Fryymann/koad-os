//! # KoadOS Command Sandbox
//!
//! Enforces security policies on agent commands before they are executed.
//! The Sandbox is config-driven, replacing the hardcoded legacy Spine logic with 
//! dynamic policies defined in `kernel.toml`.
//!
//! ## Principles
//! - **Sanctuary Rule**: Protects sensitive system paths from modification.
//! - **Blacklist Enforcement**: Blocks dangerous primitives (sudo, rm -rf).
//! - **Identity Awareness**: Allows administrative bypass for Admiral/Captain ranks.

use koad_core::config::KoadConfig;
use tracing::warn;

/// Result of a sandbox policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyResult {
    /// Command is permitted.
    Allowed,
    /// Command is denied with a reason.
    Denied(String),
}

/// The Sandbox enforces security policies based on configuration and identity.
pub struct Sandbox {
    config: KoadConfig,
}

impl Sandbox {
    /// Create a new sandbox with the given configuration.
    pub fn new(config: KoadConfig) -> Self {
        Self { config }
    }

    /// Evaluates a command against the active policies.
    pub fn evaluate(&self, agent_name: &str, agent_rank: &str, command: &str) -> PolicyResult {
        if !self.config.sandbox.enabled {
            return PolicyResult::Allowed;
        }

        // 1. Administrative Bypass (Admiral/Captain)
        if agent_rank == "Admiral" || agent_rank == "Captain" {
            return PolicyResult::Allowed;
        }

        let cmd_lower = command.to_lowercase();

        // 2. Blacklist Check
        for blacklisted in &self.config.sandbox.blacklist {
            if cmd_lower.contains(&blacklisted.to_lowercase()) {
                warn!(agent = %agent_name, command = %command, match = %blacklisted, "Sandbox: Blacklist violation");
                return PolicyResult::Denied(format!(
                    "Command contains blacklisted phrase: '{}'",
                    blacklisted
                ));
            }
        }

        // 3. Sanctuary Check (Paths)
        if let PolicyResult::Denied(reason) = self.check_sanctuary(agent_name, command) {
            return PolicyResult::Denied(reason);
        }

        PolicyResult::Allowed
    }

    fn check_sanctuary(&self, agent_name: &str, command: &str) -> PolicyResult {
        // Basic heuristic: if it looks like they are trying to touch a protected path
        // (Similar to legacy logic, but using config paths)
        for path in &self.config.sandbox.sanctuary {
            if command.contains(path)
                && (command.contains("rm ")
                    || command.contains("mv ")
                    || command.contains("cp ")
                    || command.contains("echo ")
                    || command.contains(">")
                    || command.contains("nano ")
                    || command.contains("vim "))
            {
                warn!(agent = %agent_name, command = %command, path = %path, "Sandbox: Sanctuary violation");
                return PolicyResult::Denied(format!(
                    "Attempt to modify protected path: '{}'",
                    path
                ));
            }
        }
        PolicyResult::Allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use koad_core::config::KoadConfig;

    fn mock_config() -> KoadConfig {
        KoadConfig::load().unwrap_or_else(|_| {
            // Fallback for environment without real config
            serde_json::from_str(
                r#"{
                "home": "/tmp/.koad-os",
                "sandbox": {
                    "enabled": true,
                    "blacklist": ["sudo ", "su "],
                    "sanctuary": [".koad-os"]
                }
            }"#,
            )
            .unwrap()
        })
    }

    #[test]
    fn test_sandbox_denies_blacklist() {
        let sandbox = Sandbox::new(mock_config());
        let res = sandbox.evaluate("test-agent", "Crew", "sudo rm -rf /");
        assert!(matches!(res, PolicyResult::Denied(_)));
    }

    #[test]
    fn test_sandbox_denies_sanctuary() {
        let sandbox = Sandbox::new(mock_config());
        let res = sandbox.evaluate("test-agent", "Crew", "rm -rf .koad-os/koad.db");
        assert!(matches!(res, PolicyResult::Denied(_)));
    }

    #[test]
    fn test_sandbox_allows_safe_command() {
        let sandbox = Sandbox::new(mock_config());
        let res = sandbox.evaluate("test-agent", "Crew", "cargo build");
        assert_eq!(res, PolicyResult::Allowed);
    }

    #[test]
    fn test_sandbox_admin_bypass() {
        let sandbox = Sandbox::new(mock_config());
        let res = sandbox.evaluate("tyr", "Captain", "sudo apt update");
        assert_eq!(res, PolicyResult::Allowed);
    }
}
