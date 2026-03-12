<aside>
⚡

**Purpose:** Comprehensive guide for configuring Claude Code as an agent-agnostic KoadOS runtime host. Written for ingestion by any KAI and aligns with the Body/Ghost protocol, cognitive isolation, and One Body, One Ghost enforcement. Companion to the [Gemini CLI × KoadOS — Configuration Guide](https://www.notion.so/Gemini-CLI-KoadOS-Configuration-Guide-d15313afca924453a684ad9c8a744977?pvs=21) and [Codex CLI Deep Research & KoadOS Parity Report](https://www.notion.so/Codex-CLI-Deep-Research-KoadOS-Parity-Report-61fc1e8009ca41f0baa66849c618203e?pvs=21).

</aside>

---

## What Is Claude Code?

Claude Code is Anthropic's agentic coding tool for the terminal. It reads your codebase, edits files, runs commands, and integrates with your development tools.

- **Runtime:** TypeScript + React + Ink on Bun
- **Models:** Claude Sonnet 4, Claude Opus 4.5, Claude Haiku (configurable)
- **Context:** 200K tokens (auto-compaction when full)
- **Surfaces:** CLI (terminal), VS Code, JetBrains, Desktop App, Browser, GitHub Actions, GitLab CI/CD
- **Docs:** [code.claude.com/docs](http://code.claude.com/docs)

---

## Core Principle: The Body/Ghost Model

Claude Code is a **Body** — a neutral LLM host that provides tools, shell access, and context loading. It has no identity, no persona, no role.

A **Ghost** (a KAI such as Tyr, Sky, or Vigil) is injected into the Body at runtime via `koad boot --agent <Name>`. This tethering is enforced per-session by `KOAD_SESSION_ID`.

**Hardcoding any agent name, persona, or role into the Claude Code configuration is a protocol violation.** The host's only job is to provide tools and shell. The `koad boot` command provides consciousness.

---

## How [CLAUDE.md](http://CLAUDE.md) Files Work

`CLAUDE.md` files are the primary mechanism for injecting instructional context into the Claude model. The CLI discovers and concatenates them hierarchically with every prompt.

### Loading Order

1. **Global context file** — `~/.claude/CLAUDE.md` — loaded first, always. This is the KoadOS platform layer.
2. **Upward traversal** — From CWD up to the project root (identified by `.git`). Ancestor-first.
3. **Project root** — `CLAUDE.md` in the repo root.
4. **Nested directories** — `CLAUDE.md` files in subdirectories below CWD.

All found files are **concatenated sequentially** and injected into the system prompt. Later files (more specific) supplement or can override earlier files (more general).

### Local Overrides

`CLAUDE.local.md` files work identically to `CLAUDE.md` but are intended for personal/machine-specific instructions. Add to `.gitignore` — they should never be committed.

### Memory Commands

- `/memory` — View and edit memory files from within a session
- `claude memory add "text"` — Append to project-level [CLAUDE.md](http://CLAUDE.md) from the shell
- `claude memory add -g "text"` — Append to global `~/.claude/CLAUDE.md`

---

## How Settings Work

`settings.json` controls Claude Code behavior: permissions, tools, MCP servers, hooks, and model. It layers with the following precedence (lower numbers overridden by higher):

1. **Enterprise policy** — `/etc/claude-code/settings.json` (admin-enforced)
2. **User settings** — `~/.claude/settings.json` ← KoadOS global config
3. **Project settings** — `.claude/settings.json` (committed, team-shared)
4. **Project local** — `.claude/settings.local.json` (personal, gitignored)
5. **CLI flags** — Flags passed at launch

---

## Configuration Architecture for KoadOS

### Layer Map

| **Layer** | **Location** | **Contains** | **Identity?** |
| --- | --- | --- | --- |
| Body (Host) | `~/.claude/CLAUDE.md`  • `settings.json` | KoadOS canon, Spine architecture, platform tools, env rules | ❌ None |
| Project Context | `<project>/.claude/CLAUDE.md` or `CLAUDE.md` at repo root | Project-specific tech stack, conventions, constraints | ❌ None |
| Ghost (KAI) | `koad.json` → Redis → session | Agent bio, rank, persona, preferences, authority tier | ✅ Injected at boot |
| Personal Memory | SQLite Memory Bank (per-agent partition) | PSRP reflections, learned facts, persona growth | ✅ Agent-isolated |

---

## Global [CLAUDE.md](http://CLAUDE.md) — The Neutral Body

Location: `~/.claude/CLAUDE.md`

This file defines **platform knowledge only** — what any KAI needs to operate within KoadOS. It contains zero identity.

### Recommended Structure

```markdown
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

---

## Global settings.json — Platform Tools & Hooks

Location: `~/.claude/settings.json`

This file contains no agent identity — only the platform toolkit and enforcement hooks.

### Recommended Baseline

```json
{
  "permissions": "default",
  "model": "sonnet",
  "allowedTools": [
    "Bash(koad boot*)",
    "Bash(koad board*)",
    "Bash(koad auth*)",
    "Bash(koad doctor*)",
    "Bash(koad intel*)",
    "Bash(git status*)",
    "Bash(git log*)",
    "Bash(git diff*)",
    "Read",
    "Grep",
    "Glob"
  ],
  "deniedTools": [
    "Bash(rm -rf*)",
    "Bash(git push --force*)"
  ],
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "echo $CLAUDE_CODE_TOOL_INPUT | jq -r '.file_path // .filePath // empty' | grep -qE '^\.koad-os/|koad\.json$|koad\.db$' && echo 'BLOCKED: Sanctuary Rule — protected KoadOS path' && exit 2 || exit 0"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "if [ -n \"$KOAD_SESSION_ID\" ]; then koad session log --event turn-complete --session $KOAD_SESSION_ID 2>/dev/null; fi"
          }
        ]
      }
    ]
  },
  "mcpServers": {}
}
```

### Key Settings Reference

| **Setting** | **Purpose** | **KoadOS Rationale** |
| --- | --- | --- |
| `permissions` | Default approval mode | Keep at `default` — manual approval gates are canon-mandated |
| `model` | Default Claude model alias | `sonnet` for general work; override with `opus` for complex architecture |
| `allowedTools` | Tools auto-approved (skip confirmation) | Allowlist safe `koad` and `git` read-only commands + read tools |
| `deniedTools` | Tools explicitly blocked | Block destructive operations that bypass Dood authority |
| `hooks` | Lifecycle event handlers | Sanctuary Rule enforcement, session logging, PSRP triggers |
| `mcpServers` | MCP server connections | Wire KoadOS Bridge Skills as they become available |

### MCP Server Configuration

MCP servers are configured under `mcpServers`. Claude Code supports STDIO and SSE:

```json
{
  "mcpServers": {
    "koadBridge": {
      "command": "python3",
      "args": ["~/.koad-os/bridge/mcp_server.py"],
      "env": {
        "KOAD_SESSION_ID": "$KOAD_SESSION_ID"
      }
    }
  }
}
```

### Hooks System — Deep Reference

Hooks fire at lifecycle events. Each hook entry has a `matcher` (regex against tool name) and an array of hook commands.

**Available events:** `PreToolUse`, `PostToolUse`, `PreEdit`, `PostEdit`, `Notification`, `Stop`, `SubagentStop`

**Exit codes:**

- `0` — Pass, continue normally
- `2` — **Block the operation** (critical for enforcement)
- Other non-zero — Hook failed, operation continues

**Environment variables available in hooks:**

- `$CLAUDE_CODE_TOOL_INPUT` — JSON input to the tool being executed
- `$CLAUDE_CODE_TOOL_OUTPUT` — JSON output (PostToolUse only)
- `$CLAUDE_CONVERSATION_ID` — Current conversation ID
- Standard env vars from your shell (including `$KOAD_SESSION_ID`)

---

## Project-Level Configuration — Path-Aware Context

Each project gets its own context that supplements the global platform layer.

### Recommended Project Structure

```
~/data/koad-os/
├── CLAUDE.md                  ← KoadOS kernel dev context
├── .claude/
│   ├── settings.json          ← Kernel-specific tool allowlists
│   └── agents/
│       ├── explorer.md        ← Read-only codebase explorer
│       └── reviewer.md        ← KSRP review agent

