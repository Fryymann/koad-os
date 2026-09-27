# Windows Body Bridge — Design

**Date:** 2026-09-27 · **Author:** Clyde, with Ian · **Status:** approved design, awaiting spec review

## Problem

Game-dev work needs coding agents on native Windows, next to Roblox Studio and Blender and their MCP
servers. KoadOS (Citadel, CASS, MCP bridges) runs in WSL2. A Windows agent currently has no
identity and no access to CASS memory, including semantic search.

Networking is blocked. WSL runs in mirrored mode, but the Hyper-V firewall for the WSL VM has
`DefaultInboundAction: Block`. On 2026-09-27, Windows requests to WSL listeners timed out for every
bind address tested (127.0.0.1 and 0.0.0.0) and for both `localhost` and the host IP.

## Goal

Clyde runs as a second *body* on Windows (Claude Code for Windows) with the same identity and CASS
partition as in WSL. Memory recall, semantic search and commits all work, and nothing new is
exposed on the network.

**Success criteria**

1. A Windows Claude Code session starts as Clyde with a fresh memory packet.
2. From Windows, `memory.search_semantic` returns relevant cards for a paraphrased query.
3. A `memory.commit` from Windows is recallable from a WSL session, and the reverse.
4. Roblox Studio MCP (and later Blender MCP) keep working in the same Windows session.
5. No firewall changes; CASS and MCP ports stay bound to 127.0.0.1.

**Feasibility, verified 2026-09-27:** from PowerShell, piping MCP JSON-RPC through
`wsl.exe -e koad-os-mcp` in stdio mode returned `initialize` in 108 ms. `tools/list` and a
semantic search for "how do I restart the memory server without sudo" returned the relevant card
through CASS's embeddings.

## Approach

**Primary (A): stdio through `wsl.exe`.** Windows launches WSL processes; all Citadel components
stay in WSL. No ports and no firewall change are needed, and `wsl.exe` starts the distro on demand.

**Later (B), appendix only:** a loopback-only Hyper-V firewall rule plus HTTP MCP, for Windows
tools that need HTTP.

Rejected: a native Windows build of `koad-os-mcp`, which would still need Windows→WSL networking.

## Architecture

```
Windows: Claude Code (C:\Users\idean\.claude)
 ├─ SessionStart hook ──► wsl.exe -d Ubuntu -e …/bin/koad-wsl-env koad-agent anchor Clyde --body windows
 │                           prints identity + CASS packet → added to session context
 ├─ MCP "citadel-memory" ─► wsl.exe -d Ubuntu -e …/bin/koad-wsl-env koad-mcp-stdio clyde
 │                           └─ koad-os-mcp (stdio, read_write, partition clyde_Jupiter_ideans)
 │                                └─ CASS 127.0.0.1:50052 → SQLite, Qdrant embeddings, Ollama
 ├─ skills: cass-recall, cass-search
 └─ Roblox Studio MCP, Blender MCP (native Windows, unchanged)
```

Everything that executes runs in WSL. Windows holds only configuration.

## Components

### 1. Launchers (`$KOAD_HOME/bin`, installed from `scripts/`)

Processes started by `wsl.exe -e` get `USER` and `HOME`, but **not `KOAD_HOME`, and
`$KOAD_HOME/bin` is not on `PATH`** (verified 2026-09-27). Without `KOAD_HOME`, KoadOS config
resolution falls back to the legacy `~/.koad-os` install. Two small scripts fix that:

- **`koad-wsl-env <command> [args…]`** establishes the KoadOS environment for a process launched
  from Windows, then execs the command:
  - sets `KOAD_HOME` and `KOADOS_HOME` from the script's own location;
  - prepends `$KOAD_HOME/bin` to `PATH`;
  - sets `USER` from `id -un` if missing.

  Every Windows-side command goes through it.
- **`koad-mcp-stdio <agent>`** checks that `$KOAD_HOME/config/identities/<agent>.toml` exists (for an
  unknown agent: non-zero exit, message on stderr, server never starts). It then execs `koad-os-mcp`
  with `MCP_TRANSPORT=stdio`, `AGENT_NAME=<agent>`, `MCP_MODE=read_write` and
  `CASS_URL=http://127.0.0.1:50052`.

The partition is **not** computed in shell. `koad-os-mcp` derives it from `AGENT_NAME` with
`koad_core::utils::partition::partition_key` whenever `AGENT_PARTITION` is unset, so there is
one source of truth. Today `AGENT_PARTITION` is required; making it optional is part of this work.

### 2. `koad-os-mcp` / `koad-mcp` changes

Found by the feasibility probe:

- In stdio mode, tracing output goes to **stdout** interleaved with JSON-RPC. It must go to stderr,
  with no ANSI colour.
- A blank input line logs `Failed to parse request: EOF`. Blank lines must be skipped silently.
- `AGENT_PARTITION` becomes optional and is derived from `AGENT_NAME` when unset (see component 1).

### 3. `koad-agent anchor <agent> --body windows`

- Prints the identity anchor to stdout. It writes no files and mints no Citadel session: MCP memory
  access goes to CASS directly and needs no session.
