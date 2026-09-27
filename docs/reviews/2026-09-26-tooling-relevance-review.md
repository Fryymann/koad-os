# Tooling Relevance Review — 2026-09-26

Author: Clyde (Claude Code, Opus 5.5), at Ian's request.
Status: **proposal for discussion.** Nothing here has been removed or changed.

## Why

Much of KoadOS's agent tooling dates from early 2026, when coding agents had no reliable file tools,
small context windows and no memory. Harnesses (Claude Code, Codex, Gemini CLI, opencode) and models
have improved a lot since then. This review sorts each tool by what it does **today**: working and
valuable, superseded by the harness, a stub, or misconfigured.

Every claim below comes from reading source or running the tool on 2026-09-26. Anything not
verified is marked **unverified**.

## 1. Stubs: installed but not implemented

These ship in `$KOAD_HOME/bin` or the CLI but do nothing, or only pretend to.

| Tool | Evidence | Recommendation |
|---|---|---|
| `koad-fs-mcp` | `crates/koad-cli/src/bin/koad-fs-mcp.rs`: prints `not yet implemented`, exits 1. Unchanged since the initial commit. | **Implement** (Ian wants it for local models). See §5. |
| `koad-notion-mcp` | Same 4-line stub. | **Remove.** Notion now ships an official MCP connector. |
| `koad-map` (binary) | Same 4-line stub. `koad map` works as a `koad` subcommand; the binary is a dead duplicate. | **Remove the binary.** |
| `koad-mcp` (binary) | `src/bin/koad-mcp.rs` prints `KoadOS MCP Server (Unified)` and exits. The `koad-mcp` *library* is real and used by `koad-os-mcp`. | **Remove the binary**, keep the library. |
| Signal service | `koad-citadel/src/services/signal.rs`: `send_signal` discards the payload and returns `Signal sent (stub)`; `get_signals` always returns empty. Crew already uses inbox files instead. | **Decide:** implement or delete. Today it misleads any agent that trusts `Signal dispatched`. |
| `koad fleet` (all but `board`) | `Fleet action placeholder.` | Remove or implement. |
| `koad project` | `list` prints a header and nothing else; everything else prints `not yet fully implemented in v4.1`. | Remove or implement. |
| `koad vault` (unhandled actions) | `[STUB] This vault action is not yet implemented`. | Trim the CLI to the implemented actions. |
| `koad bridge` (unhandled actions) | `Bridge action placeholder.` | Trim the CLI to the implemented actions. |
| `koad intel mind` (non-status actions) | `Mind action placeholder.` | Trim. |
| Admin `get_snippet` / `trigger_backup` RPCs | `admin.rs` returns `"Snippet content placeholder"` and backup ID `bkp-placeholder`. `koad system backup` prints `[OK]` for a backup that never happened. | **Fix or remove.** A fake-success backup is dangerous. |
| `koad saveup` step 1 | Sends the Citadel `Shutdown` RPC and prints `Hot-stream drained to durable memory`. On 2026-09-26 this **stopped the Citadel's gRPC servers (port 50051 gone) while the process stayed alive**, so systemd saw a healthy unit and didn't restart it. Every later `koad-agent boot` failed with `[OFFLINE] ... not reachable at 127.0.0.1:50051`, and `koad system status` still reported PASS. | **Fix urgently.** A save should not take down the control plane. Either drain without shutting down, or exit the process so `Restart=` brings it back. Make `system status` probe 50051, not just the socket file. |

## 2. Superseded by the harness

These solved real early-2026 problems that the harnesses now solve natively.

| Tool / protocol | What it was for | Why it's superseded | Recommendation |
|---|---|---|---|
| Anchor rule "**No-Read**" (no reading whole files over 50 lines; use `grep_search` / `read_file` with line ranges) | Small context windows, expensive tokens | Claude Code's Read already takes offset/limit, context is 1M, and `grep_search` / `read_file` are Gemini CLI tool names, not Claude Code's. | **Drop from the anchor**, or turn it into a harness-neutral guideline: "read what you need." |
| Anchor rule "**Filesystem Protocol: all file ops via `koadFsMcp`**" | No safe file tools in early harnesses | Every current harness has scoped, permissioned file tools. The mandated tool doesn't exist (§1). | **Drop for harness-hosted agents.** Keep the fs MCP for local models only (§5). |
| `koad intel snippet` | Line-range file read | Harness Read with offset/limit does this. | Remove, or keep only for harness-less models. |
| `koad-codegraph` / "Crate API Maps" in the CASS packet | Structural code maps so agents don't read files | `codegraph.db` is 12 KB and the anchor's "Crate API Maps" section is empty. The external `code-review-graph` MCP is configured for Codex and Continue, and Claude has a skill for it. | **Candidate for removal**; confirm nothing else depends on it. |
| `koad map` protocol in the anchor | Orientation in a strange directory | Useful but thin: it prints a directory tree. Harnesses list files natively. | Keep the command, drop it from the mandatory anchor. |
| `koad cognitive` | Detect agent drift | Printed `[WARN] L1: Session ID not found in environment` with a valid session sourced. **Unverified** whether it measures anything meaningful. | Review with Ian. |
| `koad xp` | Agent XP and levels | Gamification. Working memory still shows XP 807 from May, so nothing has updated it since. | Ian's call. |

