//! `koad body windows`: link Claude Code for Windows to this Citadel.
//!
//! Spec: docs/superpowers/specs/2026-09-27-windows-body-bridge-design.md

use serde_json::{json, Value};

/// Identifies the KoadOS SessionStart hook in Claude Code settings.
pub const HOOK_MARKER: &str = "/bin/koad-wsl-env koad-agent anchor ";

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

/// Whether a single hook entry (an object with a `command` field) is one
/// KoadOS installed, as opposed to a user's own hook that happens to share
/// a `SessionStart` group — or to merely mention the marker text.
fn is_koad_hook(hook: &Value) -> bool {
    hook["command"]
        .as_str()
        .is_some_and(|c| c.starts_with("wsl.exe -d ") && c.contains(HOOK_MARKER))
}

/// Strip KoadOS hook entries out of each group's `hooks` array in place,
/// dropping a group entirely once its `hooks` array becomes empty. Groups
/// that have no `hooks` array at all (malformed or simply different) are
/// left untouched.
fn strip_koad_hooks(groups: &mut Vec<Value>) {
    for group in groups.iter_mut() {
        if let Some(hooks) = group.get_mut("hooks").and_then(Value::as_array_mut) {
            hooks.retain(|h| !is_koad_hook(h));
        }
    }
    groups.retain(|group| match group.get("hooks").and_then(Value::as_array) {
        Some(hooks) => !hooks.is_empty(),
        None => true,
    });
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
    strip_koad_hooks(groups);
    groups.push(json!({"hooks": [{"type": "command", "command": command, "timeout": 30}]}));
    Ok(settings)
}

/// Remove the KoadOS SessionStart hook, leaving every other hook (including
/// other hooks in the same group) untouched. Drops the `SessionStart` array
/// once it holds no groups, and drops `hooks` once it holds no keys —
/// either container can end up empty this way even if it was already empty
/// before the KoadOS hook was ever installed.
pub fn remove_session_hook(mut settings: Value) -> Value {
    let mut drop_hooks = false;
    if let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) {
        if let Some(groups) = hooks.get_mut("SessionStart").and_then(Value::as_array_mut) {
            strip_koad_hooks(groups);
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
    fn hook_command_output_is_recognised_as_a_koad_hook() {
        // is_koad_hook and hook_command must not drift apart.
        assert!(is_koad_hook(
            &json!({"type": "command", "command": CMD, "timeout": 30})
        ));
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
    fn merge_keeps_a_user_hook_sharing_the_koad_group() {
        let settings = json!({"hooks": {"SessionStart": [
            {"hooks": [
                {"type": "command", "command": "echo mine"},
                {"type": "command", "command": CMD, "timeout": 30}
            ]}
        ]}});
        let merged = merge_session_hook(settings, CMD).unwrap();
        let groups = merged["hooks"]["SessionStart"].as_array().unwrap();
        // The original group loses only the koad hook; a fresh group is pushed for it.
        assert_eq!(groups.len(), 2);
        assert_eq!(
            groups[0]["hooks"],
            json!([{"type": "command", "command": "echo mine"}])
        );
        assert_eq!(groups[1]["hooks"][0]["command"], CMD);
    }

    #[test]
    fn merge_errors_when_hooks_is_not_an_object() {
        assert!(merge_session_hook(json!({"hooks": null}), CMD).is_err());
    }

    #[test]
    fn merge_errors_when_session_start_is_not_an_array() {
        assert!(merge_session_hook(json!({"hooks": {"SessionStart": {}}}), CMD).is_err());
    }

    #[test]
    fn merge_refuses_non_object_settings() {
        assert!(merge_session_hook(json!([1, 2]), CMD).is_err());
    }

    #[test]
    fn remove_restores_the_original_settings() {
        let original = json!({"effortLevel": "high", "mcpServers": {}});
        let merged = merge_session_hook(original.clone(), CMD).unwrap();
        assert_eq!(remove_session_hook(merged), original);
    }

    #[test]
    fn remove_keeps_a_user_hook_sharing_the_koad_group() {
        let settings = json!({"hooks": {"SessionStart": [
            {"hooks": [
                {"type": "command", "command": "echo mine"},
                {"type": "command", "command": CMD, "timeout": 30}
            ]}
        ]}});
        let removed = remove_session_hook(settings);
        assert_eq!(
            removed,
            json!({"hooks": {"SessionStart": [
                {"hooks": [{"type": "command", "command": "echo mine"}]}
            ]}})
        );
    }

    #[test]
    fn remove_keeps_a_user_hook_that_merely_mentions_the_marker_text() {
        let settings = json!({"hooks": {"SessionStart": [
            {"hooks": [{"type": "command", "command": "echo koad-agent anchor"}]}
        ]}});
        assert_eq!(remove_session_hook(settings.clone()), settings);
    }

    #[test]
    fn remove_keeps_other_session_start_groups_and_other_events() {
        let other_group = json!({"hooks": [{"type": "command", "command": "echo mine"}]});
        let settings = json!({
            "hooks": {
                "SessionStart": [other_group.clone(), {"hooks": [{"type": "command", "command": CMD, "timeout": 30}]}],
                "Stop": [{"hooks": [{"type": "command", "command": "echo stop"}]}]
            }
        });
        let removed = remove_session_hook(settings);
        assert_eq!(removed["hooks"]["SessionStart"], json!([other_group]));
        assert_eq!(
            removed["hooks"]["Stop"],
            json!([{"hooks": [{"type": "command", "command": "echo stop"}]}])
        );
    }

    #[test]
    fn remove_on_settings_without_hooks_returns_unchanged() {
        let settings = json!({"effortLevel": "high"});
        assert_eq!(remove_session_hook(settings.clone()), settings);
    }
}
