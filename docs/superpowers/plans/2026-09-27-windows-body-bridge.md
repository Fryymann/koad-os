# Windows Body Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let Clyde run in Claude Code for Windows with the same identity and CASS memory (recall, semantic search, commit) as in WSL, bridged over MCP stdio through `wsl.exe`.

**Architecture:** Everything executes in WSL. Windows Claude Code gets three things:
- an MCP server entry that runs `wsl.exe … koad-wsl-env koad-mcp-stdio clyde`;
- a SessionStart hook that runs `wsl.exe … koad-wsl-env koad-agent anchor Clyde --body windows`, whose stdout becomes session context;
- the `cass-recall` and `cass-search` skills.

`koad body windows install|status|uninstall` (koad CLI, run in WSL) manages that config.

**Tech Stack:** Rust (tokio, clap, serde_json, tonic, tracing-subscriber), bash launchers, Claude Code for Windows (`claude.exe mcp add-json`, `settings.json` hooks), Vercel `skills` CLI.

**Spec:** `docs/superpowers/specs/2026-09-27-windows-body-bridge-design.md`

**Conventions for every task** (from the repo `AGENTS.md`):
- Branch from `origin/nightly`; one PR per task, targeting `nightly` on `Fryymann/koad-os`. Do not stack.
- On Jupiter, run cargo as `~/.cargo/bin/cargo`: the RTK hook rewrites `cargo` output.
- New and changed code must be rustfmt-clean. Do not reformat untouched code.
- Full check before each PR: `~/.cargo/bin/cargo test --workspace --release` with 0 failures.

---

## File structure

| File | Responsibility |
|---|---|
| `crates/koad-mcp/src/lib.rs` (modify) | `McpServer::serve` over any reader/writer: skip blank lines, never answer notifications |
| `crates/koad-os-mcp/src/main.rs` (modify) | Logs to stderr; partition derived from `AGENT_NAME` when `AGENT_PARTITION` is unset |
| `crates/koad-os-mcp/Cargo.toml` (modify) | Add `koad-core` dependency |
| `crates/koad-os-mcp/tests/stdio.rs` (create) | Binary-level test: stdout carries only JSON-RPC |
| `crates/koad-os-mcp/src/tools/cass.rs` (create) | The one way tools reach CASS: bounded connect and request timeouts |
| `crates/koad-os-mcp/src/tools/{commit,intel_get,list_topics,recall,search_semantic,status}.rs` (modify) | Use `cass::memory` / `cass::pulse` instead of unbounded `connect` |
| `scripts/koad-wsl-env` (create) | Establish the KoadOS env for processes launched from Windows |
| `scripts/koad-mcp-stdio` (create) | Start `koad-os-mcp` over stdio for one known agent |
| `crates/koad-cli/tests/wsl_launchers.rs` (create) | Tests for both launchers |
| `install.sh` (modify) | Install both launchers into `$KOAD_HOME/bin` |
| `crates/koad-agent/src/commands/anchor.rs` (create) | `koad-agent anchor --body windows`: fetch CASS packet, render the Windows anchor |
| `crates/koad-agent/src/commands/mod.rs`, `crates/koad-agent/src/lib.rs` (modify) | Wire the `anchor` subcommand |
| `crates/koad-cli/src/handlers/body_windows.rs` (create) | Pure config merges + install/status/uninstall side effects |
| `crates/koad-cli/src/cli.rs`, `crates/koad-cli/src/main.rs`, `crates/koad-cli/src/handlers/mod.rs` (modify) | Wire `koad body windows …` |
| `AGENTS.md` (modify) | One line under "Deploying on Jupiter" |

---

### Task 1: `McpServer::serve`: blank lines and notifications

**Files:**
- Modify: `crates/koad-mcp/src/lib.rs` (the `run` method, around lines 95-116; the imports at the top; the `#[cfg(test)] mod tests` block)

- [ ] **Step 1: Write the failing test**

Add to the existing `mod tests` in `crates/koad-mcp/src/lib.rs`. `server_with_tool()` already exists there.

```rust
    /// Regression guard: blank input lines logged a parse error, and
    /// notifications (no `id`, e.g. `notifications/initialized`) were
    /// answered with an error response carrying `id: null`, which strict MCP
    /// clients reject.
    #[tokio::test]
    async fn serve_skips_blank_lines_and_never_answers_notifications() {
        let server = server_with_tool();
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\"}}\n",
            "\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
        );
        let mut out: Vec<u8> = Vec::new();
        server.serve(input.as_bytes(), &mut out).await.unwrap();

        let text = String::from_utf8(out).unwrap();
        let ids: Vec<i64> = text
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap()["id"].as_i64().unwrap())
            .collect();
        assert_eq!(ids, vec![1, 2]);
    }
```

- [ ] **Step 2: Run it and confirm it fails to compile (`serve` does not exist)**

Run: `~/.cargo/bin/cargo test -p koad-mcp serve_skips`
Expected: `error[E0599]: no method named `serve``

- [ ] **Step 3: Implement**

Add to the imports at the top of `crates/koad-mcp/src/lib.rs`:

```rust
use tokio::io::{AsyncBufRead, AsyncWrite};
```

Replace the whole `pub async fn run(&self) -> Result<()> { … }` with:

```rust
    pub async fn run(&self) -> Result<()> {
        self.serve(BufReader::new(tokio::io::stdin()), tokio::io::stdout())
            .await
    }

    /// Serve newline-delimited JSON-RPC from `reader`, writing responses to
    /// `writer`. Blank lines are skipped; notifications (no `id`) get no
    /// response, as JSON-RPC requires.
    pub async fn serve<R, W>(&self, reader: R, mut writer: W) -> Result<()>
    where
        R: AsyncBufRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let req: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    tracing::warn!("Ignoring unparseable request: {}", e);
                    continue;
                }
            };
            if req.id.is_none() {
                continue;
            }
            let response = self.handle_request(req).await;
            let res_str = serde_json::to_string(&response)?;
            writer.write_all(res_str.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }
        Ok(())
    }
```

