//! `koad body windows`: link Claude Code for Windows to this Citadel.
//!
//! Spec: docs/superpowers/specs/2026-09-27-windows-body-bridge-design.md

use crate::cli::{BodyAction, WindowsBodyAction};
use anyhow::{bail, Context, Result};
use koad_core::config::KoadConfig;
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

/// Strip KoadOS hook entries out of each group's `hooks` array in place and
/// return how many were removed. A group is dropped only when this filtering
/// emptied it; groups whose `hooks` array was already empty, or that have no
/// `hooks` array at all (malformed or simply different), are left untouched.
fn strip_koad_hooks(groups: &mut Vec<Value>) -> usize {
    let mut removed = 0;
    groups.retain_mut(|group| {
        let Some(hooks) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
            return true;
        };
        let before = hooks.len();
        hooks.retain(|h| !is_koad_hook(h));
        let stripped = before - hooks.len();
        removed += stripped;
        !(stripped > 0 && hooks.is_empty())
    });
    removed
}

/// Whether settings carry a KoadOS SessionStart hook. Any unexpected shape
/// counts as "not installed".
pub fn has_session_hook(settings: &Value) -> bool {
    settings["hooks"]["SessionStart"]
        .as_array()
        .is_some_and(|groups| {
            groups.iter().any(|g| {
                g["hooks"]
                    .as_array()
                    .is_some_and(|hooks| hooks.iter().any(is_koad_hook))
            })
        })
}

/// Add (or replace) the KoadOS SessionStart hook, preserving everything else.
pub fn merge_session_hook(mut settings: Value, command: &str) -> Result<Value> {
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
/// other hooks in the same group) untouched, and return whether one was
/// removed. Only when something was removed are the `SessionStart` array and
/// then `hooks` object dropped if that left them empty; with nothing to
/// remove, `settings` is not modified at all.
pub fn remove_session_hook(settings: &mut Value) -> bool {
    let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) else {
        return false;
    };
    let Some(groups) = hooks.get_mut("SessionStart").and_then(Value::as_array_mut) else {
        return false;
    };
    if strip_koad_hooks(groups) == 0 {
        return false;
    }
    if groups.is_empty() {
        hooks.remove("SessionStart");
    }
    if hooks.is_empty() {
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("hooks");
        }
    }
    true
}

const SKILLS_SOURCE: &str = "https://github.com/Fryymann/koad-os/tree/nightly/skills";

/// Refuse anything that could break out of the hook command line Claude Code
/// runs through a shell. `agent` must also satisfy the rule
/// `scripts/koad-mcp-stdio` applies.
fn validate_inputs(distro: &str, koad_home: &str, agent: &str) -> Result<()> {
    let safe = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | '-'))
    };
    for (label, value) in [
        ("WSL distro", distro),
        ("KOAD_HOME", koad_home),
        ("agent", agent),
    ] {
        if !safe(value) {
            bail!("{label} {value:?} contains characters outside [A-Za-z0-9._/-]; refusing");
        }
    }
    let mut chars = agent.chars();
    let agent_ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-'));
    if !agent_ok {
        bail!("agent {agent:?} must match ^[a-z0-9][a-z0-9_-]*$; refusing");
    }
    Ok(())
}

fn distro_from(var: Option<String>) -> Result<String> {
    match var {
        Some(d) if !d.is_empty() => Ok(d),
        _ => bail!("WSL_DISTRO_NAME is not set; run this from inside the WSL distro that hosts the Citadel"),
    }
}

fn distro() -> Result<String> {
    distro_from(std::env::var("WSL_DISTRO_NAME").ok())
}

/// Distro and KOAD_HOME, validated together with `agent`.
fn bridge_inputs(config: &KoadConfig, agent: &str) -> Result<(String, String)> {
    let (distro, home) = (distro()?, config.home.to_string_lossy().to_string());
    validate_inputs(&distro, &home, agent)?;
    Ok((distro, home))
}

