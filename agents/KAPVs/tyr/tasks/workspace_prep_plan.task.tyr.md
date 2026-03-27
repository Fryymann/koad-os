**Assignee:** Tyr (Captain) 

**Branch:** `nightly` 

**Priority:** Complete before Phase 4 sprint planning 

**Goal:** Clean, consolidated, accurate repo docs and structure so all agents can operate from a single source of truth without token-wasting confusion.

---

# Audit Summary

Noti audited every directory and key file in `Fryymann/koad-os` on the `nightly` branch (2026-03-27). The codebase is structurally sound — the Rust workspace compiles, services run, and the crate architecture is solid. But the **documentation and organizational layer is badly out of sync** with reality. Agents booting into this repo will read stale phase statuses, encounter ghost personnel, and find planning artifacts scattered across 4+ locations.

<aside>
🔴

**Core Problem:** An agent reading `agents/CITADEL.md` thinks we're in Phase 1. An agent reading root `AGENTS.md` thinks Pic is the Captain. An agent reading `crates/AGENTS.md` thinks we have 4 crates. None of this is true anymore.

</aside>

---

# Finding Categories

## 🔴 Critical — Actively Misleading Agents

These files contain wrong information that will cause agents to make incorrect assumptions:

| **File** | **Severity** | **Issue** |
| --- | --- | --- |
| `agents/CITADEL.md` | Critical | Says "Phase 1 — Citadel MVP Construction." We're in Phase 4. Build phases listed are the old 5-phase plan (not the new 10-phase). References `tyr_plan_review.md` (archived) and GitHub Projects board #6 (stale). |
| `agents/CREW.md` | Critical | Lists **Pic** as Captain — Pic is deprecated. **Tyr is missing from the table** (only mentioned in deployment protocols). References `~/.pic/logs/` for communication. Vigil listed in deployment protocols but is deprecated. Doesn't reflect current crew structure. |
| Root `AGENTS.md` | Critical | Personnel table (§Ⅶ) lists Pic as Captain, Helm as "Security" (should be Officer/Build Engineer). Has **duplicate section numbering** (two §Ⅱ headers). Date says 2026-03-15. Phase status says 4 (correct) but the surrounding context is stale. |
| `crates/AGENTS.md` | Critical | **Lists only 4 crates** (core, proto, citadel, cass). Workspace has **11 crates**. Missing: koad-cli, koad-board, koad-bridge-notion, koad-intelligence, koad-sandbox, koad-codegraph, koad-plugins. Says cass is "Phase 2" — it's COMPLETE. |
| `config/kernel.toml` | High | Contains `[watchdog]` section but `koad-watchdog` is deprecated/removed (confirmed in SYSTEM_[MAP.md](http://MAP.md)). Dead config that could confuse agents working on kernel.toml. |

## 🟡 Stale — Outdated but Not Dangerous

These files are behind reality but won't actively mislead agents into wrong actions:

| **File / Directory** | **Issue** |
| --- | --- |
| `new_world/DRAFT_PLAN_3.md` | Superseded by the Notion Master Dev Plan. Should be archived to `new_world/archived/`. |
| `new_world/DEVELOPMENT_PLAN_3.plan.md` | Same — superseded. Archive alongside DRAFT_PLAN_3. |
| `STARTUP_CHECKLIST.md` | Jupiter Migration checklist — one-time task, likely completed. References Docker Redis Stack, `.env.template`, `PimpedBash`. Archive or delete. |
| `docs/rebuild/DIRECTORY_CLEANUP.md` | Describes `personas/` → `config/identities/` migration. Already done. Status still says "DRAFT (Phase 1)." Archive. |
| `docs/AGENTS.md` | Mentions only 2 files out of the dozens in `docs/`. Severely incomplete. |
| `scripts/AGENTS.md` | Mentions only `koad-telemetry.sh` — there are 7 files in `scripts/`. Incomplete. |
| `agents/.claude/` context/README | Says "Phase 1 — Citadel MVP" as Claude's focus. Stale — Claude Code (Clyde) is now an Officer working on Phase 4+. |
| `agents/quests/side_quests/spine_eradication` | Spine eradication is DONE (per `updates/` log entry 2026-03-27). Quest should be marked complete/archived. |
| `docs/MINION_BOOT.md` | References "Clyde Minion Boot" with scope tier budgets. Partially relevant but needs update for the new Minion Swarm architecture (Phase 9). |

## 🟠 Structural — Organization Problems

| **Issue** | **Details** |
| --- | --- |
| **Planning artifacts in 4+ locations** | `new_world/` (strategic plans), `plans/` (implementation plans), `docs/requests/` (feature/skill requests), `agents/quests/` (side quests). No single place to find "what are we building next." |
| **No per-crate [AGENTS.md](http://AGENTS.md) files** | Only `crates/AGENTS.md` (top-level, stale). Individual crates have no onboarding docs. Agents touching `koad-citadel` have no crate-level guide to read first. |
| **Feature requests are loose files** | `docs/requests/` has ~12 request files in various subdirectories. These are effectively Icebox items living in the repo instead of a trackable system. |
| **SESSIONS_[LOG.md](http://LOG.md) is a monolith** | `agents/SESSIONS_LOG.md` is an ever-growing file (currently 170+ lines, 9 sessions). Will become a token sink as agents read it on boot. Needs archival rotation. |
| **`templates/` is underused** | Only contains `issues/`. No task manifest template, no EoW template in templates/ (EoW template is in `docs/protocols/` instead). |
| **docker-compose.yml vs. kernel.toml mismatch** | Docker Compose exposes Redis on port 6379 (TCP). kernel.toml uses `run/koad.sock` (Unix socket). May represent different deployment modes but is confusing without a comment explaining why. |

---

# Task List for Tyr

Ordered by priority. Each task is scoped to be completable in a single agent session.

## Sprint 0A — Distribution / Instance Split (Do First)

*This is the foundational change. Everything else builds on having the right files in the right layer.*

### Task 0: Separate Distribution from Instance State

KoadOS is a deployable platform. The repo is a **distribution artifact**. Two Citadel operators pulling from the same repo must never get a merge conflict. This means instance-specific state must be gitignored, with defaults/templates shipped for bootstrap.

**Config split:**

- [ ]  Create `config/defaults/` directory
- [ ]  Move `config/kernel.toml` → `config/defaults/kernel.toml` (this becomes the shipped template)
- [ ]  Move `config/redis.conf` → `config/defaults/redis.conf`
- [ ]  Update `install/bootstrap.sh` to copy defaults into the live config location if no local config exists (i.e., `cp -n config/defaults/kernel.toml config/kernel.toml`)
- [ ]  Add `config/kernel.toml` and `config/redis.conf` (the live copies) to `.gitignore`
- [ ]  Add a `config/README.md` explaining: "defaults/ ships with the repo. Live config is generated at bootstrap and is gitignored."

**State gitignore audit:**

- [ ]  Verify `.gitignore` excludes all instance state. Add any missing entries:
    - `data/db/` (all SQLite DBs)
    - `data/redis/` (Redis RDB)
    - `agents/KAPVs/*/memory/` (agent memory files)
    - `agents/SESSIONS_LOG.md` (per-instance session history)
    - `agents/bays/` (per-agent bay state DBs)
    - `run/` (sockets, PIDs)
    - `logs/` (service logs)
    - `cache/` (ephemeral session briefs)
    - `backups/` (DB snapshots)
- [ ]  Ensure `agents/KAPVs/*/identity/` structure (non-memory vault files) IS tracked — identities ship with the distro, memory does not

**Crew manifest split:**

- [ ]  `agents/CREW.md` becomes a **template** showing the expected roles and slots, not a specific deployment roster
- [ ]  Create `agents/crews/` directory with per-Citadel manifests: `agents/crews/jupiter.md`, `agents/crews/io.md`
- [ ]  Add `agents/crews/*.md` to `.gitignore` (each Citadel maintains its own crew file locally)
- [ ]  Ship `agents/crews/TEMPLATE.md` (tracked) as the starting point for new Citadels

**SYSTEM_[MAP.md](http://MAP.md) update:**

- [ ]  Add a "Distribution vs. Instance" section at the top of SYSTEM_[MAP.md](http://MAP.md) explaining the three layers (Code, Docs & Defaults, Instance State)
- [ ]  Mark which directories are instance-local vs. distribution-tracked

<aside>
🏰

**Why this is Task 0:** If Tyr rewrites [CREW.md](http://CREW.md) (Task 2) before this split happens, the rewrite will be specific to Jupiter and will conflict when Io pulls. Do the split first, then all subsequent doc fixes land cleanly in the distribution layer.

</aside>

---

## Sprint 0B — Critical Doc Fixes (Do Second)

*These fix actively misleading information. No code changes. Pure doc edits.*

### Task 1: Rewrite `agents/CITADEL.md`

- [ ]  Update status to "Phase 4 — Dynamic Tools & Containerized Sandboxes"
- [ ]  Replace old 5-phase build sequence with summary of the new 10-phase plan (Phases 0-3 COMPLETE, 4 ACTIVE, 5-10 planned)
- [ ]  Point to the Notion Master Dev Plan as the canonical roadmap source
- [ ]  Remove reference to `tyr_plan_review.md` and GitHub Projects board #6
- [ ]  Update architecture section to reflect current tri-tier model (Citadel/CASS/koad-agent)

### Task 2: Rewrite Crew Manifests

**After Task 0 creates the crew split**, update the template and Jupiter manifest:

`agents/crews/TEMPLATE.md` (tracked — ships with distro):

- [ ]  Generic role table with slots: Captain, Officers, Engineers, Crew, Contractors
- [ ]  Deployment protocols section (boot commands, sovereignty rules, communication paths)
- [ ]  Reference to `config/identities/` as identity source

`agents/crews/jupiter.md` (gitignored — Jupiter-specific):

- [ ]  Tyr as Captain
- [ ]  Clyde as Officer (Claude Code) with correct scope
- [ ]  Cid as Engineer (Codex)
- [ ]  Scribe as Crew (Gemini Flash-Lite)
- [ ]  Helm as Officer (Build Engineer)
- [ ]  Noti as Specialist (Notion AI, remote MCP)
- [ ]  Communication: `agents/inbox/` and `koad:stream:*`

`agents/CREW.md` (tracked — becomes a pointer):

- [ ]  Replace current content with a short note: "This file is the distribution template. Your Citadel's active crew manifest is in `agents/crews/<citadel>.md`. See `agents/crews/TEMPLATE.md` to create one."

### Task 3: Fix Root `AGENTS.md`

- [ ]  Fix duplicate §Ⅱ section numbering (two sections labeled "Ⅱ")
- [ ]  Update Personnel table (§Ⅶ): Remove Pic, add Tyr as Captain, fix Clyde rank to Officer, fix Helm description
- [ ]  Update date to 2026-03-27
- [ ]  Verify onboarding sequence still references correct files
- [ ]  Update "Condition" to reflect current state accurately

### Task 4: Rewrite `crates/AGENTS.md`

- [ ]  List all 11 workspace crates with current purpose and status:

| **Crate** | **Status** | **Purpose** |
| --- | --- | --- |
| `koad-core` | Complete | Shared primitives, config, session management, logging |
| `koad-proto` | Complete | gRPC bindings (tonic, auto-generated from proto/) |
| `koad-citadel` | Complete | Citadel gRPC service (:50051) — sessions, bays, signal corps, auth, state |
| `koad-cass` | Complete | CASS gRPC service (:50052) — memory, TCH, EoW pipeline |
| `koad-cli` | Complete | `koad`  • `koad-agent` binaries, all CLI subcommands |
| `koad-intelligence` | Complete | InferenceRouter, local Ollama distillation |
| `koad-codegraph` | Complete | AST-based symbol indexing (tree-sitter) |
| `koad-board` | Complete | Updates board service |
| `koad-bridge-notion` | Complete | Notion MCP bridge (Noti remote agent) |
| `koad-sandbox` | Phase 4 | Config-driven sandbox; containerized execution is active scope |
| `koad-plugins` | Phase 4 | WASM plugin runtime (wasmtime); dynamic loading is active scope |
- [ ]  Add guidelines section (RUST_[CANON.md](http://CANON.md), tonic/tokio, no legacy Spine code)

---

## Sprint 0C — Archive & Consolidate (Do Third)

*Move stale artifacts out of the active tree. Reduce noise.*

### Task 5: Archive Stale Planning Docs

- [ ]  Move `new_world/DRAFT_PLAN_3.md` → `new_world/archived/DRAFT_PLAN_3.plan.old.md`
- [ ]  Move `new_world/DEVELOPMENT_PLAN_3.plan.md` → `new_world/archived/DEVELOPMENT_PLAN_3.plan.old.md`
- [ ]  Add a `new_world/README.md` with a one-liner pointing to the Notion Master Dev Plan as the canonical source
- [ ]  Move `STARTUP_CHECKLIST.md` → `docs/rebuild/archived/STARTUP_CHECKLIST.md` (or delete if confirmed complete)
- [ ]  Mark `docs/rebuild/DIRECTORY_CLEANUP.md` as COMPLETE (or archive)

### Task 6: Consolidate Planning Locations

- [ ]  `plans/` — Audit each file (boot-telemetry, fs-mcp-integration, issue-162-admin-override, tch-context-packet). For each: if completed, archive to `plans/archived/`. If still relevant, keep.
- [ ]  `docs/requests/` — Move all feature/skill request files to `docs/requests/archived/`. These ideas now live in the Notion Icebox, not as repo files.
- [ ]  `agents/quests/side_quests/spine_eradication` — Mark as COMPLETE. Archive or add a completion note.

### Task 7: Rotate SESSIONS_[LOG.md](http://LOG.md)

- [ ]  Move all sessions except the most recent 2-3 into `agents/sessions_archive/SESSIONS_LOG_pre_20260327.md`
- [ ]  Keep `agents/SESSIONS_LOG.md` as a rolling recent-sessions file
- [ ]  Add a header comment: "Keep only the last 3 sessions here. Archive older sessions to sessions_archive/."

---

## Sprint 0D — Structural Improvements (Do Fourth)

*Add missing infrastructure for the parallel execution model.*

### Task 8: Create Per-Crate [AGENTS.md](http://AGENTS.md) Files

For each of the 3 most-touched crates, create a `crates/<crate>/AGENTS.md` with:

- Purpose (1-2 sentences)
- Current status and phase
- Source file map (key files with one-line descriptions)
- Public API surface (main structs, traits, services)
- DO NOT TOUCH areas (if any)
- Dependencies (which other crates it imports)

**Priority order:**

- [ ]  `crates/koad-citadel/AGENTS.md` (most complex crate — admin/, auth/, services/, signal_corps/, state/, workspace/)
- [ ]  `crates/koad-cass/AGENTS.md` (services/, storage/)
- [ ]  `crates/koad-cli/AGENTS.md` (builds both `koad` and `koad-agent` binaries)

### Task 9: Create Task Manifest Template

- [ ]  Create `templates/TASK_MANIFEST.md` with this structure:

```
# Task: [Title]
## Objective
[One paragraph]
## Acceptance Criteria
- [ ] ...
## Relevant Files (with line ranges)
- crates/koad-foo/src/bar.rs (L45-L120)
## DO NOT TOUCH
- crates/koad-core/ (shared, other agents may be working here)
## Token Budget
[S/M/L] — ~[8K/25K/60K] tokens
## Agent Assignment
[Clyde/Cid/Tyr]
```

### Task 10: Create Worktree Conventions Doc

- [ ]  Create `docs/protocols/WORKTREE_CONVENTIONS.md` covering:
- Branch naming: `feat/<scope>`, `task/<scope>`, `fix/<scope>`
- Worktree naming: `koad-clyde`, `koad-tyr`, `koad-cid`
- Merge rules: all merges to `nightly` via PR, Ian approval required
- Conflict avoidance: one crate per worktree, task manifests enforce file boundaries
- Cleanup: remove worktrees after merge

---

## Sprint 0E — Config Hygiene (Do Last)

### Task 11: kernel.toml Cleanup

- [ ]  Remove or comment out the `[watchdog]` section (koad-watchdog is deprecated)
- [ ]  Add a comment to `citadel_socket` noting it's reserved for future admin UDS support (not currently created by kernel startup)
- [ ]  Verify all paths are consistent with SYSTEM_[MAP.md](http://MAP.md)

### Task 12: Update Thin [AGENTS.md](http://AGENTS.md) Files

- [ ]  `docs/AGENTS.md` — List all subdirectories and their contents accurately
- [ ]  `scripts/AGENTS.md` — List all 7 scripts with one-line purposes

### Task 13: Clean `agents/.claude/`

- [ ]  Update [README.md](http://README.md) — Clyde is now Officer, not Contractor. Focus is Phase 4+, not Phase 1.
- [ ]  Update `context.md` if it references Phase 1 scope
- [ ]  Archive old `tasks/` and `memory/` entries if stale

---

# Verification Checklist

After all tasks are complete, run these checks:

- [ ]  `grep -r "Phase 1" agents/ docs/ AGENTS.md` — Should return zero hits outside of historical/archived files
- [ ]  `grep -ri "pic" agents/CREW.md agents/CITADEL.md AGENTS.md` — Should return zero hits (Pic is deprecated)
- [ ]  `grep -ri "vigil" agents/CREW.md` — Should return zero hits in active crew
- [ ]  `grep -ri "spine" agents/ AGENTS.md` — Should return zero hits outside archived quest and legacy/
- [ ]  Every crate in `Cargo.toml [workspace.members]` appears in `crates/AGENTS.md`
- [ ]  `SYSTEM_MAP.md` paths still match actual directory structure (spot check 5 paths)
- [ ]  `agents/SESSIONS_LOG.md` is ≤ 3 sessions

---

<aside>
⏱️

**Estimated Effort:** Sprints 0A-0B are pure doc edits — low token cost, high impact. Sprint 0C requires reading crate source to write accurate [AGENTS.md](http://AGENTS.md) files — moderate token cost. Sprint 0D is small cleanup. Total: ~4-6 agent sessions if done with focused task manifests.

</aside>

<aside>
🏴

**Devil's Advocate:** The biggest risk is Tyr spending tokens *reading* stale docs to figure out what's stale — which is exactly the problem we're solving. To avoid this, Tyr should use THIS page as the task brief and NOT re-audit the files listed here. The audit is done. Trust it and execute.

</aside>