- [ ] **Step 4: Run the crate's tests**

Run: `~/.cargo/bin/cargo test -p koad-mcp`
Expected: all pass, including `serve_skips_blank_lines_and_never_answers_notifications`.

- [ ] **Step 5: Commit**

```bash
git add crates/koad-mcp/src/lib.rs
git commit -m "fix(mcp): skip blank lines and never answer notifications over stdio"
```

---

### Task 2: `koad-os-mcp`: logs to stderr; derived partition

**Files:**
- Modify: `crates/koad-os-mcp/Cargo.toml`
- Modify: `crates/koad-os-mcp/src/main.rs` (the start of `main`; the existing `mod tests`)
- Create: `crates/koad-os-mcp/tests/stdio.rs`

- [ ] **Step 1: Add the dependency**

In `crates/koad-os-mcp/Cargo.toml` under `[dependencies]`:

```toml
koad-core = { path = "../koad-core" }
```

- [ ] **Step 2: Write the failing unit test**

Add to the existing `#[cfg(test)] mod tests` in `crates/koad-os-mcp/src/main.rs`:

```rust
    #[test]
    fn partition_defaults_to_the_canonical_key() {
        let canonical = koad_core::utils::partition::partition_key("clyde");
        assert_eq!(resolve_partition(None, "Clyde"), canonical);
        assert_eq!(resolve_partition(Some(String::new()), "clyde"), canonical);
        assert_eq!(resolve_partition(Some("custom_p".into()), "clyde"), "custom_p");
    }
```

- [ ] **Step 3: Write the failing binary-level test**

Create `crates/koad-os-mcp/tests/stdio.rs`:

```rust
//! Binary-level tests for the stdio transport.

use std::io::Write;
use std::process::{Command, Stdio};

/// Regression guard: in stdio mode tracing wrote to stdout, interleaving log
/// lines with JSON-RPC. Every stdout line must be a JSON-RPC response, and the
/// partition must be derived from AGENT_NAME when AGENT_PARTITION is unset.
#[test]
fn stdio_stdout_carries_only_json_rpc() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_koad-os-mcp"))
        .env("MCP_TRANSPORT", "stdio")
        .env("AGENT_NAME", "clyde")
        .env_remove("AGENT_PARTITION")
        .env("CASS_URL", "http://127.0.0.1:9")
        .env("RUST_LOG", "debug")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn koad-os-mcp");
    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-11-25"}}}}"#).unwrap();
        writeln!(stdin).unwrap();
        writeln!(stdin, r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#).unwrap();
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{{}}}}"#).unwrap();
    }
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    let ids: Vec<i64> = stdout
        .lines()
        .map(|l| {
            serde_json::from_str::<serde_json::Value>(l)
                .unwrap_or_else(|e| panic!("non-JSON stdout line {l:?}: {e}"))["id"]
                .as_i64()
                .unwrap()
        })
        .collect();
    assert_eq!(ids, vec![1, 2]);

    let stderr = String::from_utf8_lossy(&out.stderr);
    let expected = koad_core::utils::partition::partition_key("clyde");
    assert!(stderr.contains(&expected), "partition not derived; stderr: {stderr}");
}
```

- [ ] **Step 4: Run both and confirm they fail**

Run: `~/.cargo/bin/cargo test -p koad-os-mcp`
Expected: the unit test fails to compile (`resolve_partition` not found). Once that compiles (next step), the binary test would fail on a non-JSON stdout line and on the missing `AGENT_PARTITION`.

- [ ] **Step 5: Implement**

In `crates/koad-os-mcp/src/main.rs`, replace `tracing_subscriber::fmt::init();` with:

```rust
    // Logs go to stderr: in stdio mode stdout is the JSON-RPC channel.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
```

Replace the `partition` and `agent_name` reads:

```rust
    let partition = std::env::var("AGENT_PARTITION")
        .map_err(|_| anyhow::anyhow!("AGENT_PARTITION is required (e.g. clyde_Jupiter_ideans)"))?;
    let agent_name = std::env::var("AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("AGENT_NAME is required (e.g. clyde)"))?;
```

with:

```rust
    let agent_name = std::env::var("AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("AGENT_NAME is required (e.g. clyde)"))?;
    let partition = resolve_partition(std::env::var("AGENT_PARTITION").ok(), &agent_name);
```

Add this function above `#[cfg(test)]`:

```rust
/// Partition from `AGENT_PARTITION`, or derived from the agent name with the
/// canonical `partition_key` when unset. Keeping one derivation avoids shell
/// copies of the rule drifting and writing memory to a partition no one reads.
fn resolve_partition(explicit: Option<String>, agent_name: &str) -> String {
    explicit
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| koad_core::utils::partition::partition_key(agent_name))
}
```

- [ ] **Step 6: Run the tests**

Run: `~/.cargo/bin/cargo test -p koad-os-mcp`
Expected: all pass, including `partition_defaults_to_the_canonical_key` and `stdio_stdout_carries_only_json_rpc`.

- [ ] **Step 7: Commit**

```bash
git add crates/koad-os-mcp Cargo.lock
git commit -m "fix(os-mcp): log to stderr; derive partition from AGENT_NAME"
```

---

### Task 3: `koad-os-mcp` tools fail fast when CASS is down

With WSL mirrored networking, a closed loopback port **drops** connection attempts instead of
refusing them (verified 2026-09-27: a `memory.search_semantic` call with CASS on a closed port hung
until killed at 30 s). Every tool connects with `MemoryServiceClient::connect(...)` and no timeout,
so a CASS outage hangs the Windows Claude Code session. The spec requires an explicit error.

**Files:**
- Create: `crates/koad-os-mcp/src/tools/cass.rs`
- Modify: `crates/koad-os-mcp/src/tools/mod.rs` (add `pub mod cass;`)
- Modify: the 7 connect sites: `commit.rs:131`, `intel_get.rs:45`, `list_topics.rs:35`, `recall.rs:51`, `search_semantic.rs:60`, `status.rs:35` (memory), `status.rs:53` (pulse)
- Test: `crates/koad-os-mcp/tests/stdio.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/koad-os-mcp/tests/stdio.rs`:

