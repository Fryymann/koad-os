//! `koad body windows`: link Claude Code for Windows to this Citadel.
//!
//! Spec: docs/superpowers/specs/2026-09-27-windows-body-bridge-design.md

use crate::cli::{BodyAction, WindowsBodyAction};
use anyhow::{bail, Context, Result};
use koad_core::config::KoadConfig;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

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

const DETECT_TIMEOUT: Duration = Duration::from_secs(10);
const ANCHOR_TIMEOUT: Duration = Duration::from_secs(15);
const MCP_TIMEOUT: Duration = Duration::from_secs(30);
const NPX_TIMEOUT: Duration = Duration::from_secs(120);

/// Run `cmd` to completion with a deadline: stdout and stderr are captured,
/// `stdin` (if any) is fed on its own thread, and on timeout the process is
/// killed and reaped. Output still held open by a surviving grandchild is
/// given a short grace period rather than waited on forever.
fn run_bounded(cmd: &mut Command, stdin: Option<&[u8]>, timeout: Duration) -> Result<Output> {
    let mut child = cmd
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("could not start {:?}", cmd.get_program()))?;

    if let (Some(data), Some(mut pipe)) = (stdin, child.stdin.take()) {
        let data = data.to_vec();
        std::thread::spawn(move || {
            let _ = pipe.write_all(&data);
        });
    }
    let reader = |pipe: Option<Box<dyn Read + Send>>| {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut p) = pipe {
                let _ = p.read_to_end(&mut buf);
            }
            let _ = tx.send(buf);
        });
        rx
    };
    let out_rx = reader(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let err_rx = reader(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );

    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!("timed out after {}s", timeout.as_secs());
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let grace = Duration::from_secs(2);
    Ok(Output {
        status,
        stdout: out_rx.recv_timeout(grace).unwrap_or_default(),
        stderr: err_rx.recv_timeout(grace).unwrap_or_default(),
    })
}

/// Short reason for a failed or unsuccessful run: the error (e.g. a timeout)
/// or the exit code plus the last non-empty stderr line.
fn failure_reason(result: &Result<Output>) -> String {
    match result {
        Err(e) => e.to_string(),
        Ok(out) => {
            let code = out.status.code().map_or_else(
                || "killed by signal".to_string(),
                |c| format!("exit code {c}"),
            );
            let stderr = String::from_utf8_lossy(&out.stderr);
            match stderr.lines().rev().map(str::trim).find(|l| !l.is_empty()) {
                Some(line) => format!("{code}: {line}"),
                None => code,
            }
        }
    }
}

fn succeeded(result: &Result<Output>) -> bool {
    matches!(result, Ok(out) if out.status.success())
}

/// Parse JSON text, tolerating a leading UTF-8 BOM (Windows editors add one).
fn parse_json(text: &str) -> serde_json::Result<Value> {
    serde_json::from_str(text.strip_prefix('\u{feff}').unwrap_or(text))
}

