# KoadOS Rebuild — Development Plan
*Prepared: 2026-03-12 | Branch: claude/gifted-rubin | Source: DRAFT_PLAN_2.md*

---

## Development Team

| Role | Who | Tooling | Responsibilities |
|------|-----|---------|-----------------|
| **Admiral** | Ian (Dood) | — | Final authority. All phase gates, merges, and pushes require Ian approval. |
| **Captain / Primary Agent** | Tyr | Gemini CLI | KoadOS Captain, domain owner of The Citadel. Primary collaborator and reviewer. Joins as active contributor after Phase 3 (CASS online). |
| **Contractor Agent** | Claude Code (me) | Claude Code CLI | Implements work in isolated git worktrees. Submits PRs. Does not push or merge without Ian approval. |
| **Officers** | Sky, Vigil | Gemini CLI | Join as contributors after Phase 3. |

**Key implications for how we work:**
- Tyr's identity TOML and Personal Bay are the **first priority** in Phase 2 — he's the primary consumer.
- **Gemini CLI is the primary target** for `koad-agent` and CASS MCP server integration.
- I work in worktrees (current: `claude/gifted-rubin`), submit PRs, and wait for Tyr/Ian review.
- Tyr's reviews are authoritative on Citadel/CASS design decisions, subject to Ian's final approval.

---

## Primary Build Goal

> **Officer Agent Support First.** Get the Citadel to a stable baseline capable of supporting Tyr, Sky, and Vigil. Then bring CASS online. After that, all three agents contribute to the continued development of the rest of the Citadel and KoadOS.

**Top-level build sequence:**

```
1. Citadel (MVP)  →  Tyr can connect, has his Personal Bay, and can work
2. CASS           →  Tyr has memory, MCP tools, and cognitive support
3. Full KoadOS    →  Tyr (+ Sky, Vigil) contribute as active crew
```

This is not "build everything then launch." It's: build the minimum Citadel that gets Tyr online, then Tyr helps complete the rest.

---

## Architecture Summary

| System | Role | Owner |
|--------|------|-------|
| **The Citadel** | Central OS — session brokering, Personal Bays, shared state, watchdog | Tyr |
| **CASS** | Cognitive support — memory stack, MCP tools, session hydration | CASS (Citadel sub-system) |
| **`koad-agent`** | Shell prep tool — body/ghost model, context file generation | CLI tool |
| **Agent-Level Cognition** | Each agent's local mind — works without Citadel | Agent-side |

The Spine is retired. `koad-spine`, `koad-asm`, `proto/spine.proto` are **archived, not migrated**. New system built clean.

---

## Blockers — Resolve Before Phase 2 Code Begins

These must be answered and documented. Grouped by urgency.

### 🔴 Must-resolve before any Citadel code

**BLOCKER 1 — Dark Mode local persistence format**
- Path convention for offline saves: `<project_dir>/.koad-dark/<agent>/<session_id>.md`?
- Frontmatter format: TOML or JSON?
- CASS parser contract for reconciliation on reconnect
- Sovereignty Hierarchy: `Dood > Citadel State > Local Save` — conflicts → `.koad-conflict/`

**BLOCKER 2 — Tier 1 Zero-Trust enforcement**
- Gap: Tier 1 agents have unrestricted Redis write today
- Fix: Sanctuary Rule enforced at gRPC layer, not agent layer
- Must be in `proto/citadel.proto` design from day one

**BLOCKER 3 — Data migration protocol**
- What to extract from `koad.db` and `koad:*` Redis keys before archiving
- Import mapping: old episodic → L2 SQLite, old `koados_knowledge` → L3 Qdrant shared pool
- One-time `koad system migrate-v5` tool, not an ongoing compat layer

### 🟡 Must-resolve before Phase 3 (CASS) code

**BLOCKER 4 — EndOfWatch schema**
- Format: TOML frontmatter + Markdown body (agreed in principle)
- Required fields: `agent`, `session_id`, `task_id`, `status` (COMPLETED|STALLED|BLOCKED), `trace_id`, `timestamp`, `project`, `worked_on[]`, `decisions[]`, `blockers[]`, `next_steps[]`, `learnings[]`
- Confirm fields and enforcement point (`koad_session_save` MCP tool)