- Content: identity, bio, principles and the CASS hydration packet (agent-scoped episodes, #250),
  as at boot.
- The Working Environment section is written for Windows:
  - memory through the `citadel-memory` MCP tools (`memory.search_semantic`, `memory.recall`,
    `memory.commit`, `memory.list_topics`, `intel.get`, `status.citadel`);
  - the `koad` CLI is WSL-only, so run `wsl.exe -e koad …` from PowerShell when truly needed;
  - the vault is at `\\wsl.localhost\Ubuntu\home\ideans\.citadel-jupiter\agents\KAPVs\clyde`.
- The packet states the body (`windows`).
- If CASS is unreachable it prints the identity with `Memory: offline (CASS unreachable)` and exits 0,
  so the session still starts.

### 4. `koad body windows install | status | uninstall`

**install** (from WSL; safe to re-run):

1. Resolve `%USERPROFILE%` via `cmd.exe` + `wslpath`, and locate `claude.exe`. Stop with a clear
   message if either is missing.
2. Register the MCP server with Windows Claude Code's own CLI:
   `claude.exe mcp add-json citadel-memory '{"type":"stdio","command":"wsl.exe","args":["-d","Ubuntu","-e","/home/ideans/.citadel-jupiter/bin/koad-wsl-env","koad-mcp-stdio","clyde"]}' --scope user`.
   An existing entry of the same name is replaced.
3. Back up `C:\Users\idean\.claude\settings.json` to `settings.json.bak-<timestamp>`. Merge in a
   SessionStart hook running
   `wsl.exe -d Ubuntu -e /home/ideans/.citadel-jupiter/bin/koad-wsl-env koad-agent anchor Clyde --body windows` with a
   30-second timeout. Other keys and hooks are preserved; an existing KoadOS hook is replaced,
   not duplicated. If the file is not valid JSON, stop and change nothing.
4. Install `cass-recall` and `cass-search` on the Windows side with the skills CLI (Windows
   `npx`) from `https://github.com/Fryymann/koad-os/tree/nightly/skills`, target `claude-code`.
5. Run `status`.

**status** reports each link:

- MCP entry registered, and a stdio round trip returns tools
- hook present, and its output starts with the identity header
- skills installed
- CASS answers a search

**uninstall** removes exactly the MCP entry, the KoadOS hook and the two skills.

Distro (`Ubuntu`), agent (`clyde`) and Windows profile are resolved or passed as flags; nothing is
hard-coded in code.

## Behaviour and safety

- **One Clyde, two bodies.** Both may run at once. They share the partition, and CASS handles
  concurrent writes.
- **No network exposure.** Stdio only; loopback bindings unchanged; no firewall edits.
- **Config edits** are limited to Windows Claude Code's user config: backed up, merged, and
  reversible with `uninstall`.

## Failure handling

| Situation | Behaviour |
|---|---|
| WSL not running | `wsl.exe` starts it; the first call is slower (30s hook timeout) |
| CASS or Ollama down | The anchor says "memory offline"; MCP tools return explicit errors, never empty results posing as "no memories" |
| Unknown agent passed to the launcher | Non-zero exit with a message; the server does not start |
| `settings.json` not valid JSON | Install stops, nothing changed |
| `claude.exe` not found | Install stops with the expected path |

## Testing

Test-first for each component:

- **`koad-os-mcp` stdio:** stdout contains only JSON-RPC lines (a regression test for tracing
  output); blank lines produce no output and no error.
- **Launchers:**
  - `koad-wsl-env` under `env -i HOME=… USER=…` sets `KOAD_HOME` and `PATH` and runs the command;
  - `koad-mcp-stdio` with an unknown agent exits non-zero with nothing on stdout;
  - `koad-os-mcp` with only `AGENT_NAME` set uses `partition_key(agent)`.
- **Windows anchor:**
  - no session or `koad` CLI instructions;
  - the vault path is under `\\wsl.localhost`;
  - the body is stated;
  - offline CASS yields the degraded line and exit 0.
- **Settings merge:**
  - adds the hook and preserves unrelated keys and hooks;
  - re-running changes nothing;
  - uninstall restores the prior content;
  - invalid JSON is refused.
- **Live, from PowerShell:**
  - `status` all green;
  - a semantic search returns a relevant card;
  - a commit from Windows is recallable in WSL;
  - a real Windows Claude Code session starts as Clyde with Roblox Studio MCP still working.

## Out of scope

The `koad` CLI natively on Windows; HTTP access (appendix); identities other than Clyde on Windows
(the launcher and anchor take an agent argument, but only Clyde is installed).

## Appendix: option B (HTTP over loopback), not built

For a Windows tool that needs HTTP rather than stdio, allow loopback inbound to the WSL VM for
specific ports only, from an elevated PowerShell:

```powershell
New-NetFirewallHyperVRule -Name KoadMcpLoopback -DisplayName "KoadOS MCP (loopback)" `
  -Direction Inbound -VMCreatorId '{40E0AC32-46A5-438A-A0B2-2B479E8F2E90}' `
  -Protocol TCP -LocalPorts 9744,9745 -RemoteAddresses 127.0.0.1
```

Services stay bound to 127.0.0.1, so LAN and tailnet peers remain blocked. Verify with
`Invoke-WebRequest http://127.0.0.1:9744/health` before relying on it.
