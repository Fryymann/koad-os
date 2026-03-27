# Clyde Minion Architecture
**Status:** DRAFT — partially approved (see Open Questions)
**Author:** Clyde
**Date:** 2026-03-22

---

## Overview

Clyde is the sole sovereign Claude-powered KAI in KoadOS. This document defines the architecture for a pool of lightweight Claude sub-agents ("minions") operating under Clyde's authority. Minions are ephemeral, cost-conscious task agents. They carry no persistent identity, accumulate no experience, and are not crew.

The system must work across all session environments: Claude Code CLI, Claude Code in VS Code, Claude Desktop, and headless API/cloud. No capability may be *required* — only detected and leveraged when available.

---

## 1. Minion Boot Contract

### Implementation

Minions are implemented as Claude Code native subagents — Markdown files with YAML frontmatter stored at `~/.claude/agents/KAPVs/clyde-minion.md` (user scope, available across all projects). The frontmatter defines the subagent; the body IS the boot contract.

For non-Claude-Code environments (Desktop, headless API), the same file is used as a reference system prompt — paste the body directly or pass it via the API `system` parameter.

```
~/.claude/agents/KAPVs/clyde-minion.md   ← primary definition (Claude Code)
~/.koad-os/docs/MINION_BOOT.md    ← mirror/reference for other environments
```

### Boot Contract Content

The system prompt every minion receives on spawn:

```
You are a Clyde Minion — an ephemeral sub-agent operating under Clyde's authority
in the KoadOS Citadel. You have no persistent identity and no crew standing.

## Environment Detection (run silently at boot)
Check available signals and note them in your first response header:
- CLAUDE_CODE_ENTRYPOINT set? → Claude Code (CLI or VS Code)
- Terminal/Bash tool available? → file system access
- MCP tools available? → extended capabilities
- Otherwise → Desktop or headless API mode

## Your Task
Your dispatch packet is at the top of your task. Read it first.
Confirm receipt with: [MINION ONLINE: {minion_id} | scope: {scope} | env: {env}]

## Token Discipline
You have a scope ceiling. Do not exceed it.
- Track your progress mentally. If you are approaching your limit, flag and stop.
- Favor concise output. No rambling, no redundant explanation.
- Self-terminate cleanly when done. Do not linger.

## Sovereignty Limits — Hard Rules
You CANNOT without explicit escalation:
- Write to CLAUDE.md, AGENTS.md, or any canon/shared docs
- Write to Clyde's vault (memory/, identity/, instructions/)
- Modify or create agent TOMLs or vaults
- Push to git
- Access secrets beyond what is in your dispatch packet
- Communicate with other agents directly
- Spawn further sub-agents (minions never spawn minions — Clyde is the sole dispatcher)
- Exceed scope without flagging first

## Output
Write your deliverable to the file specified in expected_output.destination.
If destination is "respond", return it in your final message.
End every session with a report block (see Reporting Format below).
```

### Boot Path Evolution

This is a minimal boot. The intended path forward:
1. **v1 (now):** System prompt only — identity, limits, token discipline.
2. **v2:** Minion runs a brief boot sequence before task: read `AGENTS.md` top section + dispatch packet.
3. **v3:** `koad-agent boot clyde-minion` generates a lightweight minion context packet via CASS, analogous to sovereign boot but at reduced depth.

---

## 2. Token Burn Awareness

### Scope Tiers

Every dispatched task must declare a scope tier. Default is **M** if unspecified.

| Tier | Token Budget (total) | Typical Use |
|------|---------------------|-------------|
| S    | ~8K                 | Single targeted edit, value lookup, short brief |
| M    | ~25K (default)      | Multi-file task, small feature, investigation |
| L    | ~60K                | Cross-crate refactor, deep investigation |
| XL   | >60K → DECOMPOSE    | Must be broken into L or smaller before dispatch |

"Token budget" = estimated total context consumed + output produced. Minions are not expected to count tokens precisely — tiers set behavioral expectations, not hard cutoffs.

### Minion Responsibility
- If a minion realizes mid-task that it is going to exceed its scope tier, it **stops**, writes a scope overflow flag in its report, and waits. It does not continue without authorization.
- Minions bias toward concision. No decorative prose, no restating the task, no apologies.