/// Read and parse a JSON file; `Ok(None)` if it does not exist.
fn read_json(path: &Path) -> Result<Option<Value>> {
    if !path.exists() {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let value =
        parse_json(&text).with_context(|| format!("{} is not valid JSON", path.display()))?;
    Ok(Some(value))
}

/// Replace `path` atomically: temp file in the same directory, fsync, rename.
fn write_atomic(path: &Path, value: &Value) -> Result<()> {
    let dir = path.parent().context("path has no parent directory")?;
    std::fs::create_dir_all(dir)?;
    let mut tmp = tempfile::Builder::new()
        .prefix(&format!(
            ".{}.tmp-",
            path.file_name().unwrap_or_default().to_string_lossy()
        ))
        .tempfile_in(dir)?;
    tmp.write_all((serde_json::to_string_pretty(value)? + "\n").as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)
        .with_context(|| format!("replacing {}", path.display()))?;
    Ok(())
}

/// Where install records which Windows skills it added (and so may remove).
fn record_path(koad_home: &Path) -> PathBuf {
    koad_home.join("state/body-windows.json")
}

fn recorded_skills(path: &Path) -> Result<Vec<String>> {
    Ok(read_json(path)?
        .and_then(|v| v["skills_added"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|s| s.as_str().map(str::to_string))
        .collect())
}

fn write_recorded_skills(path: &Path, skills: Vec<String>) -> Result<()> {
    let mut record = match read_json(path)? {
        Some(Value::Object(map)) => Value::Object(map),
        _ => json!({}),
    };
    record["skills_added"] = json!(skills);
    write_atomic(path, &record)
}

/// Add `skills` to the record (union, order kept), creating it if needed.
fn record_skills_added(path: &Path, skills: &[&str]) -> Result<()> {
    let mut all = recorded_skills(path)?;
    for s in skills {
        if !all.iter().any(|r| r == s) {
            all.push(s.to_string());
        }
    }
    write_recorded_skills(path, all)
}

/// Drop `skills` from the record.
fn clear_recorded_skills(path: &Path, skills: &[&str]) -> Result<()> {
    let remaining = recorded_skills(path)?
        .into_iter()
        .filter(|r| !skills.contains(&r.as_str()))
        .collect();
    write_recorded_skills(path, remaining)
}

/// The Windows side as seen from WSL.
struct WindowsEnv {
    /// e.g. /mnt/c/Users/idean
    profile: PathBuf,
    /// e.g. /mnt/c/Users/idean/.local/bin/claude.exe
    claude: PathBuf,
}

/// A command run from a Windows-visible directory, so interop doesn't warn
/// about a UNC working directory.
fn win_cmd(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut cmd = Command::new(program);
    cmd.current_dir("/mnt/c");
    cmd
}

fn detect_windows() -> Result<WindowsEnv> {
    let out = run_bounded(
        win_cmd("cmd.exe").args(["/c", "echo %USERPROFILE%"]),
        None,
        DETECT_TIMEOUT,
    )
    .context("cmd.exe not usable; is this WSL with Windows interop?")?;
    let win = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !out.status.success() || win.is_empty() || win.contains('%') {
        bail!("could not read %USERPROFILE% through cmd.exe (got {win:?})");
    }
    let wsl = run_bounded(
        Command::new("wslpath").arg("-u").arg(&win),
        None,
        DETECT_TIMEOUT,
    )
    .context("wslpath failed")?;
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

/// Read settings.json (missing = `{}`, BOM tolerated) and pass it to `f`. If
/// `f` returns a new value, back the file up to
/// `settings.json.bak-<timestamp>` and replace it atomically, without a BOM.
/// Invalid JSON, or an error from `f`, aborts without touching anything.
/// Returns whether the file was written.
fn edit_settings(path: &Path, f: impl FnOnce(Value) -> Result<Option<Value>>) -> Result<bool> {
    let current = read_json(path)
        .map_err(|e| anyhow::anyhow!("{e:#}; nothing changed"))?
        .unwrap_or_else(|| json!({}));
    let Some(updated) = f(current)? else {
        return Ok(false);
    };
    if path.exists() {
        let dir = path.parent().context("settings path has no parent")?;
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
    write_atomic(path, &updated)?;
    Ok(true)
}

/// `edit_settings` step for install: `None` when the hook is already in place.
fn merge_edit(settings: Value, hook: &str) -> Result<Option<Value>> {
    let merged = merge_session_hook(settings.clone(), hook)?;
    Ok((merged != settings).then_some(merged))
}

/// User-scope MCP registration, read straight from `~/.claude.json` so no
/// server health check is spawned. Missing or unreadable = not registered.
fn mcp_registered_in(claude_json: &Path) -> bool {
    read_json(claude_json)
        .ok()
        .flatten()
        .is_some_and(|v| v["mcpServers"].get(MCP_NAME).is_some())
}

fn mcp_registered(win: &WindowsEnv) -> bool {
    mcp_registered_in(&win.profile.join(".claude.json"))
}

fn skill_present(win: &WindowsEnv, skill: &str) -> bool {
    win.profile
        .join(".claude/skills")
        .join(skill)
        .join("SKILL.md")
        .exists()
}

fn npx_skills(args: &[&str]) -> Result<Output> {
    let mut cmd = win_cmd("cmd.exe");
    cmd.args(["/c", "npx", "-y", "skills@1.7.0"]).args(args);
    run_bounded(&mut cmd, None, NPX_TIMEOUT)
}

fn install(config: &KoadConfig, agent: &str) -> Result<()> {
    let (distro, home) = bridge_inputs(config, agent)?;
    let win = detect_windows()?;

    let was_registered = mcp_registered(&win);
    let _ = run_bounded(
        win_cmd(&win.claude).args(["mcp", "remove", MCP_NAME, "--scope", "user"]),
        None,
        MCP_TIMEOUT,
    );
    let server = mcp_server_json(&distro, &home, agent).to_string();
    let added = run_bounded(
        win_cmd(&win.claude).args(["mcp", "add-json", MCP_NAME, &server, "--scope", "user"]),
        None,
        MCP_TIMEOUT,
    );
    if !succeeded(&added) {
        let reason = failure_reason(&added);
        if was_registered {
            bail!(
                "claude.exe mcp add-json failed ({reason}); the previous '{MCP_NAME}' \
                 registration was already removed — rerun `koad body windows install`"
            );
        }
        bail!("claude.exe mcp add-json failed ({reason})");
    }
    println!("✓ MCP server '{MCP_NAME}' registered");

    let hook = hook_command(&distro, &home, agent);
    let path = settings_path(&win);
    if edit_settings(&path, |s| merge_edit(s, &hook))? {
        println!("✓ SessionStart hook added to {}", path.display());
    } else {
        println!("- SessionStart hook already present in {}", path.display());
    }

    let mut missing = Vec::new();
    for skill in WINDOWS_SKILLS {
        if skill_present(&win, skill) {
            println!("- Skill {skill} already present; left as is");
        } else {
            missing.push(skill);
        }
    }
    if !missing.is_empty() {
        let mut args = vec!["add", SKILLS_SOURCE, "-g", "-a", "claude-code", "-s"];
        args.extend(&missing);
        args.push("-y");
        let result = npx_skills(&args);
        let added: Vec<&str> = missing
            .iter()
            .copied()
            .filter(|s| skill_present(&win, s))
            .collect();
        record_skills_added(&record_path(&config.home), &added)?;
        if !succeeded(&result) || added.len() != missing.len() {
            bail!(
                "installing skills on Windows failed ({})",
                failure_reason(&result)
            );
        }
        println!("✓ Skills installed: {}", added.join(", "));
    }

    status(config, agent)
}

/// Pipe `initialize` + a semantic search through the configured MCP command.
fn mcp_round_trip(distro: &str, home: &str, agent: &str) -> std::result::Result<(), String> {
    let requests = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"memory.search_semantic","arguments":{"query":"KoadOS","limit":1}}}"#,
        "\n",
    );
    let result = run_bounded(
        win_cmd("wsl.exe").args([
            "-d",
            distro,
            "-e",
            &format!("{home}/bin/koad-wsl-env"),
            "koad-mcp-stdio",
            agent,
        ]),
        Some(requests.as_bytes()),
        MCP_TIMEOUT,
    );
    let answered = result.as_ref().is_ok_and(|out| {
        String::from_utf8_lossy(&out.stdout).lines().any(|l| {
            parse_json(l)
                .map(|v| {
                    v["id"] == 2 && v.get("result").is_some() && v["result"]["isError"] != true
                })
                .unwrap_or(false)
        })
    });
    if answered {
        Ok(())
    } else if succeeded(&result) {
        Err(format!(
            "no successful tools/call response; {}",
            failure_reason(&result)
        ))
    } else {
        Err(failure_reason(&result))
    }
}

/// Read-only: checks every link of the bridge and changes nothing.
fn status(config: &KoadConfig, agent: &str) -> Result<()> {
    let (distro, home) = bridge_inputs(config, agent)?;
    let win = detect_windows()?;
    let mut ok = true;
    let mut check = |label: &str, result: std::result::Result<(), String>| match result {
        Ok(()) => println!("✓ {label}"),
        Err(reason) => {
            println!("✗ {label} — {reason}");
            ok = false;
        }
    };

    let claude_json = win.profile.join(".claude.json");
    check(
        "MCP server registered in Claude Code for Windows",
        if mcp_registered_in(&claude_json) {
            Ok(())
        } else {
            Err(format!(
                "no mcpServers.{MCP_NAME} in {}",
                claude_json.display()
            ))
        },
    );

    let settings = settings_path(&win);
    check(
        "SessionStart hook present",
        match read_json(&settings) {
            Ok(Some(s)) if has_session_hook(&s) => Ok(()),
            Ok(Some(_)) => Err(format!("no KoadOS hook in {}", settings.display())),
            Ok(None) => Err(format!("{} does not exist", settings.display())),
            Err(e) => Err(format!("{e:#}")),
        },
    );

    let anchor = run_bounded(
        win_cmd("wsl.exe").args([
            "-d",
            &distro,
            "-e",
            &format!("{home}/bin/koad-wsl-env"),
            "koad-agent",
            "anchor",
            agent,
            "--body",
            "windows",
        ]),
        None,
        ANCHOR_TIMEOUT,
    );
    let printed = anchor.as_ref().is_ok_and(|o| {
        String::from_utf8_lossy(&o.stdout).starts_with("# KoadOS Agent Identity Anchor")
    });
    check(
        "Hook command prints the identity anchor",
        if printed {
            Ok(())
        } else if succeeded(&anchor) {
            Err("output is not an identity anchor".to_string())
        } else {
            Err(failure_reason(&anchor))
        },
    );

    let missing: Vec<&str> = WINDOWS_SKILLS
        .into_iter()
        .filter(|s| !skill_present(&win, s))
        .collect();
    check(
        "Memory skills installed",
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!("missing: {}", missing.join(", ")))
        },
    );

    check(
        "MCP round trip: semantic search through wsl.exe",
        mcp_round_trip(&distro, &home, agent),
    );

    if !ok {
        bail!("Windows body bridge is not fully linked");
    }
    Ok(())
}