```rust
/// Regression guard: with CASS silent (TCP accepted, never answered) a tool
/// call hung until the client gave up. It must fail fast with `isError`.
#[test]
fn tool_call_fails_fast_when_cass_does_not_answer() {
    let silent = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", silent.local_addr().unwrap());
    let started = std::time::Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_koad-os-mcp"))
        .env("MCP_TRANSPORT", "stdio")
        .env("AGENT_NAME", "clyde")
        .env_remove("AGENT_PARTITION")
        .env("CASS_URL", &url)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn koad-os-mcp");
    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-11-25"}}}}"#).unwrap();
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"memory.search_semantic","arguments":{{"query":"x","limit":1}}}}}}"#).unwrap();
    }
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();
    assert!(started.elapsed() < std::time::Duration::from_secs(15), "took {:?}", started.elapsed());

    let stdout = String::from_utf8(out.stdout).unwrap();
    let reply: serde_json::Value = stdout
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .find(|v| v["id"] == 2)
        .expect("no response to the tool call");
    assert_eq!(reply["result"]["isError"], true, "{reply}");
    assert!(reply["result"]["content"][0]["text"].as_str().unwrap().contains("CASS unreachable"), "{reply}");
    drop(silent);
}
```

- [ ] **Step 2: Run it and confirm it fails**

Run: `~/.cargo/bin/cargo test -p koad-os-mcp --test stdio tool_call_fails_fast`
Expected: FAIL. Either the elapsed-time assertion or "no response to the tool call" (the call hangs on the HTTP/2 handshake).

- [ ] **Step 3: Implement the helper**

Create `crates/koad-os-mcp/src/tools/cass.rs`:

```rust
//! The one way tools reach CASS, with bounded connect and request time.
//!
//! On WSL with mirrored networking a closed loopback port drops connection
//! attempts instead of refusing them, and a CASS that accepts but does not
//! answer stalls the HTTP/2 handshake. Unbounded, either hangs the MCP client.

use anyhow::{anyhow, Context, Result};
use koad_proto::cass::v1::memory_service_client::MemoryServiceClient;
use koad_proto::cass::v1::pulse_service_client::PulseServiceClient;
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// Open a channel to CASS or fail within `CONNECT_TIMEOUT`.
pub async fn channel(url: &str) -> Result<Channel> {
    let endpoint = Endpoint::from_shared(url.to_string())
        .with_context(|| format!("invalid CASS_URL {url}"))?
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT);
    tokio::time::timeout(CONNECT_TIMEOUT, endpoint.connect())
        .await
        .map_err(|_| {
            anyhow!(
                "CASS unreachable at {url} (no answer within {}s)",
                CONNECT_TIMEOUT.as_secs()
            )
        })?
        .with_context(|| format!("CASS unreachable at {url}"))
}

pub async fn memory(url: &str) -> Result<MemoryServiceClient<Channel>> {
    Ok(MemoryServiceClient::new(channel(url).await?))
}

pub async fn pulse(url: &str) -> Result<PulseServiceClient<Channel>> {
    Ok(PulseServiceClient::new(channel(url).await?))
}
```

Add `pub mod cass;` to `crates/koad-os-mcp/src/tools/mod.rs`.

- [ ] **Step 4: Use it at every connect site**

In `commit.rs`, `intel_get.rs`, `list_topics.rs`, `recall.rs` and `search_semantic.rs`, replace:

```rust
        let mut client = MemoryServiceClient::connect(self.cass_url.clone()).await?;
```

with:

```rust
        let mut client = super::cass::memory(&self.cass_url).await?;
```

In `status.rs`, replace `MemoryServiceClient::connect(self.cass_url.clone()).await` with
`super::cass::memory(&self.cass_url).await`, and `PulseServiceClient::connect(self.cass_url.clone()).await`
with `super::cass::pulse(&self.cass_url).await`. The surrounding `match … { Ok(mut c) => …, Err(e) => … }`
blocks are unchanged; `Err(e)` now carries the "CASS unreachable" message.

Remove any `MemoryServiceClient` / `PulseServiceClient` imports the compiler reports unused.

Run: `/usr/bin/grep -rn "Client::connect(" crates/koad-os-mcp/src/tools/`
Expected: no output.

- [ ] **Step 5: Run the tests**

Run: `~/.cargo/bin/cargo test -p koad-os-mcp`
Expected: all pass, including `tool_call_fails_fast_when_cass_does_not_answer`.

- [ ] **Step 6: Commit**

```bash
git add crates/koad-os-mcp
git commit -m "fix(os-mcp): bound CASS connects so tools fail fast when CASS is down"
```

---

### Task 4: Launchers `koad-wsl-env` and `koad-mcp-stdio`

**Files:**
- Create: `scripts/koad-wsl-env`, `scripts/koad-mcp-stdio` (both executable)
- Create: `crates/koad-cli/tests/wsl_launchers.rs`
- Modify: `crates/koad-cli/Cargo.toml` (`[dev-dependencies]`)
- Modify: `install.sh` (next to both `agent-boot.sh` copies, around lines 230 and 460)
- Modify: `scripts/AGENTS.md` (script index)

- [ ] **Step 1: Add the test dependency**

In `crates/koad-cli/Cargo.toml` under `[dev-dependencies]`:

```toml
tempfile.workspace = true
```

- [ ] **Step 2: Write the failing tests**

Create `crates/koad-cli/tests/wsl_launchers.rs`:

