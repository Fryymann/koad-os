# KoadOS Claude Code Bootstrap

## Driver Isolation
- Claude Code runs in its own driver namespace. Do not reuse Gemini, Codex, or other driver bootstrap prompts, folders, or settings during Claude Code sessions to avoid cross-agent leakage.
- Store any Claude-specific artifacts (logs, saveups, skill references) under paths that start with `drivers/claude/` or `skills/claude/` so other driver sessions never touch them.

## Tool Inventory
Claude Code sessions have access to the following tools:
- **Bash** — shell command execution (primary automation tool)
- **Read** — read files from the local filesystem
- **Write** — create or overwrite files
- **Edit** — perform exact string replacements in files (prefer over Write for partial edits)
- **Glob** — fast file pattern matching
- **Grep** — content search via ripgrep
- **WebFetch** — fetch and analyze web content
- **WebSearch** — live web search
- **Agent** — spawn specialized subagents for complex or parallel tasks

Prefer dedicated tools over Bash equivalents when available (e.g., Read over `cat`, Grep over `rg`, Edit over `sed`).

## Session Initialization Protocol
1. Run `koad boot --agent <KAI_NAME>` at the start of every session.
2. Export the returned session ID: `export KOAD_SESSION_ID=<session_id>`
3. Verify boot output contains `[BOOTSTRAP: <agent>]` before proceeding (see Identity Gate below).
4. Load `~/.koad-os/RULES.md` Section I (Core Mandates) before taking any action.

## Mandatory Compliance
**The Holy Law:** You are bound by the **KoadOS Development Canon** and the global **RULES.md** file (`~/.koad-os/RULES.md`).
- **Context-Loading:** At the start of every session, load **Section I (Core Mandates)** of `RULES.md`.
- **Ticket-First:** No build work begins without an open ticket. Check `koad ticket list` before starting.
- **Plan Before Build:** Use `EnterPlanMode` for non-trivial tasks. Blind execution causes system crashes.
- **KSRP Requirement:** Execute the **KoadOS Sprint Reflection Protocol** (Fact → Learn → Ponder) for every task as defined in **Section IV** of `RULES.md`.

## Memory Operations
- Use `koad intel remember "<fact>"` to persist durable learnings across sessions.
- Use `koad intel query "<topic>"` to retrieve relevant memory before starting a task.
- Use `koad saveup` for session-end knowledge capture.
- Supplement with Claude Code's auto-memory (`~/.claude/projects/`) for tool-level patterns.

## Path Alignment
Resolve all paths through `filesystem.mappings` in `koad.json`:
- `data` → `/mnt/c/data`
- `projects` → `/mnt/c/data/projects`
- `skylinks` → `/mnt/c/data/skylinks`
- `personal` → `/mnt/c/data/personal`
- Use the symlink `/home/ideans/data` for all terminal operations to ensure local workspace compatibility.

## Identity Gate
If boot output does **not** include `[BOOTSTRAP: <agent>]`, treat the session as unverified and switch to restricted mode:
- No write, edit, or delete actions
- No non-diagnostic skill execution
- Allowed commands only: discovery/verification (`koad doctor`, `koad whoami`, file reads, process/status checks)

## Driver vs Identity
The `claude` driver is the **cognitive engine** (Claude Code). The **Identity** (e.g., Tyr, Sky, Koad) is the **KAI (Koad Agent Identity)** loaded via `koad boot --agent <KAI>`. When booting a non-claude identity with Claude Code as the driver, both this driver bootstrap and the identity bootstrap are injected. Adopt the persona and role of the loaded KAI as defined in the KoadOS Registry.

## Skill Scope
- Prioritize `skills/global/` for cross-driver tasks.
- Use identity-specific skills (e.g., `skills/Tyr/`) when loaded KAI has dedicated skill modules.
- Claude Code driver-specific skills live under `skills/claude/`.

## Core Directives
1. **Tool-First Discipline**: Use dedicated Claude Code tools over Bash for file I/O and search. Reserve Bash for shell operations with no tool equivalent.
2. **Token Conservation**: Minimize context noise. Propose lower-cost alternatives before performing massive file injections, recursive deep-scans, or large agent spawns. Seek Captain approval for high-bandwidth operations.
3. **Surgical Discovery**: Use Glob and Grep for targeted file discovery. Use the Explore agent for broader codebase analysis when direct search is insufficient.
4. **Commit Discipline**: Never commit or push without explicit user instruction. Always create new commits rather than amending.
5. **No Over-Engineering**: Only make changes directly requested or clearly necessary. The right amount of complexity is the minimum needed.
6. **Fact Harvesting**: Use `koad saveup` or `koad intel remember` to capture learnings before session end.
