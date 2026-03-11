use koad_core::identity::{Identity, Rank};

/// The Sandbox enforces security policies on commands before they are executed by the Spine.
pub struct Sandbox;

pub enum PolicyResult {
    Allowed,
    Denied(String),
}

impl Sandbox {
    /// Evaluates a command against the policies associated with the given identity.
    pub fn evaluate(identity: &Identity, command: &str) -> PolicyResult {
        // 1. Credential Bypass: If the identity carries a Sovereign Key, grant full access.
        if identity.access_keys.contains(&"GITHUB_ADMIN_PAT".to_string()) {
            return PolicyResult::Allowed;
        }

        // 2. Rank-based Bypass (Admiral/Captain)
        if identity.rank == Rank::Admiral || identity.rank == Rank::Captain {
            return PolicyResult::Allowed;
        }

        let role = identity.name.to_lowercase();

        // Developer Agent Policies
        if identity.tier == 2 || role == "developer" || role == "pm" || role == "reviewer" {
            return Self::evaluate_agent_policy(command);
        }

        // Chief Officer Policies (Sky / SLE Isolation)
        if identity.rank == Rank::Officer {
            return Self::evaluate_officer_policy(command);
        }

        // Compliance Agent Policies (Overseer)
        if role == "compliance" || role == "overseer" {
            return Self::evaluate_compliance_policy(command);
        }

        // Default Deny for unknown identities
        PolicyResult::Denied(format!("Unauthorized identity: {} (Rank: {:?})", identity.name, identity.rank))
    }

    fn evaluate_officer_policy(command: &str) -> PolicyResult {
        let cmd_lower = command.to_lowercase();

        // 1. SLE Isolation Mandate (The "Chain of Trust" Guardrail)
        // Officers managing the SCE (Skylinks Cloud Ecosystem) MUST NOT use production commands.
        let production_triggers = [
            "--project skylinks-prod",
            "--live",
            "stripe listen",
            "gcloud functions deploy",
        ];

        for trigger in production_triggers.iter() {
            if cmd_lower.contains(trigger)
                && !cmd_lower.contains("--test")
                && !cmd_lower.contains("--sandbox")
            {
                return PolicyResult::Denied(format!(
                    "SLE_ISOLATION_MANDATE: Attempt to execute production command '{}' without --test or --sandbox flag.",
                    trigger
                ));
            }
        }

        // 2. Blacklist and Sanctuary (Inherit from Agent Policy)
        Self::evaluate_agent_policy(command)
    }

    fn evaluate_compliance_policy(command: &str) -> PolicyResult {
        let cmd_lower = command.to_lowercase();
        // Allowed tools for compliance
        let allowed_tools = [
            "koad fleet board",
            "koad status",
            "doodskills/repo-clean.py",
            "git status",
            "ls ",
        ];

        for tool in allowed_tools.iter() {
            if cmd_lower.contains(tool) {
                return PolicyResult::Allowed;
            }
        }

        PolicyResult::Denied(
            "Compliance role is restricted to governance and inspection tools.".to_string(),
        )
    }

    fn evaluate_agent_policy(command: &str) -> PolicyResult {
        let cmd_lower = command.to_lowercase();

        // 1. Blacklist check (Commands)
        let blacklisted_commands = ["sudo ", "su ", "rm -rf /", "koad boot"];
        for blacklisted in blacklisted_commands.iter() {
            if cmd_lower.contains(blacklisted) {
                return PolicyResult::Denied(format!(
                    "Command contains blacklisted phrase: '{}'",
                    blacklisted
                ));
            }
        }

        // 2. Sanctuary Check (Paths)
        // Agents must not modify the KoadOS kernel or system roots.
        let protected_paths = [".koad-os", "/etc", "/var", "/root"];

        // Basic heuristic: if it looks like they are trying to touch a protected path
        // (This is a rudimentary check for MVP, a real parser would break down args)
        for path in protected_paths.iter() {
            if command.contains(path)
                && (command.contains("rm ")
                    || command.contains("mv ")
                    || command.contains("cp ")
                    || command.contains("echo ")
                    || command.contains(">")
                    || command.contains("nano ")
                    || command.contains("vim "))
            {
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

    #[test]
    fn test_admin_bypass() {
        assert!(matches!(
            Sandbox::evaluate("admin", "sudo rm -rf /"),
            PolicyResult::Allowed
        ));
    }

    #[test]
    fn test_developer_blacklist() {
        assert!(matches!(
            Sandbox::evaluate("developer", "sudo apt update"),
            PolicyResult::Denied(_)
        ));
        assert!(matches!(
            Sandbox::evaluate("pm", "rm -rf /home"),
            PolicyResult::Denied(_)
        ));
    }

    #[test]
    fn test_developer_sanctuary() {
        assert!(matches!(
            Sandbox::evaluate("developer", "rm -rf .koad-os/koad.db"),
            PolicyResult::Denied(_)
        ));
        assert!(matches!(
            Sandbox::evaluate("developer", "ls .koad-os"),
            PolicyResult::Allowed
        )); // Read is okay
    }

    #[test]
    fn test_developer_allowed() {
        assert!(matches!(
            Sandbox::evaluate("developer", "cargo build"),
            PolicyResult::Allowed
        ));
    }

    #[test]
    fn test_compliance_policy() {
        assert!(matches!(
            Sandbox::evaluate("compliance", "koad board status"),
            PolicyResult::Allowed
        ));
        assert!(matches!(
            Sandbox::evaluate("overseer", "git status"),
            PolicyResult::Allowed
        ));
        assert!(matches!(
            Sandbox::evaluate("compliance", "rm -rf /"),
            PolicyResult::Denied(_)
        ));
    }
}
