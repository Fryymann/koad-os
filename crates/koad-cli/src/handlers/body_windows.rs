//! `koad body windows`: link Claude Code for Windows to this Citadel.
//!
//! Spec: docs/superpowers/specs/2026-09-27-windows-body-bridge-design.md

use serde_json::{json, Value};

/// Identifies the KoadOS SessionStart hook in Claude Code settings.
pub const HOOK_MARKER: &str = "koad-agent anchor";

/// Name of the MCP server registered in Claude Code for Windows.
pub const MCP_NAME: &str = "citadel-memory";

/// Skills installed on the Windows side.
pub const WINDOWS_SKILLS: [&str; 2] = ["cass-recall", "cass-search"];

/// SessionStart hook command: prints the agent's anchor through WSL.
pub fn hook_command(distro: &str, koad_home: &str, agent: &str) -> String {
    format!("wsl.exe -d {distro} -e {koad_home}/bin/koad-wsl-env koad-agent anchor {agent} --body windows")
}

/// MCP server definition for `claude.exe mcp add-json`.
pub fn mcp_server_json(distro: &str, koad_home: &str, agent: &str) -> Value {
    json!({
        "type": "stdio",
        "command": "wsl.exe",
        "args": ["-d", distro, "-e", format!("{koad_home}/bin/koad-wsl-env"), "koad-mcp-stdio", agent]
    })
}

fn is_koad_group(group: &Value) -> bool {
    group["hooks"].as_array().is_some_and(|hooks| {
        hooks.iter().any(|h| {
            h["command"]
                .as_str()
                .is_some_and(|c| c.contains(HOOK_MARKER))
        })
    })
}

/// Add (or replace) the KoadOS SessionStart hook, preserving everything else.
pub fn merge_session_hook(mut settings: Value, command: &str) -> anyhow::Result<Value> {
    let obj = settings
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("settings.json is not a JSON object"))?;
    let hooks = obj
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("settings.json `hooks` is not an object"))?;
    let groups = hooks
        .entry("SessionStart")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| anyhow::anyhow!("settings.json `hooks.SessionStart` is not an array"))?;
    groups.retain(|g| !is_koad_group(g));
    groups.push(json!({"hooks": [{"type": "command", "command": command, "timeout": 30}]}));
    Ok(settings)
}

/// Remove the KoadOS SessionStart hook, dropping containers left empty.
pub fn remove_session_hook(mut settings: Value) -> Value {
    let mut drop_hooks = false;
    if let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) {
        if let Some(groups) = hooks.get_mut("SessionStart").and_then(Value::as_array_mut) {
            groups.retain(|g| !is_koad_group(g));
            if groups.is_empty() {
                hooks.remove("SessionStart");
            }
        }
        drop_hooks = hooks.is_empty();
    }
    if drop_hooks {
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("hooks");
        }
    }
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    const CMD: &str =
        "wsl.exe -d Ubuntu -e /h/.citadel-jupiter/bin/koad-wsl-env koad-agent anchor clyde --body windows";

    #[test]
    fn hook_command_runs_the_anchor_through_the_env_wrapper() {
        assert_eq!(hook_command("Ubuntu", "/h/.citadel-jupiter", "clyde"), CMD);
    }

    #[test]
    fn mcp_server_runs_the_stdio_launcher_through_wsl() {
        assert_eq!(
            mcp_server_json("Ubuntu", "/h/.citadel-jupiter", "clyde"),
            json!({
                "type": "stdio",
                "command": "wsl.exe",
                "args": ["-d", "Ubuntu", "-e", "/h/.citadel-jupiter/bin/koad-wsl-env", "koad-mcp-stdio", "clyde"]
            })
        );
    }

    #[test]
    fn merge_adds_the_hook_and_keeps_other_settings_and_hooks() {
        let other = json!({"hooks": [{"type": "command", "command": "echo mine"}]});
        let settings =
            json!({"effortLevel": "high", "hooks": {"SessionStart": [other.clone()], "Stop": []}});
        let merged = merge_session_hook(settings, CMD).unwrap();
        assert_eq!(merged["effortLevel"], "high");
        assert_eq!(merged["hooks"]["Stop"], json!([]));
        let groups = merged["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], other);
        assert_eq!(groups[1]["hooks"][0]["command"], CMD);
        assert_eq!(groups[1]["hooks"][0]["timeout"], 30);
    }

    #[test]
    fn merge_is_idempotent() {
        let once = merge_session_hook(json!({}), CMD).unwrap();
        let twice = merge_session_hook(once.clone(), CMD).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn remove_restores_the_original_settings() {
        let original = json!({"effortLevel": "high", "mcpServers": {}});
        let merged = merge_session_hook(original.clone(), CMD).unwrap();
        assert_eq!(remove_session_hook(merged), original);
    }

    #[test]
    fn merge_refuses_non_object_settings() {
        assert!(merge_session_hook(json!([1, 2]), CMD).is_err());
    }
}
