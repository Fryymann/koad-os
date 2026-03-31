**Status:** Active — v3.3 Planning 

**Owner:** Ian (Dood) + Tyr (Captain, Jupiter) 

**Coding Agents:** Gemini CLI · Claude Code · Codex 

**Date:** 2026-03-27 

**Source of Truth:** This page supersedes `new_world/DRAFT_PLAN_3.md` (which covers through Phase 4 only).

<aside>
🏰

**Multi-Citadel Architecture:** KoadOS is a deployable platform, not a single workspace. The repo (`Fryymann/koad-os`) is a **distribution artifact** — like a distro. Each developer clones it, bootstraps a local Citadel, configures their own crew, and operates independently. Citadels are **sovereign by default, optionally networked.** Ian's fleet: Jupiter (desktop, primary) and Io (laptop, secondary). Other users (e.g., Ian's dad) run fully independent Citadels with their own crews, never connected to Ian's network unless explicitly federated.

</aside>

### Distribution vs. Instance

The repo ships **three layers** with different lifecycle rules:

| **Layer** | **Git Status** | **What It Contains** |
| --- | --- | --- |
| **Code** | Tracked, shared | All crates, proto/, scripts, tests, install/ — identical on every Citadel |
| **Docs & Defaults** | Tracked, shared | [AGENTS.md](http://AGENTS.md), SYSTEM_[MAP.md](http://MAP.md), docs/, `config/defaults/` (template configs) — ship with the distro |
| **Instance State** | Gitignored, local | agents/KAPVs/*/memory/, data/db/, agents/SESSIONS_[LOG.md](http://LOG.md), config/kernel.toml (generated from defaults at bootstrap), Redis data, bay DBs — unique per Citadel |

**Rule:** If two Citadel operators pull from the same repo and get a merge conflict, something is in the wrong layer.

---

# Part 1 — Agent Team & Efficiency

*The bottleneck isn't code — it's token burn and serial execution. Fix this first and everything else accelerates.*

## 1.1 Citadel Fleet (Ian's Deployment)

| **Citadel** | **Machine** | **Captain** | **Role** |
| --- | --- | --- | --- |
| **Jupiter** | Desktop | Tyr | Primary Citadel — main development, full services (Citadel gRPC, CASS, Redis), all coding agents |
| **Io** | Laptop | Pic | Secondary Citadel — mobile development, pulls code from Jupiter via git, own crew and local memory |

Both Citadels share the same codebase (same repo, `nightly` branch). They differ in crew composition, local agent memory, and machine-specific config. Code flows Jupiter → Io via git pull. Io agents can push feature branches back for Jupiter-side merge.

## 1.2 Agent Roles & Assignments (Jupiter Crew)

| **Agent** | **Runtime** | **Best Used For** | **Typical Task Shape** |
| --- | --- | --- | --- |
| **Tyr** | Gemini CLI | Strategic oversight, plan verification, cross-crate review, architecture decisions | Read-heavy, multi-file, advisory |
| **Clyde** | Claude Code | Complex implementation — new modules, gRPC services, multi-file refactors | Write-heavy, needs deep context |
| **Cid** | Codex | Scoped tasks — tests, boilerplate, single-function impl, CI fixes | Narrow scope, high throughput |
| **Scribe** | Gemini Flash-Lite | Context distillation, map maintenance, [AGENTS.md](http://AGENTS.md) updates | Cheap doc generation |
| **Ian + Noti** | Notion AI | Planning, architecture review, spec writing, task dispatch, idea capture | Strategic / coordination |

## 1.3 Parallel Execution Model

The goal: **3 coding agents running simultaneously** on non-overlapping crates/features.

<aside>
🔑

**Git Worktrees** are the key enabler. Each agent gets its own worktree branching from `nightly` so they can work on different crates without merge conflicts.

</aside>

**Terminal Layout:**

- **Terminal 1 — Clyde (Claude Code):** Complex feature branch (e.g., `feat/minion-hangar`)
- **Terminal 2 — Tyr (Gemini CLI):** Review + architecture branch or separate crate work
- **Terminal 3 — Cid (Codex):** Test/boilerplate branch (e.g., `task/citadel-integration-tests`)
- **Ian:** Dispatch, review PRs, merge to `nightly`, capture ideas in Notion (not in the terminal)

**Worktree Setup (one-time):**

```bash
# From ~/.koad-os (main worktree)
git worktree add ../koad-clyde nightly   # Claude Code
git worktree add ../koad-tyr nightly     # Gemini CLI
git worktree add ../koad-cid nightly     # Codex
```

**Rule:** Each worktree works on a different crate or a different file set within the same crate. The task manifest (see below) enforces this.

## 1.4 Token Efficiency Playbook

Every technique here targets the same goal: **give each agent exactly what it needs and nothing more.**

### Tier 1 — Repo Infrastructure (Zero token cost, massive savings)

- [ ]  **Per-crate [AGENTS.md](http://AGENTS.md) files** — Each crate already has (or needs) an `AGENTS.md` that describes the crate's purpose, public API surface, file map, and current implementation status. Agents read this *instead of* scanning the whole crate.
- [ ]  **Task manifests** — A `.task.md` file placed in the agent's worktree before launch. Contains: objective, acceptance criteria, relevant files (with line ranges), DO NOT TOUCH list. The agent reads this first and works within scope.
- [ ]  **SYSTEM_[MAP.md](http://MAP.md) maintenance** — Already exists and is solid. Keep it current after every merge.

### Tier 2 — Context Packet Generation (Small cost, large savings)

- [ ]  **Pre-computed context files (`KOAD_CONTEXT_FILE`)** — Generate a distilled context packet per-task using Scribe (flash-lite, cheap). Includes: crate API map, relevant type signatures, recent commit summaries for touched files.
- [ ]  **Crate API Maps** — Auto-generate or manually maintain a `API_MAP.md` per crate listing public structs, traits, functions, and their file locations. Agents use this instead of `ls -R` or reading full files.

### Tier 3 — Agent Discipline (Behavioral)

- [ ]  **Surgical reads only** — `read_file` with `start_line`/`end_line`. Full-file reads are a Tier 1 Performance Violation (already in [AGENTS.md](http://AGENTS.md), enforce it).
- [ ]  **No exploratory scanning** — If it's not in the task manifest or API map, the agent asks (via inbox message) rather than exploring.
- [ ]  **Single-purpose commits** — One task = one commit = one PR. No scope creep inside agent sessions.

### Tier 4 — Agent Wandering Prevention (Config-Level Enforcement)

Behavioral rules only work if the runtime enforces them. Each agent platform has config-level guardrails:

- [ ]  **Claude Code:** `.claude/settings.local.json` — restrict file access via `allowedTools`, scope to worktree.
- [ ]  **Gemini:** `.gemini/settings.json` — scope working directory to worktree root.
- [ ]  **Codex:** `.codex/config.toml` — explicit `sandbox_permissions` for worktree only.
- [ ]  **All agents:** Only wire MCP tools relevant to the current task. No ambient tool access.

### Tier 5 — Infrastructure (Build once, saves forever)

- [ ]  **`koad-agent context` command** — (This is the koad-agent MVP — see Phase 5 below.) Auto-generates the context packet from crate metadata + git log + SYSTEM_MAP. Replaces manual context prep.
- [ ]  **Token budget per task** — Set a soft limit in the task manifest. Agent should checkpoint progress and hand off if approaching limit.

<aside>
📊

**Expected Impact:** Tier 1 + 2 alone should reduce per-agent token usage by 40-60%. Tier 5 (`koad-agent context`) is the long-term force multiplier.

</aside>

## 1.5 Session Discipline Protocol

These rules keep agent sessions tight and prevent context drift:

- **One task per session.** Don't let sessions drift into "while you're at it" territory.
- **Kill and re-launch** rather than extending a session that's lost focus.
- **Save session notes** at end of each session — becomes warm-start context for next session on that worktree.
- **Task manifest is the contract** — if the agent proposes work outside the manifest, it's a deviation. Redirect or kill.

## 1.6 Task Manifest Template

Standardized format at `templates/TASK_MANIFEST.md`. Tyr can fill these out when dispatching work:

```
# Task: [Title]
## Scope
- File: [target file path]
- Proto: [proto file if applicable]
- Tests: [test file path]
## Context files to read
- [file 1]
- [file 2]
## Do NOT read
- Any archived crates
- Any docs/ files (context already provided above)
## Acceptance criteria
- [ ] `cargo build` passes
- [ ] `cargo test [module]` passes
- [ ] [specific functional requirement]
## Constraints
- Every public function accepts TraceContext
- Use anyhow::Result for error handling
- Redis keys follow koad:[namespace]:{id} pattern
```

## 1.7 Idea Discipline Protocol

<aside>
🧊

**The Icebox Rule:** Ideas that arrive during active development go into the Icebox, not into the current sprint. No exceptions.

</aside>

**Process:**

1. Idea arrives → Ian captures it in Notion under **KoadOS Icebox** (a simple page or DB entry — title + 2-sentence description + which phase it likely belongs to)
2. During sprint planning (between phases), Ian + Tyr review the Icebox and promote items that fit the next phase
3. Everything else stays frozen

**Why this matters:** The brainstorm page (Citadel Refactor) has ~40 ideas spanning 8+ phases. Trying to hold all of them in active context is what's causing development drift. The Icebox is a pressure valve.

---

# Part 2 — Citadel Rebuild: Revised Phase Sequence

*Resequenced from the original DRAFT_PLAN_3 to account for: (a) minion swarm architecture, (b) koad-agent as a force multiplier, (c) what's already built.*

## Current State (as of 2026-03-27)

| **Component** | **Status** | **What Exists in Repo** |
| --- | --- | --- |
| `koad-citadel` | Built | gRPC kernel, session bays, signal corps, auth, state, workspace modules, Zero-Trust interceptor |
| `koad-cass` | Built | Memory services, TCH, EoW pipeline, storage layer |
| `koad-core` | Built | Shared primitives, config, session, logging |
| `koad-proto` | Built | gRPC bindings — citadel.proto, cass.proto, skill.proto |
| `koad-cli` | Built | `koad` and `koad-agent` binaries, CLI subcommands |
| `koad-intelligence` | Built | InferenceRouter, local Ollama distillation |
| `koad-sandbox` | Phase 4 | Config-driven sandbox exists; containerized execution is Phase 4 scope |
| `koad-codegraph` | Built | AST-based symbol indexing (tree-sitter) |
| `koad-plugins` | Phase 4 | WASM plugin runtime (wasmtime) — dynamic loading is Phase 4 scope |
| `koad-board` | Built | Updates board service |
| `koad-bridge-notion` | Built | Notion MCP bridge |
| Minion Swarm / Hangar | Not started | Design only (Notion brainstorm). No code, no proto, no crate. |

### Unresolved 🔴 Design Decisions

These open questions span multiple phases and should be resolved before the relevant phase begins:

- **Dark Mode local persistence format** — needs a concrete schema decision (structured `.md` with TOML frontmatter is the current proposal). *Blocks Phase 7 CASS work.*
- **Data migration protocol schema** — how legacy state (Redis dump, SQLite export) maps to new CASS memory layers. *Blocks Phase 8 `koad system migrate-v5`.*
- **Tier 1 Zero-Trust enforcement approach** — specific enforcement strategy at gRPC layer. *Should be audited in Phase 4.*
- **EndOfWatch schema** — structured format for session close-out records. *Blocks Phase 8 `koad-agent eow`.*

## Revised Phase Map

<aside>
⚠️

**Phases 0–3 are COMPLETE and locked.** Do not reopen them. The sequence below starts from Phase 4 (current) and extends through the full vision including minion swarm.

</aside>

### Phase 4 — Dynamic Tools & Containerized Sandboxes (CURRENT — In Progress)

**Goal:** Externalize tool execution. Make agents' tool use pluggable and isolated.

- [ ]  MCP Tool Registry in CASS — register/invoke MCP tools via gRPC
- [ ]  Filesystem MCP Server integration
- [ ]  Docker/Podman sandbox for arbitrary code execution
- [ ]  Dynamic library loading for custom tool implementations

**Parallelizable:** MCP Registry (Clyde) ∥ Sandbox containerization (Cid) ∥ Integration tests (Tyr review)

**Gate:** MCP Registry + Sandbox passing integration tests → Ian approval.

---

### Phase 5 — koad-agent MVP (Context Generation Engine) (Force Multiplier)

**Goal:** Make `koad-agent` the tool that makes all other agents faster. Context generation, not Citadel dependency.

<aside>
💡

**Why this is resequenced to come early:** `koad-agent context` is the single highest-ROI feature for reducing token burn across all agents. It runs in DEGRADED mode without a live Citadel — it reads config files, git state, and crate metadata directly. Every phase after this benefits from it.

</aside>

**Scope (koad-cli crate — `koad-agent` binary):**

- [ ]  `koad-agent context <crate>` — Generate a context packet from: crate [AGENTS.md](http://AGENTS.md) + API_[MAP.md](http://MAP.md) + recent git log + SYSTEM_[MAP.md](http://MAP.md) extract. Output: a single `.context.md` file an agent can read as its first action.
- [ ]  `koad-agent boot <identity>` — Load identity TOML from `config/identities/`, generate CLI config for the target runtime (Claude Code [CLAUDE.md](http://CLAUDE.md), Gemini .gemini, Codex config), set environment variables.
- [ ]  `koad-agent task <manifest>` — Validate a task manifest against crate boundaries, check for file overlap with other active tasks (worktree-aware).
- [ ]  `koad-agent inspect` — Show current shell state (KoadOS env vars, active identity, worktree info).
- [ ]  `koad-agent clear` — Unset all KoadOS environment variables (clean shell reset).
- [ ]  **Preflight validator** — Check required env vars, report READY / NOT READY before agent launch.
- [ ]  **Env var exporter** — `eval $(koad-agent --ghost sky --export)` pattern for shell integration.
- [ ]  Degraded mode: all of the above works with **zero running services** (no Citadel, no CASS, no Redis). Just filesystem reads.

**Parallelizable:** `context` subcommand (Clyde) ∥ `boot` subcommand (Cid) ∥ Identity TOML cleanup (Tyr)

**Gate:** `koad-agent context koad-citadel` produces a usable context packet that measurably reduces token usage in a test session → Ian approval.

---

### Phase 6 — Canon Lock (Documentation Distillation) (Stabilization)

**Goal:** Freeze the architecture. Distill the Notion brainstorm into repo-canonical docs. After this phase, Notion is for *ideas* and the repo is for *truth*.

- [ ]  Distill Citadel Refactor brainstorm (Notion [Citadel Refactor — Brainstorm & Research](https://www.notion.so/Citadel-Refactor-Brainstorm-Research-ff598ede2a0048998e3262119fd13cef?pvs=21)) into `docs/rebuild/ARCHITECTURE.md` — canonical architecture reference
- [ ]  Write `docs/rebuild/MINION_SWARM_SPEC.md` — extracted from Notion Minion Architecture page, frozen as implementation spec
- [ ]  Write `docs/CONVENTIONS.md` — coding standards, TraceContext requirement, error handling, Redis key patterns
- [ ]  Write `docs/PROTO_GUIDE.md` — how to read and extend `citadel.proto`
- [ ]  Update all crate [AGENTS.md](http://AGENTS.md) files to reflect current state (post-Phase 4/5)
- [ ]  Update SYSTEM_[MAP.md](http://MAP.md) with any new paths/crates
- [ ]  Archive `new_world/DRAFT_PLAN_3.md` → `new_world/archived/` and replace with a pointer to this Notion plan page

**Parallelizable:** Scribe (cheap) handles doc generation ∥ Tyr reviews for accuracy ∥ Cid updates SYSTEM_MAP

**Gate:** All crate [AGENTS.md](http://AGENTS.md) files pass a completeness check (purpose, API surface, file map, status) → Ian approval.

---

### Phase 7 — CASS Expansion (Memory Stack + MCP Server) (Core Cognition)

**Goal:** Full cognitive support. CASS becomes the memory backbone that agents query at boot and during work.

**Memory Layer Architecture:**

The four-layer memory stack that CASS implements:

- **L1:** Redis Stack upgrade + vector search config (hot working memory)
- **L2:** SQLite WAL episodic store (schema, write/read, 90-day retention cron)
- **L3:** Qdrant deployment + per-agent collection provisioning (semantic long-term)
- **L4:** SQLite procedural memory (separate schema, no decay — skills and procedures)
- **Memory Insurance:** WORM ledger (`ledger.jsonl`), automatic vault snapshots

**Implementation Tasks:**

- [ ]  **FactCard CRUD** — Full create/read/update/delete for structured memory entries via gRPC
- [ ]  **CASS MCP Server** — Expose CASS memory and context services as MCP tools so external agents (Claude Code, Gemini) can query them natively. Tool inventory:
    - `koad_intel_commit` — commit new intelligence/facts
    - `koad_intel_query` — query existing intelligence
    - `koad_memory_hydrate` — hydrate agent context from memory
    - `koad_status` — report system/session status
    - `koad_session_save` — persist session state
    - `koad_session_restore` — restore session from saved state
    - `koad_context_archive` — archive context for later retrieval
    - `koad_map_add` — add entries to agent knowledge maps
- [ ]  **Three-Tier Context Hydration (TCH)** — Implement the full pipeline: Boot Context → Working Set → Deep Recall, with token-budget-aware truncation
- [ ]  **Dark Mode reconciliation** — Offline-to-online memory sync (agent works offline, reconnects, CASS merges)
- [ ]  **Brain Drain Protocol** — Clean shutdown sequence: flush L1 → L2, commit pending intel, release lease
- [ ]  **Post-compaction recovery hook** — Recovery path after memory compaction events

**Parallelizable:** MCP Server (Clyde) ∥ FactCard CRUD (Cid) ∥ TCH pipeline design doc (Tyr)

**Gate:** An agent can boot, call `koad-agent context`, then query CASS MCP for relevant memories — full loop works → Ian approval.

---

### Phase 8 — koad-agent Full (CASS Integration) (Agent Maturity)

**Goal:** `koad-agent` connects to live CASS. Context packets now include memory hydration from CASS, not just filesystem.

- [ ]  `koad-agent context` now queries CASS for relevant FactCards and injects them into the context packet
- [ ]  `koad-agent eow` — Triggers End-of-Watch pipeline: session summary → CASS storage → XP ledger update (depends on EndOfWatch schema resolution)
- [ ]  `koad-agent status` — Reports current session state, active tasks, token usage estimate
- [ ]  `koad system migrate-v5` — CLI command to import legacy knowledge (Redis dump, SQLite export) into new CASS memory stack (depends on data migration protocol schema resolution)
- [ ]  **KoadOS Core Contract v2.4** — Version bump and update to Operational Infrastructure section reflecting CASS integration
- [ ]  Graceful degradation: if CASS is down, falls back to Phase 5 behavior (filesystem-only)

**Gate:** Full boot → work → EoW cycle completes with CASS integration → Ian approval.

---

### Phase 9 — Minion Swarm Hangar (Architecture Extension)

**Goal:** Implement the Micro-Swarm Hangar from the Minion Architecture design. Citadel can spawn, monitor, and collect disposable minion agents.

<aside>
🐝

**Design hooks from Phase 4 (sandbox) and Phase 7 (CASS MCP) are prerequisites.** The Hangar orchestrates minions that run in sandboxes and report through CASS.

</aside>

- [ ]  **Hangar Manager** in `koad-citadel` — Lifecycle management for minion instances (spawn, heartbeat, collect, terminate)
- [ ]  **Minion proto** — New `minion.proto` defining spawn requests, status reports, result collection
- [ ]  **Model Router integration** — Tie into `koad-intelligence` InferenceRouter for T1-T4 model tiering per minion task
- [ ]  **Task Delegation Protocol** — Structured format for breaking a parent task into minion sub-tasks with dependency graph
- [ ]  **Output Evaluation** — Quality gate before minion output is merged (configurable: auto-merge for T1, review-required for T3+)
- [ ]  **VRAM Arbiter** — Resource-aware scheduling (GPU/memory limits per concurrent minion)

**Parallelizable:** Hangar Manager (Clyde) ∥ Minion proto (Cid) ∥ Model Router wiring (Tyr)

**Gate:** Citadel can spawn 3 concurrent minions on different tasks, collect results, and report to CASS → Ian approval.

---

### Phase 10 — Advanced Features (Post-Core) (Future)

**Goal:** The vision features. Only start these after Phase 9 is stable.

**Agent-to-Agent Signaling (A2A-S):**

- [ ]  **Ghost Mailbox** — Redis keys for per-agent message queues
- [ ]  **`koad signal` CLI** — Send signals between agents from command line
- [ ]  **Boot-time signal delivery** — Agents check their mailbox on boot and process queued messages
- [ ]  **Real-time pub/sub via Signal Corps** — Full inter-agent event streaming

**Citadel Federation:**

- [ ]  **Federation Protocol** — Optional cross-Citadel knowledge sync, fleet-level task coordination, shared fact replication. Sovereign by default — federation is opt-in per Citadel pair.

**Growth & Intelligence:**

- [ ]  **Growth System** — XP ledger → level progression → capability unlocks
- [ ]  **Mem0 advanced hooks** — Interaction loop, semantic cache, contradiction detection
- [ ]  **Neo4j Knowledge Graph** — Replace/augment SQLite memory with graph-based knowledge store

**Infrastructure & Interface:**

- [ ]  **KoadStream Integration** — Live event stream for real-time monitoring dashboard
- [ ]  **koad-bridge-notion enhancements** — Two-way Notion sync for specs, task status, memory
- [ ]  **Plugin Marketplace** — Community/internal WASM plugin distribution
- [ ]  **TUI / Web Deck (v6+)** — Interface layer for KoadOS monitoring and control
- [ ]  **Crew Briefings generator** — Auto-generated briefing docs for crew coordination

**Portability:**

- [ ]  **Clean clone + `koad install`** — Streamlined setup for external devs (portability and shareability)

*These are Icebox items until Phase 9 gate passes.*

---

# Part 3 — Execution Rhythm

## Sprint Structure

Each phase = 1 sprint. Sprints have a fixed structure:

1. **Sprint Planning (Ian + Tyr, 30 min)** — Review phase goals, break into task manifests, assign to agents, check Icebox for promotions
2. **Parallel Execution (Clyde + Cid + Tyr)** — 3 agents working simultaneously on non-overlapping tasks using worktrees
3. **Daily Merge Window (Ian)** — Review PRs, merge to `nightly`, update SYSTEM_MAP if needed
4. **Gate Check (Ian + Tyr)** — Phase acceptance criteria met? Ship it or iterate.
5. **Phase Retro (Ian + Noti)** — What worked, what burned tokens, update efficiency playbook

## Daily Execution Pattern

**Morning (15 min):**

1. Check what's in progress across worktrees
2. Review any completed Codex background tasks
3. Pick the highest-priority unfinished task from the current phase
4. Write or update the task manifest for today's focus agent

**Work session:**

1. Launch agent in the scoped worktree with its task manifest
2. Monitor progress — redirect if agent wanders
3. If one agent completes, dispatch next task or switch to a different worktree
4. If tokens run low on one provider, switch to another agent/provider

**End of day (10 min):**

1. Save session notes for each active worktree
2. Update phase checklist (check off completed items)
3. Ice any new ideas that came up
4. Note what the next morning should start on

## Immediate Next Actions (This Week)

- [x]  Create this plan (you're reading it)
- [ ]  **Set up 3 Git worktrees** — `koad-clyde`, `koad-tyr`, `koad-cid`
- [ ]  **Write task manifests for Phase 4 remaining items** — One `.task.md` per agent assignment
- [ ]  **Create the KoadOS Icebox page** in Notion — Move all non-Phase-4 ideas there
- [ ]  **Update per-crate [AGENTS.md](http://AGENTS.md) files** — At minimum: `koad-citadel`, `koad-cass`, `koad-cli` (these are the most-touched crates)
- [ ]  **Generate API_[MAP.md](http://MAP.md)** for `koad-citadel` and `koad-cass` — Can be done by Scribe (cheap)

---

# Part 4 — Getting Tyr On Track

*After Ian is oriented with this plan, the next step is repo docs so Tyr can operate independently.*

## What Tyr Needs in the Repo

1. **Updated `AGENTS.md`** (root) — Reflect the new phase sequence and agent team structure
2. **Updated `agents/CITADEL.md`** — Current mission brief pointing to this plan as source of truth
3. **Updated `agents/CREW.md`** — Add Cid (Codex) role, update role descriptions to match table above
4. **Task manifest template** at `templates/TASK_MANIFEST.md` — Standardized format Tyr can fill out when dispatching work
5. **Worktree conventions doc** at `docs/protocols/WORKTREE_CONVENTIONS.md` — Branch naming, merge rules, conflict avoidance

*We'll tackle these as the next work item after this plan is reviewed.*

---

# Reference Links

- [Vigil — CLAUDE.md](https://www.notion.so/Vigil-CLAUDE-md-321fe8ecae8f806780abd3c65016de36?pvs=21) — Codebase audit and hardcoded values inventory
- [KoadOS Mission](https://www.notion.so/KoadOS-Mission-dc4e13f70ab5497395e055c4eaf000fa?pvs=21) — The vision this all serves
- [Citadel Refactor — Brainstorm & Research](https://www.notion.so/Citadel-Refactor-Brainstorm-Research-ff598ede2a0048998e3262119fd13cef?pvs=21) — Original brainstorm (Notion is for ideas, repo is for truth post-Phase 6)
- [KoadOS Minion Architecture](https://www.notion.so/KoadOS-Minion-Architecture-095b5309831d41588ec4fe6a97a38ac2?pvs=21) — Minion swarm design reference

---

<aside>
🏴

**Devil's Advocate Note:** This plan assumes Phases 0-3 are genuinely complete and stable. If there's hidden tech debt in `koad-citadel` or `koad-cass` (e.g., incomplete gRPC methods, missing error handling, placeholder implementations), that will surface during Phase 4 and should be addressed as hotfixes, not as a reason to reopen earlier phases. Tyr should audit this during the first sprint planning.

</aside>

[Workspace Prep Plan — Tyr Task Brief](https://www.notion.so/Workspace-Prep-Plan-Tyr-Task-Brief-f04f12ebe9e3405cb9407327488415a2?pvs=21)