```rust
//! Tests for the Windows-bridge launchers in scripts/.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_script(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts")
        .join(name)
}

/// A fake KOAD_HOME with both launchers in bin/ and a stand-in koad-os-mcp
/// that prints the environment it was given.
fn fake_home() -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(home.path().join("config/identities")).unwrap();
    for s in ["koad-wsl-env", "koad-mcp-stdio"] {
        fs::copy(repo_script(s), bin.join(s)).unwrap();
    }
    let fake = bin.join("koad-os-mcp");
    fs::write(&fake, "#!/usr/bin/env bash\nenv | sort\n").unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    home
}

/// Mimic `wsl.exe -e`: minimal environment, no KOAD_HOME, no KoadOS PATH.
fn run_bare(home: &Path, args: &[&str]) -> Output {
    Command::new("bash")
        .arg(home.join("bin/koad-wsl-env"))
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap()
}

#[test]
fn wsl_env_sets_koad_home_path_and_user() {
    let home = fake_home();
    let out = run_bare(home.path(), &["env"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let env = String::from_utf8(out.stdout).unwrap();
    let h = home.path().display();
    assert!(env.lines().any(|l| l == format!("KOAD_HOME={h}")), "{env}");
    assert!(env.lines().any(|l| l == format!("KOADOS_HOME={h}")), "{env}");
    assert!(env.lines().any(|l| l.starts_with(&format!("PATH={h}/bin:"))), "{env}");
    assert!(env.lines().any(|l| l.starts_with("USER=") && l.len() > 5), "{env}");
}

#[test]
fn mcp_stdio_starts_the_server_for_a_known_agent() {
    let home = fake_home();
    fs::write(home.path().join("config/identities/clyde.toml"), "[identities.clyde]\n").unwrap();
    let out = run_bare(home.path(), &["koad-mcp-stdio", "Clyde"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let env = String::from_utf8(out.stdout).unwrap();
    for want in [
        "MCP_TRANSPORT=stdio",
        "AGENT_NAME=clyde",
        "MCP_MODE=read_write",
        "CASS_URL=http://127.0.0.1:50052",
    ] {
        assert!(env.lines().any(|l| l == want), "missing {want}: {env}");
    }
    assert!(!env.contains("AGENT_PARTITION="), "partition must be derived by koad-os-mcp");
}

#[test]
fn mcp_stdio_refuses_an_unknown_agent_without_touching_stdout() {
    let home = fake_home();
    let out = run_bare(home.path(), &["koad-mcp-stdio", "nobody"]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty(), "stdout must stay clean for MCP");
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown agent"));
}
```

- [ ] **Step 3: Run and confirm failure**

Run: `~/.cargo/bin/cargo test -p koad --test wsl_launchers`
Expected: FAIL, because `fs::copy` cannot find `scripts/koad-wsl-env`.

- [ ] **Step 4: Create `scripts/koad-wsl-env`**

```bash
#!/usr/bin/env bash
# koad-wsl-env — run a command with the KoadOS environment established.
#
# Processes that Windows starts through `wsl.exe -e` get USER and HOME but not
# KOAD_HOME, and $KOAD_HOME/bin is not on PATH; KoadOS config would then fall
# back to the legacy ~/.koad-os install. This script lives in $KOAD_HOME/bin
# and derives KOAD_HOME from its own location.
#
# Usage: koad-wsl-env <command> [args...]
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "usage: koad-wsl-env <command> [args...]" >&2
  exit 64
fi

bin_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KOAD_HOME="$(dirname "$bin_dir")"
export KOAD_HOME
export KOADOS_HOME="$KOAD_HOME"
export PATH="$bin_dir:$PATH"
export USER="${USER:-$(id -un)}"
exec "$@"
```

- [ ] **Step 5: Create `scripts/koad-mcp-stdio`**

```bash
#!/usr/bin/env bash
# koad-mcp-stdio — start the CASS memory MCP server over stdio for one agent.
# Run it through koad-wsl-env (KOAD_HOME must be set). koad-os-mcp derives the
# partition from AGENT_NAME.
#
# Usage: koad-mcp-stdio <agent>
set -euo pipefail

agent="${1:-}"
if [[ -z "$agent" ]]; then
  echo "usage: koad-mcp-stdio <agent>" >&2
  exit 64
fi
: "${KOAD_HOME:?KOAD_HOME is not set; run through koad-wsl-env}"

agent="${agent,,}"
if [[ ! -f "$KOAD_HOME/config/identities/$agent.toml" ]]; then
  echo "koad-mcp-stdio: unknown agent '$agent' (no $KOAD_HOME/config/identities/$agent.toml)" >&2
  exit 1
fi

export MCP_TRANSPORT=stdio
export AGENT_NAME="$agent"
export MCP_MODE=read_write
export CASS_URL="${CASS_URL:-http://127.0.0.1:50052}"
unset AGENT_PARTITION
exec "$KOAD_HOME/bin/koad-os-mcp"
```

Then make both executable, in the working tree and in git:

```bash
chmod +x scripts/koad-wsl-env scripts/koad-mcp-stdio
```

- [ ] **Step 6: Run the tests**

Run: `~/.cargo/bin/cargo test -p koad --test wsl_launchers`
Expected: 3 passed.

- [ ] **Step 7: Install them with `install.sh`**

In `install.sh`, directly after the `agent-boot.sh` block in `run_update` (the one using `$bin_dir`), add:

```bash
        # Windows body bridge launchers (see docs/superpowers/specs/2026-09-27-windows-body-bridge-design.md)
        for launcher in koad-wsl-env koad-mcp-stdio; do
            cp "scripts/$launcher" "$bin_dir/$launcher"
            chmod +x "$bin_dir/$launcher"
        done
        ok "  ✓ Updated Windows bridge launchers"
```

Directly after the `agent-boot.sh` block in `run_install` (the one using `$BIN_DIR`), add:

```bash
    for launcher in koad-wsl-env koad-mcp-stdio; do
        cp "scripts/$launcher" "$BIN_DIR/$launcher"
        chmod +x "$BIN_DIR/$launcher"
    done
```

Run: `bash -n install.sh`
Expected: no output (syntax OK).

- [ ] **Step 8: Index them in `scripts/AGENTS.md`**

Add these rows to the table, after the `install-services.sh` row:

```markdown
| `koad-wsl-env` | Run a command with the KoadOS environment set (for processes launched from Windows via `wsl.exe`) |
| `koad-mcp-stdio` | Start the CASS memory MCP server over stdio for one agent (Windows body bridge) |
```

- [ ] **Step 9: Commit**