**BLOCKER 5 — `personas/` vs `config/identities/` discrepancy**
- Flight Manual §7.1 describes `personas/` + `bodies/`; CLAUDE.md shows `config/identities/`
- Decision: canonical path before any identity file work
- Recommendation: `config/identities/` wins (already exists, confirmed in KoadConfig)

**BLOCKER 6 — Env var decisions**
- Secret Manager vs `.env` precedence: Secret Manager → env var → TOML default?
- Shell profile injection: auto-append `source ~/.koad-os/.env` to `.bashrc` or confirm?
- Preflight failure: hard stop on missing required secret, or DEGRADED (CASS offline)?

**BLOCKER 7 — CLI command surface**
- All `koad spine *` retired; full new surface needed before code:
  `koad citadel *`, `koad agent *`, `koad signal *`, `koad intel *`, `koad dood *`
- Needs Ian approval as locked spec before CLI code is written

**BLOCKER 8 — Personal Bay open questions (from DRAFT_PLAN_2)**
- Bay storage backend: Redis (volatile) or SQLite/file-based (survives Citadel restart)?
- Bay provisioning trigger: auto on TOML detection at boot, or explicit `koad citadel provision-bay`?
- Credential vault: thin broker (bay holds references, Citadel resolves) or cached copy?
- Bay isolation enforcement mechanism: network layer, application layer, or both?
- Operator (Ian) bay: standard bay or special elevated access outside the model?
- Multi-session: same ghost in two shells — shared bay or sub-slots?

**BLOCKER 9 — Body/Ghost boot open questions (from DRAFT_PLAN_2)**
- Shell export mechanism: `eval $(koad-agent --ghost sky --export)` — confirm as standard
- Citadel registration: opt-in `--register`, automatic on prepare, or not at this layer?
- Context file placement: project root write, or KoadOS-managed path + symlink?
- Context file cleanup on `koad-agent clear`: delete generated files or leave?
- CASS MCP server port: fixed from `kernel.toml` or dynamically assigned?
- v1 hook set: Sanctuary Rule check (pre-tool-use) = high priority. KSRP lint, GitHub Issue enforcement = medium. Define v1 scope.

---

## Phase 0 — Knowledge Extraction & Archive
*Save everything. Archive the old. Start clean.*

**0.1 — Legacy state extraction (run NOW on live system)**
- Dump all Redis `koad:*` key-value pairs
- Export SQLite `koad.db` (episodic memories, task outcomes, `koados_knowledge`)
- Capture active session state
- Output: raw export files → raw material for `koad system migrate-v5`
- **Gate: confirm extraction complete before 0.2**

