## Purpose

A repeatable, full-system health check for the KoadOS platform. Invoke this protocol after any major codebase modifications to detect gaps, regressions, broken integrations, and state drift across all KoadOS subsystems.

This is a **read-only audit**. The agent executing this check **MUST NOT fix anything**. It produces a structured **SIC Report** that the Admiral (Ian) uses to prioritize remediation.

---

## Invocation

```bash
# Boot the agent, then paste this protocol as the task directive
koad boot --agent <Name> --project koad-os
```

Alternatively, reference this document via `@import` or paste it directly into the session prompt.

**Recommended agent:** Gemini CLI (`gemini-2.5-pro`) — the 1M token context window allows full-system scanning without context pressure.

**Estimated time:** 15–30 minutes depending on codebase size.

---

## Pre-Check Setup

Before beginning the audit passes, execute these setup steps:

1. Confirm you are in the `koad-os` project root (`pwd`, verify `.git` and `.koad` markers exist)
2. Run `git status` — record the current branch, commit hash, and working tree state
3. Run `koad doctor` — record the output as the baseline system status
4. Confirm Redis is reachable: `redis-cli ping` → expect `PONG`
5. Confirm the Spine is running: `koad system status` — record output
6. Record the current date/time as the SIC timestamp

If any pre-check fails, **log it immediately** as a `critical` finding and continue with remaining passes where possible.

---

## Audit Passes

Execute each pass in order. For every finding, classify severity using the KSRP scale:

- `info` — Observation only. No action required.
- `warning` — Minor issue. Does not block operations but should be addressed.
- `error` — Significant issue. Must be fixed to restore full system health.
- `critical` — Blocker. Immediate escalation to the Admiral.

---

### Pass 1 — Build & Compilation Integrity

**Goal:** Confirm the entire codebase compiles cleanly.

```bash
cargo build 2>&1
cargo build --release 2>&1
```

**Check for:**

- Any compilation errors → `critical`
- Any compilation warnings → `warning` (list each with [file:line](file:line))
- Unused imports, dead code, deprecated API usage → `warning`

---

### Pass 2 — Lint & Style Compliance

**Goal:** Confirm code meets KoadOS style standards.

```bash
cargo fmt --check 2>&1
cargo clippy -- -D warnings 2>&1
```

**Check for:**

- Formatting violations → `warning`
- Clippy warnings treated as errors → `error`
- Any `#[allow(...)]` suppressions that look like they're hiding real issues → `warning`

---

### Pass 3 — Test Suite & Coverage

**Goal:** Confirm all tests pass and coverage is adequate.

```bash
cargo test 2>&1
cargo llvm-cov --summary-only 2>&1
```

**Check for:**

- Any test failures → `error` (list each with test name and failure message)
- Tests that panic instead of asserting → `warning`
- Coverage below 70% on any crate → `warning`
- Coverage below 50% on any crate → `error`
- Missing negative/failure test cases for critical paths (Spine, ASM, Sentinel) → `warning`

---

### Pass 4 — Dependency & Security Audit

**Goal:** Confirm no vulnerable or outdated dependencies.

```bash
cargo audit 2>&1
cargo outdated 2>&1
```

**Check for:**

- Known vulnerabilities (RUSTSEC advisories) → `critical`
- Major version updates available for key dependencies → `warning`
- Yanked crate versions in use → `error`

---

### Pass 5 — Spine & Core Services Health

**Goal:** Confirm the Spine, ASM, Sentinel, and Watchdog are operational and correctly integrated.

```bash
koad system status
koad system health
```

**Check for:**

- Spine (kspine) gRPC service not responding → `critical`
- ASM heartbeat monitoring not running → `critical`
- Sentinel not performing context hydration on boot → `error`
- Watchdog not detecting/pruning ghost processes → `error`
- Any service reporting degraded status → `error`
- Services running but with stale PID files → `warning`

**Integration checks:**

- Run `koad boot --agent Tyr` (or any registered agent) in a test capacity and verify:
    - Identity loads from `koad.json` → cached in Redis → session tethered via `KOAD_SESSION_ID`
    - Sentinel hydrates personal memory from SQLite into Redis Hot Stream
    - Session appears on `koad:sessions` bus
- If possible, attempt a second boot in the same session — confirm it is **rejected** (One Body, One Ghost enforcement)
- Terminate the test session and verify ASM purges the volatile context

---

### Pass 6 — Redis Engine Room Integrity

**Goal:** Confirm Redis state is clean and consistent.