fn uninstall(config: &KoadConfig) -> Result<()> {
    let win = detect_windows()?;
    let mut failed = false;

    if mcp_registered(&win) {
        let result = run_bounded(
            win_cmd(&win.claude).args(["mcp", "remove", MCP_NAME, "--scope", "user"]),
            None,
            MCP_TIMEOUT,
        );
        if succeeded(&result) {
            println!("✓ MCP server '{MCP_NAME}' removed");
        } else {
            println!(
                "✗ claude.exe mcp remove {MCP_NAME} failed — {}",
                failure_reason(&result)
            );
            failed = true;
        }
    } else {
        println!("- MCP server '{MCP_NAME}' not registered; nothing to remove");
    }

    let path = settings_path(&win);
    if edit_settings(&path, |mut s| Ok(remove_session_hook(&mut s).then_some(s)))? {
        println!("✓ SessionStart hook removed from {}", path.display());
    } else {
        println!(
            "- No KoadOS SessionStart hook in {}; nothing to remove",
            path.display()
        );
    }

    let record = record_path(&config.home);
    let recorded = recorded_skills(&record)?;
    let mut ours = Vec::new();
    for skill in WINDOWS_SKILLS {
        if recorded.iter().any(|r| r == skill) {
            ours.push(skill);
        } else if skill_present(&win, skill) {
            println!("- Skill {skill} was not installed by koad; left in place");
        }
    }
    let (present, gone): (Vec<&str>, Vec<&str>) =
        ours.into_iter().partition(|s| skill_present(&win, s));
    if !gone.is_empty() {
        println!("- Skills already gone: {}", gone.join(", "));
        clear_recorded_skills(&record, &gone)?;
    }
    if !present.is_empty() {
        let mut args = vec!["remove"];
        args.extend(&present);
        args.extend(["-g", "-y"]);
        let result = npx_skills(&args);
        let removed: Vec<&str> = present
            .iter()
            .copied()
            .filter(|s| !skill_present(&win, s))
            .collect();
        clear_recorded_skills(&record, &removed)?;
        if removed.len() == present.len() {
            println!("✓ Skills removed: {}", removed.join(", "));
        } else {
            println!("✗ Removing skills failed — {}", failure_reason(&result));
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
            WindowsBodyAction::Uninstall => uninstall(config),
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

    #[test]
    fn run_bounded_kills_a_process_that_overruns() {
        let start = std::time::Instant::now();
        let err =
            run_bounded(Command::new("sleep").arg("5"), None, Duration::from_secs(1)).unwrap_err();
        assert!(err.to_string().contains("timed out after 1s"), "{err}");
        assert!(start.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn run_bounded_returns_output_and_feeds_stdin() {
        let out =
            run_bounded(Command::new("echo").arg("hi"), None, Duration::from_secs(5)).unwrap();
        assert!(out.status.success());
        assert_eq!(out.stdout, b"hi\n");
        let out = run_bounded(
            &mut Command::new("cat"),
            Some(b"piped"),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(out.stdout, b"piped");
    }

    #[test]
    fn failure_reason_shows_exit_code_and_last_stderr_line() {
        let out = run_bounded(
            Command::new("sh").args(["-c", "echo one >&2; echo two >&2; exit 3"]),
            None,
            Duration::from_secs(5),
        );
        assert_eq!(failure_reason(&out), "exit code 3: two");
    }

    #[test]
    fn skill_record_merges_reads_and_clears() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state/body-windows.json");
        assert!(
            recorded_skills(&path).unwrap().is_empty(),
            "no record = nothing"
        );
        record_skills_added(&path, &["cass-recall"]).unwrap();
        record_skills_added(&path, &["cass-search", "cass-recall"]).unwrap();
        assert_eq!(
            recorded_skills(&path).unwrap(),
            vec!["cass-recall", "cass-search"]
        );
        clear_recorded_skills(&path, &["cass-recall"]).unwrap();
        assert_eq!(recorded_skills(&path).unwrap(), vec!["cass-search"]);
    }

    #[test]
    fn skill_record_keeps_other_fields_and_tolerates_a_bom() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("body-windows.json");
        std::fs::write(
            &path,
            "\u{feff}{\"other\": 1, \"skills_added\": [\"cass-search\"]}",
        )
        .unwrap();
        record_skills_added(&path, &["cass-recall"]).unwrap();
        let v = read_json(&path).unwrap().unwrap();
        assert_eq!(v["other"], 1);
        assert_eq!(
            recorded_skills(&path).unwrap(),
            vec!["cass-search", "cass-recall"]
        );
    }

    #[test]
    fn edit_settings_tolerates_a_bom_and_writes_without_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "\u{feff}{\r\n\"a\":1\r\n}\r\n").unwrap();
        assert!(edit_settings(&path, |s| merge_edit(s, CMD)).unwrap());
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.starts_with('\u{feff}'));
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["a"], 1);
        assert!(has_session_hook(&v));
    }

    #[test]
    fn merge_edit_is_a_no_op_when_the_hook_is_already_there() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{}").unwrap();
        assert!(edit_settings(&path, |s| merge_edit(s, CMD)).unwrap());
        let before = dir_entries(dir.path());
        assert!(!edit_settings(&path, |s| merge_edit(s, CMD)).unwrap());
        assert_eq!(dir_entries(dir.path()), before, "no second backup");
    }

    #[test]
    fn mcp_registration_is_read_from_claude_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".claude.json");
        assert!(!mcp_registered_in(&path));
        std::fs::write(&path, "{\"mcpServers\": {\"other\": {}}}").unwrap();
        assert!(!mcp_registered_in(&path));
        std::fs::write(
            &path,
            "\u{feff}{\"mcpServers\": {\"citadel-memory\": {\"type\": \"stdio\"}}}",
        )
        .unwrap();
        assert!(mcp_registered_in(&path));
        std::fs::write(&path, "{ not json").unwrap();
        assert!(!mcp_registered_in(&path));
    }
}
