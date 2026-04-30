# Design: Agent Boot Skill & KoadOS Claude Plugin

**Date:** 2026-04-30
**Status:** Approved
**Author:** Clyde

---

## Overview

Convert the `agent-boot` bash function into a self-contained Claude Code skill, distributed as part of a new `koad-agents` plugin that ships with the `koad-os` repo. The skill wraps the boot sequence for terminal-based coding agents, delegates to canonical bash logic inside the plugin, and supports three boot levels with `standard` as the default.

---

## Goals

- Unify agent boot behavior across all harnesses (CLI, subagents, CI) through a single skill
- Make the boot logic canonical and versioned inside `koad-os` — not scattered in `koad-functions.sh`
- Open the door for future Citadel skills to be distributed as a proper Claude plugin

---

## Plugin Structure

Lives at `koad-os/plugin/`:

```
koad-os/
  plugin/
    package.json          # Claude plugin manifest (name: koad-agents)
    bin/
      agent-boot.sh       # Canonical boot logic (moved from koad-functions.sh)
    skills/
      agent-boot/
        SKILL.md          # Core skill: frontmatter, protocol, level routing
        quick.md          # Level A: boot only
        standard.md       # Level B: boot + orient (default)
        full.md           # Level C: boot + orient + tasks + Condition Green
```

`package.json` is minimal — name, version (aligned to OS release cycle), no `main` entry (pure skill plugin, no JS hook).

---

## Skill Protocol

### Invocation

```
/agent-boot <name>           # default: standard
/agent-boot <name> --quick
/agent-boot <name> --full
```

### `SKILL.md` — Core

- Frontmatter: `name: agent-boot`, triggers on session start or explicit boot request
- Parses agent name and optional level flag
- Branches to the appropriate level sub-file

### `quick.md` — Level A

```
1. source $KOAD_HOME/plugin/bin/agent-boot.sh via koad-functions wrapper
2. Done — await user direction
```

Use case: mid-session re-hydration, scoped subagent spawn, CI context.

### `standard.md` — Level B (default)

```
1. Run agent-boot (via koad-functions wrapper)
2. Run: koad map look
3. Run: koad system status (Redis / Citadel / CASS health)
4. Summarize: identity confirmed, service state, open items from working memory
```

Use case: normal session open.

### `full.md` — Level C

```
1. Run agent-boot (via koad-functions wrapper)
2. Run: koad map look
3. Run: koad system status
4. Read open tasks from vault tasks directory
5. Assert Condition Green — flag any blockers before proceeding
6. Deliver full situational report to Dood
```

Use case: new major session, post-incident recovery, inter-agent handoff.

---

## Integration & Data Flow

```
Agent invokes /agent-boot <name> [--quick|--full]
       ↓
SKILL.md → parse args → branch to level sub-file
       ↓
Level sub-file → run agent-boot via koad-functions.sh wrapper
       ↓
koad-functions.sh (thin wrapper — preserves eval/export scope):
  function agent-boot() { source "$KOAD_HOME/plugin/bin/agent-boot.sh" "$@" }
       ↓
plugin/bin/agent-boot.sh (canonical logic):
  → display cached brief
  → detect runtime (env signals → TOML fallback)
  → set WSL LD_LIBRARY_PATH if applicable
  → eval $(koad-agent boot <name>) → env vars hydrated into shell
       ↓
[standard/full] koad map look → situational awareness
[standard/full] koad system status → service health
[full] read vault tasks → open items
[full] assert Condition Green → flag blockers
       ↓
Agent reports boot complete + session state
```

**Key constraint:** `agent-boot.sh` must be sourced (not exec'd as a subprocess) so `eval`'d env var exports propagate to the calling shell. `koad-functions.sh` retains the shell function wrapper for exactly this reason. The canonical logic moves; the shell mechanics stay.

---

## What Does NOT Change

- `koad-agent` Rust binary — no changes
- `koad-agent boot <name>` output format — no changes
- `koad-functions.sh` shell function signature — no changes (body becomes a delegation call)
- Any existing `agent-boot tyr` / `agent-boot clyde` call sites — no changes

---

## Out of Scope

- VS Code extension support (terminal-only)
- Non-bash harnesses (Gemini, Codex boot logic is a separate concern)
- Modifying `koad-agent` boot output

---

## Open Questions

- None — all decisions resolved during design session.