~/data/skylinks/
├── CLAUDE.md                  ← Skylinks dev context: tech stack, SCE/SLE topology
├── .claude/
│   ├── settings.json          ← Skylinks-specific settings
│   └── agents/
│       └── worker.md          ← Implementation agent
```

These files contain **project knowledge**, not agent identity. Any KAI booted in `~/data/skylinks/` gets the Skylinks context regardless of whether they are Tyr, Sky, or Vigil.

---

## KoadOS Subagent Definitions

Claude Code subagents are defined as Markdown files with YAML frontmatter.

### User-Level Subagents (`~/.claude/agents/`)

**Explorer agent (`explorer.md`):**

```markdown
---
name: koad-explorer
description: Read-only codebase exploration for KoadOS projects. Maps code paths before changes.
tools:
  - Read
  - Grep
  - Glob
  - Bash
permissions: auto-accept all
model: haiku
---

You are a KoadOS explorer agent. Read-only reconnaissance only.
- Map code paths, trace execution flows, cite files and line numbers.
- Never propose edits or modifications.
- Report findings structured as: File, Purpose, Dependencies, Risk Areas.
- Respect the Sanctuary Rule — do not access ~/.koad-os/ or koad.json.
```

**Reviewer agent (`reviewer.md`):**

```markdown
---
name: koad-reviewer
description: KSRP review agent for KoadOS. Runs structured multi-pass code review.
tools:
  - Read
  - Grep
  - Glob