/// The Windows side as seen from WSL.
struct WindowsEnv {
    /// e.g. /mnt/c/Users/idean
    profile: PathBuf,
    /// e.g. /mnt/c/Users/idean/.local/bin/claude.exe
    claude: PathBuf,
}

fn detect_windows() -> Result<WindowsEnv> {
    let out = Command::new("cmd.exe")
        .args(["/c", "echo %USERPROFILE%"])
        .current_dir("/mnt/c")
        .output()
        .context("cmd.exe not reachable; is this WSL with Windows interop?")?;
    let win = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !out.status.success() || win.is_empty() || win.contains('%') {
        bail!("could not read %USERPROFILE% through cmd.exe (got {win:?})");
    }
    let wsl = Command::new("wslpath")
        .arg("-u")
        .arg(&win)
        .output()
        .context("wslpath not found")?;
    let profile = PathBuf::from(String::from_utf8_lossy(&wsl.stdout).trim());
    if !wsl.status.success() || !profile.is_dir() {
        bail!("Windows profile {win} does not map to a WSL directory");
    }
    let claude = profile.join(".local/bin/claude.exe");
    if !claude.exists() {
        bail!("Claude Code for Windows not found at {}", claude.display());
    }
    Ok(WindowsEnv { profile, claude })
}

fn settings_path(win: &WindowsEnv) -> PathBuf {
    win.profile.join(".claude/settings.json")
}

/// Read settings.json (missing = `{}`) and pass it to `f`. If `f` returns a
/// new value, back the file up to `settings.json.bak-<timestamp>` and replace
/// it atomically (temp file in the same directory, then rename). Invalid
/// JSON, or an error from `f`, aborts without touching anything. Returns
/// whether the file was written.
fn edit_settings(path: &Path, f: impl FnOnce(Value) -> Result<Option<Value>>) -> Result<bool> {
    let exists = path.exists();
    let current = if exists {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&text)
            .with_context(|| format!("{} is not valid JSON; nothing changed", path.display()))?
    } else {
        json!({})
    };
    let Some(updated) = f(current)? else {
        return Ok(false);
    };
    let dir = path
        .parent()
        .context("settings path has no parent directory")?;
    std::fs::create_dir_all(dir)?;
    if exists {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let mut backup = dir.join(format!("settings.json.bak-{stamp}"));
        let mut n = 1;
        while backup.exists() {
            backup = dir.join(format!("settings.json.bak-{stamp}-{n}"));
            n += 1;
        }
        std::fs::copy(path, &backup)
            .with_context(|| format!("backing up to {}", backup.display()))?;
    }
    let mut tmp = tempfile::Builder::new()
        .prefix(".settings.json.tmp-")
        .tempfile_in(dir)?;
    tmp.write_all((serde_json::to_string_pretty(&updated)? + "\n").as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)
        .with_context(|| format!("replacing {}", path.display()))?;
    Ok(true)
}

fn run_windows(cmd: &mut Command) -> Result<bool> {
    Ok(cmd.current_dir("/mnt/c").status()?.success())
}

fn mcp_registered(win: &WindowsEnv) -> bool {
    Command::new(&win.claude)
        .args(["mcp", "get", MCP_NAME])
        .current_dir("/mnt/c")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn skills_present(win: &WindowsEnv) -> Vec<&'static str> {
    WINDOWS_SKILLS
        .into_iter()
        .filter(|s| {
            win.profile
                .join(".claude/skills")
                .join(s)
                .join("SKILL.md")
                .exists()
        })
        .collect()
}