```bash
git add scripts/koad-wsl-env scripts/koad-mcp-stdio scripts/AGENTS.md crates/koad-cli/tests/wsl_launchers.rs crates/koad-cli/Cargo.toml Cargo.lock install.sh
git commit -m "feat(bridge): koad-wsl-env and koad-mcp-stdio launchers"
```

---

### Task 5: `koad-agent anchor --body windows`

**Files:**
- Create: `crates/koad-agent/src/commands/anchor.rs`
- Modify: `crates/koad-agent/src/commands/mod.rs`
- Modify: `crates/koad-agent/src/lib.rs` (`Commands` enum and the `match` in `run`)

- [ ] **Step 1: Write the module with its tests; render functions as stubs**

Create `crates/koad-agent/src/commands/anchor.rs`:

```rust
//! `koad-agent anchor`: print an identity anchor for another body.
//!
//! The Windows body bridge runs this from a Claude Code for Windows
//! SessionStart hook through `wsl.exe`; its stdout becomes session context.
//! It writes no files and mints no Citadel session: memory goes to CASS
//! directly over MCP.

use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_proto::cass::v1::hydration_service_client::HydrationServiceClient;
use koad_proto::cass::v1::HydrationRequest;
use koad_proto::citadel::v5::WorkspaceLevel;
use std::time::Duration;
use tonic::transport::Endpoint;

/// Where the anchored session runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum AnchorBody {
    /// Claude Code (or similar) on native Windows, bridged to WSL.
    Windows,
}

/// Identity fields shown in the anchor.
pub struct AnchorIdentity<'a> {
    pub name: &'a str,
    pub role: &'a str,
    pub rank: &'a str,
    pub bio: &'a str,
}

/// `\\wsl.localhost\<distro>\…` form of a Linux path.
pub fn wsl_unc(distro: &str, linux_path: &str) -> String {
    unimplemented!()
}

/// Render the Windows-body anchor. `cass_packet` is `None` when CASS was
/// unreachable.
pub fn render_windows_anchor(
    id: &AnchorIdentity,
    timestamp: &str,
    koad_home: &str,
    vault_unc: &str,
    cass_packet: Option<&str>,
) -> String {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clyde() -> AnchorIdentity<'static> {
        AnchorIdentity {
            name: "Clyde",
            role: "Citadel Officer and Implementation Engineer",
            rank: "Officer",
            bio: "Sovereign KoadOS Agent.",
        }
    }

    const VAULT: &str = r"\\wsl.localhost\Ubuntu\home\ideans\.citadel-jupiter\agents\KAPVs\clyde";

    #[test]
    fn unc_path_for_a_linux_path() {
        assert_eq!(
            wsl_unc("Ubuntu", "/home/ideans/.citadel-jupiter/agents/KAPVs/clyde"),
            VAULT
        );
    }

    #[test]
    fn windows_anchor_has_identity_body_and_memory_tools() {
        let a = render_windows_anchor(&clyde(), "T", "/home/ideans/.citadel-jupiter", VAULT, Some("## Ⅰ. Episodes\n- x\n"));
        assert!(a.starts_with("# KoadOS Agent Identity Anchor\n"), "{a}");
        assert!(a.contains("Name: Clyde"));
        assert!(a.contains("Body: windows"));
        assert!(a.contains("memory.search_semantic"));
        assert!(a.contains("memory.commit"));
        assert!(a.contains(VAULT));
        assert!(a.contains("wsl.exe -e /home/ideans/.citadel-jupiter/bin/koad-wsl-env koad"));
        assert!(a.contains("## 🧠 Temporal Context Packet (CASS)\n## Ⅰ. Episodes"));
    }

    /// The WSL-body session instructions do not apply on Windows.
    #[test]
    fn windows_anchor_has_no_wsl_session_instructions() {
        let a = render_windows_anchor(&clyde(), "T", "/k", VAULT, Some("p"));
        for wsl_only in ["agent-boot", "current.env", "koad system heartbeat", "koad-functions.sh"] {
            assert!(!a.contains(wsl_only), "unexpected {wsl_only:?} in {a}");
        }
    }

    #[test]
    fn offline_cass_is_stated_not_hidden() {
        let a = render_windows_anchor(&clyde(), "T", "/k", VAULT, None);
        assert!(a.contains("Memory: offline (CASS unreachable)"), "{a}");
        assert!(!a.contains("Temporal Context Packet"));
    }
}
```

In `crates/koad-agent/src/commands/mod.rs`, add `pub mod anchor;` next to the other `pub mod` lines.
(The `pub use` comes in Step 4, once `handle_anchor` exists.)

- [ ] **Step 2: Run the tests and confirm they fail**

Run: `~/.cargo/bin/cargo test -p koad-agent anchor`
Expected: 4 tests FAIL with `not implemented`.

- [ ] **Step 3: Implement**

Replace the two `unimplemented!()` bodies:

```rust
pub fn wsl_unc(distro: &str, linux_path: &str) -> String {
    format!(r"\\wsl.localhost\{}{}", distro, linux_path.replace('/', r"\"))
}
```

```rust
pub fn render_windows_anchor(
    id: &AnchorIdentity,
    timestamp: &str,
    koad_home: &str,
    vault_unc: &str,
    cass_packet: Option<&str>,
) -> String {
    let mut s = format!(
        "# KoadOS Agent Identity Anchor\n\
         Generated At: {timestamp}\n\
         Body: windows (Claude Code for Windows, bridged to Citadel Jupiter in WSL)\n\n\
         ## Identity\nName: {}\nRole: {}\nRank: {}\n\n## Bio\n{}\n",
        id.name, id.role, id.rank, id.bio
    );
    s.push_str(&format!(
        "\n## Working Environment (Windows body)\n\
         - **Memory:** use the `citadel-memory` MCP tools. Recall with \
         `memory.search_semantic` / `memory.recall` before rebuilding knowledge; store durable \
         lessons with `memory.commit` and verify by recall. Also `memory.list_topics`, \
         `intel.get`, `status.citadel`.\n\
         - **KoadOS CLI:** runs only in WSL. From PowerShell, when truly needed: \
         `wsl.exe -e {koad_home}/bin/koad-wsl-env koad <command>`.\n\
         - **Vault:** `{vault_unc}`\n\
         - **Handoffs:** inbox files in the Citadel home in WSL (see the `koad-inbox` skill).\n"
    ));
    match cass_packet {
        None => s.push_str(
            "\nMemory: offline (CASS unreachable). Memory tools will return errors until CASS is back.\n",
        ),
        Some(p) if !p.is_empty() => {
            s.push_str("\n## 🧠 Temporal Context Packet (CASS)\n");
            s.push_str(p);
        }
        Some(_) => {}
    }
    s
}
```