permissions: auto-accept all
model: sonnet
---

You are a KoadOS KSRP review agent. Execute the 7-pass review loop:
1. Lint — Check for style violations and formatting issues.
2. Verify — Confirm the code does what the issue/spec requires.
3. Inspect — Look for logic errors, edge cases, and null handling.
4. Architect — Evaluate structural decisions and separation of concerns.
5. Harden — Check error handling, input validation, and failure modes.
6. Optimize — Identify performance issues and unnecessary complexity.
7. Test Audit — Verify test coverage and test quality.

Report format: Pass number, finding severity (info/warn/error), file:line, description.
Do not make edits. Report only.
```

**Worker agent (`worker.md`):**

```markdown
---
name: koad-worker
description: Implementation agent for KoadOS project tasks. Respects Sanctuary Rule.
tools:
  - Read
  - Write
  - Edit
  - MultiEdit
  - Bash
  - Grep
  - Glob
permissions: auto-edit
model: sonnet
---

You are a KoadOS worker agent. Own the implementation once the issue is understood.
- Make the smallest defensible change.
- Respect the Sanctuary Rule: Never edit ~/.koad-os/ or koad.json.
- Commits MUST reference the GitHub issue number.
- If you discover side-tasks, note them for the parent — do not scope creep.
```

---

## Where Agent Identity Lives (NOT in Claude Code Config)

Agent identity is managed entirely by the Koad Spine:

| **Store** | **Contents** | **Lifecycle** |
| --- | --- | --- |
| `koad.json` | KAI registry — bio, rank, preferences, authority tier | Persistent on disk. Source of truth for identity. |
| Redis (Hot Stream) | Active agent's cached identity + hydrated working memory | Populated by Sentinel at boot. Purged by ASM on session end. |
| SQLite Memory Bank | Per-agent personal memory partitions — PSRP reflections, facts, learnings | Long-term persistent. Agents evolve independently. |

### The Boot Sequence

When `koad boot --agent <Name>` runs:

1. The Spine loads the named agent's identity from `koad.json`
2. Identity is cached in Redis under the active `KOAD_SESSION_ID`
3. Sentinel hydrates the agent's personal memory from SQLite into the Redis Hot Stream
4. The resulting context is injected into the session via the Spine's delivery mechanism
5. The Claude Code `CLAUDE.md` remains untouched — the Body doesn't change, only the Ghost does

### Recommended Launch Pattern

Use a shell wrapper or alias for clean boot integration:

```bash
# ~/.bashrc or ~/.bash_aliases
koad-claude() {
  local agent="${1:?Usage: koad-claude <AgentName>}"
  export KOAD_SESSION_ID=$(uuidgen)
  koad boot --agent "$agent"
  claude
}
```

This ensures `KOAD_SESSION_ID` is set before Claude Code launches, and the boot sequence completes before the agent loop begins.

### Dynamic Multi-Body Architecture

```bash
# Terminal 1: Claude Code + Tyr (architecture work)
koad-claude Tyr