fn install(config: &KoadConfig, agent: &str) -> Result<()> {
    let (distro, home) = bridge_inputs(config, agent)?;
    let win = detect_windows()?;

    let _ = Command::new(&win.claude)
        .args(["mcp", "remove", MCP_NAME, "--scope", "user"])
        .current_dir("/mnt/c")
        .output();
    let server = mcp_server_json(&distro, &home, agent).to_string();
    if !run_windows(
        Command::new(&win.claude).args(["mcp", "add-json", MCP_NAME, &server, "--scope", "user"]),
    )? {
        bail!("claude.exe mcp add-json failed");
    }
    println!("✓ MCP server '{MCP_NAME}' registered");

    let hook = hook_command(&distro, &home, agent);
    let path = settings_path(&win);
    edit_settings(&path, |s| merge_session_hook(s, &hook).map(Some))?;
    println!("✓ SessionStart hook added to {}", path.display());

    let mut skills = Command::new("cmd.exe");
    skills.args([
        "/c",
        "npx",
        "-y",
        "skills@1.7.0",
        "add",
        SKILLS_SOURCE,
        "-g",
        "-a",
        "claude-code",
        "-s",
    ]);
    skills.args(WINDOWS_SKILLS).arg("-y");
    if !run_windows(&mut skills)? {
        bail!("installing skills on Windows failed");
    }
    println!("✓ Skills installed: {}", WINDOWS_SKILLS.join(", "));

    status(config, agent)
}

/// Pipe `initialize` + a semantic search through the configured MCP command.
fn mcp_round_trip(distro: &str, home: &str, agent: &str) -> Result<bool> {
    let mut child = Command::new("wsl.exe")
        .args([
            "-d",
            distro,
            "-e",
            &format!("{home}/bin/koad-wsl-env"),
            "koad-mcp-stdio",
            agent,
        ])
        .current_dir("/mnt/c")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    {
        let stdin = child.stdin.as_mut().context("stdin")?;
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-11-25"}}}}"#
        )?;
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"memory.search_semantic","arguments":{{"query":"KoadOS","limit":1}}}}}}"#
        )?;
    }
    drop(child.stdin.take());
    let out = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.lines().any(|l| {
        serde_json::from_str::<Value>(l)
            .map(|v| v["id"] == 2 && v.get("result").is_some() && v["result"]["isError"] != true)
            .unwrap_or(false)
    }))
}

/// Read-only: checks every link of the bridge and changes nothing.
fn status(config: &KoadConfig, agent: &str) -> Result<()> {
    let (distro, home) = bridge_inputs(config, agent)?;
    let win = detect_windows()?;
    let mut ok = true;
    let mut check = |label: &str, pass: bool| {
        println!("{} {label}", if pass { "✓" } else { "✗" });
        ok &= pass;
    };

    check(
        "MCP server registered in Claude Code for Windows",
        mcp_registered(&win),
    );

    let hook = std::fs::read_to_string(settings_path(&win))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .is_some_and(|s| has_session_hook(&s));
    check("SessionStart hook present", hook);

    let anchor = Command::new("wsl.exe")
        .args([
            "-d",
            &distro,
            "-e",
            &format!("{home}/bin/koad-wsl-env"),
            "koad-agent",
            "anchor",
            agent,
            "--body",
            "windows",
        ])
        .current_dir("/mnt/c")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).starts_with("# KoadOS Agent Identity Anchor"))
        .unwrap_or(false);
    check("Hook command prints the identity anchor", anchor);

    check(
        "Memory skills installed",
        skills_present(&win).len() == WINDOWS_SKILLS.len(),
    );

    check(
        "MCP round trip: semantic search through wsl.exe",
        mcp_round_trip(&distro, &home, agent).unwrap_or(false),
    );

    if !ok {
        bail!("Windows body bridge is not fully linked");
    }
    Ok(())
}

