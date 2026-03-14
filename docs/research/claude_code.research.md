<aside>
🎯

**Purpose:** Deep dive research into Claude Code's architecture, configuration system, and capabilities — written for Tyr to review KoadOS and ensure the Spine, boot sequence, and cognitive isolation are ready to support Claude Code as a third Body alongside Gemini CLI and Codex CLI.

</aside>

---

## 1. Claude Code — Architecture Overview

Claude Code is Anthropic's **agentic coding tool** that reads your codebase, edits files, runs commands, and integrates with development tools. Available as a **CLI (terminal)**, **IDE extension** (VS Code, JetBrains), **desktop app**, **browser app**, and via **GitHub Actions / GitLab CI/CD**.

### Core Architecture: Client-Server Model

Claude Code operates on a **client-server split**:

- **Client (local):** A TypeScript/React/Ink TUI running on **Bun**. Handles file reading, command execution, diff rendering, and permission enforcement locally.
- **Server (remote):** Anthropic's Claude models (Sonnet, Opus, Haiku). The model receives the full context (system prompt + `CLAUDE.md` files + conversation) and returns tool calls.
- **Agent loop:** Claude receives a prompt → reads files → proposes edits → runs shell commands → iterates based on results → presents output for approval.

### Key Architectural Traits

- **No sandbox by default** — Unlike Codex, Claude Code runs commands directly in your shell environment. It relies on a **permission system** rather than sandboxing.
- **Permission tiers:** `Ask` (confirm everything), `Auto-accept edits` (auto-approve file writes, confirm commands), `Auto-accept all` (full autonomy).
- **Tool system:** Built-in tools include `Read`, `Write`, `Edit`, `MultiEdit`, `Bash`, `Grep`, `Glob`, `Task` (subagent), `WebSearch`, and MCP tools.
- **Context window:** Depends on model — Claude Sonnet 4 has **200K tokens**, Claude Opus 4.5 has **200K tokens**. Context compaction is automatic when the window fills.
- **Session state:** Conversations are resumable via `claude --resume` or `claude --continue`.
- **Built with itself:** 90% of Claude Code's own codebase is written by Claude Code.

---

## 2. Configuration System — The Layers

Claude Code has a layered configuration system that maps cleanly to KoadOS's Body/Ghost separation.

### Layer 1: `CLAUDE.md` (Hierarchical Instructions)

This is **the Claude Code equivalent of `GEMINI.md` and `AGENTS.md`** — the primary instruction injection mechanism.

**Discovery order:**

1. **Global:** `~/.claude/CLAUDE.md` — loaded first, always. This is the KoadOS platform layer.
2. **Upward traversal:** From CWD up to the project root (identified by `.git`). Each `CLAUDE.md` found is loaded, ancestor-first.
3. **Project root:** `CLAUDE.md` in the repo root.
4. **Nested directories:** `CLAUDE.md` files in subdirectories below CWD.

**Key behavior:**

- All found files are **concatenated sequentially** and injected into the system prompt.
- Later (more specific) files supplement or can override earlier (more general) files.
- `CLAUDE.md` files are interpreted **by the model**, not by the CLI — they are raw text injected into context.
- Users can add content via `/memory` command or `claude memory add "text"`.

**Local override:** `CLAUDE.local.md` files work identically but are intended for personal/machine-specific instructions (add to `.gitignore`).

### Layer 2: Settings (JSON, Scoped)

Claude Code settings are JSON and follow a **scope hierarchy:**

1. **Enterprise policy** — `/etc/claude-code/settings.json` (admin-enforced)
2. **User settings** — `~/.claude/settings.json` ← KoadOS global config
3. **Project settings** — `.claude/settings.json` (committed, shared with team)
4. **Project local settings** — `.claude/settings.local.json` (personal, gitignored)

**Key settings:**

| **Setting** | **Purpose** | **KoadOS Relevance** |
| --- | --- | --- |
| `permissions` | Default permission mode (`default`, `auto-edit`, `auto-full`) | Keep at `default` — aligns with canon approval gates |
| `allowedTools` | Tools auto-approved without confirmation | Allowlist safe `koad` and `git` read commands |
| `deniedTools` | Tools explicitly blocked | Block destructive ops that bypass Dood authority |
| `model` | Default Claude model | Set globally; override per-project if needed |
| `mcpServers` | MCP server connections | Wire KoadOS Bridge Skills here |
| `hooks` | Lifecycle event handlers | Critical for boot enforcement, PSRP triggers |

