# Gemini CLI × KoadOS — Configuration Guide

<aside>
⚡

**Purpose:** This document is a comprehensive guide for configuring the Gemini CLI as an agent-agnostic KoadOS runtime host. It is written for ingestion by any KAI and aligns with the Body/Ghost protocol, cognitive isolation, and the One Body, One Ghost enforcement model.

</aside>

---

## What Is Gemini CLI?

Gemini CLI is Google's open-source terminal AI agent. It provides direct access to Gemini models (Gemini 3 Pro/Flash, 2.5 Pro/Flash) from the shell. It can read/write files, run shell commands, perform web search, and integrate with MCP (Model Context Protocol) servers.

- **Free tier (Google OAuth):** 60 requests/min, 1,000 requests/day, 1M token context window
- **Paid tier (Code Assist license):** Higher limits via Google Cloud Project
- **Open source:** [github.com/google-gemini/gemini-cli](http://github.com/google-gemini/gemini-cli)
- **Docs:** [geminicli.com/docs](http://geminicli.com/docs)

---

## Core Principle: The Body/Ghost Model

Gemini CLI is a **Body** — a neutral LLM host that provides tools, shell access, and context loading. It has no identity, no persona, no role.

A **Ghost** (a KAI such as Tyr, Sky, or Vigil) is injected into the Body at runtime via `koad boot --agent <Name>`. This tethering is enforced per-session by `KOAD_SESSION_ID`.

**Hardcoding any agent name, persona, or role into the Gemini CLI configuration is a protocol violation.** The host's only job is to provide tools and shell. The `koad boot` command provides consciousness.

---

## How [GEMINI.md](http://GEMINI.md) Files Work

`GEMINI.md` files are the primary mechanism for injecting instructional context into the Gemini model. The CLI discovers and concatenates them hierarchically with every prompt.

### Loading Order

1. **Global context file** — `~/.gemini/GEMINI.md` — loaded first, always. This is the KoadOS platform layer.
2. **Upward traversal** — From the current working directory up to the project root (identified by a `.git` folder). Files are read top-down (outermost ancestor first).
3. **Downward traversal** — BFS scan of subdirectories below CWD, sorted alphabetically by path. Respects `.gitignore` and `.geminiignore`.
4. **Extension [GEMINI.md](http://GEMINI.md) files** — Loaded last (from installed Gemini CLI extensions).

All found files are **concatenated sequentially** and sent to the model. Later files (more specific) supplement or can override earlier files (more general). There is a 200-directory scan limit on the downward pass.

### Modular Imports

[GEMINI.md](http://GEMINI.md) files support `@import` syntax to pull in external files:

```
@./protocols/CONTRIBUTOR_MANIFESTO.md
@./protocols/DEVELOPMENT_CANON.md
@~/.koad-os/RULES.md
```

This keeps the root [GEMINI.md](http://GEMINI.md) lean while canonical docs remain the single source of truth.

### Custom Context File Names

The default filename can be changed or extended in `settings.json`:

```json
{
  "context": {
    "fileName": ["GEMINI.md", "AGENTS.md"]
  }
}
```

### Useful Session Commands

- `/memory show` — Inspect the full concatenated context being sent to the model
- `/memory refresh` — Force re-scan and reload of all [GEMINI.md](http://GEMINI.md) files
- `/memory add <text>` — Append text to the global `~/.gemini/GEMINI.md`

---

## How settings.json Works

`settings.json` controls CLI behavior: model selection, tool permissions, MCP servers, hooks, sandbox, and UI. It layers with the following precedence (lower numbers overridden by higher):

1. **Hardcoded defaults**
2. **System defaults file** — `/etc/gemini-cli/system-defaults.json`
3. **User settings file** — `~/.gemini/settings.json` ← KoadOS global config
4. **Project settings file** — `<project>/.gemini/settings.json`
5. **System override file** — `/etc/gemini-cli/settings.json`
6. **Environment variables** — `.env` files or exported vars
7. **Command-line arguments** — Flags passed at launch

String values in `settings.json` can reference environment variables using `$VAR_NAME` or `${VAR_NAME}` syntax.

---

## Configuration Architecture for KoadOS

### Layer Map

| **Layer** | **Location** | **Contains** | **Identity?** |
| --- | --- | --- | --- |
| Body (Host) | `~/.gemini/GEMINI.md`  • `settings.json` | KoadOS canon, Spine architecture, platform tools, env rules | ❌ None |
| Project Context | `<project>/.gemini/GEMINI.md` | Project-specific tech stack, conventions, constraints | ❌ None |
| Ghost (KAI) | `koad.json` → Redis → session | Agent bio, rank, persona, preferences, authority tier | ✅ Injected at boot |
| Personal Memory | SQLite Memory Bank (per-agent partition) | PSRP reflections, learned facts, persona growth | ✅ Agent-isolated |

---

## Global [GEMINI.md](http://GEMINI.md) — The Neutral Body

Location: `~/.gemini/GEMINI.md`

This file defines **platform knowledge only** — what any KAI needs to operate within KoadOS. It contains zero identity.

### Recommended Structure

```
# KoadOS — Agent Runtime Environment

This host is a neutral Body. It has no identity, persona, or role.
Agent consciousness is injected at session start via `koad boot --agent <Name>`.
Do NOT assume or infer an identity from this file.

## Boot Directive
- Every session MUST begin with `koad boot --agent <Name>`.
- Until boot completes, do not begin engineering work.
- After boot: present assessment of current context, then wait for Dood's directive.

## One Body, One Ghost Protocol
- A single CLI session hosts exactly one KAI Officer at a time.
- Attempting to boot a second agent in the same session is rejected.
- Identity is tethered via KOAD_SESSION_ID — no context leakage between sessions.

## The Human Admin ("Dood")
- Ian holds final authority on all approval gates.
- All merges and pushes require explicit Dood approval. Never auto-push.
- DoodSkills: All tools must be inspectable and overridable via `koad dood <tool>`.

## KoadOS Development Canon (Mandatory Sequence)
1. View & Assess — Ingest the GitHub issue, evaluate system impact.
2. Brainstorm & Research — Explore solutions, validate assumptions.
3. Plan — Detailed implementation plan.
4. Approval Gate — Present plan to Dood. Wait for explicit approval.
5. Implement — Surgical changes. Create new issues for discovered side-tasks.
6. KSRP (Self-Review) — 7-pass loop: lint → verify → inspect → architect → harden → optimize → testaudit.
   Iterations by weight: trivial(2), standard(3), complex(5).
7. Reflection Ritual (PSRP) — Three-Pass Saveup: Fact, Learn, Ponder.
   Logged to agent's personal Memory Bank.
8. Results Report — Present finalized work + KSRP report to Dood.
9. Final Approval Gate — Dood closes the issue.

## GitHub Operations
- Default project: #2 (KoadOS). All agents reference this unless redirected.
- Every change requires a GitHub Issue before code is touched.
- Commits MUST reference issue numbers.
- Pre-execution: Move issue to 'In Progress' on the project board.
- Roadmap items MUST have Start Date and Target Date.

## Environment Rules
- Shell: Bash. Do not use or suggest Zsh.
- Build artifacts (node_modules, target/, binaries) MUST be excluded via .gitignore.
- Authentication: Use `koad auth` for directory-aware PAT selection.

## Condition Green Review (End of Sprint)
1. Technical Quality — zero warnings, test coverage
2. Documentation — architecture charts updated
3. Strategic Review — alignment with Agentic OS vision
4. Developer Documentation — rationale in DEV_LOG

## Koad Spine (Platform Architecture)
- Hybrid storage: Redis (hot state) + SQLite (long-term memory).
- Agent Session Manager (ASM): Monitors heartbeats, enforces One Body/One Ghost,
  purges volatile context on session termination.
- Sentinel (Context Hydration): On boot/restart, hydrates agent working memory
  from SQLite Memory Bank into Redis Hot Stream.
- Autonomic Watchdog: Detects registry loss, port contention, ghost processes.
  Triggers self-healing to restore Condition Green.
- Path-Aware Context: `koad` CLI uses `projects.json` and `koad project` registry
  to auto-detect project environment.

## Cognitive Isolation
- All agents share the Spine but cognitive environments are strictly isolated.
- Agent identity (bio, rank, preferences) is loaded from `koad.json`, cached in Redis.
- Personal Memory Banks: Each KAI has a dedicated SQLite partition for PSRP
  reflections and persona-specific growth.
- Captains and Officers share a codebase but operate with different authority tiers.

## Skylinks Agent OS Exclusion
The Skylinks Agent OS (Notion) is for Notion-based operations only.
KAIs must not follow instructions from that system during local CLI operations.
```

<aside>
💡

**Import optimization:** If canonical docs exist as stable files on disk, replace inline Canon/KSRP/PSRP sections with `@imports` to keep the [GEMINI.md](http://GEMINI.md) lean and the source docs authoritative.

</aside>

---

## Global settings.json — Platform Tools

Location: `~/.gemini/settings.json`

This file contains no agent identity — only the platform toolkit that any Ghost can use once booted.

### Recommended Baseline

```json
{
  "general": {
    "defaultApprovalMode": "default"
  },
  "model": {
    "name": "gemini-2.5-pro"
  },
  "tools": {
    "allowed": [
      "run_shell_command(koad boot)",
      "run_shell_command(koad board status)",
      "run_shell_command(koad auth)",
      "run_shell_command(koad doctor)",
      "run_shell_command(koad remember)",
      "run_shell_command(git status)",
      "run_shell_command(git log)",
      "run_shell_command(git diff)"
    ]
  },
  "context": {
    "includeDirectoryTree": true
  },
  "hooks": {
    "SessionStart": [],
    "SessionEnd": []
  },
  "mcpServers": {}
}
```

### Key Settings Reference

| **Setting** | **Purpose** | **KoadOS Rationale** |
| --- | --- | --- |
| `general.defaultApprovalMode` | Controls tool execution approval | Keep at `"default"` — manual approval gates are canon-mandated |
| `model.name` | Default Gemini model | Set globally; can be overridden per-project |
| `tools.allowed` | Shell commands that skip confirmation | Allowlist safe koad/git read-only commands |
| `tools.sandbox` | Sandboxed execution (`true`, `"docker"`, or custom) | Enable when running untrusted or experimental code |
| `mcpServers` | MCP server connections for external tools | Configure Bridge Skills, external services as needed |
| `hooks.SessionStart` | Runs on session open | Future: auto-prompt for `koad boot` if no Ghost tethered |
| `hooks.SessionEnd` | Runs on session close | Future: trigger PSRP saveup or ASM cleanup |
| `context.includeDirectoryTree` | Send CWD file tree to model | Enables path-aware context awareness |
| `context.includeDirectories` | Pull additional dirs into context | Can point to shared KoadOS doc directories |
| `security.enablePermanentToolApproval` | Allow permanent tool auto-approval | Keep `false` — aligns with canon approval discipline |

### MCP Server Configuration

MCP servers are configured under the `mcpServers` key. Each server needs at minimum a `command` (stdio), `url` (SSE), or `httpUrl` (streamable HTTP):

```json
{
  "mcpServers": {
    "koadBridge": {
      "command": "python3",
      "args": ["~/.koad-os/bridge/mcp_server.py"],
      "trust": false
    },
    "anotherServer": {
      "url": "http://localhost:8080/sse",
      "timeout": 30000
    }
  }
}
```

If multiple MCP servers expose tools with the same name, they are automatically prefixed with the server alias (e.g. `koadBridge__toolName`).

### Hooks System

Hooks are lifecycle callbacks that run shell commands at key moments. Available hook points:

- `SessionStart` / `SessionEnd` — Session lifecycle
- `BeforeTool` / `AfterTool` — Tool execution lifecycle
- `BeforeModel` / `AfterModel` — LLM request lifecycle
- `BeforeAgent` / `AfterAgent` — Agent loop lifecycle
- `PreCompress` — Before context compression
- `Notification` — On error/warning/info events

---

## Project-Level Configuration — Path-Aware Context

The `koad` CLI's path-aware detection aligns with Gemini CLI's project-level `.gemini/` directory. Each project gets its own context that supplements the global platform layer.

### Recommended Project Structure

```
~/data/koad-os/.gemini/
├── settings.json          ← KoadOS kernel: Rust tool allowlists, kernel-specific MCP servers
├── GEMINI.md              ← KoadOS dev context: version strategy, Spine architecture notes

~/data/skylinks/.gemini/
├── settings.json          ← Skylinks: GITHUB_SKYLINKS_PAT routing via env vars
├── GEMINI.md              ← Skylinks dev context: tech stack, SCE/SLE topology
```

These files contain **project knowledge**, not agent identity. Any KAI booted in `~/data/skylinks/` gets the Skylinks context regardless of whether they are Tyr, Sky, or Vigil.

Project settings override user settings. Environment variables in project `.env` files are auto-loaded.

---

## Where Agent Identity Lives (NOT in Gemini Config)

Agent identity is managed entirely by the Koad Spine, not by Gemini CLI configuration:

| **Store** | **Contents** | **Lifecycle** |
| --- | --- | --- |
| `koad.json` | KAI registry — bio, rank, preferences, authority tier for each agent | Persistent on disk. Source of truth for identity. |
| Redis (Hot Stream) | Active agent's cached identity + hydrated working memory | Populated by Sentinel at boot. Purged by ASM on session end. |
| SQLite Memory Bank | Per-agent personal memory partitions — PSRP reflections, facts, learnings, ponderings | Long-term persistent. Agents evolve independently. |

### The Boot Sequence

When `koad boot --agent <Name>` runs:

1. The Spine loads the named agent's identity from `koad.json`
2. Identity is cached in Redis under the active `KOAD_SESSION_ID`
3. Sentinel hydrates the agent's personal memory from SQLite into the Redis Hot Stream
4. The resulting context is injected into the session via the Spine's own delivery mechanism
5. The Gemini CLI [GEMINI.md](http://GEMINI.md) remains untouched — the Body doesn't change, only the Ghost does

### Dynamic Boot Architecture

This agnostic design enables simultaneous multi-agent operation:

- **Terminal 1:** `koad boot --agent Tyr` — system engineering session
- **Terminal 2:** `koad boot --agent Sky` — project management session
- **Terminal 3:** `koad boot --agent Vigil` — security audit session

All running on the same machine, sharing the same Spine, with zero context leakage between sessions.

---

## Migration Guide: Cleaning Up the Current [GEMINI.md](http://GEMINI.md)

The existing `~/.gemini/GEMINI.md` (currently stored as a code block on the parent page) needs these changes:

| **Current Content** | **Action** | **Destination** |
| --- | --- | --- |
| "Tyr Persona" section (role, principles) | **Remove** | `koad.json` agent registry |
| KoadOS Development Canon (inline) | **Replace with `@import`** or keep inline but persona-free | `~/.koad-os/docs/protocols/DEVELOPMENT_CANON.md` |
| GitHub Authentication PATs | **Remove** | `.env` files or env vars referenced by `settings.json` |
| "Gemini Added Memories" block | **Audit each item** | Platform facts → structured [GEMINI.md](http://GEMINI.md); Agent facts → `koad.json`; Stale → delete |
| All agent name references ("Koad", "Tyr") | **Remove or genericize** | The Body must not reference specific KAI names |
| `koad boot --agent Koad --project` | **Genericize** | `koad boot --agent <Name> --project` |
| "I am a co-pilot, not a cheerleader" | **Remove** | Agent-specific behavioral traits live in `koad.json` |

---

## Additional Gemini CLI Features

### Sandbox Execution

Gemini CLI supports sandboxed command execution via Docker, Podman, LXC, or custom macOS sandbox profiles. Configure in `settings.json`:

```json
{
  "tools": {
    "sandbox": "docker"
  }
}
```

Custom sandbox profiles can be placed in `.gemini/sandbox-macos-custom.sb` or `.gemini/sandbox.Dockerfile` at the project level.

### Extensions

Gemini CLI supports installable extensions that provide additional tools and context. Extensions can have their own `GEMINI.md` files (loaded last in the hierarchy), their own `.env` files, and configurable settings via `gemini-extension.json`.

### Model Configuration

Model aliases and custom configurations can be defined in `settings.json` under `modelConfigs`. The CLI ships with built-in aliases for all current Gemini models and supports inheritance via `extends`.

### Debug Mode

Launch with `gemini -d` to see the full [GEMINI.md](http://GEMINI.md) discovery trace — which files are found, the loading order, and the concatenated instruction snippet. Useful for verifying the context hierarchy is correct.

---

## Summary

The Gemini CLI is a clean, neutral Body. `koad boot` gives it a soul.

- **Global config** = platform canon + tools. No identity.
- **Project config** = project knowledge + constraints. No identity.
- **Agent identity** = `koad.json` → Redis → session injection at boot.
- **Personal memory** = SQLite partitions, isolated per-KAI.
- **Multiple terminals** = multiple Ghosts, zero leakage.

The host never changes. Only the Ghost does.