```bash
redis-cli ping
redis-cli info keyspace
redis-cli keys "koad:sessions:*"
redis-cli keys "koad:agent:*"
```

**Check for:**

- Redis unreachable → `critical`
- Orphaned session keys (sessions with no active process) → `error`
- Stale WAKE entries → `warning`
- Keys without TTL that should have one → `warning`
- Session keys that don't correspond to any registered agent in `koad.json` → `error`
- `koad:sessions` pub/sub bus not functioning → `critical`

---

### Pass 7 — SQLite Memory Bank Integrity

**Goal:** Confirm the persistent memory store is healthy and partitions are intact.

```bash
sqlite3 ~/.koad-os/koad.db "PRAGMA integrity_check;"
sqlite3 ~/.koad-os/koad.db ".tables"
sqlite3 ~/.koad-os/koad.db "SELECT COUNT(*) FROM <agent_memory_table>;"
```

**Check for:**

- `PRAGMA integrity_check` returns anything other than `ok` → `critical`
- Missing expected tables → `error`
- Agent memory partitions that don't correspond to registered agents → `warning`
- PSRP entries with overwritten (non-append) patterns → `error` (violates append-only rule)
- Empty memory partitions for agents that have completed tasks → `warning`

---

### Pass 8 — Agent Registry Validation

**Goal:** Confirm `koad.json` is well-formed and all agents are properly registered.

```bash
cat ~/.koad-os/koad.json | python3 -m json.tool
```

**Check for:**

- JSON parse errors → `critical`
- Any agent missing required fields (`bio`, `rank`, `preferences`, `authority_tier`) → `error`
- Duplicate agent names or session IDs → `error`
- Agents referenced in code or configs but not in the registry → `error`
- Agents in the registry that appear decommissioned (no recent activity, no memory entries) → `info`
- Authority tier assignments that seem incorrect (e.g., a Tier 3 support agent with Tier 1 privileges) → `warning`

---

### Pass 9 — Configuration Integrity (Body/Ghost Compliance)

**Goal:** Confirm all config files follow the agent-agnostic Body/Ghost model.

**Gemini layer:**

```bash
cat ~/.gemini/GEMINI.md
cat ~/.gemini/settings.json | python3 -m json.tool
```

**Codex layer:**

```bash
cat ~/.codex/AGENTS.md 2>/dev/null || echo "NOT FOUND"
cat ~/.codex/config.toml 2>/dev/null || echo "NOT FOUND"
```

**Project-level:**

```bash
cat ~/data/koad-os/.gemini/GEMINI.md 2>/dev/null || echo "NOT FOUND"
cat ~/data/koad-os/.gemini/settings.json 2>/dev/null || echo "NOT FOUND"
cat ~/data/skylinks/.gemini/GEMINI.md 2>/dev/null || echo "NOT FOUND"
```

**Check for:**