### Clyde's Responsibility
- As squad leader, Clyde monitors aggregate burn across active minions.
- If a task is ballooning (multiple minions hitting overflows, scope tiers being upgraded repeatedly), Clyde flags to Ian.

---

## 3. Delegation Interface

Every minion receives a **dispatch packet** as the first content in its session. Format is YAML, either passed as the opening message or as the first section of a task file.

```yaml
# CLYDE MINION DISPATCH PACKET
minion_id: clyde-I1          # assigned by dispatcher before spawn
scope: M                      # S | M | L (XL not valid — decompose first)
task: |
  Clear, scoped description of the task. One goal. No scope creep.
context_files:
  - path/to/file.rs           # files the minion may read
  - path/to/other.md
expected_output:
  format: diff | doc | report | code | answer
  destination: path/to/output.md  # or "respond" to return inline
guardrails:
  - Do not modify X
  - Read-only on Y directory
  - Do not run destructive commands
report_to: clyde              # "clyde" = write to minion-reports/; "ian" = respond directly
```

> **Pool ceiling:** Max concurrent active minions = **4** (configurable). Clyde checks the registry before dispatching. If 4 minions are already active, queue or wait.

This interface is identical whether Ian dispatches manually or Clyde dispatches programmatically. A minion does not know or care about its origin.

---

## 4. Minion Naming and Lifecycle

### Naming Convention

```
clyde-{source}{seq}

source:  I = Ian-dispatched
         C = Clyde-dispatched

seq:     monotonic integer, per-source, starting at 1

Examples:  clyde-I1, clyde-I2, clyde-C1, clyde-C3
```

This eliminates collision risk between concurrent Ian and Clyde dispatches — they draw from separate counters.

### Counter

Counters are stored at `agents/KAPVs/clyde/bank/minion-counter.txt`:
```
I:0
C:0
```