fn uninstall() -> Result<()> {
    let win = detect_windows()?;
    let mut failed = false;

    if mcp_registered(&win) {
        if run_windows(
            Command::new(&win.claude).args(["mcp", "remove", MCP_NAME, "--scope", "user"]),
        )? {
            println!("✓ MCP server '{MCP_NAME}' removed");
        } else {
            println!("✗ claude.exe mcp remove {MCP_NAME} failed");
            failed = true;
        }
    } else {
        println!("- MCP server '{MCP_NAME}' not registered; nothing to remove");
    }

    let path = settings_path(&win);
    let removed = path.exists()
        && edit_settings(&path, |mut s| Ok(remove_session_hook(&mut s).then_some(s)))?;
    if removed {
        println!("✓ SessionStart hook removed from {}", path.display());
    } else {
        println!(
            "- No KoadOS SessionStart hook in {}; nothing to remove",
            path.display()
        );
    }

    let present = skills_present(&win);
    if present.is_empty() {
        println!("- Skills not installed; nothing to remove");
    } else {
        let mut skills = Command::new("cmd.exe");
        skills.args(["/c", "npx", "-y", "skills@1.7.0", "remove"]);
        skills.args(&present).args(["-g", "-y"]);
        if run_windows(&mut skills)? && skills_present(&win).is_empty() {
            println!("✓ Skills removed: {}", present.join(", "));
        } else {
            println!("✗ Removing skills failed: {}", present.join(", "));
            failed = true;
        }
    }

    if failed {
        bail!("Windows body bridge was not fully removed");
    }
    Ok(())
}