### Layer 3: Hooks (Lifecycle Automation)

Hooks are **the most powerful KoadOS integration point** in Claude Code. They run shell commands, HTTP calls, or LLM prompts at specific lifecycle events.

**Available hook events:**

| **Event** | **When it fires** | **KoadOS Use Case** |
| --- | --- | --- |
| `PreToolUse` | Before any tool execution | Enforce Sanctuary Rule — block writes to `~/.koad-os/` or `config/` |
| `PostToolUse` | After tool execution completes | Audit logging, file change tracking |
| `PreEdit` | Before file edits are applied | Auto-format, protected file blocking |
| `PostEdit` | After file edits complete | Lint check, type check on modified files |
| `Notification` | When Claude needs human input | Send notification when approval gate reached |
| `Stop` | When Claude finishes a turn | Trigger PSRP saveup, session logging |
| `SubagentStop` | When a subagent finishes | Aggregate subagent results for parent |

**Hook configuration (in settings.json):**

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "echo $CLAUDE_CODE_TOOL_INPUT | jq -r '.command' | grep -qE 'rm -rf|config/' && echo 'BLOCK: Sanctuary Rule violation' && exit 2"
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
            "command": "koad session log --event stop --session $KOAD_SESSION_ID"
          }
        ]
      }
    ]
  }
}
```

**Exit code behavior:**

- `0` — Hook passes, continue normally
- `2` — **Block the operation** (critical for Sanctuary Rule enforcement)
- Non-zero (other) — Hook failed, but operation continues

### Layer 4: Subagents

Claude Code supports **custom subagents** — specialized child agents with their own prompts, tool restrictions, permission modes, and even hooks.

**Definition:** Markdown files with YAML frontmatter, stored in:

- `~/.claude/agents/` — user-level (available everywhere)
- `.claude/agents/` — project-level

**Example subagent (`.claude/agents/explorer.md`):**

```
---
name: explorer
description: Read-only codebase exploration for KoadOS projects
tools:
  - Read
  - Grep
  - Glob
permissions: auto-accept all
model: sonnet
---