**0.2 — Vigil pre-archive cleanup**
- Remove ~30 leaked/temp/stale files (Vigil's `cleanup_sweep_3-11-2026`)
- Clean `.koad-os/legacy/` redundancy
- Archive `sdk/python` stub

**0.3 — Archive old codebase**
- Move to `archive/` branch: `crates/koad-spine/`, `crates/koad-asm/`, `proto/spine.proto`
- Remove `koad-spine` and `koad-asm` from active `Cargo.toml` workspace
- Tag clearly: `archive/spine-era`
- Remaining active crates: `koad-core`, `koad-proto`, `koad-cli`, `koad-watchdog`, `koad-board`, `koad-bridge-notion`

**Ian approval to close Phase 0.**

---

## Phase 1 — Lock Canon & Resolve Blockers
*Blueprint approved before build begins.*

**1.1 — Resolve all 🔴 blockers (1–3)**
- Decision records written for each
- Ian approval on each

**1.2 — Write `proto/citadel.proto` from scratch**
- Clean service/message/RPC names — no Spine references
- Services: `CitadelSession`, `CitadelSector`, `CitadelSignal`, `CitadelBay`
- Every mutation RPC carries `TraceContext`
- Zero-Trust baked in: auth context on every call (closes BLOCKER 2)
- Must support bidirectional streaming for logs and signals

**1.3 — TOML config bootstrap templates**
- `config/kernel.toml` — ports, socket paths, timeouts, memory insurance settings
- `config/filesystem.toml` — workspace symlinks, data dirs
- `config/registry.toml` — service and agent registration
- `config/identities/<agent>.toml` starters for sky, tyr, vigil, ian (using Ghost Config format from DRAFT_PLAN_2)
- `config/integrations/skylinks.toml` and `config/integrations/wsl.toml` stubs
- `.env.template` — secrets and runtime overrides only

**1.4 — Lock CLI command surface (BLOCKER 7)**
- Full inventory of `koad citadel *`, `koad agent *`, `koad signal *`, `koad intel *`, `koad dood *`, `koad system *`
- Ian approval as spec before any CLI code

**Ian approval to close Phase 1 and begin Phase 2.**

---

## Phase 2 — Build the Citadel MVP
*The minimum Citadel that lets agents come online.*

**New crate:** `crates/koad-citadel/`

The MVP Citadel is scoped to support agents — not all Citadel features need to ship before CASS. The goal is: agents can boot, connect to their bay, and work.

### 2.1 — Citadel Core: Session brokering & Personal Bays

**Session brokering:**
- Agent registration (replace old ASM flow)
- Heartbeat model (push-model: agent emits, Citadel listens):
  - Isolated heartbeat thread (separate from agent's work thread)
  - Interval: 10s default; DARK threshold: ≥30s (2-3× interval + consecutive miss count)
  - Monotonic sequence counter per heartbeat
  - Session-ID-aware: distinguish reboot (new `KOAD_SESSION_ID`) from continuation
  - Explicit deregistration on clean shutdown (timeout-based teardown only for crashes)
- One Body One Ghost: Redis atomic lease via `KOAD_SESSION_ID`; second boot rejected if alive

**Personal Bays (MVP scope):**

| Bay Component | MVP | Deferred |
|---------------|-----|----------|
| Dedicated connection slot | ✅ | |
| Session log | ✅ | |
| Health record | ✅ | |
| Filesystem map (assigned paths from TOML) | ✅ | |
| Filesystem map (self-registered paths) | ✅ | |
| Credential vault | ✅ (thin broker) | |
| Tool manifest | ✅ | |
| Cognition context | Deferred to CASS | ✅ |

Bay lifecycle: provision on `identities/*.toml` detection at boot (or `koad citadel provision-bay`), persist across sessions, return to STANDBY on disconnect.

**Bay storage decision:** SQLite-backed for bay state (survives Citadel restarts) + Redis for live connection status and heartbeat tracking.

### 2.2 — Citadel Core: Redis state management

- Redis as single source of truth for hot state
- Read-through cache via keyspace notifications (`keyspace@0:koad:sessions:*`) — disposable replica, NOT authoritative
- CQRS pattern: Redis = Data Plane (reads), gRPC = Control Plane (mutations)
- Rule: reads → Redis, mutations → gRPC, telemetry → Redis Streams
- Sector Locking protocol (re-implemented clean)
- Key namespace: `koad:session:*`, `koad:bay:<agent>:*`, `koad:stream:*`, `koad:mailbox:<agent>`

### 2.3 — Zero-Trust gRPC enforcement (baked in from day one)

- Every state mutation through gRPC with `TraceContext`
- CIP (Cognitive Integrity Protocol): Tier 1 restriction at gRPC layer — no unrestricted sovereign key writes
- Protected keys: `identities`, `identity_roles`, `knowledge`, `principles`, `canon_rules`
- No unauthenticated endpoints — closes the `:50051` unauthenticated gap from the old system
- **Red Team test before Phase 3:** can Tier 2 agent access another agent's bay or sovereign keys?

### 2.4 — Docking State Machine

Formal agent lifecycle — all states and transitions:

```
DORMANT → DOCKING → HYDRATING → ACTIVE → WORKING → DARK → TEARDOWN → DORMANT
```

| Transition | Trigger | What happens |
|------------|---------|--------------|
| DORMANT → DOCKING | `koad-agent --ghost <name>` invoked | Shell prep begins; identity TOML loaded; env vars exported |
| DOCKING → HYDRATING | Redis atomic lease acquired | CASS begins memory hydration; context files generated; MCP wired |
| HYDRATING → ACTIVE | Preflight passes; context files written | Shell READY; AI CLI can launch |
| ACTIVE → WORKING | AI CLI launched; first interaction begins | Agent online |
| WORKING → DARK | >30s without heartbeat ACK (2-3 consecutive misses) | Agent continues locally; buffers work for sync |
| DARK → WORKING | Heartbeat restored | CASS reconciles local saves with bay state |
| DARK → TEARDOWN | Timeout >5m in DARK state | Citadel initiates teardown; session considered lost |
| WORKING → TEARDOWN | `koad-agent clear` or CLI exit | Brain Drain protocol runs; EndOfWatch generated |
| TEARDOWN → DORMANT | Brain Drain complete | All state persisted; lease released; bay → STANDBY |

**Brain Drain Protocol (TEARDOWN):**
1. Auto-generate EndOfWatch summary (learnings, decisions, blockers, next steps)
2. Flush L1 Redis working memory → L2 SQLite episodic
3. Commit pending `koad_intel_commit` entries
4. Persist filesystem map updates to bay
5. Release Redis atomic lease
6. Bay → STANDBY

MVP note: in Phase 2, Brain Drain flushes to bay. Full L2/L3/L4 persistence is Phase 3 (CASS).

### 2.5 — Signal Corps & Event Bus

- Redis Streams integration: `koad:stream:*` schema
- Signal Packet broadcast format (JSON)
- Station-wide observability broadcasts

### 2.6 — Trace ID system

- `TraceContext` on every gRPC call
- `audit_trail` table in SQLite
- `koad dood pulse`, `koad dood inspect`, `koad watch --raw`

### 2.7 — Workspace Manager (Git Worktree orchestration)

- Provision isolated worktrees per task: `~/.koad-os/workspaces/{agent_name}/{task_id}/`
- Register worktrees in Redis: `koad:workspaces:{path_hash}` → `{ agent, issue_id, trace_id, created_at, branch }`
- Mount worktree path to agent's bay filesystem map at provisioning
- SLE/prod-adjacent repos: `.env.sandbox` with mock credentials only (enforced from `registry.toml` security tier)
- Debris Sweep: warn after 72h idle → Tyr notification after 24h → archive (not delete) after 48h
- Open questions to resolve: branch naming convention, concurrent worktree limit, cross-agent read-only access for review, worktree-to-PR automation, sparse checkout for monorepo

### 2.8 — Watchdog/Sentinel integration

- `koad-watchdog` crate wired into Citadel
- Self-healing, health monitoring, port contention detection

**Ian approval to close Phase 2 and begin Phase 3.**

---

## Phase 3 — Build CASS
*The brain. Memory, identity continuity, cognitive tools.*

**New crate:** `crates/koad-cass/`

After CASS is online, agents can boot with full memory and live tool access. This is the point where Tyr, Sky, and Vigil join as active contributors.

### 3.1 — 4-Layer Memory Stack

| Layer | Tool | CASS Role |
|-------|------|-----------|
| L1 Working memory | Redis Stack + Vector Search | Hot session state, semantic cache, pub/sub — expires at TTL |
| L2 Episodic | SQLite (WAL mode) | Conversation logs, task history — 90-day retention |
| L3 Semantic / long-term | Qdrant (self-hosted) | Per-agent private collections + shared `koados_knowledge` |
| L4 Procedural | SQLite (dedicated schema) | Skills, patterns, learned behaviors — no decay |

- Deploy Qdrant
- Integrate Mem0 OSS as memory middleware (importance scoring 0.0–1.0, decay, contradiction detection, cross-session continuity)
- Private collections: `sky_memories`, `tyr_memories`, `vigil_memories`
- Shared: `koados_knowledge` (all agents R/W), `task_outcomes` (read-shared, write-restricted to completing agent)
- Contradiction policy: surface `MEMORY_CONFLICT` event to originating agent; defer to most recent until Ian resolves
- Importance + decay: low-importance stale memories decay weekly → cold storage; L4 procedural exempt

### 3.2 — Memory Insurance (Triple-Redundancy)

**Three sinks — all must ACK before Brain Drain completes:**
1. **SQLite** (L2/L4) — primary durable store, WAL mode
2. **WORM Ledger** (`ledger.jsonl`) — append-only, no overwrite/truncate. Format: `{ timestamp, trace_id, agent, layer, action, payload_hash, payload }`. The "black box."
3. **Notion Cloud backup** — async periodic sync; failure → retry queue (does not block Brain Drain)

**Automatic Vault:**
- `VACUUM INTO` timestamped SQLite snapshots every 10 commits or 4 hours (whichever first)
- Stored in `~/.koad-os/backups/`, configurable retention in `kernel.toml [memory_insurance]`

**Anti-Deletion Protection:**
- SQLite `ON DELETE` triggers redirect deleted rows to `audit_trail` table (not real deletion)
- Only Ian can execute true purge via `koad system purge --confirm`
- Agents can only mark memories deprecated (decay score → 0)

**Recovery commands:**
- `koad system restore --from ledger`
- `koad system restore --from cloud`
- `koad system restore --from snapshot <timestamp>`

### 3.3 — CASS MCP Server

Exposes CASS tools via Model Context Protocol (localhost, wired into CLI configs by `koad-agent`):

| MCP Tool | Function |
|----------|----------|
| `koad_intel_commit` | Write to L2 episodic (intelligence bank) |
| `koad_intel_query` | Retrieve from memory bank |
| `koad_memory_hydrate` | Pull session context on demand |
| `koad_status` | Citadel/CASS health check |
| `koad_map_add` | Register filesystem path to Personal Bay |
| `koad_session_save` | Persist working state (enforces EndOfWatch schema) |
| `koad_session_restore` | Read latest session notes |
| `koad_context_archive` | Archive verbose content to L2 with 1-line summary |
| `koad_signal_send` | Send async signal to another agent's mailbox (A2A-S) |
| `koad_signal_read` | Read pending signals from own mailbox |
| `koad_hydrate_from` | Read-only cross-agent context query (TCH) |

Graceful degradation: if Citadel offline, `koad-agent` omits MCP config; agent operates without CASS tools.

### 3.4 — ASM integration as CASS sub-system

- Memory hydration on agent connect: Mem0 runs `memory.search()` across L1+L2+L3 before ghost fully online
- Session re-establishment on reconnect
- Handoff reconciliation: CASS reconciles local `.koad-dark/` saves with bay state on DARK → WORKING

### 3.5 — Dark Mode persistence & reconciliation

- Standardized save format: TOML frontmatter + Markdown body (path from BLOCKER 1 resolution)
- CASS parser for reconciliation
- Sovereignty Hierarchy: `Dood > Citadel State > Local Save`
- Conflicts quarantined to `.koad-conflict/`
- `MEMORY_CONFLICT` events surfaced to agent and Tyr

### 3.6 — EndOfWatch schema enforcement

- Structured schema enforced in `koad_session_save`
- Required fields from BLOCKER 4
- Auto-generated at TEARDOWN; also triggerable on compaction events
- Stored in bay session log AND tagged to `koados_knowledge` (project/topic tagged)

### 3.7 — A2A-S — Agent-to-Agent Signal Protocol

- **Ghost Mailbox**: Redis keys `koad:mailbox:<agent_name>` — JSON signal payloads: `{ sender, timestamp, priority, message, issue_ref }`
- **Boot-time delivery**: `koad-agent` checks mailbox during preparation, injects pending signals into context file (Tier 2 content). High-priority signals flagged prominently.
- **In-session**: `koad_signal_send` + `koad_signal_read` MCP tools
- **CLI**: `koad signal <agent_name> -m "message" [-p HIGH|NORMAL|LOW]`
- **Lifecycle**: signals consumed (marked read) when target agent's prep hydrates them; unread signals older than configurable TTL (default 7 days) → archived to L2
- **Open questions**: HIGH-priority escalation to Ian (Notion/email)? Mailbox capacity limit? EndOfWatch trigger on compaction events?

### 3.8 — TCH — Temporal Context Hydration

- `koad hydrate --from <agent_name> --topic <topic_id>` (CLI) or `koad_hydrate_from` (MCP)
- Returns read-only summary: EndOfWatch summaries + relevant `koad_intel_commit` entries + `task_outcomes`
- Does NOT return raw turn history or private L3 memories
- Permission model: cross-agent hydration requires opt-in in source agent's identity TOML: `[sharing] allow_hydration_from = ["tyr", "vigil"]` — default: no cross-agent access
- TCH queries logged to `audit_trail` for Tyr visibility

### 3.9 — `koad system migrate-v5` (data migration tool)

- Import Phase 0 extracted data into new CASS memory stack
- Old episodic data → L2 SQLite; old `koados_knowledge` Redis → L3 Qdrant shared pool
- **Migration dry run gate:** validate on a subset of extracted data before any destructive operation

**Ian approval to close Phase 3. Agents (Tyr, Sky, Vigil) join as contributors after this point.**

---

## Phase 4 — Build `koad-agent`
*Shell preparation tool. Body/Ghost model.*

**New crate:** `crates/koad-agent/`

### 4.1 — Boot sequence (9-step prepare flow)

Standard invocation: `eval $(koad-agent --ghost sky --export)`

Steps run by `koad-agent --ghost <name>`:
1. Read identity TOML: `~/.koad-os/config/identities/<name>.toml`
2. Export env vars: `KOAD_AGENT_NAME`, `KOAD_HOME`, `KOAD_SESSION_ID`, etc.
3. Generate context files based on target CLI:
   - `AGENTS.md` — always generated (universal; covers Codex + Gemini natively)
   - `CLAUDE.md` — generated when `model = "claude"`
   - Content: Tier 1 always-loaded core + Tier 2 task-relevant (3-tier progressive disclosure)
4. Generate CLI config files:
   - `.claude/settings.local.json` — permissions, hooks, MCP servers
   - `.gemini/settings.json` — model, tools, MCP servers
   - `.codex/config.toml` — model, approval policy, MCP servers
5. Wire CASS MCP server into CLI config (if Citadel reachable)
6. Hydrate session context (if Citadel reachable) — pull last session notes from CASS + Qdrant → append to context file
7. Generate system prompt append file for Claude's `--append-system-prompt`
8. Run preflight validation: required env vars? context files? CASS reachable? One Body One Ghost lease available?
9. Report: READY / DEGRADED / NOT READY + suggested launch command

**Does NOT launch the AI CLI — always a manual step.**

### 4.2 — Three-Tier Context Hydration

- **Tier 1 — Always-Loaded Core (<2,000 tokens):** Agent identity, condensed Prime Directives, One Body One Ghost, Sanctuary Rule summary, active session continuity note, table of contents for Tier 2/3
- **Tier 2 — Task-Relevant (3,000–8,000 tokens):** Loaded by `koad-agent` from ghost config — station conventions, Dev Canon, filesystem map, recent CASS session notes. Scope driven by `memory_scope` in identity TOML
- **Tier 3 — On-Demand (0 tokens at boot):** Never in boot file. Agent pulls via CASS MCP tools during session — full Prime Directives, Core Contract sections, historical outcomes, cross-agent `koados_knowledge`

`koad-agent` generates `AGENTS.md` dynamically on every boot — never stored persistently, always fresh. Token budget controlled by `max_boot_tokens` in identity TOML `[context]` section.

**Context lifecycle in-session:**
1. Observation masking — old tool outputs archived to CASS L2, replaced with 1-line summaries
2. Structured note-taking — agent writes state to Personal Bay via `koad_session_save`
3. CLI compaction (safety net) — KoadOS post-compaction hook re-injects Tier 1 identity + latest session notes
4. Sub-agent delegation — maps to Micro-Swarm Hangar (future)

### 4.3 — Ghost config format (identity TOML)

```toml
[identity]
name = "Sky"
role = "Officer"
model = "gemini"
instructions = "~/.koad-os/config/identities/sky.instructions.md"
memory_scope = "sle"

[context]
prime_directives = "~/.koad-os/config/shared/prime-directives.md"
dev_canon = "~/.koad-os/config/shared/development-canon.md"
includes = ["~/.koad-os/config/shared/koados-glossary.md"]
max_boot_tokens = 8000

[env]
KOAD_AGENT_NAME = "Sky"
KOAD_AGENT_ROLE = "Officer"
KOAD_AGENT_INSTRUCTIONS = "~/.koad-os/config/identities/sky.instructions.md"
KOAD_MEMORY_SCOPE = "sle"

[requires]
vars = ["NOTION_TOKEN", "GITHUB_TOKEN", "KOAD_HOME"]

[optional]
vars = ["GCLOUD_PROJECT"]

[hooks]
pre_tool_use = "~/.koad-os/hooks/sanctuary-check.sh"
post_edit = "~/.koad-os/hooks/ksrp-lint.sh"

[sharing]
allow_hydration_from = ["tyr", "vigil"]   # TCH opt-in
```

### 4.4 — CLI modes

- `koad-agent inspect` / `koad-agent status` — health summary: READY / DEGRADED / NOT READY
- `koad-agent --ghost <name> [--export]` — prepare shell for named ghost
- `koad-agent set <VAR> <value>` — manual var correction without full re-prepare
- `koad-agent clear` — unset all KoadOS session vars; trigger EndOfWatch

### 4.5 — Context file hierarchy (three CLIs)

- **Global** (`~/.koad-os/config/shared/AGENTS.md`) — agent identity, KoadOS canon, Prime Directives
- **Project/Station** (`~/projects/<station>/AGENTS.md`) — station context, tech stack, conventions
- **Subdirectory** (`~/projects/<station>/src/<module>/AGENTS.md`) — module-specific rules

Global file is auto-generated on every ghost boot. Project + subdir files are persistent, manually maintained.

### 4.6 — `koad install` interactive installer

```
1. Detect environment (native Linux vs. WSL) — confirm with developer
2. Confirm workspace paths, data dirs, KOAD_HOME
3. Prompt for each required env var / secret (with descriptions + validation)
4. Write secrets to ~/.koad-os/.env; non-secret config to TOML files
5. Guide through creating Captain Agent:
   a. Choose name, role, model
   b. Generate identities/<captain>.toml
   c. Provision Personal Bay for Captain Agent
6. Run preflight check — validate all vars, paths, TOML files
7. Exit: READY or NEEDS ATTENTION
```

**Ian approval to close Phase 4.**

---

## Phase 5 — Integration & Documentation
*Wire everything together. Terminology scrub. Docs update.*

### 5.1 — Spine Terminology Scrub (docs-only — no legacy code to grep)

Scrub surfaces:
- Notion: Core Contract, Global Canon, all agent instruction pages, Tyr Briefs, AGENTS.md files
- GitHub: issue titles/descriptions, Project #2 board, README and repo markdown
- Rule: no blind find-and-replace — each occurrence reviewed for context (Citadel vs. CASS vs. agent-local)
- Document every replacement in a scrub log (GitHub issue or Notion sub-page)

Replacement vocabulary:

| Old term | Replace with |
|----------|-------------|
| The Koad Spine | The Citadel |
| Spine (cognitive/session layer) | CASS |
| Spine session tethering | Citadel session brokering |
| Lost Spine connection | Disconnected from the Citadel |
| k-spine | citadel (CLI/code contexts) |

### 5.2 — Agent instruction update

- Single coordinated pass: Sky, Tyr, Vigil instruction pages (simultaneous — no mixed vocabulary period)
- Tyr's domain: updated to Citadel ownership and captain scope

### 5.3 — KoadOS Core Contract v2.4

- New architecture canonical
- KoadOS Operating Philosophy section ("Koados" pseudonym retired)
- Updated Operational Infrastructure section

### 5.4 — Tyr Brief

- Announce refactor completion
- Define Tyr's expanded Citadel domain

### 5.5 — External README

- Written after `koad install` locked (Phase 4 complete)
- For external developers: fresh clone → `koad install` → READY Citadel boot

### 5.6 — Full integration test

- Fresh clone + `koad install` on a clean environment
- Validate: READY Citadel boot, Personal Bay provisioned, CASS MCP tools available, agent boot works

**Ian final approval to close Phase 5.**

---

## Phase 6 — Agent Growth System & Advanced Memory (Gated)
*After Phase 5 stable and approved.*

1. **Mem0 advanced hooks** — agent interaction loop, semantic cache, contradiction detection mid-session
2. **Agent Journals** — append-only crew logs (`koad journal add "..."`, `koad_journal_add` MCP tool)
3. **Introspection Cycles** — structured self-reflection at EndOfWatch (5 questions → journal + Knowledge Indexer)
4. **Knowledge Broadcasting** — micro-agent Knowledge Indexer → `koados_knowledge` Qdrant + `knowledge:new-learning` Signal Corps event
5. **A2A-S advanced** — HIGH-priority signal escalation to Ian, mailbox capacity limits
6. **Full TCH** — expanded cross-agent context scope (filtered L3 semantic + EndOfWatch)

---

## Phase 7 — Future (Gated)
*Requires Phase 6 stability and explicit approval.*

1. **Neo4j + Graphiti** — temporal knowledge graph (L5 memory layer)
2. **Micro-Swarm Hangar** — ephemeral task-scoped micro-agents, Intake Drone, Idea Pipeline (`koad dispatch`)
3. **Native CLI sub-agent integration** — `koad-agent` generates sub-agent configs for Claude Code / Codex native sub-agent features
4. **TUI / Web Deck** — v6+ interface layer

---

## Current Codebase State (Worktree: gifted-rubin)

**Cargo workspace (current):**

| Crate | Action |
|-------|--------|
| `koad-core` | Keep |
| `koad-proto` | Keep — will be extended with `citadel.proto` |
| `koad-cli` | Keep — new command surface added |
| `koad-watchdog` | Keep — wire into Citadel (Phase 2.8) |
| `koad-board` | Keep |
| `koad-bridge-notion` | Keep |
| `koad-spine` | Archive Phase 0.3 |
| `koad-asm` | Archive Phase 0.3 |

**Proto files:**
- `proto/spine.proto` → archive Phase 0.3
- `proto/skill.proto` → assess (likely keep or adapt)
- `proto/citadel.proto` → new, Phase 1.2

**Config (currently in repo):**
- `config/kernel.toml`, `config/registry.toml`, `config/identities/`, `config/integrations/` — exist, review and update in Phase 1.3

---

## Gate Summary

| Gate | Condition | Action |
|------|-----------|--------|
| Phase 0 | Legacy extraction confirmed; old crates archived | Ian approval |
| Phase 1 | All 🔴 blockers resolved; proto written; CLI surface locked | Ian approval |
| Phase 1.5 | `citadel.proto` supports bidirectional streaming | Must pass before Phase 2 |
| Phase 2 Red Team | Tier 2 agent cannot access another bay or sovereign Redis keys | Must pass before Phase 3 |
| Phase 3 migration dry run | `koad system migrate-v5` validated on subset of extracted data | Must pass before archive |
| Phase 3 | CASS online; agents boot with full memory and MCP tools | Ian approval — agents join as contributors |
| Phase 5 | Fresh install test passes on clean environment | Ian final approval |

---

## First Actions (Ready to Execute on Ian Approval)

These can be drafted in parallel and presented as one approval package:

1. **Run legacy state extraction** — dump Redis `koad:*` and SQLite `koad.db` (do this first, before anything is archived)
2. **BLOCKER 5 decision** — `config/identities/` wins; `personas/`/`bodies/` clarified (quick, unblocks identity work)
3. **BLOCKER 7 draft** — new CLI command surface for Ian review
4. **BLOCKER 4 confirmation** — EndOfWatch schema already 90% defined in DRAFT_PLAN_2; needs Ian sign-off
5. **BLOCKER 9 decision** — confirm `eval $(koad-agent --ghost sky --export)` as standard invocation