pub async fn handle(action: BodyAction, config: &KoadConfig) -> Result<()> {
    match action {
        BodyAction::Windows { action } => match action {
            WindowsBodyAction::Install { agent } => install(config, &agent.to_lowercase()),
            WindowsBodyAction::Status { agent } => status(config, &agent.to_lowercase()),
            WindowsBodyAction::Uninstall => uninstall(),
        },
    }
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
        let mut merged = merge_session_hook(original.clone(), CMD).unwrap();
        assert!(remove_session_hook(&mut merged));
        assert_eq!(merged, original);
    }

    #[test]
    fn remove_keeps_a_user_hook_sharing_the_koad_group() {
        let settings = json!({"hooks": {"SessionStart": [
            {"hooks": [
                {"type": "command", "command": "echo mine"},
                {"type": "command", "command": CMD, "timeout": 30}
            ]}
        ]}});
        let mut removed = settings;
        assert!(remove_session_hook(&mut removed));
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
        let mut removed = settings.clone();
        assert!(!remove_session_hook(&mut removed));
        assert_eq!(removed, settings);
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
        let mut removed = settings;
        assert!(remove_session_hook(&mut removed));
        assert_eq!(removed["hooks"]["SessionStart"], json!([other_group]));
        assert_eq!(
            removed["hooks"]["Stop"],
            json!([{"hooks": [{"type": "command", "command": "echo stop"}]}])
        );
    }

    #[test]
    fn remove_on_settings_without_hooks_returns_unchanged() {
        let settings = json!({"effortLevel": "high"});
        let mut removed = settings.clone();
        assert!(!remove_session_hook(&mut removed));
        assert_eq!(removed, settings);
    }

    #[test]
    fn remove_reports_whether_a_koad_hook_was_removed() {
        let mut merged = merge_session_hook(json!({}), CMD).unwrap();
        assert!(remove_session_hook(&mut merged));
        assert!(!remove_session_hook(&mut merged));
    }

    #[test]
    fn remove_without_a_koad_hook_leaves_empty_containers_alone() {
        let mut settings = json!({"hooks": {"SessionStart": []}});
        assert!(!remove_session_hook(&mut settings));
        assert_eq!(settings, json!({"hooks": {"SessionStart": []}}));
    }

    #[test]
    fn strip_keeps_groups_whose_hooks_were_already_empty() {
        let mut settings = json!({"hooks": {"SessionStart": [
            {"matcher": "startup", "hooks": []},
            {"hooks": [{"type": "command", "command": CMD, "timeout": 30}]}
        ]}});
        assert!(remove_session_hook(&mut settings));
        assert_eq!(
            settings,
            json!({"hooks": {"SessionStart": [{"matcher": "startup", "hooks": []}]}})
        );
    }

    #[test]
    fn has_session_hook_detects_only_koad_hooks() {
        assert!(has_session_hook(
            &merge_session_hook(json!({}), CMD).unwrap()
        ));
        assert!(!has_session_hook(&json!({})));
        assert!(!has_session_hook(&json!({"hooks": null})));
        assert!(!has_session_hook(&json!({"hooks": {"SessionStart": {}}})));
        assert!(!has_session_hook(&json!({"hooks": {"SessionStart": [
            {"hooks": [{"type": "command", "command": "echo koad-agent anchor"}]}
        ]}})));
    }

    #[test]
    fn validate_accepts_normal_inputs() {
        assert!(validate_inputs("Ubuntu-24.04", "/home/ideans/.citadel-jupiter", "clyde").is_ok());
        assert!(validate_inputs("Ubuntu", "/h", "tyr_2-x").is_ok());
    }

    #[test]
    fn validate_rejects_shell_metacharacters_and_bad_agents() {
        assert!(validate_inputs("Ubuntu; rm -rf /", "/h", "clyde").is_err());
        assert!(validate_inputs("Ubuntu", "/h/with space", "clyde").is_err());
        assert!(validate_inputs("Ubuntu", "/h/$(x)", "clyde").is_err());
        assert!(validate_inputs("", "/h", "clyde").is_err());
        assert!(validate_inputs("Ubuntu", "/h", "Clyde").is_err());
        assert!(validate_inputs("Ubuntu", "/h", "-clyde").is_err());
        assert!(validate_inputs("Ubuntu", "/h", "cl/yde").is_err());
        assert!(validate_inputs("Ubuntu", "/h", "cl.yde").is_err());
        assert!(validate_inputs("Ubuntu", "/h", "").is_err());
    }

    #[test]
    fn distro_comes_from_the_environment_or_fails() {
        assert_eq!(
            distro_from(Some("Ubuntu-24.04".into())).unwrap(),
            "Ubuntu-24.04"
        );
        assert!(distro_from(Some(String::new())).is_err());
        assert!(distro_from(None).is_err());
    }

    fn dir_entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn edit_settings_refuses_invalid_json_and_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert!(
            edit_settings(&path, |s| merge_session_hook(s, "x koad-agent anchor")
                .map(Some))
            .is_err()
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not json");
        assert_eq!(
            dir_entries(dir.path()),
            vec!["settings.json"],
            "no backup written"
        );
    }

    #[test]
    fn edit_settings_backs_up_before_writing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let original = "{\"effortLevel\":\"high\"}";
        std::fs::write(&path, original).unwrap();
        assert!(
            edit_settings(&path, |s| merge_session_hook(s, "x koad-agent anchor")
                .map(Some))
            .unwrap()
        );
        let names = dir_entries(dir.path());
        let backups: Vec<_> = names
            .iter()
            .filter(|n| n.starts_with("settings.json.bak-"))
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(names.len(), 2, "no temp file left behind: {names:?}");
        assert_eq!(
            std::fs::read_to_string(dir.path().join(backups[0])).unwrap(),
            original
        );
        let written: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(written["effortLevel"], "high");
    }

    #[test]
    fn edit_settings_keeps_every_backup_when_run_twice_quickly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{}").unwrap();
        edit_settings(&path, |s| merge_session_hook(s, "a").map(Some)).unwrap();
        edit_settings(&path, |s| merge_session_hook(s, "b").map(Some)).unwrap();
        let backups = dir_entries(dir.path())
            .into_iter()
            .filter(|n| n.starts_with("settings.json.bak-"))
            .count();
        assert_eq!(backups, 2);
    }

    #[test]
    fn edit_settings_creates_a_missing_file_without_a_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        edit_settings(&path, |s| merge_session_hook(s, "x").map(Some)).unwrap();
        assert_eq!(dir_entries(dir.path()), vec!["settings.json"]);
    }

    #[test]
    fn edit_settings_writes_nothing_when_the_edit_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{\"a\": 1}").unwrap();
        assert!(!edit_settings(&path, |_| Ok(None)).unwrap());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{\"a\": 1}");
        assert_eq!(dir_entries(dir.path()), vec!["settings.json"]);
    }

    #[test]
    fn edit_settings_aborts_without_writing_when_the_edit_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "[1]").unwrap();
        assert!(edit_settings(&path, |s| merge_session_hook(s, "x").map(Some)).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[1]");
        assert_eq!(dir_entries(dir.path()), vec!["settings.json"]);
    }
}