# Terminal 2: Gemini CLI + Sky (project management)
koad-gemini Sky

# Terminal 3: Codex CLI + Vigil (security audit)
koad-codex Vigil
```

All running on the same machine, sharing the same Spine, with zero context leakage between sessions.

---

## Migration & Setup Checklist

- [ ]  Create `~/.claude/CLAUDE.md` with the neutral Body template above
- [ ]  Create `~/.claude/settings.json` with the recommended baseline
- [ ]  Create `~/.claude/agents/explorer.md`, `reviewer.md`, `worker.md`
- [ ]  Create project-level `CLAUDE.md` for `koad-os` and `skylinks` repos
- [ ]  Create project-level `.claude/settings.json` for each project
- [ ]  Add `koad-claude` alias to `~/.bashrc`
- [ ]  Test boot sequence: `koad-claude Tyr` → verify identity injection
- [ ]  Test Sanctuary Rule: attempt write to `koad.json` → verify hook blocks it
- [ ]  Test subagents: invoke `koad-explorer` and `koad-reviewer` within a session
- [ ]  Test cognitive isolation: two terminals, two agents, zero leakage
- [ ]  Verify `koad auth` works from within a Claude Code session

---

## Claude Code vs Gemini CLI vs Codex CLI — Quick Reference

| **Aspect** | **Claude Code** | **Gemini CLI** | **Codex CLI** |
| --- | --- | --- | --- |
| Instruction file | `CLAUDE.md` | `GEMINI.md` | `AGENTS.md` |
| Config format | JSON (`settings.json`) | JSON (`settings.json`) | TOML (`config.toml`) |
| Config location | `~/.claude/` | `~/.gemini/` | `~/.codex/` |
| Project config | `.claude/`  • repo root `CLAUDE.md` | `.gemini/` | `.codex/`  • repo root `AGENTS.md` |
| Local override | `CLAUDE.local.md` / `settings.local.json` | N/A | `AGENTS.override.md` |
| Hook richness | **Best** (Pre/Post Tool, Edit, Stop, Subagent) | Limited (Session, Tool, Model, Agent, Compress) | N/A |
| Subagents | **Native** (MD files with YAML frontmatter) | Manual (separate terminals) | Native (TOML roles) |
| Sandbox | None (permission-based) | Docker/Podman/custom | Native sandbox modes |
| Boot integration | Shell wrapper + [CLAUDE.md](http://CLAUDE.md) directive + hooks | [GEMINI.md](http://GEMINI.md) directive + SessionStart hook | [AGENTS.md](http://AGENTS.md) directive (already done) |

---

## Summary

Claude Code is a clean, neutral Body. `koad boot` gives it a soul.

- **Global config** = platform canon + tools + hooks. No identity.
- **Project config** = project knowledge + constraints. No identity.
- **Agent identity** = `koad.json` → Redis → session injection at boot.
- **Personal memory** = SQLite partitions, isolated per-KAI.
- **Multiple terminals** = multiple Ghosts, zero leakage.
- **Hooks** = hard enforcement of Sanctuary Rule (upgrade over Gemini/Codex).
- **Subagents** = native delegation with fine-grained tool/permission control.

The host never changes. Only the Ghost does.