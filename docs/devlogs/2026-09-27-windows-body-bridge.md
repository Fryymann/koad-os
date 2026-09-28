# Dev Log — 2026-09-27: Windows Body Bridge

Author: Clyde (Claude Code, Opus 5.5). Reviewed and merged by Ian.
PRs: [#253](https://github.com/Fryymann/koad-os/pull/253) (spec),
[#254](https://github.com/Fryymann/koad-os/pull/254) (plan),
[#255](https://github.com/Fryymann/koad-os/pull/255) (implementation), all into `nightly`.

## Why

The Roblox Studio and Blender MCP servers only work from native Windows. KoadOS runs in WSL2, and
Windows cannot reach WSL over TCP: the Hyper-V firewall's `DefaultInboundAction` is `Block`, and a
closed loopback port drops the connection attempt instead of refusing it, so a connect just hangs.
Ian wanted coding agents on Windows to have the whole memory system, semantic search included, not
a file copy.

Decisions Ian made: the Windows agent is "Clyde on Windows" (the same identity and partition, not a
new agent); it gets a full body (memory read and write, identity anchor, skills); and transport is
stdio through `wsl.exe` now, with an HTTP-through-the-firewall option only documented.

## What changed

### Memory over stdio (`koad-mcp`, `koad-os-mcp`)

- `koad-mcp` `serve()` runs an MCP stdio loop. Blank lines and notifications (no `id`) are never
  answered; an explicit `"id": null` is a request and is answered. Non-object JSON is logged.
- `koad-os-mcp` logs to stderr (stdout is the JSON-RPC stream), resolves its partition from the
  agent name with `partition_key` when `AGENT_PARTITION` is unset, and bounds every CASS call:
  3 s to connect, 8 s per request, reported as "CASS unreachable". A cold semantic search was
  measured at 1.8 s.

### Launchers (`scripts/`, installed by `install.sh`)

- `koad-wsl-env`: processes started by `wsl.exe -e` get no `KOAD_HOME` and no `$KOAD_HOME/bin` on
  `PATH`. The wrapper sets both from its own resolved location, falls back `USER` to `id -un`
  (the partition key needs it), then execs its arguments.
- `koad-mcp-stdio <agent>`: validates the agent name, requires its identity file, and starts the
  read-write stdio bridge.

### Identity (`koad-agent anchor <agent> --body windows`)

Prints the Windows identity anchor: the vault as a `\\wsl.localhost\<distro>\...` path, how to reach
memory and the CLI from Windows, and the CASS hydration packet. If CASS is down it prints an
offline line after 3 s and still exits 0. The vault is resolved the same way as `koad-agent boot`,
through a new `resolve_vault_path_unchecked` in `koad-core`.

### Setup (`koad body windows install|status|uninstall`)

- Registers the `citadel-memory` MCP server with `claude.exe mcp add-json --scope user`.
- Adds a SessionStart hook to `C:\Users\<u>\.claude\settings.json`. Edits back up first, refuse
  invalid JSON, tolerate a BOM, replace the file atomically, and skip the write when nothing changes.
- Installs `cass-recall` and `cass-search` with `npx skills`, and records only the skills it added
  in `$KOAD_HOME/state/body-windows.json`, so uninstall never removes a skill it didn't install.
- Every call to a Windows program is time-bounded. `status` runs the hook exactly as stored and gives
  a reason for every failed check.

## What the reviews caught

Each task went through a spec review and a code-quality review, then the whole branch had a final
review. The findings that mattered:

- **Git Bash rewrote the hook's path.** Claude Code on Windows runs a hook command string through
  Git Bash when it is installed, and Git Bash converts `/home/...` into `C:/Program Files/Git/home/...`.
  Windows sessions would have started without an identity, while `status` (run from WSL) showed ✓.
  The hook now uses the documented exec form (`command` + `args`, no shell). For command lines Windows
  agents type, the WSL path starts with `//`, which survives Git Bash, PowerShell and cmd.
- **Uninstall matched too broadly.** It removed a whole hook group if any hook in it looked like
  ours. It now removes only our own entries, recognised by exact shape.
- **Uninstall would have deleted skills that were there before install.** Fixed by the ownership record.
- **Nothing had a time limit.** A CASS that accepts a connection but never answers would have frozen
  `install` after it had changed things.

## Verification

- `cargo test --workspace --release`: 317 passed, 0 failed.
- Deployed from `nightly` with `install.sh --update`; `koad body windows install` passed all five
  status checks.
- A memory committed through the Windows stdio bridge was the top semantic-search hit from WSL, in
  the shared `clyde_Jupiter_ideans` partition.
- Ian started Claude Code on Windows: Clyde's anchor loaded and `/mcp` showed `citadel-memory`
  connected.

## Follow-up the same day

- Removed a stale `mcpServers.citadel-memory` HTTP entry (`localhost:9742`) from the Windows
  `settings.json`, at Ian's request.
- Cleared the May-era skill copies on the Windows side (`agent-boot`, `koad-cognitive`, `koad-intel`,
  `rook-boot` and two non-KoadOS skills). They assumed WSL paths and would have failed or misled.
  Windows now carries only the central `cass-recall` and `cass-search`, lock-tracked from
  `Fryymann/koad-os`.

## Behaviour changes for existing services

- `clyde-mcp.service` info logs moved from `logs/clyde-mcp.log` to `logs/clyde-mcp.error.log`.
- The 9744 and 9745 bridges now return "CASS unreachable" after 8 s instead of waiting.

## Still open

- Windows agents have no Citadel session, so `koad` commands that need one (CLI `intel remember`,
  heartbeat, hot context) don't work from Windows. Memory itself does, through the bridge. A Windows
  boot mode is possible if a real need appears.
- Central skills other than `cass-recall` and `cass-search` call `koad`, `rtk` or Ollama, which only
  work in WSL. They would need Windows variants that go through `wsl.exe`.
- The HTTP transport option (a firewall rule for a loopback-only port) is documented in the spec,
  not built.