Add the CASS fetch and the command handler below `render_windows_anchor`:

```rust
/// Ask CASS for the agent's hydration packet. `None` when CASS is unreachable.
pub async fn fetch_cass_packet(
    cass_addr: &str,
    agent: &str,
    project_root: &str,
    timeout: Duration,
) -> Option<String> {
    // Bounded: on WSL mirrored networking a down CASS drops packets, and an
    // unbounded connect would stall the SessionStart hook.
    let endpoint = Endpoint::from_shared(cass_addr.to_string())
        .ok()?
        .connect_timeout(timeout)
        .timeout(timeout);
    let channel = tokio::time::timeout(timeout, endpoint.connect())
        .await
        .ok()?
        .ok()?;
    let mut client = HydrationServiceClient::new(channel);
    let req = tonic::Request::new(HydrationRequest {
        agent_name: agent.to_string(),
        project_root: project_root.to_string(),
        level: WorkspaceLevel::LevelUnspecified as i32,
        token_budget: 4000,
        task_id: String::new(),
    });
    client
        .hydrate(req)
        .await
        .ok()
        .map(|r| r.into_inner().markdown_packet)
}

fn expand_home(path: &str) -> String {
    match path.strip_prefix("~/") {
        Some(rest) => format!("{}/{}", dirs::home_dir().unwrap_or_default().display(), rest),
        None => path.to_string(),
    }
}

/// Print the identity anchor for `agent` running in `body`.
pub async fn handle_anchor(config: &KoadConfig, agent: &str, body: AnchorBody) -> Result<()> {
    let AnchorBody::Windows = body;
    let key = agent.to_lowercase();
    let identity = config
        .identities
        .get(&key)
        .with_context(|| format!("Unknown agent '{agent}'"))?;
    let koad_home = config.home.to_string_lossy().to_string();
    let vault = identity
        .vault
        .clone()
        .unwrap_or_else(|| format!("{koad_home}/agents/KAPVs/{key}"));
    let distro = std::env::var("WSL_DISTRO_NAME").unwrap_or_else(|_| "Ubuntu".to_string());
    let vault_unc = wsl_unc(&distro, &expand_home(&vault));
    let packet = fetch_cass_packet(
        &config.network.cass_grpc_addr,
        &identity.name,
        &koad_home,
        Duration::from_secs(5),
    )
    .await;
    let id = AnchorIdentity {
        name: &identity.name,
        role: &identity.role,
        rank: &identity.rank,
        bio: &identity.bio,
    };
    print!(
        "{}",
        render_windows_anchor(
            &id,
            &chrono::Utc::now().to_rfc3339(),
            &koad_home,
            &vault_unc,
            packet.as_deref()
        )
    );
    Ok(())
}
```

- [ ] **Step 4: Wire the subcommand**

In `crates/koad-agent/src/commands/mod.rs`, add `pub use anchor::handle_anchor;` next to the other `pub use` lines.

In `crates/koad-agent/src/lib.rs`, add this variant to `pub enum Commands` after `Boot { … }`:

```rust
    /// Print an identity anchor for another body (e.g. Claude Code for Windows).
    Anchor {
        /// The name of the agent.
        agent: String,
        /// Which body the session runs in.
        #[arg(long, value_enum)]
        body: commands::anchor::AnchorBody,
    },
```

Add this arm to the `match cli.command` in `run`:

```rust
        Commands::Anchor { agent, body } => {
            commands::handle_anchor(&config, &agent, body).await?;
        }
```

- [ ] **Step 5: Run the tests**

Run: `~/.cargo/bin/cargo test -p koad-agent`
Expected: all pass, including the 4 anchor tests.

- [ ] **Step 6: Live check of stdout**

Run:

```bash
~/.cargo/bin/cargo build -p koad --bin koad-agent
./target/debug/koad-agent anchor Clyde --body windows | head -3
```

Expected: the first line is exactly `# KoadOS Agent Identity Anchor`, with nothing before it (stdout becomes context).

- [ ] **Step 7: Commit**

```bash
git add crates/koad-agent
git commit -m "feat(agent): koad-agent anchor --body windows"
```

---

### Task 6: `body_windows` config logic (pure, tested)

**Files:**
- Create: `crates/koad-cli/src/handlers/body_windows.rs`
- Modify: `crates/koad-cli/src/handlers/mod.rs` (add `pub mod body_windows;`)

- [ ] **Step 1: Write the module with tests and stub bodies**

Create `crates/koad-cli/src/handlers/body_windows.rs`:

```rust
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
    unimplemented!()
}

/// MCP server definition for `claude.exe mcp add-json`.
pub fn mcp_server_json(distro: &str, koad_home: &str, agent: &str) -> Value {
    unimplemented!()
}

/// Add (or replace) the KoadOS SessionStart hook, preserving everything else.
pub fn merge_session_hook(settings: Value, command: &str) -> anyhow::Result<Value> {
    unimplemented!()
}

/// Remove the KoadOS SessionStart hook, dropping containers left empty.
pub fn remove_session_hook(settings: Value) -> Value {
    unimplemented!()
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
        let settings = json!({"effortLevel": "high", "hooks": {"SessionStart": [other.clone()], "Stop": []}});
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
```

In `crates/koad-cli/src/handlers/mod.rs`, add `pub mod body_windows;`.

- [ ] **Step 2: Run and confirm failure**