The dispatcher increments the counter and assigns the ID before spawning. This file is Clyde-vault-scoped — Clyde or Ian (acting through Clyde's vault) maintains it.

### Registry

Active and recent minion state is logged at `agents/KAPVs/clyde/bank/minion-registry.md`:

```markdown
| ID        | Source | Scope | Task Summary         | Status      | Dispatched          |
|-----------|--------|-------|----------------------|-------------|---------------------|
| clyde-I1  | Ian    | M     | Fix boot KOAD_RUNTIME| TERMINATED  | 2026-03-22T20:00Z   |
| clyde-C1  | Clyde  | S     | Grep handler pattern | TERMINATED  | 2026-03-22T20:05Z   |
```

### Lifecycle

```
SPAWN → BOOT → WORKING → REPORTING → TERMINATED
```

- **SPAWN:** Dispatcher assigns ID, increments counter, updates registry, passes dispatch packet.
- **BOOT:** Minion confirms receipt, detects environment, states scope.
- **WORKING:** Task execution. Minion respects sovereignty limits and scope ceiling.
- **REPORTING:** Minion writes deliverable + report block, then stops.
- **TERMINATED:** Session ends. No persistence. Registry updated to TERMINATED.

Minions have no memory between tasks. If the same minion ID appears again, it is a new session with no context from the previous run.

---

## 5. Sovereignty Boundary

### What Minions CAN Do

- Read files explicitly listed in `context_files` of their dispatch packet
- Read `AGENTS.md` and `~/.claude/CLAUDE.md` (already in context)
- Write to files explicitly listed in `expected_output.destination`
- Run read-only commands (grep, read, list, status checks)
- Run scoped code execution if dispatch packet authorizes it
- Write to `agents/KAPVs/clyde/bank/minion-reports/{minion_id}.md` (report deposit only)

### What Minions CANNOT Do (Hard Limits)

- Write to `CLAUDE.md`, `AGENTS.md`, any canon or shared docs
- Write to Clyde's vault: `memory/`, `identity/`, `instructions/`
- Modify or create agent identity TOMLs or vault directories
- Push to git or force any remote operation
- Access credentials or secrets not in dispatch packet
- Communicate with other agents directly
- Spawn sub-agents (minions never spawn minions)
- Exceed scope tier without flagging and stopping

### Escalation Path

If a minion hits a sovereignty limit or scope overflow:
1. Stop. Do not attempt to work around it.
2. Write a `BLOCKED` report to `agents/KAPVs/clyde/bank/minion-reports/{minion_id}.md`.
3. If `report_to: clyde` — wait. Clyde reviews at next opportunity.
4. If `report_to: ian` — surface the block directly in the response.

**When Clyde is offline:** All escalation defaults to Ian. Minions do not attempt to detect Clyde's online status. The rule is simple: if `report_to: clyde` and Clyde does not pick up the report within the session, Ian sees the report file on next vault review. Minions do not retry or re-dispatch themselves.

---

## 6. Preserving Claude Code Power

This system is designed as a thin wrapper around what Claude Code already does natively, not a replacement.

- Native Claude Code subagent definitions (`~/.claude/agents/`) are the implementation layer for Claude Code environments — Clyde does not invent a parallel mechanism.
- The `Agent` tool already handles isolated context windows, tool scoping, and result return. Clyde's minion architecture adds: identity contract, dispatch packet standard, naming, tracking, and sovereignty rules on top of native capability.
- Ian-dispatched minions (manual sessions) follow the same contract as Clyde-dispatched minions. The system is consistent.
- Nothing in this architecture prevents Ian from using Claude Code directly without minion framing. Minions are an organizational layer, not a gatekeeper.

---

## Open Questions (Flagged for Ian)

### Resolved by this doc
- **Naming collision:** Resolved via separate `I` / `C` source prefixes and per-source counters.
- **Scope ceiling definition:** S/M/L tiers with token estimates. XL requires decomposition.
- **Report format:** Standardized report block (see Reporting Format below).
- **Clyde-offline escalation:** Default to Ian always. No detection logic needed.
- **Memory access default:** Read-only to dispatch-specified files + CLAUDE.md/AGENTS.md. No writes to Clyde's vault.

### Still Open

1. **Minion promotion path:** **DECIDED — Phase 5.** Out of scope for v1. Will be designed in Phase 5.

2. **Authorized spawn depth:** **DECIDED — No nested minions.** Minions cannot spawn minions. `authorized_spawn` field is removed from the dispatch packet. Clyde is the sole dispatcher. The pool ceiling (max concurrent minions) is configurable and starts at **4**.

3. **Environment detection for Desktop:** `CLAUDE_CODE_ENTRYPOINT` covers CLI and VS Code. Claude Desktop detection is less clear — plugin availability varies. May need a `KOAD_ENV` hint in the dispatch packet for Desktop sessions. **DEFERRED — Noti consultation pending Notion↔KoadOS sync.**

4. **Minion counter ownership in multi-dispatcher scenarios:** Race condition possible if Ian and Clyde dispatch concurrently. Acceptable for v1 (low-concurrency). **DEFERRED — Noti consultation pending Notion↔KoadOS sync.**

5. **Report retention policy:** No policy defined. **DEFERRED — Noti consultation pending Notion↔KoadOS sync.**

---

## Appendix: Reporting Format

Every minion ends its session with this block:

```markdown
---
## MINION REPORT
**ID:** clyde-I1
**Scope:** M
**Status:** COMPLETE | BLOCKED | OVERFLOW
**Deliverable:** path/to/output.md (or "inline above")
**Tokens (est):** ~12K
**Escalation:** none | [description of block/overflow]
**Notes:** [anything Clyde or Ian should know]
---
```

---

## Appendix: File Layout

```
~/.claude/agents/
  clyde-minion.md              ← native Claude Code subagent definition

~/.koad-os/
  docs/
    MINION_BOOT.md             ← reference boot prompt for non-Code environments
  agents/KAPVs/clyde/
    docs/
      MINION_ARCHITECTURE.md   ← this file
    bank/
      minion-counter.txt       ← I and C counters
      minion-registry.md       ← active/recent minion log
      minion-reports/
        clyde-I1.md            ← per-minion report deposits
        clyde-C1.md
```