## 3. Actively harmful as built

| Issue | Evidence | Recommendation |
|---|---|---|
| **Boot overwrites the project's `CLAUDE.md`** | `crates/koad-agent/src/commands/boot.rs:415,418,421` write the anchor to `GEMINI.md`, `CLAUDE.md` and `AGENTS.md` in the **current directory** with a plain `fs::write`. The home-directory copies on the lines before use `safe_write_anchor`. At boot on 2026-09-26 it replaced `survival-game/CLAUDE.md`, a tracked file holding that project's real instructions. The same thing happened to skylinks on 2026-05-02. | **Highest-value fix.** Write identity only to user-level files (`~/.claude/CLAUDE.md`, etc.), never to a project's instruction files. |
| **90-second session lease; heartbeat doesn't extend it** | Documented in the agent-boot skill. Memory writes fail after 90s unless the agent re-mints. | Fix the heartbeat, or lengthen the lease for harness sessions. |
| **`install.sh --update` reports success without restarting services** | `sudo -n` fails and it prints a warning, but it still ends `Citadel installations have been successfully updated`. This has bitten three sessions (June, twice; September). | End with a clear **"services still running old binaries"** banner plus the exact restart command, or check `/proc/<pid>/exe` itself. |

## 4. Misconfigured right now

| Issue | Evidence | Recommendation |
|---|---|---|
| Codex `citadel-memory` points at a dead port | `~/.codex/config.toml` → `127.0.0.1:9746`. Nothing listens on 9746. | Add a Codex CASS MCP unit or point Codex at an existing one. |
| `hermes-cass-mcp.service` runs a stale binary | PID 476 started 17:21, before both deploys. Still has the protocol-version echo bug fixed in #223. | `systemctl --user restart hermes-cass-mcp.service`, with Hermes/Ian's go-ahead. |
| MCP servers bound to all interfaces | `koad-os-mcp` on `0.0.0.0:9744` and `0.0.0.0:9745`; Qdrant on `*:6333/6334`; unknown listener on `*:9743` (owner not visible without root). | Bind to `127.0.0.1` unless LAN access is intended; identify 9743. |

## 5. `koad-fs-mcp`: keep, but build it thin

Ian wants filesystem access for local models. Two facts shape the design:

1. **An MCP server only helps a model whose harness is an MCP client.** Local models running under
   Gemini CLI (configured: `qwen3:14b` as the `generalist` agent), opencode or Continue already get
   the harness's own file tools. The real gap is raw Ollama tool-calling with no harness, such as
   `ollama-delegate` Branch B, and any harness without file tools.
2. **The original plan was to wrap the official server** (`@modelcontextprotocol/server-filesystem`,
   see `crates/koad-cli/.koad-os/docs/requests/skill_request__agent_tbd_mcp_filesystem_server_integration.md`).
   That's still the right call. It already provides allow-listed roots, read, write, edit with diff
   preview, and search. Node is available on Jupiter.

Suggested shape: `koad-fs-mcp` becomes a thin launcher that resolves the agent's allowed roots from
its identity TOML and KAPV, then execs the official server with those roots. KoadOS contributes the
per-agent scoping; it doesn't reimplement file I/O.

## 6. Keep: working and valuable

- **CASS + citadel-memory MCP.** Semantic recall works: a paraphrased query returned the right fact
  first. This is the part harnesses still don't give you: durable, cross-agent, cross-harness memory.
- **`koad intel remember` / `ponder`, the updates board, KAPV vaults.** Working, and they give real
  continuity between sessions and between agents.
- **`agent-boot` identity hydration**, once the `CLAUDE.md` overwrite and lease issues (§3) are fixed.
- **`ollama-delegate`**: live VRAM check plus a model-routing matrix for local models.
- **`install.sh`**, with the restart messaging fix.
- **The WASM plugin host** (`koad-plugins`): real and now tested, but see the note below.

**Unverified:** whether anything registers WASM tools in practice. The registry is in-memory only;
no SQLite database has a tool or plugin table. Registered tools are lost on every CASS restart, so
either nothing relies on them or something re-registers them at boot. Worth confirming before
investing more in the plugin path.

## Suggested order

1. Stop boot from overwriting project `CLAUDE.md` / `AGENTS.md` / `GEMINI.md` (§3).
2. Restart `hermes-cass-mcp`; fix Codex's MCP port; bind MCP servers and Qdrant to localhost (§4).
3. Fix `saveup` step 1, which takes the Citadel down while systemd and `system status` report it
   healthy, then the other fake-success paths: `system backup` and the `install.sh` restart message (§1, §3).
4. Trim the anchor: drop the No-Read and `koadFsMcp` mandates and the map protocol for
   harness-hosted agents (§2).
5. Delete the stub binaries and placeholder CLI actions, or file issues to implement them (§1).
6. Build `koad-fs-mcp` as a scoped launcher for the official filesystem server (§5).
7. Decide on the signal service, `koad-codegraph`, `koad xp` and `koad cognitive`.