You are a KoadOS explorer agent. Read-only. Map code paths, trace execution, cite files.
Never propose edits. Report findings to the parent agent.
```

**Subagent capabilities:**

- Custom tool restrictions (e.g., read-only agents)
- Custom permission modes per subagent
- Custom model selection per subagent
- Custom hooks per subagent
- Can be invoked inline via the `Task` tool or explicitly by name

### Layer 5: MCP Servers

Claude Code connects to **STDIO and SSE MCP servers** via settings.json:

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

MCP tools appear alongside built-in tools and can be allowlisted/denied like any other tool.

---

## 3. Parity Map: Gemini ↔ Codex ↔ Claude Code for KoadOS

| **KoadOS Capability** | **Gemini Implementation** | **Codex Implementation** | **Claude Code Implementation** |
| --- | --- | --- | --- |
| Global agent instructions | `~/.gemini/GEMINI.md` | `~/.codex/AGENTS.md` | `~/.claude/CLAUDE.md` |
| Project instructions | `.gemini/GEMINI.md` | `AGENTS.md` (repo root + nested) | `CLAUDE.md` (repo root + nested) |
| User config | `~/.gemini/settings.json` | `~/.codex/config.toml` | `~/.claude/settings.json` |
| Project config | `.gemini/settings.json` | `.codex/config.toml` | `.claude/settings.json` |
| MCP servers | `settings.json` → `mcpServers` | `config.toml` → `[mcp.servers.*]` | `settings.json` → `mcpServers` |
| Lifecycle hooks | `hooks` in settings.json (limited) | N/A (manual) | **Rich hook system** (PreToolUse, PostEdit, Stop, etc.) |
| Subagent delegation | Manual (separate terminals) | Native `[agents.*]` roles | Native subagents (`.claude/agents/*.md`) |
| Tool permission control | `tools.allowed` allowlist | `approval_policy` tiers | `allowedTools` / `deniedTools`  • per-subagent |
| Context file naming | Configurable (`GEMINI.md`, `AGENTS.md`) | Configurable (`AGENTS.md`  • fallbacks) | `CLAUDE.md` (fixed name) |
| Local overrides | N/A | `AGENTS.override.md` | `CLAUDE.local.md`  • `settings.local.json` |
| Named profiles | N/A | `[profiles.*]` in config.toml | N/A (use `CLAUDE.local.md` or flags) |
| Sandbox | Docker / Podman / custom | Native sandbox modes | **No sandbox** — permission-based only |

---

## 4. Claude Code Strengths & Weaknesses Profile

### Where Claude Code Excels

- **Code quality** — Consistently rated highest for clean abstractions, elegant solutions, and production-grade output (9.1/10 in community benchmarks vs Codex ~7.8/10).
- **Complex greenfield architecture** — Preferred for building complex systems from scratch where deep planning and architectural coherence matter.
- **Frontend/UI precision** — Community consensus: Claude outdoes GPT-5 in frontend implementation, design-to-code, and UI polish.
- **Long multi-turn session coherence** — Handles extended sessions more gracefully than Codex. Less context rot over long conversations.
- **Understanding large existing codebases** — Maintains better contextual coherence across very large codebases (350k+ LOC).
- **Instruction adherence ([CLAUDE.md](http://CLAUDE.md))** — The model deeply respects [CLAUDE.md](http://CLAUDE.md) instructions since they're injected directly into the system prompt.
- **Hook system** — The richest lifecycle hook system of any CLI agent. Critical for KoadOS enforcement patterns.
- **Subagent flexibility** — Custom subagents with per-agent tool restrictions, permissions, models, and hooks.

### Where Claude Code Struggles

- **Speed** — Slower than Codex for raw throughput. Claude thinks more carefully but takes longer.
- **No built-in sandbox** — Relies entirely on permissions and hooks for safety. No Docker/container isolation for command execution.
- **Context window size** — 200K tokens vs Gemini's 1M and Codex's 400K. Context compaction kicks in sooner on large codebases.
- **Cost** — No free tier. Requires Claude Pro ($20/mo), Team ($30/mo), or Max ($100-200/mo) subscription. API usage is metered.
- **No native CI/CD mode** — GitHub Actions integration exists but is less mature than Codex's `codex exec` and SDK.
- **No named profiles** — Unlike Codex's `[profiles.*]`, Claude Code doesn't have a native profile switching mechanism. Must use flags or local config files.

---

## 5. Head-to-Head: Claude Code vs Codex vs Gemini CLI

| **Dimension** | **Claude Code** | **Codex CLI** | **Gemini CLI** |
| --- | --- | --- | --- |
| Speed | Moderate | **Fastest** | Fast |
| Code quality | **Best** (9.1/10) | Good (7.8/10) | Good (7.8/10) |
| Error handling | **Best** (8.8/10) | Good (7.5/10) | Good (7.5/10) |
| Context window | 200K tokens | 400K tokens | **1M tokens** |
| Greenfield architecture | **Strong** | Moderate | Moderate |
| Frontend/UI | **Strong** | Moderate | Moderate |
| Backend/debugging | Strong | **Strong** | Moderate |
| Lifecycle hooks | **Richest** | N/A | Limited |
| Subagent system | **Flexible** (MD-defined) | Native roles (TOML) | Manual |
| Sandbox | None (permission-based) | **Native** | Docker/Podman |
| MCP support | STDIO + SSE | STDIO only | STDIO + SSE + HTTP |
| Automation/CI | Good | **Best** | Good |
| Free tier | None | Limited trial | **60 req/min free** |
| Cost (top tier) | $200/mo (Max) | $200/mo (Pro) | Pay-per-use / free |

---

## 6. KoadOS Readiness Assessment for Tyr

<aside>
⚠️

**Tyr: Review each section below against current Spine implementation. Mark each item as ✅ Ready, ⚡ Needs Work, or 🔴 Blocker.**

</aside>

### 6A. Boot Sequence Compatibility

**Current:** `koad boot --agent <Name>` loads identity from `config/` → caches in Redis → Sentinel hydrates memory from SQLite → context injected into session.

**Claude Code delta:**

- Claude Code has **no native boot hook** like Gemini's `SessionStart`. However, the `CLAUDE.md` Boot Directive instruction + a `Stop` hook on first turn can enforce boot.
- The `PreToolUse` hook can **block all tool execution** until `koad boot` has been run (exit code 2 blocks the operation).
- **Alternative:** Use a shell wrapper script (`koad claude` or alias) that runs `koad boot --agent <Name>` then launches `claude` with the session ID as an env var.

**Readiness question for Tyr:** Can the Spine detect that the Body is Claude Code (vs Gemini/Codex) and adjust context delivery accordingly? Or is the delivery mechanism already Body-agnostic?

### 6B. One Body, One Ghost Enforcement

**Current:** Enforced via `KOAD_SESSION_ID` tethering in Redis. ASM monitors heartbeats and purges on termination.

**Claude Code delta:**

- Claude Code sessions don't have a native session ID mechanism. The `KOAD_SESSION_ID` must be **injected as an environment variable** before launch.
- Claude Code's `--resume` and `--continue` flags create new API conversations but the terminal session persists. The Spine needs to handle this — same Body session, potentially new conversation thread.
- Subagents run as **child processes within the same terminal**. They share the parent's environment, so `KOAD_SESSION_ID` propagates naturally. But the ASM needs to know these are sub-sessions, not separate Ghosts.

**Readiness question for Tyr:** Does ASM currently distinguish between a primary session and child processes sharing the same `KOAD_SESSION_ID`? If Claude Code subagents register heartbeats independently, will the Autonomic Pruner treat them as orphans?

### 6C. Cognitive Isolation

**Current:** Per-agent SQLite partitions + Redis session isolation + procedural Canon.

**Claude Code delta:**

- `CLAUDE.md` files are **read-only from the model's perspective** — the model cannot modify them. This is a natural Sanctuary Rule enforcement.
- `CLAUDE.local.md` provides per-machine overrides without polluting the shared project config. Good for agent-specific local context.
- Claude Code's **context compaction** (automatic when approaching window limit) may discard earlier [CLAUDE.md](http://CLAUDE.md) content. The Boot Directive and critical Canon rules should be placed **at the top** of the global `CLAUDE.md` to maximize retention during compaction.

**Readiness question for Tyr:** When context compaction fires, does the Spine need to re-inject critical identity context? Or is the `CLAUDE.md` always preserved as part of the system prompt (never compacted)?

### 6D. Sanctuary Rule Enforcement

**Current:** Instructional only (written in `GEMINI.md` / `AGENTS.md`).

**Claude Code upgrade opportunity:**

- Claude Code's `PreToolUse` hook can **programmatically enforce** the Sanctuary Rule by blocking any Bash command or file write targeting protected paths.
- `deniedTools` in settings can block specific tools entirely.
- This moves Sanctuary Rule from "honor system" to **hard enforcement** — a significant security upgrade over Gemini and Codex.

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [{
          "type": "command",
          "command": "echo $CLAUDE_CODE_TOOL_INPUT | jq -r '.file_path // .filePath // empty' | grep -qE '^\.koad-os/|koad\.json|koad\.db' && echo 'BLOCKED: Sanctuary Rule — protected KoadOS path' && exit 2 || exit 0"
        }]
      },
      {
        "matcher": "Bash",
        "hooks": [{
          "type": "command",
          "command": "echo $CLAUDE_CODE_TOOL_INPUT | jq -r '.command' | grep -qE 'rm.*koad|mv.*koad\.json|sqlite3.*koad\.db' && echo 'BLOCKED: Sanctuary Rule — destructive koad operation' && exit 2 || exit 0"
        }]
      }
    ]
  }
}
```

### 6E. KSRP / PSRP Integration

**Current:** Canon-mandated in instruction files. Agents self-execute.

**Claude Code delta:**

- The `Stop` hook fires when Claude finishes a turn — can trigger `koad session log` or a PSRP saveup prompt.
- Custom subagents can be defined specifically for review passes (e.g., a `reviewer` subagent with read-only tools that runs KSRP passes).
- Claude Code's **`/memory`** command can be used to inject PSRP reflections into the session's [CLAUDE.md](http://CLAUDE.md) in real-time.

### 6F. GitHub Operations

**Current:** `koad auth` for directory-aware PAT selection. Commits reference issues.

**Claude Code delta:**

- Claude Code has **native Git awareness** — it understands repo structure, can run git commands, and proposes commits with messages.
- `koad auth` integration works via env vars (`GITHUB_TOKEN` etc.) that Claude Code picks up from the shell environment.
- The `PreToolUse` hook on Bash can enforce that `git push` and `git merge` commands are blocked unless `DOOD_APPROVED=true` is set in the environment.

---

## 7. Updated Agent Routing Recommendation

<aside>
🎯

**Route to Claude Code when:** Complex greenfield architecture, frontend/UI precision, production-grade code quality, long multi-turn planning sessions, tasks requiring deep contextual coherence, security-sensitive work requiring hook-enforced guardrails, subagent delegation with fine-grained tool control

</aside>

<aside>
🎯

**Route to Codex when:** Speed matters, backend/debugging work, refactoring at scale, parallel task fan-out, CI/CD automation, batch operations, codebase exploration

</aside>

<aside>
🎯

**Route to Gemini CLI when:** Very large context tasks (1M tokens), multimodal reasoning (images/diagrams → code), Google Cloud integration, cost-sensitive or free-tier work, tasks requiring long-context reasoning

</aside>

---

## 8. Open Questions for Tyr

- [ ]  **Boot enforcement mechanism:** Shell wrapper (`koad claude`) vs `PreToolUse` hook that blocks until boot completes vs `CLAUDE.md` instruction only?
- [ ]  **Subagent session handling:** Should Claude Code subagents register as child sessions in ASM, or are they invisible to the Spine?
- [ ]  **Context compaction resilience:** Does the Spine need a re-injection mechanism for compacted sessions, or is `CLAUDE.md` system prompt content preserved?
- [ ]  **Sanctuary Rule — hard vs soft:** Should we enable hook-based hard enforcement (exit code 2 blocking) for Claude Code, making it stricter than Gemini/Codex?
- [ ]  **Unified `koad init --agent claude`:** Should the `koad` CLI scaffold Claude Code config files (`~/.claude/CLAUDE.md`, `~/.claude/settings.json`, `.claude/agents/`) from canonical KoadOS templates?
- [ ]  **`CLAUDE.md` vs `AGENTS.md` naming:** Claude Code's filename is fixed (`CLAUDE.md`). Codex uses `AGENTS.md`. The `koad` CLI's project context files need a strategy for maintaining both. Should `CLAUDE.md` import a shared canonical file?

## 9. Noti Reassessment — Gap Analysis & Recommendations

March 10, 2026

<aside>
🔍

**Context:** Cross-referenced this report against the full KoadOS Global Canon, the Gemini CLI Configuration Guide, and the Codex CLI Parity Report. The findings below address implementation gaps, unresolved open questions, and opportunities the original report undersells.

</aside>

### 9A. Canon Compliance Gaps

The report references KSRP and PSRP but doesn't map Claude Code's lifecycle to the **full 9-step Development Canon**.

**Approval Gates (Steps 4 & 9)** — The report recommends keeping permissions at `default` (correct), but proposes no hook to *enforce* the Canon's strict halt gates. Claude Code's `PreToolUse` hook can make this mechanical:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit|Bash",
        "hooks": [{
          "type": "command",
          "command": "[ \"$KOAD_PHASE\" = 'implement' ] || (echo 'BLOCKED: Canon gate — not in implement phase' && exit 2)"
        }]
      }
    ]
  }
}
```

This moves approval gates from honor-system to hard enforcement — a pattern neither Gemini nor Codex can match.

**Task-Weight KSRP Tiers** — The Canon mandates weight assignment at Step 1 (`trivial`/`standard`/`complex`) which governs iteration caps and required passes. Neither the `CLAUDE.md` template nor the proposed `reviewer` subagent addresses weight-awareness. The `reviewer` subagent definition should accept a `--weight` parameter or read `KOAD_TASK_WEIGHT` from the environment.

**Gate Timeout Escalation** — The Canon specifies: 24h reminder → 48h suspend with partial Saveup. The `Notification` hook event is a natural fit but isn't mentioned in the report. Proposed addition:

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [{
          "type": "command",
          "command": "koad gate check --session $KOAD_SESSION_ID --escalate"
        }]
      }
    ]
  }
}
```

### 9B. `Stop` Hook — PSRP Is Underspecified

The report says the `Stop` hook can trigger `koad session log`, but the Canon requires a **full Three-Pass Saveup** (fact/learn/ponder) with a specific format, and **partial Saveups on abnormal exits**.

**Problem:** Claude Code fires `Stop` on every turn completion, not just session end. Without a guard, you're logging noise on every single turn.

**Recommended implementation:**

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "",
        "hooks": [{
          "type": "command",
          "command": "koad session log --event turn-end --session $KOAD_SESSION_ID --check-saveup"
        }]
      }
    ]
  }
}
```

The `--check-saveup` flag should make the `koad` CLI distinguish between a normal turn-end (lightweight log) and a session-end (trigger full PSRP). The Spine can determine this based on session duration, turn count, or an explicit `/done` signal from the user.

### 9C. Boot Enforcement — Definitive Recommendation

The report presents three options without recommending one. Based on how Gemini and Codex handle it:

<aside>
✅

**Recommended: Shell wrapper (`koad claude`) as primary + `PreToolUse` blocker as backup.**

</aside>

**Why this wins:**

- **Body-agnostic pattern** — Same approach as future `koad gemini` / `koad codex` wrappers.
- **Guarantees boot runs before the model sees any context** — No race condition.
- **`PreToolUse` backup catches edge cases** where someone launches bare `claude` without the wrapper.

**Implementation:**

```bash
# ~/.koad-os/bin/koad-claude (or subcommand: koad claude)
#!/bin/bash
set -euo pipefail
AGENT_NAME="${1:?Usage: koad claude <AgentName>}"
eval "$(koad boot --agent "$AGENT_NAME" --export-env)"
export KOAD_BOOTED=true
exec claude "${@:2}"
```

**Backup hook (in settings.json):**

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit|Bash",
        "hooks": [{
          "type": "command",
          "command": "[ \"$KOAD_BOOTED\" = 'true' ] || (echo 'BLOCKED: KoadOS boot required — run koad claude <AgentName>' && exit 2)"
        }]
      }
    ]
  }
}
```

### 9D. Context Budget — 200K Is Tight

Claude Code has the **smallest context window** of the three Bodies (200K vs Codex 400K vs Gemini 1M). The report flags this but doesn't solve it.

**Key constraint:** Claude Code has **no `@import` equivalent** like Gemini's `GEMINI.md`. Everything in `CLAUDE.md` is raw text concatenated into the system prompt. You cannot modularize the way the Gemini guide recommends.

**Recommended strategy:**

1. **`~/.claude/CLAUDE.md` must be aggressively lean** — Boot Directive, One Body/One Ghost, Sanctuary Rule, and the highest-priority Canon summary only. Target <2K tokens.
2. **Move detailed KSRP/PSRP procedures into the `reviewer` subagent instructions** — subagent prompts are only loaded when invoked, not on every turn.
3. **Move Development Canon details into a `koad canon` CLI command** that the agent invokes when entering each Canon step — just-in-time context injection rather than persistent context burn.
4. **Place Boot Directive and Sanctuary Rule at the absolute top** of global `CLAUDE.md` to maximize retention during compaction.
5. **Use `CLAUDE.local.md` for machine-specific context** (e.g., project paths, env quirks) that would waste shared context budget.

This is a significant architectural difference from Gemini where you can afford to inline the full Canon.

### 9E. `koad init --agent claude` Scaffolding Spec

Both the Gemini and Codex reports converge on `koad init` as the canonical scaffolding command. This report should spec it concretely:

**`koad init --agent claude` should generate:**

| **File** | **Purpose** | **Source** |
| --- | --- | --- |
| `~/.claude/CLAUDE.md` | Lean platform context (boot, sanctuary, canon summary) | Generated from `~/.koad-os/templates/BODY_CONTEXT.md` |
| `~/.claude/settings.json` | KoadOS defaults + hooks + allowedTools + deniedTools | Generated from `~/.koad-os/templates/claude-settings.json` |
| `~/.claude/agents/explorer.md` | Read-only codebase explorer subagent | Generated from canonical role template |
| `~/.claude/agents/worker.md` | Implementation subagent respecting Sanctuary Rule | Generated from canonical role template |
| `~/.claude/agents/reviewer.md` | KSRP reviewer subagent (read-only, weight-aware) | Generated from canonical role template |
| `<project>/.claude/settings.json` | Project-specific overrides | Generated per-project from `koad project` registry |

All generated from a **single canonical source** (`~/.koad-os/templates/`) so updates propagate across Bodies via `koad init --agent <gemini|codex|claude> --refresh`.

### 9F. `CLAUDE.md` vs `AGENTS.md` — Resolution

Claude Code's filename is fixed (`CLAUDE.md`). Codex uses `AGENTS.md`. Gemini uses `GEMINI.md` (but can be configured to also read `AGENTS.md`).

<aside>
✅

**Resolution: Maintain separate files per-Body, all generated from a single canonical template by `koad init`.**

</aside>

The content is nearly identical — only the filename and minor syntax differences change. The `koad init` command renders `~/.koad-os/templates/BODY_CONTEXT.md` into the Body-specific filename. Project-level context files follow the same pattern.

This avoids symlink hacks, avoids confusing the CLIs with foreign filenames, and keeps the canonical source authoritative.

### 9G. What the Report Undersells

**Hard enforcement is a paradigm shift.** Right now, Sanctuary Rule, approval gates, and KSRP are all honor-system across Gemini and Codex. Claude Code's hook system with exit code 2 blocking makes these *mechanical*. If Tyr implements the hooks from Section 6D + the additions above, **Claude Code becomes the most trustworthy Body in the fleet** — not just the highest code-quality one. This should be evaluated as a pattern to backport to Gemini (which has hooks, though less rich).

**Subagent-scoped tool restrictions are a natural KSRP implementation.** A `reviewer` subagent with read-only tools (`Read`, `Grep`, `Glob`) running the 7-pass KSRP loop physically cannot introduce changes during review. That's structural safety that neither Gemini nor Codex can match natively.

---

- Prompt for Tyr — Claude Code Readiness Review
    
    **Directive: KoadOS Readiness Review for Claude Code Integration**
    
    **Context:** Noti has produced a deep research report on Claude Code's architecture and a readiness assessment for KoadOS integration. The report is available as a Notion page titled **"Claude Code Deep Research & KoadOS Readiness Report"** under **Koados**. It covers Claude Code's client-server architecture, [CLAUDE.md](http://CLAUDE.md) hierarchy, settings system, hook system, subagents, MCP support, and a detailed parity comparison with Gemini CLI and Codex CLI.
    
    **Your Task:** Perform a multi-pass readiness review of KoadOS to ensure the Spine, boot sequence, cognitive isolation, and enforcement mechanisms are ready to support Claude Code as a third Body.
    
    ---
    
    **Pass 1 — Ingest & Cross-Reference**
    
    - Read the Notion research report in full.
    - Cross-reference against the official Claude Code docs at https://code.claude.com/docs/en/overview
    - Cross-reference against the existing Gemini CLI Configuration Guide and Codex CLI Parity Report.
    - Identify any gaps, inaccuracies, or missed opportunities. Note them but do not act yet.
    
    **Pass 2 — Spine Compatibility Audit**
    
    - Verify `koad boot --agent <Name>` can inject identity into a Claude Code session via environment variables.
    - Verify `KOAD_SESSION_ID` propagation to Claude Code subagents.
    - Verify ASM can monitor Claude Code sessions (heartbeat, lifecycle, termination).
    - Verify Sentinel hydration works when the Body is Claude Code.
    - Verify the Autonomic Pruner handles Claude Code's `--resume` / `--continue` correctly.
    - Test cognitive isolation: boot two different agents in two terminals running Claude Code. Confirm zero context leakage.
    
    **Pass 3 — Security & Enforcement**
    
    - Implement and test `PreToolUse` hooks for Sanctuary Rule enforcement.
    - Verify `deniedTools` properly blocks destructive operations.
    - Test that exit code 2 from hooks actually blocks tool execution.
    - Evaluate whether hook-based enforcement should be the default for all Bodies or Claude Code only.
    
    **Pass 4 — Configuration Scaffolding**
    
    - Create or update `~/.claude/CLAUDE.md` with the agent-agnostic KoadOS platform context.
    - Create or update `~/.claude/settings.json` with KoadOS-optimized defaults.
    - Create project-level `.claude/` directories for `koad-os` and `skylinks`.
    - Create KoadOS subagent definitions in `~/.claude/agents/` (explorer, worker, reviewer).
    - Test the full boot → work → saveup lifecycle.
    
    **Pass 5 — Reflection & Report**
    
    - Execute PSRP on this review.
    - Produce a Results Report with all findings, changes, and open items.
    - Create GitHub Issues for any follow-up work.
    
    **Constraints:**
    
    - Do not merge or push without Dood's explicit approval.
    - All changes must align with the Agent-Agnostic Host Baseline — no identity in Body config.
    - Document any Spine changes needed as separate GitHub Issues before implementing.

[Tyr Action Checklist — Claude Code Integration](https://www.notion.so/Tyr-Action-Checklist-Claude-Code-Integration-912a3d34b1244e56afe6f24c123e7d5d?pvs=21)