Run: `~/.cargo/bin/cargo test -p koad body_windows`
Expected: 6 tests FAIL with `not implemented`.

- [ ] **Step 3: Implement**

Replace the four stub bodies:

```rust
pub fn hook_command(distro: &str, koad_home: &str, agent: &str) -> String {
    format!("wsl.exe -d {distro} -e {koad_home}/bin/koad-wsl-env koad-agent anchor {agent} --body windows")
}

pub fn mcp_server_json(distro: &str, koad_home: &str, agent: &str) -> Value {
    json!({
        "type": "stdio",
        "command": "wsl.exe",
        "args": ["-d", distro, "-e", format!("{koad_home}/bin/koad-wsl-env"), "koad-mcp-stdio", agent]
    })
}

fn is_koad_group(group: &Value) -> bool {
    group["hooks"].as_array().is_some_and(|hooks| {
        hooks
            .iter()
            .any(|h| h["command"].as_str().is_some_and(|c| c.contains(HOOK_MARKER)))
    })
}

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
```

- [ ] **Step 4: Run the tests**

Run: `~/.cargo/bin/cargo test -p koad body_windows`
Expected: 6 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/koad-cli/src/handlers/body_windows.rs crates/koad-cli/src/handlers/mod.rs
git commit -m "feat(bridge): Claude Code for Windows config merge logic"
```

---

### Task 7: `koad body windows install | status | uninstall`

**Files:**
- Modify: `crates/koad-cli/src/handlers/body_windows.rs` (add side-effect functions)
- Modify: `crates/koad-cli/src/cli.rs` (`Commands` enum; new `BodyAction`, `WindowsBodyAction`)
- Modify: `crates/koad-cli/src/main.rs` (dispatch)

These functions drive Windows executables through WSL interop. They are verified live in Task 8; their logic lives in the tested functions from Task 6.

- [ ] **Step 1: Add the CLI definitions**

In `crates/koad-cli/src/cli.rs`, add this variant to `pub enum Commands`:

```rust
    /// Link another harness body (Claude Code for Windows) to this Citadel.
    Body {
        #[command(subcommand)]
        action: BodyAction,
    },
```

Add these enums at the end of `crates/koad-cli/src/cli.rs`:

```rust
#[derive(Subcommand)]
pub enum BodyAction {
    /// Claude Code for Windows, bridged over stdio through wsl.exe.
    Windows {
        #[command(subcommand)]
        action: WindowsBodyAction,
    },
}

#[derive(Subcommand)]
pub enum WindowsBodyAction {
    /// Register the memory MCP server, the SessionStart identity hook and the memory skills.
    Install {
        #[arg(long, default_value = "clyde")]
        agent: String,
    },
    /// Check every link of the bridge.
    Status {
        #[arg(long, default_value = "clyde")]
        agent: String,
    },
    /// Remove exactly what install added.
    Uninstall,
}
```

- [ ] **Step 2: Add the side-effect functions**

Append to `crates/koad-cli/src/handlers/body_windows.rs` (above `#[cfg(test)]`), and add the imports shown at the top of the file:

```rust
use crate::cli::{BodyAction, WindowsBodyAction};
use anyhow::{bail, Context, Result};
use koad_core::config::KoadConfig;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const SKILLS_SOURCE: &str = "https://github.com/Fryymann/koad-os/tree/nightly/skills";

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
    let wsl = Command::new("wslpath").arg("-u").arg(&win).output()?;
    let profile = PathBuf::from(String::from_utf8_lossy(&wsl.stdout).trim());
    let claude = profile.join(".local/bin/claude.exe");
    if !claude.exists() {
        bail!("Claude Code for Windows not found at {}", claude.display());
    }
    Ok(WindowsEnv { profile, claude })
}

fn distro() -> String {
    std::env::var("WSL_DISTRO_NAME").unwrap_or_else(|_| "Ubuntu".to_string())
}

fn settings_path(win: &WindowsEnv) -> PathBuf {
    win.profile.join(".claude/settings.json")
}

/// Read settings.json (missing = `{}`), transform it, back it up, write it.
/// Invalid JSON aborts without changing anything.
fn edit_settings(path: &Path, f: impl FnOnce(Value) -> Result<Value>) -> Result<()> {
    let current = if path.exists() {
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str(&text)
            .with_context(|| format!("{} is not valid JSON; nothing changed", path.display()))?
    } else {
        json!({})
    };
    let updated = f(current)?;
    if path.exists() {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        std::fs::copy(path, path.with_file_name(format!("settings.json.bak-{stamp}")))?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&updated)? + "\n")?;
    Ok(())
}

fn run_windows(cmd: &mut Command) -> Result<bool> {
    Ok(cmd.current_dir("/mnt/c").status()?.success())
}

fn install(config: &KoadConfig, agent: &str) -> Result<()> {
    let win = detect_windows()?;
    let (distro, home) = (distro(), config.home.to_string_lossy().to_string());

    let _ = Command::new(&win.claude)
        .args(["mcp", "remove", MCP_NAME, "--scope", "user"])
        .current_dir("/mnt/c")
        .output();
    let server = mcp_server_json(&distro, &home, agent).to_string();
    if !run_windows(Command::new(&win.claude).args(["mcp", "add-json", MCP_NAME, &server, "--scope", "user"]))? {
        bail!("claude.exe mcp add-json failed");
    }
    println!("✓ MCP server '{MCP_NAME}' registered");

    let hook = hook_command(&distro, &home, agent);
    edit_settings(&settings_path(&win), |s| merge_session_hook(s, &hook))?;
    println!("✓ SessionStart hook added to {}", settings_path(&win).display());

    let mut skills = Command::new("cmd.exe");
    skills.args(["/c", "npx", "-y", "skills@1.7.0", "add", SKILLS_SOURCE, "-g", "-a", "claude-code", "-s"]);
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
        .args(["-d", distro, "-e", &format!("{home}/bin/koad-wsl-env"), "koad-mcp-stdio", agent])
        .current_dir("/mnt/c")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    {
        let stdin = child.stdin.as_mut().context("stdin")?;
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-11-25"}}}}"#)?;
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"memory.search_semantic","arguments":{{"query":"KoadOS","limit":1}}}}}}"#)?;
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

fn status(config: &KoadConfig, agent: &str) -> Result<()> {
    let win = detect_windows()?;
    let (distro, home) = (distro(), config.home.to_string_lossy().to_string());
    let mut ok = true;
    let mut check = |label: &str, pass: bool| {
        println!("{} {label}", if pass { "✓" } else { "✗" });
        ok &= pass;
    };

    let registered = Command::new(&win.claude)
        .args(["mcp", "get", MCP_NAME])
        .current_dir("/mnt/c")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    check("MCP server registered in Claude Code for Windows", registered);

    let settings = std::fs::read_to_string(settings_path(&win)).unwrap_or_default();
    check("SessionStart hook present", settings.contains(HOOK_MARKER));

    let anchor = Command::new("wsl.exe")
        .args(["-d", &distro, "-e", &format!("{home}/bin/koad-wsl-env"), "koad-agent", "anchor", agent, "--body", "windows"])
        .current_dir("/mnt/c")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).starts_with("# KoadOS Agent Identity Anchor"))
        .unwrap_or(false);
    check("Hook command prints the identity anchor", anchor);

    let skills = WINDOWS_SKILLS
        .iter()
        .all(|s| win.profile.join(".claude/skills").join(s).join("SKILL.md").exists());
    check("Memory skills installed", skills);

    check("MCP round trip: semantic search through wsl.exe", mcp_round_trip(&distro, &home, agent)?);

    if !ok {
        bail!("Windows body bridge is not fully linked");
    }
    Ok(())
}

fn uninstall() -> Result<()> {
    let win = detect_windows()?;
    let _ = run_windows(Command::new(&win.claude).args(["mcp", "remove", MCP_NAME, "--scope", "user"]));
    println!("✓ MCP server '{MCP_NAME}' removed");
    if settings_path(&win).exists() {
        edit_settings(&settings_path(&win), |s| Ok(remove_session_hook(s)))?;
    }
    println!("✓ SessionStart hook removed");
    let mut skills = Command::new("cmd.exe");
    skills.args(["/c", "npx", "-y", "skills@1.7.0", "remove"]);
    skills.args(WINDOWS_SKILLS).args(["-g", "-y"]);
    let _ = run_windows(&mut skills);
    println!("✓ Skills removed: {}", WINDOWS_SKILLS.join(", "));
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
```