- Any hardcoded agent name, persona, or "I am" statement in global configs → `error`
- Boot Directive missing from global [GEMINI.md](http://GEMINI.md) or [AGENTS.md](http://AGENTS.md) → `error`
- One Body, One Ghost protocol not declared → `error`
- `@import` references that don't resolve to existing files → `error`
- `settings.json`: `enablePermanentToolApproval` set to `true` → `error`
- `settings.json`: `defaultApprovalMode` not set to `"default"` → `warning`
- Missing project-level configs for active projects → `warning`
- Stale or contradictory instructions between global and project layers → `warning`

---

### Pass 10 — Secrets & Security Scan

**Goal:** Confirm no secrets are exposed in the codebase or configs.

```bash
rg -i '(api[_-]?key|secret|token|password|bearer|private[_-]?key)\s*[:=]' --type rust --type toml --type json --type md -g '!target/' -g '!node_modules/' .
rg -i '(ghp_|sk-|AIza|AKIA|xox[bpas]-)' -g '!target/' -g '!node_modules/' .
```

**Check for:**

- Any actual secret values (API keys, tokens, passwords) hardcoded in source → `critical`
- Secret-like patterns in non-.env files → `error`
- `.env` files committed to git → `critical` (check `git ls-files '*.env'`)
- Secrets in git history → `warning` (check `git log --all -p | rg -c '(ghp_|sk-|AIza)'` — sample only)
- `koad auth` not configured for directory-aware PAT selection → `warning`

---

### Pass 11 — Cross-Module Integration Check

**Goal:** Confirm all subsystems are wired together and communicating correctly. This is the most important pass — it catches the **gaps between systems** that individual module checks miss.

**Check for:**

- **Spine → Redis:** Boot an agent, verify identity lands in Redis. Kill the session, verify cleanup.
- **Spine → SQLite:** After boot, verify Sentinel hydrated memory from SQLite into Redis. Compare key counts.
- **CLI → Spine:** Run `koad system status`, `koad board status`, `koad auth` — all should return valid responses. Any command that errors or hangs → `error`.
- **CLI → GitHub:** Run `koad board status` or equivalent — verify it connects to Project #2 and returns current board state.
- **Config → Runtime:** Verify that `GEMINI.md` `@imports` are actually loaded at runtime (use `gemini -d` to check discovery trace if possible).
- **Watchdog → ASM:** Manually create a stale session key in Redis, wait 5–10 seconds, verify the Watchdog prunes it.
- **Memory round-trip:** Write a test fact via `koad intel remember "SIC test fact"`, then query it back. Verify it persists in SQLite and is retrievable.
- **Any command that silently fails** (returns success code but produces no output or wrong output) → `error`
- **Any integration that worked previously but now errors** (regression from recent changes) → `error`

---

### Pass 12 — Git & Repository Hygiene

**Goal:** Confirm the repo is in a clean, canonical state.

```bash
git status
git log --oneline -20
git branch -a
git stash list
```

**Check for:**

- Uncommitted changes in working tree → `warning`
- Untracked files that should be gitignored → `warning`
- Stale branches (no commits in 30+ days, not merged) → `info`
- Recent commits without issue number references → `warning`
- Stashed changes that might be forgotten work → `info`
- `.gitignore` missing entries for `target/`, `node_modules/`, or other build artifacts → `error`

---

## SIC Report Template

After completing all passes, produce the report in this exact format:

```markdown
# KoadOS Systems Integrity Check (SIC) Report

**Date:** YYYY-MM-DD HH:MM (timezone)
**Agent:** <Name>
**Model:** <model used>
**Branch:** <branch>
**Commit:** <short hash>
**Baseline (koad doctor):** <summary>

---

## Summary

| Severity | Count |
|----------|-------|
| critical | X     |
| error    | X     |
| warning  | X     |
| info     | X     |

**Overall Status:** CONDITION GREEN | CONDITION YELLOW | CONDITION RED

- CONDITION GREEN: 0 critical, 0 error
- CONDITION YELLOW: 0 critical, 1+ error
- CONDITION RED: 1+ critical

---

## Findings by Pass

### Pass 1 — Build & Compilation
| # | Severity | Finding | Location | Recommended Action |
|---|----------|---------|----------|--------------------|
| 1 | ...      | ...     | ...      | ...                |

### Pass 2 — Lint & Style
...

(Repeat for all 12 passes. If a pass is clean, state: "✅ No findings.")

---

## Critical Path Items (Immediate Attention)

(List all critical and error findings here, grouped by subsystem, with recommended remediation order)

---

## Regression Watch

(List any findings that appear to be regressions from recent changes — things that previously worked but are now broken. Cross-reference against recent git log to identify which commits likely introduced the issue.)

---

## Recommendations

(Prioritized list of remediation actions. Group by: "Fix Now", "Fix This Sprint", "Track for Later")
```

---

## Constraints

- **Read-only.** Do not modify any files, configs, databases, or state. This is an audit.
- **No fixes.** Even if you know the fix, log it as a finding with a recommended action. Do not apply it.
- **No pushes or commits.** Do not touch git history.
- **Do not skip passes.** If a pass cannot be executed (e.g., tool not installed), log it as an `error` finding with the reason and continue.
- **Use `rg`, `bat`, `cat`, and standard Unix tools** before invoking LLM reasoning for discovery. Preserve intelligence budget for analysis.
- **Time-box each pass to 3 minutes.** If a pass requires deeper investigation than 3 minutes allows, log what you found so far and note "deeper investigation recommended" in the finding.
- **Output the SIC Report** to `stdout` AND to `~/.koad-os/sic-reports/SIC-YYYY-MM-DD.md` for historical tracking.

---

## Post-SIC Actions (for the Admiral)

After reviewing the SIC Report:

1. Triage findings by severity
2. Create GitHub Issues for all `error` and `critical` findings
3. Assign remediation to the appropriate KAI (Tyr for system/infra, Vigil for security, Sky for project/ops)
4. Re-run SIC after remediation to confirm CONDITION GREEN