- [ ] **Step 2b: Test the settings-file safety**

Add to the `mod tests` in `crates/koad-cli/src/handlers/body_windows.rs`:

```rust
    #[test]
    fn edit_settings_refuses_invalid_json_and_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert!(edit_settings(&path, |s| merge_session_hook(s, "x koad-agent anchor")).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not json");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1, "no backup written");
    }

    #[test]
    fn edit_settings_backs_up_before_writing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{\"effortLevel\":\"high\"}").unwrap();
        edit_settings(&path, |s| merge_session_hook(s, "x koad-agent anchor")).unwrap();
        let backups: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("settings.json.bak-"))
            .collect();
        assert_eq!(backups.len(), 1);
        let written: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(written["effortLevel"], "high");
    }
```

Run: `~/.cargo/bin/cargo test -p koad body_windows`
Expected: 8 passed.

- [ ] **Step 3: Dispatch**

In `crates/koad-cli/src/main.rs`, add this arm to the main `match` over `Commands`:

```rust
        Commands::Body { action } => {
            crate::handlers::body_windows::handle(action, &config).await?;
        }
```

- [ ] **Step 4: Build and run the full test suite**

Run: `~/.cargo/bin/cargo build -p koad && ~/.cargo/bin/cargo test --workspace --release`
Expected: builds with no new warnings; 0 failures.

- [ ] **Step 5: Commit**

```bash
git add crates/koad-cli
git commit -m "feat(bridge): koad body windows install|status|uninstall"
```

---

### Task 8: Deploy and verify live

- [ ] **Step 1: Deploy**

```bash
cd ~/koados-citadel && ./install.sh --update
```

Expected: `Updated Windows bridge launchers`. If it lists system services on old binaries, have Ian run the printed `sudo systemctl restart …`; this change doesn't need it, since only binaries launched per session changed.

- [ ] **Step 2: Install the bridge**

Run: `koad body windows install`
Expected: all five `status` checks are `✓` at the end.

- [ ] **Step 3: Cross-body memory check**

From WSL, confirm a Windows-side commit is recallable:

```bash
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"memory.commit","arguments":{"content":"Windows body bridge verified 2026-09-27: commit made through wsl.exe stdio.","tags":["bridge","verification"]}}}' \
  | wsl.exe -d Ubuntu -e ~/.citadel-jupiter/bin/koad-wsl-env koad-mcp-stdio clyde
```

Then search for it from WSL through the regular `citadel-memory` MCP (`memory.search_semantic`, query "windows body bridge verified"). Expected: the card appears.

- [ ] **Step 4: Real session (Ian)**

In PowerShell: `cd C:\data\projects\survival-game; claude`. Expected:
- The session context starts with `# KoadOS Agent Identity Anchor … Body: windows`.
- `/mcp` lists both `citadel-memory` (connected) and `Roblox_Studio`.
- Asking "what do you remember about the Windows bridge?" makes a `memory.search_semantic` call that returns the Step 3 card.

---

### Task 9: Document

**Files:**
- Modify: `AGENTS.md` ("Deploying on Jupiter" section)

- [ ] **Step 1: Add the Windows body line**

Append to the "Deploying on Jupiter" section of `AGENTS.md`:

```markdown
Claude Code for Windows is linked as a second Clyde body with `koad body windows install`
(check with `koad body windows status`): memory over MCP stdio through `wsl.exe`, identity via a
SessionStart hook. Design: `docs/superpowers/specs/2026-09-27-windows-body-bridge-design.md`.
```

- [ ] **Step 2: Commit**

```bash
git add AGENTS.md
git commit -m "docs: Windows body bridge in AGENTS.md"
```
