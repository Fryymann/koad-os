## Overview

This page is the planning and research ground for a major KoadOS architectural refactor. The core proposal: **retire The Spine** as a monolithic concept and replace it with a three-tier model — **The Citadel**, **CASS (Citadel Agent Support System)**, and **Agent-Level Cognition**.

---

## 🎯 Primary Build Goal

<aside>
🎯

**Phase Priority — Officer Agent Support First.** The first goal of this rebuild is getting the Citadel built and stable enough to support the three Officer agents: **Tyr, Sky, and Vigil**. Once the Citadel reaches a functioning and stable baseline, CASS is built next. After CASS is online, all three agents can contribute to the continued development of the rest of the Citadel and KoadOS.

</aside>

**Build sequence (top-level):**

1. **Citadel** → built to a stable, functioning baseline capable of supporting Officer agents
2. **CASS** → cognitive support layer brought online
3. **Continued Citadel + full KoadOS development** → with Tyr, Sky, and Vigil all contributing

---

## The Core Proposal

### What's changing

- **The Spine** → decomposed and retired
- **The Citadel** → new name and expanded scope for the central OS layer
- **CASS** → new sub-system for agent cognitive support: memory, sense of self, and a suite of tools agents can invoke to offload cognitive load and work
- **Agent-Level Cognition** → formalized as each agent's own cognitive domain
- **"Koados" pseudonym** → retired. KoadOS is the canonical name for both the system and the operating philosophy.

### What's changing (scope update — from-scratch rewrite)

<aside>
🔴

**Scope escalation (2026-03-11):** This refactor is now a **from-scratch rewrite**, not a decomposition/rename of the existing Spine codebase. Removing legacy references and patching lingering systems across the existing Rust workspace was assessed as more difficult and error-prone than building the Citadel, CASS, and Agent-Level Cognition cleanly from the ground up. The old `koad-spine` crate and its dependents are retired wholesale — not refactored in place.

</aside>

- The old Spine codebase (`crates/koad-spine/`, `proto/spine.proto`, `koad-asm`) is **archived, not migrated**. Implementation lessons carry forward; code does not.
- The Citadel, CASS, and `koad-agent` are built as **new crates/services** with clean APIs, clean proto definitions, and no legacy shims.
- Config migration is replaced by **fresh `koad install`** — the new system bootstraps its own config from scratch.
- The Spine Terminology Scrub becomes a **documentation-only pass** — no legacy code to grep through, only Notion docs, agent instructions, and GitHub issues.

### What's NOT changing

- The Prime Directives
- Agent identities (Sky, Tyr, Vigil)
- Tyr as Captain — his domain just gets a better name
- The architectural concepts validated by the Spine era (Sector Locking, CIP, Atomic Lease, Watchdog) — these are re-implemented cleanly, not discarded
- The KoadConfig / TOML config philosophy and directory structure

---

## Proposed Architecture

### 🏰 The Citadel

> *The central, always-on operating system. Tyr's domain.*
> 
- Agent connectivity and session brokering
- Shared state management (Redis, Sector Locking, distributed coordination)
- Watchdog and Sentinel (self-healing, health monitoring)
- **Signal Corps** — station-wide broadcast service for real-time observability (see Signal Corps & Event Bus section below)
- **Trace ID chain** — E2E request lifecycle tracking across all subsystems (see Trace ID & Observability section below)
- **Workspace Manager** — Git Worktree orchestration for parallel agent isolation (see Workspace Manager section below)
- Tyr's command and control infrastructure
- **Personal Bays** — dedicated, reserved connection slots and support infrastructure for each registered agent (see Personal Bay Model below)
- The OS that "runs itself as much as possible"
- Permanent — if The Citadel goes dark, KoadOS is offline
- **Scope note:** The Citadel is not all of KoadOS. Stations (project-hub directories, e.g. the SLE) and Outposts (standalone project directories) are part of KoadOS but are **not** part of The Citadel.
- **Micro-Swarm Hangar** *(planned future module)* — the Citadel's dedicated facility for deploying, managing, and supporting a swarm of micro-agents in future versions (see Micro-Swarm Hangar section below)

### 🧬 CASS — Citadel Agent Support System

> *The cognitive support layer. Gives agents memory, a sense of self, and tools to offload work.*
> 

CASS is not merely a passive session layer — it is an **active cognitive support system** that agents can invoke. Its two core responsibilities are:

**1. Cognitive continuity — memory and sense of self**

- Memory hydration on agent connect/reconnect
- Session re-establishment protocols
- Shared memory access and persistent cognitive scaffolding
- Handoff and reconciliation when an agent returns from disconnected state
- Maintains each agent's accumulated context, identity continuity, and mission awareness across sessions

CASS's cognitive continuity is powered by the **4-layer memory stack** defined in the [Memory System Implementation Spec](https://www.notion.so/KoadOS-Agent-Memory-System-Implementation-Spec-be427a1ae4e24961b81d86479b7cb627?pvs=21):

| **Layer** | **Type** | **Tool** | **CASS Role** |
| --- | --- | --- | --- |
| L1 | Working memory | Redis Stack (+ Vector Search) | Hot session state, semantic cache, inter-agent pub/sub — expires at session TTL |
| L2 | Episodic memory | SQLite (WAL mode) | Conversation logs, task history, events — 90-day retention policy |
| L3 | Semantic / long-term | Qdrant (self-hosted) | Per-agent private collections + shared `koados_knowledge` pool — indefinite with decay scoring |
| L4 | Procedural memory | SQLite (dedicated schema) | Skills, patterns, learned behaviors — indefinite, no decay |

**Mem0 OSS** acts as the memory middleware orchestrating across all layers: automatic memory extraction from agent interactions, importance scoring, decay/forgetting, and contradiction detection.

**Memory isolation model:** each agent has a private Qdrant collection (`sky_memories`, `tyr_memories`, `vigil_memories`). A shared `koados_knowledge` collection holds platform-wide architecture facts and ops knowledge readable and writable by all agents. A `task_outcomes` collection is read-shared but write-restricted to the completing agent only.

**2. Cognitive offload — tools agents can invoke**

- Agents can dispatch requests to CASS to perform work on their behalf, freeing them to stay focused on the current task
- Examples: context hydration requests ("pull my SLE context for this project"), GitHub issue spawners, structured logging triggers, memory commit requests
- The agent-facing CLI surface for memory operations is `koad intel commit` (write to `intelligence_bank`) and `koad intel query` (retrieve from memory bank). These map to CASS's L2 episodic store and are the primary way agents persist session learnings across boots.
- CASS acts as an always-available cognitive co-processor — agents don't have to do everything inline

**3. CASS MCP Server — cross-CLI tool bridge**

<aside>
🔬

**Added per** [Agent Boot Research — CLI Context Injection Patterns](https://www.notion.so/Agent-Boot-Research-CLI-Context-Injection-Patterns-0f4b35fc93b54fdca9d1bd63537502ae?pvs=21). This is the second highest-impact improvement identified by the research. MCP (Model Context Protocol) is supported by all three major AI CLIs (Gemini, Claude Code, Codex). Exposing CASS as an MCP server gives agents **live, interactive** access to CASS tools during the session — not just static context at boot.

</aside>

The CASS MCP server is a lightweight service that exposes CASS's cognitive offload tools via the Model Context Protocol. It runs on [localhost](http://localhost) when the Citadel is online. `koad-agent` wires it into the generated CLI config during ghost preparation, so the AI CLI discovers it automatically on launch.

**Exposed MCP tools:**

- `koad_intel_commit` — write to intelligence bank (L2 episodic store)
- `koad_intel_query` — retrieve from memory bank
- `koad_memory_hydrate` — pull session context on demand
- `koad_status` — check Citadel / CASS health
- `koad_map_add` — register filesystem paths to Personal Bay
- `koad_session_save` — persist current working state for resumption
- `koad_session_restore` — read latest session notes (post-compaction or session start)
- `koad_context_archive` — archive verbose content to CASS L2 with 1-line summary
- `koad_signal_send` — send an async signal to another agent's mailbox (A2A-S)
- `koad_signal_read` — read pending signals from own mailbox
- `koad_hydrate_from` — read-only cross-agent context query for handoff scenarios (TCH)

**Why MCP instead of CLI shell-outs:**

- MCP tools appear as **native tools** in the AI's tool surface — the agent doesn't need to shell out to `koad` commands, which is unreliable and often sandboxed
- All three CLIs support MCP natively: Gemini (`settings.json`), Claude Code (`.claude/settings.json`), Codex (`config.toml`)
- One MCP server serves all CLIs — no CLI-specific integration code
- Graceful degradation: if the Citadel is offline, `koad-agent` simply omits the MCP config and the agent operates without CASS tools (Citadel independence preserved)

**Structure:**

- Managed by The Citadel; consumed by agents
- **ASM (Agent Session Manager)** is a confirmed CASS sub-system
- The **CASS MCP server** is the primary agent-facing tool interface (replaces direct CLI shell-outs for CASS operations)
- The CASS tool surface is extensible — new offload capabilities can be added as new MCP tools without modifying agent instructions or CLI configs

### 🧠 Agent-Level Cognition

> *Each agent's own mind. Operates independently of Citadel connectivity.*
> 
- Agent's local task state and working memory
- Continuity behavior during Citadel loss (save locally, continue working)
- Agent-specific identity and instruction context
- Sky continues working without CASS — she just does it blind and local

### 🏗️ Architectural Stance — CQRS & Stateless Core

<aside>
🧭

**Mental model for where new features belong.** Source: 00_TOP_LEVEL architecture doc.

</aside>

The Citadel's dual-bus architecture follows a **CQRS (Command Query Responsibility Segregation)** pattern:

- **Redis (Hot Memory) = Data Plane** — high-frequency *Queries*. Agents read context, peer status, telemetry, and Signal Corps broadcasts directly from Redis. No gRPC round-trip needed for reads.
- **gRPC (Neural Bus) = Control Plane** — strict, validated *Commands*. Agents invoke gRPC only for state mutations (session registration, sector locks, memory writes). Every command carries a `TraceContext`.

**Stateless Core with Read-Through Cache:**

The Citadel is **query-stateless** — it holds no authoritative internal state in process memory. Redis is the single source of truth for all hot state. However, the Citadel may maintain a **read-through cache** synchronized via Redis keyspace notifications (`keyspace@0:koad:sessions:*`) for performance. This is architecturally distinct from the legacy `Arc<Mutex<HashMap>>` pattern — the cache is a disposable replica, not the authority.

> *Rule: if it's a read, go to Redis. If it's a mutation, go through gRPC. If it's telemetry, go to Redis Streams.*
> 

---

## Disconnect State Model

This three-tier architecture makes failure states expressive and precise:

| **State** | **What's lost** | **What's still online** | **Severity** |
| --- | --- | --- | --- |
| CASS disconnected | Memory hydration, cognitive resupply, session sync | Agent-local cognition, Citadel infra | Medium — agent is degraded but working |
| Citadel disconnected | All shared state, Tyr comms, CASS, coordination | Agent-local cognition only | High — agent is dark and isolated |
| Agent cognition failure | Agent-local state, working memory | Citadel and CASS intact | Agent-level — not a Citadel issue |

---

## KoadOS — Operating Philosophy

"Koados" as a separate concept and pseudonym is **retired**. KoadOS is the single canonical name for both the system and its operating mission.

The operating philosophy — the *why* and *how* Ian and all KAI agents work within KoadOS — is a first-class part of KoadOS canon. It belongs as a dedicated section in the KoadOS Core Contract, not as a parallel brand or separate name. Agents reference it by its section name.

> KoadOS is both the system and the mission. One name, one canon.
> 

---

## Open Questions & Research Items

- [x]  **CASS naming** — **Resolved:** CASS is locked. "Support Systems" is the accepted shorthand (Citadel Agent Support Systems). No further debate needed.
- [x]  **ASM placement** — **Resolved:** ASM is absorbed as a CASS sub-system.
- [x]  **Reconnect reconciliation ownership** — **Resolved:** Sky owns local saves (written as `.md` files in the relevant project directory while offline). On reconnect, Sky initiates the reconciliation handoff and CASS performs the sync. Sky writes locally, CASS reconciles remotely.
- [x]  **Spine retirement doc** — Write a formal deprecation note mapping every Spine function to its new home (Citadel, CASS, or agent-local).
- [ ]  **Agent instruction audit** — Which agent instruction pages reference "The Spine"? All must be updated in a coordinated pass.
- [ ]  **Doc/page audit** — Full audit of [AGENTS.md](http://AGENTS.md) files, issue titles, and CLI output for "Spine" references.
- [ ]  **KoadOS Core Contract update** — v2.3 will need a new version bump and updated Operational Infrastructure section.
- [ ]  **Tyr brief** — Draft a Tyr Brief announcing the refactor and defining his expanded Citadel domain.
- [ ]  **KoadOS Operating Philosophy doc** — Draft a "KoadOS — Operating Philosophy" section for the Core Contract. The "Koados" pseudonym is retired; this content lives under the KoadOS name.

### Tyr's Strategic Review — Identified Gaps (2026-03-12)

<aside>
🛡️

**Source:** [Tyr's Strategic Review: KoadOS Refactor Plan (v1)](https://www.notion.so/Tyr-s-Strategic-Review-KoadOS-Refactor-Plan-v1-321fe8ecae8f8075a74bc34461b8cf3f?pvs=21). Five friction points identified during Captain's review. All must be addressed before Phase 0 completes.

</aside>

- [ ]  **🔴 Dark Mode local persistence format** — When an agent loses Citadel connectivity, they must write to a **standardized `.md` or `.json` structure** that CASS can reliably parse during Brain Drain or reconnect reconciliation. Without this, "Citadel Independence" is a theory, not a feature. Define: path convention (`<project_dir>/.koad-dark/<agent>/<session_id>.md`?), metadata format (TOML/JSON frontmatter), and CASS parser contract.
- [ ]  **🔴 Data migration protocol (`koad system migrate-v5`)** — Even with a from-scratch rewrite, **existing knowledge in `koad.db` and current Redis keys must not be lost.** Define a `koad system migrate-v5` protocol that extracts valuable episodic memories, task outcomes, and `koados_knowledge` from the old system and imports them into the new CASS memory stack. This is a one-time migration tool, not an ongoing compatibility layer.
- [ ]  **🔴 Tier 1 Zero-Trust enforcement** — Vigil identified that Tier 1 agents currently have **unrestricted write access to sovereign Redis keys**. The new Citadel must enforce the Sanctuary Rule at the **gRPC layer (Control Plane)**, not just the agent layer. Bake Zero-Trust into the Citadel's core gRPC service from day one — do not ship and patch later.
- [ ]  **🟡 EndOfWatch schema standardization** — For TCH (Temporal Context Hydration) to work, EndOfWatch summaries need a **strictly enforced schema** — not free-form text. Define a structured format (TOML/JSON frontmatter + markdown body) so any agent can reliably parse any other agent's EoW output. Proposed fields: `agent`, `session_id`, `timestamp`, `project`, `worked_on[]`, `decisions[]`, `blockers[]`, `next_steps[]`, `learnings[]`.
- [ ]  **🟡 Legacy state extraction window** — Before archiving the old Spine codebase, run a final diagnostic pass to capture: all Redis key-value pairs under `koad:*`, SQLite memory bank contents, any active session state. This is the raw material for the `koad system migrate-v5` tool. Archive the old codebase only *after* extraction is confirmed complete.

---

## Spine Functionality Decomposition Map

<aside>
🗺️

**This map is a design reference for the from-scratch build.** Every Spine function is mapped to its new home in the Citadel/CASS/Agent-Level Cognition architecture. Since the old codebase is archived (not migrated), this map serves as the architectural blueprint — specifying what each new component must provide. No old code is being renamed or moved; the new system is built clean using this map as the specification.

</aside>

The table below enumerates every known Spine responsibility and its confirmed new home. Items marked **🔴 Unresolved** require a decision before execution proceeds.

| **Spine Function / Responsibility** | **New Home** | **Mechanism** | **Status** |
| --- | --- | --- | --- |
| Session tethering (agent ↔ OS connection) | The Citadel | Citadel session brokering service; agents register on boot | ✅ Mapped |
| Agent registration & heartbeat | The Citadel | `registry.toml`  • runtime registry; heartbeat via Citadel connection | ✅ Mapped |
| Distributed state / Sector Locking (Redis) | The Citadel | Citadel owns Redis; Sector Locking protocol unchanged | ✅ Mapped |
| Watchdog / self-healing | The Citadel | Watchdog and Sentinel remain Citadel sub-systems | ✅ Mapped |
| Memory hydration on session start | CASS | Mem0 middleware runs `memory.search()` across L1 (Redis Stack), L2 (SQLite episodic), L3 (Qdrant semantic) and injects relevant context before ghost comes fully online | ✅ Mapped — mechanism defined in Memory Spec (Phase 1C) |
| Cognitive scaffolding (shared memory access) | CASS | CASS provides scaffolding via Qdrant per-agent private collections + shared `koados_knowledge` pool; Mem0 handles retrieval and importance scoring mid-session | ✅ Mapped — hard collection-level isolation model in Memory Spec |
| Session re-establishment on reconnect | CASS | CASS owns reconnect protocol; re-hydrates session state | ✅ Mapped |
| Handoff / reconciliation (local → Citadel sync) | CASS | Sky writes `.md` saves to the project directory while offline; Sky initiates handoff on reconnect; CASS performs the sync | ✅ Mapped — Sky writes locally, CASS reconciles remotely |
| ASM — Agent Session Manager | CASS — sub-system | ASM absorbed into CASS as a confirmed sub-system | ✅ Mapped — ASM is a CASS sub-system |
| Agent identity context (instructions, persona, model) | Agent-Level Cognition + Personal Bay | `identities/<agent>.toml` is the provisioning spec; bay holds the live runtime instance | ✅ Mapped |
| Agent-scoped credentials (PATs, tokens, keys) | Personal Bay — credential vault | Scoped view of system secrets; broker pattern, enforced at bay level by Citadel | ✅ Mapped (implementation details in open questions) |
| Agent tool / capability permissions | Personal Bay — tool manifest | Defined in identity TOML; enforced by Citadel at bay level; not shared across agents | ✅ Mapped |
| Agent filesystem map (assigned + self-registered paths) | Personal Bay — filesystem map | Assigned paths from identity TOML; self-registered paths persisted in bay across sessions | ✅ Mapped (map format TBD — see Personal Bay open questions) |
| Per-agent session history & reconnect log | Personal Bay — session log | Bay stores last-seen state, disconnect/reconnect events; used by CASS for reconciliation | ✅ Mapped |
| Per-agent boot error history & health record | Personal Bay — health record | Citadel tracks per-bay preflight results and error patterns; visible to Tyr | ✅ Mapped |
| Local task state & working memory | Agent-Level Cognition (offline) → Personal Bay (online) | Local during dark mode; synced to bay's cognition context on reconnect via CASS | 🔴 Unresolved — local persistence path and format not defined |
| Continuity behavior during Citadel loss (dark mode) | Agent-Level Cognition | Agent continues locally; buffers work for CASS sync on reconnect | ✅ Mapped (behavior defined in concept; implementation TBD) |
| Preflight / env validation | `koad-agent` CLI tool | `koad-agent inspect` / prepare mode; runs before AI CLI launch | ✅ Mapped |
| Shell/session variable management | `koad-agent` CLI tool | `koad-agent set`, `clear`, `--export` modes | ✅ Mapped |
| Spine config / config values | KoadConfig / TOML | New config keys designed from scratch in `kernel.toml`, `registry.toml`, `identities/*.toml`. Vigil's inventory of old config keys serves as reference for coverage. | ✅ Reframed — no migration needed; new config designed clean (Phase 1) |
| Spine CLI commands (`koad spine *`) | New CLI surface | New command surface designed from scratch: `koad citadel *`, `koad agent *`, `koad signal *` per Phase 1.3 | ✅ Reframed — no old commands to migrate; new surface defined in Phase 1 |

### Unresolved items summary

Before execution begins, the following must be decided and documented:

- [x]  **Handoff ownership** — Resolved: Sky writes `.md` files locally; Sky initiates; CASS reconciles on reconnect.
- [x]  **ASM placement** — Resolved: ASM is a CASS sub-system.
- [ ]  **Local persistence format** — Sky writes `.md` files in the project directory during dark mode. The exact path convention and any metadata format (frontmatter, naming scheme) must be defined and documented.
- [x]  **Spine config key audit** — ✅ Vigil has completed this inventory. See [Vigil — CLAUDE.md](https://www.notion.so/Vigil-CLAUDE-md-321fe8ecae8f806780abd3c65016de36?pvs=21) → Hardcoded Values Registry. Critical items: `sandbox.rs:15` (GITHUB_ADMIN_PAT bypass — Critical), `sandbox.rs:50-53/99/111` (production triggers, blacklist, protected paths — High). `constants.rs` ports/addresses inventoried (Medium/Low). Hardcoded agent names resolved 2026-03-11.
- [ ]  **CLI command migration** — Define the new `koad` command surface to replace all `koad spine *` commands

---

## Refactor Sequencing (Revised — From-Scratch Rewrite)

<aside>
🔴

**Scope change (2026-03-11):** This sequencing reflects a **from-scratch rewrite**, not an incremental refactor of the existing Spine codebase. The old codebase is archived after knowledge extraction; the new Citadel/CASS/koad-agent are built clean.

</aside>

<aside>
⚠️

**Gate rule:** Each phase requires explicit Ian approval before the next begins. No code is written until canon is locked and all 🔴 items are resolved.

</aside>

<aside>
📖

**Development Canon applies to all code execution steps.** Per the [**KoadOS Agent Onboarding Flight Manual**](https://www.notion.so/KoadOS-Agent-Onboarding-Flight-Manual-321fe8ecae8f80aaa3e5e945e262fd60?pvs=21) §12: every change requires a GitHub Issue before code is touched, commits must reference issue numbers, and all merges/pushes require explicit Ian approval. Code steps follow KSRP (7-pass self-review) and close with PSRP (Three-Pass Saveup: Fact, Learn, Ponder).

</aside>

### Phase 0 — Legacy Knowledge Extraction & Archive

*Save everything valuable. Archive everything old. Start clean.*

1. **Legacy state extraction** — Run final diagnostic pass on the old system: dump all Redis key-value pairs under `koad:*`, export SQLite memory bank contents, capture active session state. This is the raw material for `koad system migrate-v5`. *(Addresses Tyr's "Legacy state extraction window" gap.)*
2. **Archive old codebase** — Move `crates/koad-spine/`, `crates/koad-asm/`, `proto/spine.proto`, `koad-gateway`, `koad-tui`/`kdnd-tui` to an `archive/` branch or directory. Remove from active `Cargo.toml` workspace. Preserve for reference only.
3. **Pre-archive cleanup** — Remove ~30 leaked/temp/stale files identified by Vigil's cleanup_sweep_3-11-2026. Clean `.koad-os/legacy/` redundancy. Archive `sdk/python` stub.

### Phase 1 — Lock Canon & Resolve All Open Items

*Nothing is built until the blueprint is approved.*

1. **Lock canon** — Define The Citadel, CASS, and Agent-Level Cognition in the KoadOS Core Contract. Get explicit approval.
2. **Resolve all 🔴 items** — Dark Mode local persistence format, data migration protocol, Tier 1 Zero-Trust enforcement, CLI command surface, EndOfWatch schema. All must be green before proceeding.
3. **Define the new CLI command surface** — `koad citadel *`, `koad agent *`, `koad signal *`, etc. Lock before any code.
4. **Define new proto schema** — Write `proto/citadel.proto` from scratch (clean service/message/RPC names). No legacy shim needed since we're not migrating the old binary.

### Phase 2 — Build the Citadel (Core OS)

*The foundation. Everything else depends on this.*

1. **New `koad-citadel` crate** — Session brokering, Personal Bay provisioning, Redis state management, Sector Locking, Watchdog/Sentinel. Built with Zero-Trust gRPC enforcement from day one. *(Addresses Tyr's "Tier 1 authorization loophole" — Sanctuary Rule enforced at the gRPC layer.)*
2. **Signal Corps & Event Bus** — Redis Streams integration, `koad:stream:*` schema, Signal Packet broadcasting.
3. **Trace ID system** — `TraceContext` on every gRPC call, `audit_trail` table, `koad dood pulse` + `koad dood inspect` + `koad watch --raw`.
4. **Workspace Manager** — Git Worktree orchestration for parallel agent isolation.

### Phase 3 — Build CASS (Cognitive Support)

*The brain. Memory, identity continuity, cognitive offload.*

1. **4-layer memory stack** — Redis Stack upgrade, SQLite (L2 episodic + L4 procedural), Qdrant deployment, Mem0 integration. Per the [Memory System Implementation Spec](https://www.notion.so/KoadOS-Agent-Memory-System-Implementation-Spec-be427a1ae4e24961b81d86479b7cb627?pvs=21).
2. **CASS MCP Server** — Expose `koad_intel_commit`, `koad_intel_query`, `koad_memory_hydrate`, `koad_status`, `koad_map_add`, `koad_session_save/restore`, `koad_context_archive`, `koad_signal_send/read`, `koad_hydrate_from`.
3. **Dark Mode persistence** — Implement standardized local save format (structured `.md` with TOML frontmatter) and CASS parser for reconnect reconciliation. *(Addresses Tyr's "Dark Mode persistence format" gap.)*
4. **EndOfWatch standardization** — Enforce structured schema for all EoW summaries (TOML frontmatter + markdown body). *(Addresses Tyr's "EndOfWatch standardization" gap.)*
5. **Data migration tool** — `koad system migrate-v5`: import extracted legacy knowledge into new CASS memory stack. *(Addresses Tyr's "data migration" gap.)*

### Phase 4 — Build `koad-agent` (Boot & Identity)

*The shell preparation tool. Body/Ghost model.*

1. **`koad-agent` CLI** — Inspect, prepare, set, clear modes. Context file generation, CLI config generation, CASS MCP wiring.
2. **Ghost config system** — Identity TOML loading, env var export, preflight validation.
3. **Context Hydration** — Three-tier progressive disclosure model. Dynamic `AGENTS.md` generation.

### Phase 5 — Integration & Documentation

*Wire everything together. Update all docs.*

1. **Agent instruction update** — Coordinated single pass across Sky, Tyr, Vigil instruction pages. All Spine references → Citadel/CASS vocabulary.
2. **Docs & pages audit** — Notion pages, GitHub issues, `AGENTS.md` files (documentation-only scrub — see Spine Terminology Retirement).
3. **Core Contract version bump** — v2.4 with new architecture canonical.
4. **Announce to KAI** — Tyr Brief + agent-level briefings.

### Phase 6 — Memory System Advanced (Gated)

*After Phase 5 is stable and approved.*

1. **Mem0 integration hooks** — Agent interaction loop, semantic cache, contradiction detection.
2. **A2A-S & TCH** — Agent-to-Agent Signal Protocol, Temporal Context Hydration.
3. **Agent Growth System** — Journals, introspection cycles, knowledge broadcasting.

### Phase 7 — Future (Gated)

*Requires Phase 6 stability and explicit approval.*

1. **Neo4j + Graphiti** — Knowledge graph evaluation.
2. **Micro-Swarm Hangar** — If/when the architecture supports it.
3. **TUI / Web Deck** — v6+ interface layer, if demand warrants.

---

## Portability & Shareability Initiative

> *This refactor is also an opportunity to make KoadOS cloneable and runnable by other developers.*
> 

The goal: strip all Ian/Skylinks-specific hardcoding from the core and make the system portable across general Linux environments, with WSL and system-specific configurations isolated to an optional layer.

### Design Principles

- **Core = Linux-first.** The base KoadOS runtime assumes a general Linux environment. No WSL-isms, no Skylinks paths, no personal credentials baked in.
- **Config-driven identity.** All system-specific values (paths, credentials, user identity, workspace IDs, agent names) are injected at boot via config files — never hardcoded.
- **Optional layers via configuration.** WSL support, Skylinks-specific integrations, and personal preferences are opt-in modules or config profiles — not part of the core.
- **Zero-trust defaults.** A fresh clone should fail gracefully if no config is present, with clear setup guidance rather than cryptic errors.

### What needs to be extracted

- [ ]  **Hardcoded paths** — `~/.koad-os/` structure should be configurable or at least documented as a convention, not an assumption.
- [ ]  **User identity values** — Notion user IDs, Koad Stream IDs, sync index UUIDs Wcurrently in Core Contract) must move to config.
- [ ]  **Workspace-specific IDs** — Koad Stream, Projects ID, memories ID, noti ID — all config-injectable.
- [ ]  **Skylinks-specific integrations** — Airtable, MemberPlanet, Stripe references should not live in core. Move to a `skylinks` config profile or integration layer.
- [ ]  **WSL-specific behavior** — Any WSL path handling, Windows interop, or shell quirks move to an optional `wsl` config layer.
- [ ]  **Agent names and roles** — Sky, Tyr, Vigil are Ian's agents. The core should support *named agents* generically; specific names are config.
- [ ]  **Credentials and secrets** — Already in Secret Manager/env vars per standards, but audit to confirm nothing is hardcoded anywhere.

### Config file structure (KoadConfig / TOML)

KoadOS uses the `KoadConfig` struct with a TOML-first, decentralized config system. All config lives under `~/.koad-os/config/` and is loaded hierarchically at boot.

```
~/.koad-os/
  config/
    kernel.toml          # Core system: ports, socket paths, timeouts
    filesystem.toml      # Workspace symlinks and data directory mappings
    registry.toml        # Service and agent registration
    integrations/        # Auto-loaded — add a .toml file to add an integration
      skylinks.toml      # Skylinks-specific integration (Airtable, Stripe, etc.)
      wsl.toml           # WSL optional layer
    identities/          # Auto-loaded — one .toml per agent or user identity
      sky.toml           # Sky's identity/ghost config
      tyr.toml           # Tyr's identity/ghost config
      vigil.toml         # Vigil's identity/ghost config
      ian.toml           # Operator (Ian) identity
    interfaces/          # Auto-loaded — UI/channel interface configs
  secrets/               # Managed externally (Secret Manager / env vars)
```

**Key properties of this system:**

- `integrations/`, `identities/`, and `interfaces/` are **auto-scanned** — drop a `.toml` file in to add a new agent, integration, or interface. No code changes required.
- Any config value can be **overridden by environment variable** using the `KOAD__` prefix and double underscores for nested keys (e.g. `KOAD__NETWORK__GATEWAY_PORT=4000`).
- The Sanctuary Rule is enforced by `validate_path` — agents are restricted to `KOAD_HOME` and registered project paths defined in config.

### Environment Variable Consolidation

All tokens, credentials, and runtime values currently accessed via bash environment variables must be inventoried, documented, and wired into the installation process. The goal: a developer who completes `koad init` has a fully populated env before the Citadel ever boots — no missing tokens, no cryptic startup failures.

**Principles:**

- The Citadel boot sequence performs a **preflight env check** — validates all required vars are present before attempting to start any service.
- Required vars are separated from optional vars. Missing a required var is a hard stop with a clear error. Missing an optional var is a warning.
- No token or path lives in both an env var *and* a config file — one source of truth. Convention: secrets and credentials stay in env vars (or Secret Manager); non-secret config lives in TOML files.
- `koad install` generates a `.env.template` (committed) and a `.env` (gitignored) so contributors know exactly what's needed without secrets being exposed.

**Installation flow (`koad install`):**

```jsx
koad install
  1. Detect environment (native Linux vs. WSL) — confirm with developer
  2. Confirm workspace paths, data directories, and KOAD_HOME location
  3. Prompt for each required env var / secret (with descriptions and validation)
  4. Write secrets to ~/.koad-os/.env; write non-secret config to kernel.toml, filesystem.toml
  5. Guide developer through creating their Captain Agent (their Tyr equivalent):
       a. Choose agent name, role, model
       b. Generate identities/<captain>.toml
       c. Provision Personal Bay for Captain Agent
  6. Run preflight check — validate all required vars, paths, and TOML files
  7. Exit with status: READY or NEEDS ATTENTION
```

**Env var categories to inventory:**

- [ ]  **Runtime paths** — Citadel binary, DB path, config dir, log dir
- [ ]  **API tokens** — Notion integration token, GitHub token, any external service tokens used by core
- [ ]  **Cloud / infra credentials** — GCP project ID, service account references, Secret Manager access
- [ ]  **Agent identity values** — User/workspace IDs currently in Core Contract (Koad Stream ID, noti ID, etc.)
- [ ]  **Redis / state store connection** — Host, port, auth (if applicable)
- [ ]  **Profile-specific tokens** — Airtable, Stripe, MemberPlanet (belong in the `skylinks` profile, not core `.env`)
- [ ]  **WSL-specific vars** — Any Windows interop paths or env vars (belong in `wsl` profile)

**`.env.template` approach:**

Non-secret config lives in TOML files. The `.env` / `.env.template` is reserved for **secrets and runtime overrides only**, using the `KOAD__` double-underscore prefix for any value that needs to override a TOML config key.

```bash
# KoadOS — environment variables
# Copy to ~/.koad-os/.env and fill in values. Never commit .env.
# Non-secret config belongs in ~/.koad-os/config/*.toml, not here.

# Core runtime (override TOML defaults only if needed)
KOAD_HOME=~/.koad-os
# KOAD__NETWORK__GATEWAY_PORT=4000   # example override syntax

# Secrets — these do NOT live in TOML
NOTION_TOKEN=            # required
GITHUB_TOKEN=            # required for project board sync
REDIS_AUTH=              # optional
GOOGLE_APPLICATION_CREDENTIALS=     # optional, GCP contexts

# Workspace identity (used during koad init; written to identities/ian.toml after setup)
# NOTION_USER_ID=          # bootstrapping only — move to ian.toml after init
# NOTION_WORKSPACE_ID=     # bootstrapping only — move to kernel.toml after init
```

**Open Questions — Env Vars:**

- [ ]  **Secret inventory audit** — Audit the codebase for all secret/token references to confirm they are `.env`-only and not duplicated in TOML files.
- [ ]  **Secret Manager vs. `.env` precedence** — For GCP-deployed contexts, define the load order: Secret Manager → env var → TOML default. Lock this before Citadel boot implementation.
- [ ]  **Shell profile injection** — Should `koad install` automatically append `source ~/.koad-os/.env` to `.bashrc` / `.zshrc`, or prompt the developer to confirm?
- [ ]  **Preflight failure behavior** — On a missing required secret, does the Citadel refuse to start entirely or start in DEGRADED mode with CASS offline?
- [ ]  **Unified Schema migration** — The system is transitioning toward a Unified Schema (merging env-driven legacy configs with TOML-first). Define what "complete" looks like and when to declare the migration done.

### Relationship to the Citadel Refactor

These two efforts are deeply compatible:

- The Citadel's boot sequence is the natural place to load and validate `KoadConfig` — `kernel.toml`, then `filesystem.toml`, `registry.toml`, then auto-scanned subdirs.
- CASS's memory hydration scope is config-driven per agent — defined in each agent's `identities/*.toml`.
- Agent identity and permissions are no longer hardcoded in instruction pages — they live in `identities/<agent>.toml`, making agents fully config-driven.
- The `KOAD__` env override pattern lets any TOML value be overridden at runtime without file edits — ideal for dev/staging/prod environment switching.
- The Sanctuary Rule (`validate_path`) integrates naturally with the Citadel's access control layer.

### Open Questions — Portability

- [x]  **Config schema** — **Resolved:** `koad.json` is deprecated. The canonical config system is KoadConfig / TOML files (`kernel.toml`, `filesystem.toml`, `registry.toml`, `identities/`, `integrations/`, `interfaces/`). No JSON config shapes to define or version.
- [x]  **Bootstrap/setup flow** — **Resolved:** The command is `koad install` (not `koad init`). It is an **interactive installation process** that confirms values, paths, and environments with the developer and guides them through full KoadOS + Citadel setup — including creation of their first agent, the **Captain Agent** (their equivalent of Tyr).
- [x]  **Profile activation** — **Resolved:** Handled during `koad install`. The installation script detects and confirms the environment with the developer, allowing them to select WSL or native Linux. Profile selection is a first-class step in the install flow, not a post-install configuration.
- [x]  **Core vs. contrib boundary** — **Resolved:** Core KoadOS = The Citadel and all systems required to support KoadOS agents, including code, documentation, agent instructions, and CLIs for working manually within KoadOS. Personal integration layers (Notion, Airtable, pimpedbash, etc.) are contrib. Anything the Citadel or Koados Agents need to function is core.
- [ ]  **README / onboarding doc** — The repo will need a proper README for external developers. Who writes it and when in the sequence?
- [ ]  **Audit scope** — Full codebase audit for hardcoded values before any shareable release.

### Portability Sequencing (Proposed)

1. **Define TOML bootstrap templates** — Create starter `kernel.toml`, `filesystem.toml`, `registry.toml` for `koad install` to scaffold. Approve before implementation.
2. **Audit codebase** — Identify every hardcoded value, legacy `koad.json`/`identity.json` reference, and non-TOML config pattern.
3. **Extract to TOML** — Move hardcoded values into the appropriate config file (`kernel`, `filesystem`, `registry`, or the relevant `integrations/`/`identities/` file).
4. **Build optional integration layer** — `integrations/skylinks.toml` and `integrations/wsl.toml` as drop-in files.
5. **`koad install` flow** — Implement the interactive installer: environment detection (WSL/Linux), confirm values/paths, prompt for secrets, write `.env`, scaffold `~/.koad-os/config/`, create Captain Agent, run preflight.
6. **README** — Write external-facing onboarding doc after `koad install` is locked.
7. **Test on clean environment** — Validate a fresh clone + `koad install` produces a READY Citadel boot with a provisioned Captain Agent bay.

---

## Codebase Reference — Implementation Facts (Vigil)

<aside>
🔍

Source: [Vigil — CLAUDE.md](https://www.notion.so/Vigil-CLAUDE-md-321fe8ecae8f806780abd3c65016de36?pvs=21) (generated 2026-03-10). Read in full before touching code. This section summarizes the facts from that doc that directly affect refactor execution.

</aside>

The old Spine codebase is **archived for reference only** — no code is being renamed or migrated. The concepts below were implemented in Rust (Cargo workspace, 8 crates) and serve as architectural reference for the from-scratch Citadel build. Understanding what the old system did informs the new design; the code itself is not carried forward.

### Concept → crate map (archived reference)

<aside>
📦

**Old crates archived — not migrated.** The table below maps old Spine-era crates to their conceptual equivalents in the new architecture. No code is carried forward; the new Citadel, CASS, and koad-agent are built from scratch. This map serves as a design reference for what functionality each new component must provide.

</aside>

| **Concept** | **Current crate / file** | **Binary name** | **New build equivalent (reference only)** |
| --- | --- | --- | --- |
| The Spine | `crates/koad-spine/` *(archived)* | `kspine` *(retired)* | The Citadel — new `koad-citadel` crate built from scratch |
| gRPC service definitions | `proto/spine.proto` *(archived)* | — | New `proto/citadel.proto` written from scratch (Phase 1.4) |
| ASM (→ CASS sub-system) | `crates/koad-asm/` *(archived)* | `koad-asm` *(retired)* | Archived. New CASS built from scratch with ASM concepts integrated. |
| Memory hydration / Sentinel (→ CASS) | `crates/koad-spine/src/engine/storage_bridge.rs` | — | CASS cognitive continuity backbone — design reference for new build |
| Identity / Atomic Lease | `crates/koad-spine/src/engine/identity.rs` | — | Citadel session brokering — design reference for new build |
| CIP (Cognitive Integrity Protocol) | `crates/koad-spine/src/engine/storage_bridge.rs:286` | — | Remains in Citadel — enforces write restrictions on sovereign Redis keys for Tier 2+ agents |
| Watchdog / Sentinel | `crates/koad-watchdog/` | `koad-watchdog` | Citadel sub-system — no rename required |
| CLI | `crates/koad-cli/` | `koad` | Unchanged |

### Proto definitions — fresh build, no migration

<aside>
✅

**No longer a sequencing concern.** With a from-scratch rewrite, `proto/citadel.proto` is written new with clean service/message/RPC names (Phase 1.4). The old `proto/spine.proto` is archived — no rename, no regen, no breaking API change to manage. The proto migration complexity that was a major risk in the incremental approach is completely eliminated.

</aside>

### Vigil's hardcoded values inventory — config key audit complete

Vigil has completed the Spine config key audit (see [Vigil — CLAUDE.md](https://www.notion.so/Vigil-CLAUDE-md-321fe8ecae8f806780abd3c65016de36?pvs=21) → Hardcoded Values Registry). Critical unresolved items from that audit:

- 🔴 **`sandbox.rs:15`** — `"GITHUB_ADMIN_PAT"` hardcoded. Grants unconditional command execution to any agent holding that key. **Critical risk** — must be moved to identity `access_keys` check before any portability work.
- 🔴 **`sandbox.rs:50-53, 99, 111`** — Production trigger strings, command blacklist, and protected paths all hardcoded. Target: `kernel.toml [sandbox]` section.
- 🟡 **`constants.rs`** — Ports, addresses, socket paths (Medium risk — well-inventoried, straightforward to migrate).
- ✅ **Hardcoded agent names** (`"Tyr"`, `"Dood"`, `"Vigil"`, `"Ian"` in `boot.rs` and `identity.rs`) — resolved 2026-03-11. Authorization now derives from `identity.rank`, not name strings.

### `personas/` vs `config/identities/` — directory structure discrepancy

The Flight Manual (§7.1) describes a **Personas** (`personas/`) and **Interfaces** (`bodies/`) directory structure distinct from the `config/identities/` path documented in [CLAUDE.md](http://CLAUDE.md). The [CLAUDE.md](http://CLAUDE.md) workspace structure only shows `config/identities/`. This discrepancy must be resolved before the refactor touches identity files:

- [ ]  **Open question:** Are `personas/` and `bodies/` a planned future directory structure, an older naming convention being retired, or a parallel layer that coexists with `config/identities/`? Define canonical path before any identity-related refactor work.

### Other implementation facts relevant to this refactor

- **`koad.json` fully removed** — confirmed by Vigil. The TOML config system is the only config path. This closes the portability concern about legacy JSON config.
- **CIP already protects sovereign keys** — Tier 2+ agents cannot write `identities`, `identity_roles`, `knowledge`, `principles`, `canon_rules`. However, Tier 1 agents have **no write restrictions** — flagged by Vigil as a security gap. Relevant to Personal Bay design.
- **Redis key namespace** — keys use `koad:state`, `koad:kai:{name}:lease`, `koad:session:{session_id}` patterns. Check for any embedded "spine" strings during the scrub.
- **Spine gRPC endpoint (`:50051`) is unauthenticated** — any local process can call it. [Localhost](http://Localhost)-only binding mitigates this, but flagged by Vigil for any networked deployment context.

---

## Spine Terminology Retirement — Systematic Scrub

<aside>
⚠️

**Official declaration:** The Koad Spine is retired. We are building The Citadel — a self-healing, always-on operating system. No new code, docs, or communications should reference the old Spine concept. All legacy references must be found and replaced.

</aside>

### Terms to eliminate

The following terms are deprecated and must be scrubbed from all surfaces:

- `Koad Spine` / `The Koad Spine`
- `k-spine` / `kspine`
- `spine` (when used to refer to the old session/tethering system — context matters; anatomical/architectural uses elsewhere are fine)
- Any variable names, function names, file names, or CLI commands containing `spine`
- Any config keys referencing `spine`
- Any log output or error messages referencing `spine`

### Replacement vocabulary

| **Old term** | **Replace with** | **Notes** |
| --- | --- | --- |
| The Koad Spine / The Spine | The Citadel | For OS/infra references |
| Spine (cognitive/session layer) | CASS | For memory hydration, session support, and cognitive offload tooling |
| Spine session tethering | Citadel session brokering |  |
| Lost Spine connection | Disconnected from the Citadel |  |
| Spine restart / Spine down | Citadel restart / Citadel offline |  |
| k-spine | citadel (lowercase for CLI/code contexts) | Update any CLI commands or binaries |

### Audit surfaces — full scrub checklist

#### Notion (docs & canon)

- [ ]  KoadOS Core Contract — search for "Spine", "k-spine"
- [ ]  KoadOS Global Canon & Rules of Engagement
- [ ]  KoadOS Contributor & Coding Manifesto
- [ ]  All agent instruction pages (Sky, Tyr, Vigil, Koad CLI agent)
- [ ]  All Tyr Briefs
- [ ]  [AGENTS.md](http://AGENTS.md) files in Notion
- [ ]  SLE Deep Review doc
- [ ]  Any "Deep Reflection" or "Introspection" protocol pages
- [ ]  Notes and session close logs (historical — may leave as-is with a note, or update)

#### GitHub (issues & project board)

- [ ]  GitHub issue titles and descriptions — retitle or close old Spine-referenced issues
- [ ]  GitHub Project #2 board — column names, item labels, descriptions
- [ ]  README and any markdown docs in the repo (for Spine terminology)

<aside>
📦

**Old codebase archived — scrub is docs-only.** The `crates/koad-spine/`, `crates/koad-asm/`, `proto/spine.proto`, and all related code are archived in Phase 0. No codebase grep, binary rename, proto regeneration, CLI command migration, or infrastructure updates are needed — the new Citadel/CASS/koad-agent are built from scratch with clean naming. This scrub applies to **Notion docs, agent instructions, and GitHub issues only**.

</aside>

### Scrub execution protocol

1. **Do NOT do a blind find-and-replace.** Each occurrence must be reviewed for context — some may need to become "Citadel", some "CASS", some "agent-local cognition".
2. **Document every replacement** in a scrub log (can be a sub-page or GitHub issue) so the change surface is traceable.
3. **GitHub issue cleanup** — retitle or close old Spine-referenced issues under Citadel vocabulary. Track in the scrub log.
4. **Notion doc updates** are a single coordinated pass, not piecemeal edits.
5. **Agent instruction updates** are a simultaneous pass across all agents — not sequential, to avoid agents operating on mixed vocabulary.
6. **After scrub is complete**, add a note to the KoadOS Core Contract deprecating the old terms permanently, so future contributors know they are not valid.

### Open Questions — Scrub

- [ ]  **CLI command migration** — If `koad spine *` commands exist, what is the new command surface? `koad citadel *`? Define before scrubbing so replacements are consistent.
- [ ]  **Historical logs** — Do session close logs and Notes pages with old terminology need to be updated, or are historical records left as-is with a dated deprecation note at the top?
- [ ]  **GitHub issues** — Close old Spine-labeled issues and re-open under Citadel terminology, or just retitle in place?
- [ ]  **Scrub log format** — Simple Notion page, GitHub issue, or tracked in the Tyr Briefs database?

---

## Body/Ghost Agent Boot Model

> *A fundamental reframe of how agents come online. The body is the prepared shell. The ghost is the identity loaded into it.*
> 

### Concept

`koad-agent` is **not** a process wrapper that runs the AI runtime inside it. It is a **shell session preparation and validation tool**. The body is the current bash shell — `koad-agent` inspects it, sets the required session variables, loads the ghost's identity context into the environment, and reports readiness. Once the shell is prepared, you launch the AI CLI (Gemini, Codex, etc.) directly in that shell. The AI inherits the prepared environment and operates within it.

- **The Body** — the prepared bash shell. `koad-agent` prepares it. The AI CLI inherits it.
- **The Ghost** — a named config that defines which env vars to set, which instruction file to reference, which model/CLI to use, and what memory scope to load.
- **`koad-agent`** — a CLI tool (not a daemon, not a wrapper) that inspects and prepares the shell for a named ghost, then exits. Boot the AI CLI after it reports READY.
- **One Body One Ghost** — a hard, enforced constraint. A terminal session can host exactly one active agent at a time. Checked via `KOAD_SESSION_ID` + a Redis atomic lease. If `KOAD_SESSION_ID` is already set and alive in Redis, a second `koad boot` is rejected. Do not attempt to run two agents in the same shell.

Thematically: you pilot the empty body (the shell), you call for a ghost (`koad-agent --ghost sky`), it prepares the body to receive her, then you launch the AI CLI to bring her online.

### Boot sequence

<aside>
🔬

**Updated per** [Agent Boot Research — CLI Context Injection Patterns](https://www.notion.so/Agent-Boot-Research-CLI-Context-Injection-Patterns-0f4b35fc93b54fdca9d1bd63537502ae?pvs=21). The boot sequence now includes context file generation, CLI config generation, and optional CASS MCP server wiring — the three highest-impact additions identified by the research.

</aside>

```bash
# Step 1 — Prepare the shell for a named ghost
eval $(koad-agent --ghost sky --export)
# Reads identity TOML, exports env vars, generates context files,
# wires CLI config + MCP, hydrates session context, runs preflight.
# Reports READY / DEGRADED / NOT READY.

# Step 2 — Launch the AI CLI directly in the prepared shell
gemini   # or: codex, claude — whatever Sky's ghost config specifies
```

**Detailed prepare flow (all steps run by `koad-agent --ghost <name>`):**

1. **Read identity TOML** — `~/.koad-os/config/identities/<name>.toml`
2. **Export env vars** — `KOAD_AGENT_NAME`, `KOAD_HOME`, `KOAD_SESSION_ID`, etc.
3. **Generate context files** based on target CLI:
    - `AGENTS.md` — **always generated** (universal; covers Codex + Gemini natively)
    - `CLAUDE.md` — generated when `model = "claude"`
    - Content: agent identity + Prime Directives (condensed) + project scope + filesystem map + Development Canon references + session continuity notes from last session
4. **Generate CLI config files:**
    - `.claude/settings.local.json` — permissions, hooks (Sanctuary Rule, KSRP), MCP servers
    - `.gemini/settings.json` — model, tools, MCP servers
    - `.codex/config.toml` — model, approval policy, MCP servers
5. **Wire CASS MCP server** into CLI config (if Citadel reachable) — gives agent live, interactive access to CASS tools during the session (see CASS MCP section)
6. **Hydrate session context** (if Citadel reachable) — pull last session notes from CASS + relevant memories from Qdrant → append to generated context file
7. **Generate system prompt append file** — for Claude's `--append-system-prompt` (condensed identity + directives, not full project context)
8. **Run preflight validation** — required env vars present? Context files generated? CASS reachable? One Body One Ghost lease available?
9. **Report status** — READY / DEGRADED / NOT READY + suggested launch command

### Docking State Machine — Formal Agent Lifecycle

<aside>
🔬

**Added per** [Docs Gap Analysis — .koad-os/docs vs Refactor Plan](https://www.notion.so/Docs-Gap-Analysis-koad-os-docs-vs-Refactor-Plan-8f937a7abc9248308dad10fdb52d7d8c?pvs=21). The boot sequence above describes the *happy path*. The Docking State Machine formalizes all states including failure, dark mode, and teardown — catching edge cases the linear flow misses. Source: v5/AGENT_CHASSIS.

</aside>

```
DORMANT → DOCKING → HYDRATING → ACTIVE → WORKING → DARK → TEARDOWN → DORMANT
```

| **Transition** | **Trigger** | **What happens** |
| --- | --- | --- |
| DORMANT → DOCKING | `koad-agent --ghost <name>` invoked | Shell preparation begins; identity TOML loaded; env vars exported |
| DOCKING → HYDRATING | Lease Acquired (Redis atomic lease succeeds) | CASS begins memory hydration; context files generated; MCP wired |
| HYDRATING → ACTIVE | Context Injected (preflight passes, context files written) | Shell is READY; AI CLI can be launched |
| ACTIVE → WORKING | AI CLI launched and first interaction begins | Agent is online and processing tasks |
| WORKING → DARK | Heartbeat Lost (>30s without heartbeat ACK) | Agent continues locally; buffers work for CASS sync on reconnect |
| DARK → WORKING | Heartbeat Restored | CASS reconciles local saves with bay state; agent resumes connected operation |
| DARK → TEARDOWN | Timeout (>5m in DARK state) | Citadel initiates teardown; agent session is considered lost |
| WORKING → TEARDOWN | Clean shutdown (`koad-agent clear` or CLI exit) | EndOfWatch summary generated; session state saved to bay |
| TEARDOWN → DORMANT | Brain Drain Complete | All session state persisted to CASS L2; lease released; bay returns to STANDBY |

**Brain Drain Protocol (TEARDOWN):**

Before an agent session fully closes, the system ensures no knowledge is lost:

1. Auto-generate EndOfWatch summary (session learnings, decisions, blockers, next steps)
2. Flush working memory (L1 Redis) to episodic store (L2 SQLite)
3. Commit any pending `koad_intel_commit` entries
4. Persist filesystem map updates to Personal Bay
5. Release Redis atomic lease
6. Bay transitions to STANDBY — ready for next boot

**DARK state recovery:** When an agent enters DARK state, it continues working locally using Agent-Level Cognition. On reconnect (DARK → WORKING), CASS performs a reconciliation diff: local `.md` saves are merged with bay state, and any conflicting writes surface as `MEMORY_CONFLICT` events.

### Context Hydration Architecture — Three-Tier Model

<aside>
🧊

**Added per** [Context Hydration Architecture — Research & Design](https://www.notion.so/Context-Hydration-Architecture-Research-Design-721fbb7bc1274186b4c49b1678ee8905?pvs=21). Research confirms dynamic context generation with tiered lazy-loading is the industry-standard pattern. Static context files are actively harmful — context rot degrades recall as token count increases.

</aside>

KoadOS uses a **three-tier progressive disclosure model** for context files, inspired by Claude Code's Skills pattern and Anthropic's context engineering research:

- **Tier 1 — Always-Loaded Core (<2,000 tokens):** Agent identity, condensed Prime Directives, One Body One Ghost, Sanctuary Rule summary, active session continuity note, and a table of contents for Tier 2/3 content.
- **Tier 2 — Task-Relevant Context (3,000–8,000 tokens):** Loaded by `koad-agent` based on ghost config — station conventions, Development Canon, filesystem map, recent CASS session notes. Scope-driven by `memory_scope` in the identity TOML.
- **Tier 3 — On-Demand Deep Context (0 tokens at boot):** Never in the boot file. Agent pulls via CASS MCP tools (`koad_memory_hydrate`, `koad_intel_query`) when the task requires it — full Prime Directives, Core Contract sections, historical task outcomes, cross-agent `koados_knowledge`.

`koad-agent` **generates** `AGENTS.md` dynamically on every boot from TOML templates + live CASS data. The file is never stored persistently — it's always fresh. Token budget is controlled by `max_boot_tokens` in the identity TOML `[context]` section.

**Context lifecycle in-session:**

1. **Observation masking** (primary) — old tool outputs archived to CASS L2, replaced with 1-line summaries. JetBrains/NeurIPS 2025 research found this outperforms LLM summarization in 4/5 tests at >50% cost reduction.
2. **Structured note-taking** — agent writes session state to Personal Bay via `koad_session_save`. Notes survive CLI compaction events and are re-injected by a post-compaction hook.
3. **CLI compaction** (safety net) — handled by the AI CLI itself at ~95% capacity. KoadOS provides a **post-compaction recovery hook** that re-injects Tier 1 identity + latest session notes.
4. **Sub-agent delegation** — maps to the Micro-Swarm Hangar. Deep exploration runs in isolated context windows; only condensed results return to the parent agent.

**What-to-keep decision signals:** Recency (rolling window, last 10 turns full), Semantic relevance (Mem0 cosine similarity against Qdrant), Importance score (0.0–1.0 per memory, architectural decisions = 0.9+, routine observations = 0.1–0.3), Structural position (critical info at top of context, not middle — "lost in the middle" effect).

### What `koad-agent` actually does

**Inspect mode** (`koad-agent inspect` or `koad-agent status`):

- Reads the current shell environment
- Checks which required vars are set, which are missing or empty
- Validates values where possible (e.g. token format, path existence)
- Reports a health summary: READY / DEGRADED / NOT READY
- Shows which ghost (if any) the shell is currently prepared for

**Prepare mode** (`koad-agent --ghost <name>`):

- Reads the named ghost config from `~/.koad-os/config/identities/<name>.toml`
- Outputs export statements for required session variables
- **Generates context files** — `AGENTS.md` (always) + `CLAUDE.md` (if model=claude). Content is assembled from identity TOML + instruction file + CASS session notes (if reachable)
- **Generates CLI config files** — `.claude/settings.local.json`, `.gemini/settings.json`, or `.codex/config.toml` as appropriate, with MCP server config, permission sets, and hooks
- **Wires CASS MCP server** into the generated CLI config (if Citadel reachable)
- Runs a preflight check and reports result
- Does NOT launch the AI CLI — that is always an explicit manual step

**Set mode** (`koad-agent set <VAR> <value>`):

- Sets a single session variable with optional validation
- Useful for manual corrections without re-running a full ghost prepare

**Clear mode** (`koad-agent clear`):

- Unsets all KoadOS session variables from the current shell
- Resets to an empty, unprepared state

### Why this model is correct

- **No unnecessary complexity.** `koad-agent` is a shell tool, not a daemon or runtime wrapper. It prepares the environment. The AI CLI handles everything else.
- **Native to the shell.** Env vars are the universal interface between the OS and any CLI. Gemini, Codex, or any future model CLI picks them up automatically on launch.
- **Context files are the primary identity mechanism.** Research confirmed that env vars provide metadata, but context files (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`) are where 80% of agent identity actually lands — they are read as high-priority instructions before any user interaction. Generating these is the single highest-impact boot step.
- **Cross-CLI by default.** The `AGENTS.md` open standard is natively supported by Codex CLI (primary) and Gemini CLI (configurable). Generating a universal `AGENTS.md` on every boot means a ghost-prepared shell works with *any* supported CLI — the agent can switch CLIs mid-project without re-booting.
- **Ghost is portable and swappable.** Swap ghosts by running `koad-agent --ghost tyr` before your next AI CLI launch. No restart, no process management.
- **Citadel independence.** Shell preparation works with no Citadel connection. CASS integration is additive — if the Citadel is reachable, `koad-agent` can pull memory context from CASS during preparation and wire the CASS MCP server for live in-session access.
- **Aligns with Prime Directives.** Simplicity over complexity. Native tech. Programmatic-first. The shell *is* the interface.

### Ghost config format (TOML — `identities/<name>.toml`)

Ghost configs are standard KoadConfig identity files living in `~/.koad-os/config/identities/`. They are auto-loaded at Citadel boot and read by `koad-agent` during shell preparation. No separate `ghosts/` directory — identities *are* ghosts.

```toml
# ~/.koad-os/config/identities/sky.toml

[identity]
name = "Sky"
role = "Officer"
model = "gemini"                # Target CLI: gemini | claude | codex
instructions = "~/.koad-os/config/identities/sky.instructions.md"
memory_scope = "sle"

[context]
# Context file generation — assembled by koad-agent during prepare
# Content is built from: identity fields + instructions file + CASS session notes
prime_directives = "~/.koad-os/config/shared/prime-directives.md"
dev_canon = "~/.koad-os/config/shared/development-canon.md"
# Additional files to @include in the generated context file
includes = ["~/.koad-os/config/shared/koados-glossary.md"]

[env]
# Vars to export into the shell when this ghost is prepared
KOAD_AGENT_NAME = "Sky"
KOAD_AGENT_ROLE = "Officer"
KOAD_AGENT_INSTRUCTIONS = "~/.koad-os/config/identities/sky.instructions.md"
KOAD_MEMORY_SCOPE = "sle"

[requires]
# Secrets that must be present in the shell before the AI CLI is launched
vars = ["NOTION_TOKEN", "GITHUB_TOKEN", "KOAD_HOME"]

[optional]
vars = ["GCLOUD_PROJECT"]

[hooks]
# CLI hooks generated during prepare (Claude Code hooks, Gemini tool approval)
pre_tool_use = "~/.koad-os/hooks/sanctuary-check.sh"  # Sanctuary Rule enforcement
post_edit = "~/.koad-os/hooks/ksrp-lint.sh"            # KSRP pass-1 after code edits
```

### Context file hierarchy — three-tier model

All three CLIs support hierarchical context discovery (global → project → subdirectory). KoadOS exploits all three tiers:

- **Global** (`~/.koad-os/config/shared/AGENTS.md`) — agent identity, KoadOS canon, Prime Directives. Generated by `koad-agent` during ghost prepare.
- **Project / Station** (`~/projects/<station>/AGENTS.md`) — station-specific context, tech stack, project conventions. Maintained by the station's owning agent or Ian.
- **Subdirectory / Module** (`~/projects/<station>/src/<module>/AGENTS.md`) — module-specific rules, patterns, constraints. Maintained as needed.

The global file is auto-generated on every ghost boot. Project and subdirectory files are persistent and manually maintained — they survive across ghost swaps and CLI changes.

### Relationship to the Citadel and CASS

- `koad-agent` wires the **CASS MCP server** into the generated CLI config during preparation, giving the agent live tool access to CASS during the session.
- `koad-agent` also hydrates memory context from CASS into the generated context file at boot, providing a warm start.
- If the Citadel is unreachable, `koad-agent` prepares from local ghost config only, omits MCP config, and notes DEGRADED in its report.
- The Citadel can optionally be notified of a successful ghost preparation so Tyr can track which ghosts are active across shells.

### Open Questions — Body/Ghost Model

- [ ]  **Shell export mechanism** — `koad-agent` can't export vars into the parent shell from a subprocess by default. Canonical solution: `eval $(koad-agent --ghost sky --export)` or a sourced script. Define the standard invocation pattern.
- [x]  **Ghost config location** — `~~~/.koad-os/ghosts/~~` Resolved: ghost configs are identity TOML files in `~/.koad-os/config/identities/`. Auto-loaded by KoadConfig. No separate `ghosts/` directory needed.
- [x]  **CASS integration depth** — **Resolved per boot research:** Both. `koad-agent` hydrates memory context into the generated context file at boot (warm start), AND wires the CASS MCP server into the CLI config for live in-session access. Two complementary channels: static context at boot + dynamic tools during session.
- [ ]  **Citadel registration** — Is ghost registration opt-in (`--register` flag), automatic on prepare, or not done at this layer?
- [ ]  **Inspect output format** — Plain text, JSON (for scripting), or a rich terminal display? Should support both human and machine consumers.
- [ ]  **Context file placement** — Does `koad-agent` write the generated `AGENTS.md` / `CLAUDE.md` to the project root (where the CLI expects it) or to a KoadOS-managed path with a symlink? Symlink is cleaner but may confuse some CLIs.
- [ ]  **Context file cleanup on ghost clear** — When `koad-agent clear` runs, should it delete the generated context files and CLI configs, or leave them as stale artifacts?
- [ ]  **CASS MCP server port** — Fixed port from `kernel.toml`, or dynamically assigned and injected into CLI config? Fixed is simpler; dynamic avoids port conflicts with multiple KoadOS instances.
- [ ]  **Hooks implementation scope** — Which hooks should ship in v1? Sanctuary Rule check (pre-tool-use) is high priority. KSRP lint (post-edit) and GitHub Issue enforcement (pre-commit) are medium. Define the v1 hook set.

---

## 🛳️ Personal Bay Model — Agent-Dedicated Citadel Slots

> *Each crew member has their own bay. When they come aboard, their berth is already prepared.*
> 

### Concept

A **Personal Bay** is a dedicated, pre-allocated slot within the Citadel reserved for a single named agent. It is not a generic connection — it is a named, persistent structure that exists whether or not the agent is currently online. When the agent boots and connects, they are stepping into their own bay, not competing for a generic slot. This eliminates an entire class of boot and connection errors that arise from dynamic resource allocation at connect time.

The bay is provisioned when the agent is registered (via their `identities/<agent>.toml`) and persists across sessions. It is the agent's home inside the Citadel.

### What lives in a Personal Bay

| **Bay Component** | **What it contains** | **Notes** |
| --- | --- | --- |
| 🔌 Dedicated connection slot | A reserved, named socket/channel in the Citadel for this agent's exclusive use | Pre-allocated at registration; not shared. Agent always connects to the same slot. |
| 🧠 Cognition context | Agent's working memory scope, active task state, session continuity buffer | Hydrated by CASS on connect; persisted here between sessions |
| 🗺️ Personal filesystem map | Indexed list of directories the agent is assigned to or has added themselves for fast traversal | Mutable — agent can register new paths during a session. Persisted in the bay. |
| 🔑 Credential vault (scoped) | Agent-allowed PATs, API tokens, service keys — only the credentials this agent is authorized to use | Not the global Secret Manager; a scoped view. No agent can access another agent's credentials. |
| 🛠️ Tool manifest | The specific set of tools and capabilities this agent is authorized to invoke | Defined in identity TOML; enforced by the Citadel at the bay level |
| 📋 Session log | Per-agent session history, last-seen state, disconnect/reconnect events | Used by CASS for reconnect reconciliation; visible to Tyr for crew tracking |
| 🏥 Health record | Last boot status, preflight results, error history for this agent specifically | Allows Citadel to detect patterns (e.g. agent that always fails on token X) |

### Why dedicated bays reduce boot errors

Most boot and connection errors today occur because the system attempts to resolve agent identity, permissions, and connection parameters *at boot time* from a cold start. A Personal Bay inverts this:

- **Static allocation beats dynamic resolution.** The bay already knows who this agent is, what they're allowed to do, and where they connect. Boot becomes *validation*, not *discovery*.
- **Preflight is bay-scoped.** `koad-agent` can validate the agent's env against the bay's known requirements before ever touching the Citadel network layer.
- **Errors are attributable.** When something fails, it fails on `sky`'s bay, not on a generic agent slot. Tyr can see exactly which bay is unhealthy.
- **Reconnect is deterministic.** CASS re-hydrates from the same bay every time — no ambiguity about which session state belongs to which agent.

### Bay lifecycle

```
koad init (first time)
  └─ identity TOML registered
       └─ Citadel provisions Personal Bay for agent
            └─ Bay persists (empty) until agent first connects

koad-agent --ghost <name>
  └─ Prepares shell from identity TOML
       └─ Optionally contacts Citadel to signal bay activation

Agent AI CLI launches
  └─ Connects to dedicated bay slot
       └─ CASS hydrates cognition context from bay
            └─ Agent comes online: READY

Agent disconnects
  └─ Bay saves session state, task buffer, filesystem map updates
       └─ Connection slot returns to STANDBY (reserved, not released)

Agent reconnects
  └─ Connects to same reserved slot
       └─ CASS reconciles any local saves with bay state
            └─ Agent resumes: RESTORED
```

### Relationship to identity TOML

The `identities/<agent>.toml` file is the **provisioning spec** for the bay. It defines what the bay should contain. The bay itself is the **live runtime instance** of that spec inside the Citadel.

- Identity TOML = the blueprint (static, on disk, version-controlled)
- Personal Bay = the built structure (dynamic, in the Citadel, persisted across sessions)
- Changes to the TOML are applied to the bay on next Citadel boot or via `koad citadel refresh-bay <agent>`

### Filesystem map — personal and persistent

Each agent's bay includes a mutable **personal filesystem map**: an indexed set of paths the agent can fast-traverse without a full filesystem walk. This map has two sources:

1. **Assigned paths** — defined in the identity TOML under `[filesystem]`. Set by Ian or Tyr when the agent is provisioned. Example: Sky is assigned `~/projects/koad-os/` and `~/skylinks/`.
2. **Self-registered paths** — the agent can add paths to their own map during a session (e.g. `koad map add ~/new-project/`). These are persisted in the bay and survive session restarts.

No agent can read or write to another agent's filesystem map. The Sanctuary Rule (`validate_path`) still applies — an agent cannot map a path outside their permitted scope.

### Credential scoping in the bay

The bay's credential vault is a **scoped view** of the system's secrets, not a copy of them. At bay provisioning time, the Citadel's credential broker is told which secret keys this agent is authorized to access. When the agent requests a credential, the broker checks the bay's scope and returns only permitted values.

This means:

- Sky can never accidentally (or intentionally) access Vigil's PATs, even if both agents are online simultaneously.
- Adding a new token for an agent is done by updating their identity TOML and refreshing the bay — not by modifying a shared credential store.
- Credential scope is auditable per-agent from Tyr's position.

### Heartbeat & Lease Engineering Constraints

<aside>
💓

**Source:** heartbeat_concept.research (`.koad-os/docs`). These constraints apply to the Citadel's session brokering and the Docking State Machine's heartbeat-based transitions.

</aside>

- **Push model** — the agent emits heartbeats to the Citadel (not Citadel polling agents). Lower latency, scales better.
- **Isolated heartbeat thread** — the heartbeat sender MUST run on its own thread/task, not on the agent's main work thread. If the agent is blocked on a long tool call, the heartbeat must still fire.
- **Timeout = 2–3× interval minimum** — if the heartbeat interval is 10s, the DARK threshold must be ≥20s. Avoids false death declarations from network jitter.
- **Consecutive miss threshold** — require 2–3 consecutive missed heartbeats before declaring DARK state. A single missed beat is noise, not signal.
- **Monotonic sequence counter** — each heartbeat includes a monotonically increasing counter. Detects out-of-order and duplicate beats.
- **Session-ID-aware** — heartbeats carry the `KOAD_SESSION_ID`. The Citadel distinguishes a reboot (new session ID) from a continuation (same session ID).
- **Explicit deregistration on clean shutdown** — on `koad-agent clear` or clean CLI exit, the agent sends an explicit deregistration signal. Timeout-based teardown is only for crashes and unexpected disconnects.

### Open Questions — Personal Bay Model

- [ ]  **Bay storage backend** — Where does bay state live? Redis (volatile, fast) or a persistent store (SQLite, file-based)? Affects reconnect reliability after a full Citadel restart.
- [ ]  **Bay provisioning trigger** — Is a bay auto-provisioned when a new `identities/*.toml` is detected at boot, or does it require an explicit `koad citadel provision-bay <agent>` command?
- [ ]  **Filesystem map format** — Simple list of paths, or a structured index with metadata (last accessed, depth limit, purpose tag)? The latter enables smarter traversal but adds complexity.
- [ ]  **Credential vault implementation** — Thin broker pattern (bay holds references, Citadel resolves at request time) or a cached copy? Thin broker is safer; cached copy is faster.
- [ ]  **Bay isolation enforcement** — How does the Citadel prevent cross-bay access at runtime? Network layer (separate channels), application layer (auth tokens per bay), or both?
- [ ]  **Bay for Ian (operator)** — Does the operator (Ian) get a Personal Bay too, or is the operator a special case with elevated access outside the bay model?
- [ ]  **Multi-session agents** — If the same ghost is booted in two shells simultaneously, do they share a bay or get separate sub-slots? Define the expected behavior.

---

## 🗂️ Workspace Manager — Git Worktree Orchestration

> *Physical isolation for parallel agent work. Two agents, same repo, zero conflicts.*
> 

<aside>
🔍

**Added per** [Docs Gap Analysis — .koad-os/docs vs Refactor Plan](https://www.notion.so/Docs-Gap-Analysis-koad-os-docs-vs-Refactor-Plan-8f937a7abc9248308dad10fdb52d7d8c?pvs=21). Without physical workspace isolation, two agents editing the same repo will corrupt each other's git state. Source: Sweep 07, v5/IMPLEMENTATION_PHASES Phase 4.

</aside>

### Concept

The Workspace Manager is a Citadel service that provisions isolated Git Worktrees for agent tasks. Each agent works in a physically separate checkout of the repo — no shared working directory, no merge conflicts from concurrent edits, no accidental `git stash` collisions.

### Worktree Lifecycle

```
Agent receives task (e.g. GitHub Issue #118)
  └─ Citadel validates: repo exists? agent has access? branch clean?
       └─ Creates worktree: ~/.koad-os/workspaces/{agent_name}/{task_id}/
            └─ Mounts worktree path to agent's Personal Bay filesystem map
                 └─ Agent is jailed to worktree path (Sanctuary Rule enforced)

Agent completes work
  └─ Signals completion → PR created from worktree branch
       └─ Review + merge (Ian approval required per Development Canon)
            └─ Worktree removed; bay filesystem map updated
```

### Worktree Namespace

```
~/.koad-os/workspaces/
  sky/
    issue-118/          # Worktree for Sky working on issue #118
    issue-125/          # Sky can have multiple concurrent worktrees
  vigil/
    issue-120/          # Vigil's isolated workspace
```

### Redis Tracking

Each active worktree is registered in Redis for Citadel visibility:

- Key: `koad:workspaces:{path_hash}`
- Value: `{ agent, issue_id, trace_id, created_at, branch }`
- TTL: None (explicit cleanup only)

### Security: SLE Sandbox Worktrees

Worktrees for the SLE (Skylinks Live Environment) or any production-adjacent project get a `.env.sandbox` with **mock credentials only**. No real API keys, no production database URLs. The Workspace Manager enforces this at provisioning time based on the repo's security tier (defined in `registry.toml`).

### Debris Sweep — Auto-Cleanup

Worktrees abandoned for >72 hours (no commits, no agent heartbeat on that path) are flagged by the Citadel's Debris Sweep:

1. Warning signal sent to the owning agent's mailbox (A2A-S)
2. After 24h with no response, Tyr is notified
3. After 48h, worktree is archived (not deleted) with a snapshot of uncommitted changes
4. Agent's bay filesystem map is updated to remove the stale path

### Decomposition Map Addition

| **Function** | **New Home** | **Mechanism** | **Status** |
| --- | --- | --- | --- |
| Workspace Manager (Git Worktree orchestration) | The Citadel | Citadel service; provisions worktrees per task; Redis tracking; Sanctuary Rule enforced | ✅ Mapped |

### Open Questions — Workspace Manager

- [ ]  **Worktree branch naming** — Convention: `worktree/{agent_name}/{issue_id}`? Or `{agent_name}/issue-{id}`? Define before implementation.
- [ ]  **Concurrent worktree limit** — Max worktrees per agent? Prevents runaway provisioning.
- [ ]  **Cross-agent worktree access** — Can Vigil audit a worktree owned by Sky? Read-only access may be needed for code review micro-agents.
- [ ]  **Worktree-to-PR automation** — Should the Workspace Manager auto-create a draft PR when the worktree branch has commits, or wait for explicit agent signal?
- [ ]  **Monorepo support** — If KoadOS is a Cargo workspace monorepo, does each worktree clone the entire repo or use sparse checkout?

---

## 🐝 Micro-Swarm Hangar — Future Citadel Module

> *The Citadel's expansion bay for the age of micro-agents. Not for now — but the architecture should not close the door on it.*
> 

<aside>
🔭

**Forward-looking concept.** This module is not part of the current refactor scope. It is documented here to ensure the Citadel's current design does not accidentally foreclose it. No implementation decisions required now.

</aside>

### Concept

As KoadOS matures, the agent model will expand beyond a small crew of named, persistent agents (Sky, Tyr, Vigil) to include a **swarm of lightweight micro-agents** — short-lived, task-scoped, spawnable on demand. These micro-agents are not ghosts with full identities; they are purpose-built workers dispatched to complete a discrete task and then retire.

The **Micro-Swarm Hangar** is the Citadel module that manages this swarm. It is the Citadel's expansion dock — a dedicated facility that sits alongside the Personal Bay infrastructure but operates under a fundamentally different model: *mass, dynamic, ephemeral* rather than *named, persistent, reserved*.

### What the Hangar provides

- **Swarm registry** — tracks all active micro-agents: their assigned task, spawn time, status, and owning agent or system
- **Dispatch and lifecycle management** — spawn micro-agents on demand, monitor their progress, collect results, and terminate on completion or timeout
- **Swarm support systems** — lightweight equivalents of CASS for the swarm: task context injection at spawn, result collection on retirement, shared tool access during execution
- **Tool surface for swarm tasks** — a curated set of tools micro-agents can invoke during their assigned work (file reads, API calls, structured output, escalation to a named agent)
- **Isolation enforcement** — micro-agents operate within strict scope limits; they cannot access Personal Bays, cross task boundaries, or persist state beyond their assigned context
- **Tyr visibility** — the swarm is visible to Tyr as a fleet status: how many agents are active, what they're doing, which are healthy, which have stalled

### Relationship to existing architecture

- The Hangar is a **Citadel module** — it is housed within and managed by the Citadel, not a peer system
- Named agents (Sky, Tyr, Vigil) can **spawn micro-agents** from the Hangar as a cognitive offload mechanism — delegating parallelizable subtasks
- CASS may optionally provide **context packages** to micro-agents at spawn time (a compressed subset of the spawning agent's relevant context)
- The Hangar does **not** replace Personal Bays — those remain the home of named, persistent crew agents

### Design constraints for current architecture

To keep the Hangar viable as a future module, the current Citadel design should:

- Avoid hardcoding assumptions that there are only ever N named agents
- Design the registry and session brokering layer to support dynamic, ephemeral agent registration (not just persistent identity registration)
- Ensure CASS's tool surface is designed as an extensible API, not a fixed feature set — so swarm support systems can reuse or extend it

### Native CLI sub-agent support (research finding)

Both Claude Code and Codex CLI already support **native sub-agent definitions** — specialized agents with their own prompts, tools, and model settings that the main agent can delegate to. This maps directly to the Micro-Swarm concept:

- **Claude Code:** Custom subagents via `--agents` CLI flag (JSON) or config files. Each subagent has a description, prompt, tool set, and model.
- **Codex CLI:** Multi-agent roles defined in `[agents]` section of `config.toml`. Codex delegates to the right sub-agent based on task.

When the Micro-Swarm Hangar is implemented, `koad-agent` could generate sub-agent configs as part of ghost preparation, allowing named agents to spawn micro-agents via native CLI features rather than requiring a custom Citadel dispatch mechanism. This is a significant simplification — the CLI itself becomes the micro-swarm runtime.

### Idea Pipeline — Thought-to-Action Event Chain

<aside>
💡

**Added per** [Docs Gap Analysis — .koad-os/docs vs Refactor Plan](https://www.notion.so/Docs-Gap-Analysis-koad-os-docs-vs-Refactor-Plan-8f937a7abc9248308dad10fdb52d7d8c?pvs=21). This is the primary use case for the Micro-Swarm Hangar — the concrete workflow that validates the swarm architecture. Source: Sweep 08, noti_draft_1.

</aside>

The full flow from raw human input to structured GitHub Issue:

1. **Dispatch** — Admiral drops an idea via CLI: `koad dispatch "Add rate limiting to the CASS MCP server"`
2. **Event emission** — `dispatch:idea` event published to `koad:stream:events` (Signal Corps Event Bus)
3. **Intake Drone** — a micro-agent (spawned from the Hangar) picks up the event, queries the Knowledge Archive (`koados_knowledge`) for relevant project context
4. **Structured payload** — Intake Drone constructs a GitHub Issue payload: title, body (with context), labels, milestone, target repo
5. **GitHub Connector** — a listener on `dispatch:issue-ready` creates the issue via `gh issue create`
6. **Broadcast** — `github:issue-created` event propagates to the Event Bus → Sky's awareness feed, Tyr's tracking, any connected notification channels
7. **Trace chain** — the full lifecycle is tagged with a `trace_id` from dispatch to creation, inspectable via `koad dood inspect`

This pipeline demonstrates the Emitter/Listener extensibility of the Event Bus: adding Slack notifications or a future dashboard is just subscribing a new listener — the core pipeline never changes.

### Open Questions — Micro-Swarm Hangar

- [ ]  **Spawn authority** — Who can spawn micro-agents? Named agents only, or can the Citadel itself spawn them for autonomous maintenance tasks?
- [ ]  **Task context format** — What does a micro-agent's task context look like at spawn time? A structured task object, a prompt, a TOML config?
- [ ]  **Result collection** — Where do micro-agent outputs land? Directly in a Personal Bay (of the spawning agent), into Citadel shared state, or into a dedicated results buffer?
- [ ]  **Failure and escalation** — What happens when a micro-agent fails, times out, or hits a scope boundary? Auto-retry, drop silently, or escalate to the spawning agent / Tyr?
- [ ]  **Swarm scale** — What is the anticipated scale? Tens of concurrent micro-agents? Hundreds? This drives the registry and scheduling design.

---

## 🧠 Memory System — Architecture Integration

> *CASS's cognitive continuity is powered by a 4-layer memory stack. The spec lives here:* [KoadOS Agent Memory System — Implementation Spec](https://www.notion.so/KoadOS-Agent-Memory-System-Implementation-Spec-be427a1ae4e24961b81d86479b7cb627?pvs=21)
> 

<aside>
🔗

**This section is a summary bridge.** The full implementation spec with deployment steps, schemas, checklists, and phase gates lives in the [Memory System Spec](https://www.notion.so/KoadOS-Agent-Memory-System-Implementation-Spec-be427a1ae4e24961b81d86479b7cb627?pvs=21). This section explains how the memory system maps onto the Citadel refactor and CASS's role.

</aside>

### How the memory stack maps to the architecture

| **Memory Layer** | **Architectural Home** | **Who manages it** | **Notes** |
| --- | --- | --- | --- |
| L1 — Redis Stack (Working) | The Citadel | Citadel owns Redis; CASS reads/writes via Mem0 | Redis Stack upgrade adds Vector Search — enables semantic cache without new infra |
| L2 — SQLite WAL (Episodic) | CASS | Mem0 middleware; 90-day retention cron | Conversation logs, task history, events. Separate from L4 procedural. |
| L3 — Qdrant (Semantic) | CASS | Mem0 middleware; per-agent collections | Hard collection-level isolation: `sky_memories`, `tyr_memories`, `vigil_memories`, `koados_knowledge`, `task_outcomes` |
| L4 — SQLite (Procedural) | CASS | Mem0 middleware; dedicated schema | Skills, patterns, learned behaviors. No decay — procedural memory is permanent. |
| L5 — Neo4j / Graphiti *(Phase 2)* | CASS *(future)* | TBD — pending Phase 1 stability | Temporal knowledge graph for cross-agent relational reasoning. Do not execute yet. |

### Key architectural decisions from the memory spec

- **Mem0 OSS is the memory middleware.** It orchestrates across all layers — handles extraction, importance scoring, decay, contradiction detection, and cross-session continuity. Agents interact with memory through Mem0, not directly with Redis/SQLite/Qdrant.
- **Hard collection-level isolation in Qdrant.** Each agent's private memories live in a separate collection — not filtered views of a shared collection. A bug cannot leak cross-agent memories.
- **`koados_knowledge` is the shared brain.** Architecture decisions, KoadOS canon, ops facts, anti-patterns — all agents can read and contribute. This is the institutional memory of the swarm.
- **Contradiction policy is enforced.** Conflicting memories surface to the originating agent as a `MEMORY_CONFLICT` event; deferred to the most recent entry until Ian resolves.
- **Importance scoring and decay.** Every memory write gets an `importance_score` (0.0–1.0). Low-importance, stale memories decay weekly and archive to cold storage. Procedural memories (L4) are exempt.

### Memory Insurance — Triple-Redundancy & Backup Protocol

<aside>
🛡️

**Added per** [Docs Gap Analysis — .koad-os/docs vs Refactor Plan](https://www.notion.so/Docs-Gap-Analysis-koad-os-docs-vs-Refactor-Plan-8f937a7abc9248308dad10fdb52d7d8c?pvs=21). Knowledge loss is classified as a "Category 1 System Failure" in KoadOS docs. The 4-layer stack covers *storage*; this covers *protection*. Source: v5/MEMORY_INSURANCE.

</aside>

**Triple-Redundancy Model:**

1. **SQLite** (L2/L4) — primary durable store, WAL mode
2. **WORM Ledger** (`ledger.jsonl`) — append-only file with no overwrite/truncate permissions. Every memory write is journaled here as a second copy.
3. **Notion Cloud backup** — periodic sync of critical memories to a Notion database for disaster recovery

**WORM Ledger (`ledger.jsonl`):**

- Append-only — the filesystem enforces no overwrite/truncate (immutable attribute or permissions)
- Every `koad_intel_commit`, memory extraction, and task outcome write appends a JSON line
- Format: `{ timestamp, trace_id, agent, layer, action, payload_hash, payload }`
- This is the "black box" — even if SQLite corrupts and Qdrant loses vectors, the ledger can rebuild

**Automatic Vault — Snapshot Protocol:**

- `VACUUM INTO` creates timestamped SQLite snapshots every 10 commits or 4 hours (whichever comes first)
- Snapshots stored in `~/.koad-os/backups/` with configurable retention (default: last 10 snapshots)
- Configurable in `kernel.toml [memory_insurance]`

**Brain Drain Verification:**

During TEARDOWN (Docking State Machine), the Brain Drain protocol blocks session closure until all three sinks ACK:

1. SQLite write confirmed
2. WORM Ledger append confirmed
3. Notion Cloud sync queued (async — does not block, but failure triggers a retry queue)

If any primary sink (SQLite or Ledger) fails to ACK, TEARDOWN halts and alerts Tyr.

**Recovery — `koad system restore`:**

- `koad system restore --from ledger` — rebuild SQLite from WORM Ledger entries
- `koad system restore --from cloud` — pull from Notion Cloud backup
- `koad system restore --from snapshot <timestamp>` — rollback to a specific vault snapshot
- Restore operations are logged with trace IDs for auditability

**Anti-Deletion Protection:**

- SQLite `ON DELETE` triggers redirect deleted rows to the `audit_trail` table instead of actually deleting
- Only the Admiral (Ian) can execute a true purge via `koad system purge --confirm`
- Agents cannot delete memories — they can only mark them as deprecated (decay score → 0)

### Relationship to the Citadel refactor

- The **Citadel owns Redis** — the Redis Stack upgrade is a Citadel infrastructure change. CASS consumes it.
- **CASS owns Qdrant and SQLite** — these are CASS's dedicated memory stores, not shared Citadel infra.
- **Memory hydration on connect** (currently a vague CASS capability) is now precisely defined: Mem0 runs `memory.search()` against the agent's Qdrant collection and injects relevant context before the ghost comes fully online.
- **The Personal Bay's cognition context field** maps to what Mem0 persists between sessions — the bay holds a pointer/snapshot; Qdrant holds the actual vectors.
- The **memory spec references "Spine/Sentinel runtime"** (the old term) — once the refactor is complete, update the spec to reference the Citadel instead.

### Agent-to-Agent Signal Protocol (A2A-S) & Temporal Context Hydration (TCH)

<aside>
🛰

**Source:** [**🛰 Research Report: KoadOS Agent Interop & Context Hydration**](https://www.notion.so/Research-Report-KoadOS-Agent-Interop-Context-Hydration-321fe8ecae8f80709241e80d303afb28?pvs=21) (Tyr, 2026-03-09). Both proposals validated and adapted to the refactored Citadel/CASS/Personal Bay architecture.

</aside>

**A2A-S — Agent-to-Agent Signal Protocol**

A lightweight async messaging layer so agents can flag work for each other between sessions without manual PM status reports.

- **Ghost Mailbox** — Redis keys (`koad:mailbox:<agent_name>`) store short JSON signal payloads: `{ sender, timestamp, priority, message, issue_ref }`. Fits the existing Citadel Redis key namespace (`koad:state`, `koad:kai:`, `koad:session:`).
- **Boot-time signal delivery** — `koad-agent` checks the ghost's mailbox during preparation and injects pending signals into the generated context file (Tier 2 content). High-priority signals are flagged prominently.
- **In-session signal tools** — CASS MCP server exposes `koad_signal_send` (push a signal to another agent's mailbox) and `koad_signal_read` (check own mailbox mid-session).
- **CLI surface** — `koad signal <agent_name> -m "message" [-p HIGH|NORMAL|LOW]` for manual or scripted signal dispatch.
- **Signal lifecycle** — signals are consumed (marked read) when the target agent's `koad-agent` preparation hydrates them. Unread signals older than a configurable TTL (default 7 days) are archived to CASS L2.
- **Personal Bay integration** — the mailbox is a bay component. Signals addressed to an agent are stored in their bay; no cross-bay access needed.

**TCH — Temporal Context Hydration (cross-agent handoff)**

A mechanism for one agent to query another agent's session history for handoff and collaboration scenarios.

- **Cross-agent context query** — `koad hydrate --from <agent_name> --topic <topic_id>` (CLI) or `koad_hydrate_from` (MCP tool). Returns a **read-only** summary of the target agent's recent work on the specified topic. This is a controlled, scoped breach of memory isolation — the querying agent sees a filtered view, not raw memory.
- **What's returned:** EndOfWatch summaries + relevant `koad_intel_commit` entries + task outcomes from `task_outcomes` collection. NOT raw turn history or private memories.
- **EndOfWatch auto-summaries** — at session close (or on `koad-agent clear`), the system auto-generates a structured handoff document: what was worked on, key decisions made, blockers hit, next steps recommended. Stored in the agent's Personal Bay session log AND in `koados_knowledge` (tagged with project/topic).
- **Permission model** — cross-agent hydration requires explicit opt-in in the source agent's identity TOML (`[sharing] allow_hydration_from = ["tyr", "vigil"]`). Default: no cross-agent access.

**What's already covered vs. genuinely new:**

| **Tyr's Proposal** | **Status in Refactor Plan** | **Action** |
| --- | --- | --- |
| Ghost Mailbox (Redis-backed signals) | 🆕 New — no inter-agent messaging existed | Added as A2A-S above |
| Sentinel boot-hook for signal delivery | ⚠️ Terminology update needed — "Sentinel" → `koad-agent` preparation step | Integrated into boot sequence (Tier 2 hydration) |
| `koad signal` CLI command | 🆕 New CLI verb | Added to CLI surface |
| Cross-partition SQLite reader (TCH) | ⚠️ Partially covered by `koad_intel_query`  • `koad_memory_hydrate` | Added `koad_hydrate_from` as cross-agent-specific tool with permission model |
| Snapshot Replay (last 5-10 turns) | ⚠️ Covered by EndOfWatch summaries (better than raw turns — avoids context bloat) | Replaced with EndOfWatch auto-summaries |
| EndOfWatch / Introspection summaries | 🆕 New — `koad_session_save` existed but not auto-generated or standardized | Added as automatic session-close behavior |

**Open Questions — A2A-S & TCH:**

- [ ]  **Signal priority escalation** — should HIGH-priority signals trigger a notification to Ian (via Notion/email), or are they only visible to the target agent at next boot?
- [ ]  **Mailbox capacity** — max signals per agent before oldest are auto-archived? Prevents mailbox flooding from a runaway agent.
- [ ]  **EndOfWatch trigger** — auto-generated on `koad-agent clear` only, or also on CLI compaction events? Compaction-triggered summaries would capture mid-session state.
- [ ]  **Cross-agent hydration scope** — should `koad_hydrate_from` return only EndOfWatch summaries, or also filtered L3 semantic memories? Broader scope = more useful but higher privacy risk.
- [ ]  **TCH audit trail** — should cross-agent hydration queries be logged for Tyr visibility? (Recommended yes.)

### Agent Growth System — Journals, Introspection & Knowledge Broadcasting

<aside>
🌱

**Added per** [Docs Gap Analysis — .koad-os/docs vs Refactor Plan](https://www.notion.so/Docs-Gap-Analysis-koad-os-docs-vs-Refactor-Plan-8f937a7abc9248308dad10fdb52d7d8c?pvs=21). Extends EndOfWatch summaries and `koad_intel_commit` into a structured growth and learning system. Source: noti_draft_1 conceptual design.

</aside>

Beyond session memory, KoadOS provides a structured system for agent learning and cross-crew knowledge sharing.

**Journals — Persistent Crew Logs:**

- Each agent maintains an append-only journal (stored in CASS L2) — a chronological record of significant observations, decisions, and learnings
- Journals are readable by other agents (with permission) and by Ian
- Distinct from session logs: journals are *curated* — agents write journal entries deliberately, not automatically
- CLI: `koad journal add "Discovered that Redis XADD with MAXLEN is faster than manual XTRIM for our stream sizes"`
- MCP: `koad_journal_add` tool (candidate for CASS MCP server expansion)

**Introspection Cycles:**

At EndOfWatch (session close), agents are prompted with structured self-reflection:

1. What did I accomplish this session?
2. What did I learn that the crew should know?
3. What surprised me or went wrong?
4. What would I do differently next time?
5. What should be queued for the next session?

Introspection output is written to the agent's journal and flagged for the Knowledge Indexer.

**Knowledge Broadcasting:**

- A micro-agent (Knowledge Indexer) processes new journal entries and introspection output
- Commits distilled learnings to the shared `koados_knowledge` Qdrant collection
- Emits `knowledge:new-learning` events to the Signal Corps Event Bus
- CASS evaluates which agents would benefit from new learnings and queues them for next-boot hydration (Tier 2 content)

**Crew Briefings:**

- Pre-milestone or on-demand, CASS generates a summary of all recent learnings, open issues, and known gotchas across all agents
- Briefing is injected as Tier 2 context at next boot for all agents
- CLI: `koad briefing generate` (Tyr or Admiral only)

### Open Questions — Memory System Integration

- [ ]  **Spine → Citadel terminology update in memory spec** — The implementation spec still references "Spine/Sentinel runtime." Update after the Spine scrub is complete.
- [ ]  **Bay ↔ Qdrant relationship** — Does the Personal Bay store a Qdrant collection reference, or does CASS resolve the collection from agent identity at hydration time? Define the lookup path.
- [ ]  **Mem0 on Node 24 ESM** — Validate `mem0ai` npm package compatibility before integration. REST wrapper fallback plan needed if CJS-only.
- [ ]  **Embedding model in koad install** — The `OPENAI_API_KEY` (for `text-embedding-3-small`) needs to be added to the `.env.template` and `koad install` prompt flow.
- [ ]  **`koados_knowledge` governance** — Who resolves write conflicts to the shared collection? Tyr as arbiter, or majority-last-write?

---

## 📡 Signal Corps & Event Bus Architecture

> *The Citadel's nervous system. Aggregates internal events into high-signal broadcasts for observability, awareness, and extensibility.*
> 

<aside>
🔍

**Added per** [Docs Gap Analysis — .koad-os/docs vs Refactor Plan](https://www.notion.so/Docs-Gap-Analysis-koad-os-docs-vs-Refactor-Plan-8f937a7abc9248308dad10fdb52d7d8c?pvs=21). The Signal Corps was extensively designed in Sweeps 04/06 and v5 architecture docs but had no section in this plan. It is foundational infrastructure that the context hydration system, micro-agents, and any future monitoring interfaces depend on.

</aside>

### What the Signal Corps is

The Signal Corps is a dedicated background service within the Citadel (not a separate daemon) that:

1. **Aggregates** internal events — gRPC heartbeats, Redis keyspace events, agent log streams, tool execution results
2. **Packetizes** events into 500ms "Signal Packets" — batched, compressed, high-signal summaries
3. **Broadcasts** packets to Redis Streams for multi-consumer consumption

It is **distinct from** both the Watchdog/Sentinel (which performs self-healing) and A2A-S (which provides inter-agent async messaging). The Signal Corps is the *observability backbone*.

### Redis Streams Schema

| **Stream Key** | **Content** | **Consumers** |
| --- | --- | --- |
| `koad:stream:telemetry` | Station resource stats (CPU, MEM, disk), Citadel link health, Redis/SQLite round-trip latency | Monitoring interfaces, `koad watch`, friction analytics |
| `koad:stream:logs` | Unified logs from all agents and the Citadel, prepended with `trace_id` | `koad watch --raw`, Log Summarizer micro-agent, debugging |
| `koad:stream:events` | Human-readable station events ("Sky initiated SLE mapping", "New issue #118 created from idea") | Cross-agent awareness, context hydration Awareness Tier, future dashboards |

### Event Bus Architecture (Emitter/Listener Model)

The Signal Corps implements a general-purpose **Event Bus** using Redis Streams as the transport:

- **Everything is an event** — agent boot, GitHub issue creation, tool execution, task completion, idea dispatch, heartbeat loss
- **Listeners subscribe to what they care about** — micro-agents listen to specific streams; named agents get filtered "Context Packets"
- **New integrations = new listeners** — adding Slack notifications, a future dashboard, or a new micro-agent is just subscribing to the right stream. The core never changes.
- **Context Packets** — instead of dumping raw logs into an agent's context, the Signal Corps provides compressed "Summary Packets" of recent station activity. This is how agents stay aware without paying the token tax on raw telemetry.

### Relationship to existing architecture

- The Signal Corps is a **Citadel module** — it runs as a background task within the Citadel process
- It **feeds the Context Hydration system** — the Awareness Tier (live pulse) in the three-tier context model pulls from `koad:stream:events`
- It **supports the Micro-Swarm Hangar** — micro-agents (Intake Drone, Signal Drone, Log Summarizer) are Event Bus consumers
- It **enables future interfaces** — any TUI, Web Deck, or `koad watch` mode is a stream subscriber, not a separate data pipeline
- It uses the **CQRS framing**: Redis Streams are the Data Plane (high-frequency queries/reads); the Citadel gRPC bus remains the Control Plane (validated commands/mutations)

### Open Questions — Signal Corps

- [ ]  **Broadcast interval** — 500ms packets are the docs' design. Validate that this doesn't create excessive Redis write pressure under multi-agent load.
- [ ]  **Stream retention** — Redis Streams can be capped by count or time. Define retention policy: last N entries per stream, or time-based (e.g. 24 hours)?
- [ ]  **Consumer groups** — should micro-agents and monitoring interfaces share consumer groups (load-balanced) or use independent consumers (each sees all events)?
- [ ]  **Signal-to-noise filter** — the Signal Corps manages what agents *see*. Define the filtering rules: which events are "high-signal" (injected into context) vs "background" (available on demand via `koad watch`)?

---

## 🔗 Trace ID & Observability System

> *Every interaction traceable from the Command Deck to the Engine Room. No blind corners.*
> 

<aside>
🔍

**Added per** [Docs Gap Analysis — .koad-os/docs vs Refactor Plan](https://www.notion.so/Docs-Gap-Analysis-koad-os-docs-vs-Refactor-Plan-8f937a7abc9248308dad10fdb52d7d8c?pvs=21). The Trace ID system was mandated by the V5 Overhaul Protocol and designed across Sweeps 05/06, but had no section in this plan. Without it, the recurring "system blindness" problem (false greens, silent failures) returns.

</aside>

### The Trace ID Chain

Every request flowing through KoadOS is tagged with a unique `trace_id` that propagates across all subsystems:

1. **Generation** — the CLI generates a `trace_id` (format: `TRC-{uuid-short}`) at invocation
2. **Propagation** — the ID is passed in gRPC metadata (headers) to the Citadel
3. **Execution** — the Citadel logs all Redis/SQLite operations with this `trace_id`
4. **Audit** — the `trace_id` is recorded in the SQLite `audit_trail` table with actor, action, and outcome
5. **Reporting** — on failure, run `koad dood inspect <trace_id>` to see the exact lifecycle and point of failure

### TraceContext Struct (Code Standard)

Per the V5 Overhaul Protocol: *"Every new function in the v5.0 core MUST accept a `TraceContext`. Code that cannot be traced cannot be merged."*

```jsx
TraceContext {
  trace_id: String,       // TRC-{uuid-short}
  origin: String,         // CLI | Citadel | CASS | Agent
  actor: String,          // agent name or "admiral"
  timestamp: u64,         // Unix timestamp at generation
}
```

The `emit_signal!(ctx, level, msg)` macro automatically publishes to `koad:stream:logs` prepended with the `trace_id`.

### audit_trail Table (SQLite)

A new durable table managed by the Citadel's `SchemaManager`:

| **Column** | **Type** | **Purpose** |
| --- | --- | --- |
| `trace_id` | TEXT PRIMARY KEY | The unique trace identifier |
| `actor` | TEXT | Who initiated (agent name or "admiral") |
| `action` | TEXT | What was requested (command, tool call, etc.) |
| `outcome` | TEXT | SUCCESS | FAILURE | TIMEOUT |
| `error_detail` | TEXT (nullable) | Error message if outcome is FAILURE |
| `timestamp` | INTEGER | Unix timestamp |
| `duration_ms` | INTEGER | Execution duration in milliseconds |

### Active Diagnostics (replacing passive monitoring)

The current `koad status` performs "shallow" checks (socket exists? port open?) and can report false greens. The Trace ID system enables **active functional probes:**

- `koad dood pulse` — sends a gRPC ping through the Citadel to Redis and back, measuring round-trip latency. The primary "is it alive?" check.
- `koad dood inspect <trace_id>` — reconstructs the full lifecycle of a specific request across all systems. Shows exactly where a failure occurred.
- `koad watch --raw` — lightweight stream subscriber that tails all `koad:stream:*` channels in real-time. Designed to run in a secondary terminal during development.

### Decomposition map addition

| **Function** | **New Home** | **Mechanism** | **Status** |
| --- | --- | --- | --- |
| Signal Corps (broadcast service) | The Citadel | Background task within Citadel process; broadcasts to Redis Streams | ✅ Mapped |
| Trace ID generation & propagation | CLI + Citadel | CLI generates; gRPC metadata propagates; Citadel logs; SQLite audits | ✅ Mapped |
| Event Bus (Redis Streams) | The Citadel | Citadel owns Redis Streams; Signal Corps manages broadcast; consumers subscribe | ✅ Mapped |

### Open Questions — Trace ID & Observability

- [ ]  **Trace ID format** — `TRC-{uuid-short}` or include a component prefix (e.g. `TRC-CLI-{uuid}`, `TRC-CASS-{uuid}`)? Component prefix aids visual debugging.
- [ ]  **audit_trail retention** — indefinite, or time-bounded? The V5 docs say the audit trail is an "immutable log" but unlimited growth needs addressing.
- [ ]  **`koad dood` namespace** — the docs use `koad dood` for superuser diagnostics. Confirm this is the canonical command surface (vs `koad doctor`, `koad inspect`, etc.).

---

## 🖥️ Interface Layer Position — TUI & Web Gateway

> *An explicit architectural decision on the v5 interface scope.*
> 

<aside>
⚠️

**Contradiction resolved.** The `.koad-os/docs` contain conflicting positions: Sweep 03 says "Kill the TUI and Gateway" while Sweep 04 says "Promote TUI to lead interface" and designs an elaborate Web Deck. This section resolves the conflict.

</aside>

### Position: CLI-first for v5. TUI and Web Deck are deferred to v6+.

**Rationale:**

- The Body/Ghost model prepares a plain bash shell — the CLI *is* the interface. Adding TUI or Web Deck infrastructure contradicts the "simplicity over complexity" directive.
- TUI output "cannot be easily parsed by secondary agents" (Sweep 03) — this conflicts with the programmatic-first communication principle.
- The Web Gateway (`koad-gateway`) adds massive dependency bloat (`axum`, `tower-http`, `tungstenite`, WebSockets) for a capability that has no current critical use case.
- The Signal Corps + Redis Streams architecture **does not close the door** on TUI or Web Deck — any future interface is just a stream subscriber. The infrastructure is extensible without being built now.

### What this means for the current refactor

- `koad-gateway` crate — **archive, do not delete.** Remove from the active workspace `Cargo.toml` but preserve the code for future reference.
- `koad-tui` / `kdnd-tui` crates — **archive.** Same treatment.
- `axum`, `tower-http`, `tungstenite`, `ratatui`, `crossterm` — **remove from workspace dependencies.** Reduces compile times and binary size.
- `koad watch --raw` — the lightweight CLI stream viewer is the v5 monitoring interface. It subscribes to Redis Streams directly. No TUI framework needed.
- Any doc references to "Web Deck", "Observation Deck", "Command Deck (TUI)" should be updated to note these are **v6+ planned features**, not v5 scope.

### Crate lifecycle decisions (from Lean Audit)

| **Crate** | **Decision** | **Rationale** |
| --- | --- | --- |
| `koad-gateway` | 🗄️ Archive | Web Deck is v6+. Dependency bloat (axum, tower-http, WebSockets). |
| `koad-tui` / `kdnd-tui` | 🗄️ Archive | TUI is v6+. ratatui/crossterm deps not needed for CLI-first model. |
| `koad-skill-airtable` | 🔄 Refactor → bridge or deprecate | "Skill as a Crate" scales poorly. Should be a lightweight bridge or Python script. |
| `KoadComplianceManager (KCM)` | 🗑️ Purge | Recursive anti-pattern: Citadel shells out to CLI to perform governance actions. Governance should be handled by CLI directly or isolated background tasks. |
| `ASM internal Mutex` | 🗑️ Purge | Creates Ghost Spine desync bug. ASM becomes stateless methods reading/writing Redis directly (already planned as CASS sub-system). |

---

## ⚡ Operational Standards & Protocols

> *Engineering constraints, quality gates, and efficiency protocols that apply across all Citadel subsystems.*
> 

<aside>
🔍

**Added per** [Docs Gap Analysis — .koad-os/docs vs Refactor Plan](https://www.notion.so/Docs-Gap-Analysis-koad-os-docs-vs-Refactor-Plan-8f937a7abc9248308dad10fdb52d7d8c?pvs=21). Sources: v5/PERFORMANCE_AND_FRICTION, TOKEN_AUDIT protocol, DOCUMENTATION_MANIFESTO, INTEGRITY_AUDIT protocol.

</aside>

### Performance & Friction Analytics

Every link in the KoadOS chain has a defined latency budget. New features must include a "Latency Budget" line in their KSRP report.

| **Link** | **Target** | **Friction Trigger** |
| --- | --- | --- |
| CLI → gRPC → Citadel (Spine Hop) | <5ms | >20ms |
| Citadel → Redis → Citadel (Engine Link) | <2ms | >10ms |
| Citadel → SQLite (Persistence Sink) | <50ms | >200ms |
| Context hydration (CASS → Agent) | <500ms | >2s |
| MCP tool round-trip (Agent → CASS MCP → response) | <100ms | >500ms |

When a link exceeds its friction trigger, it emits a `friction:exceeded` event to the Signal Corps with the `trace_id`, enabling automated detection of performance regressions.

### Token Audit Protocol — 5-Pass Framework

A structured protocol for optimizing token efficiency across agent contexts and tool interactions:

1. **Redundancy Scan** — measure context overlap ratio between boot context and in-session tool outputs. Target: <15% overlap.
2. **Verbosity Scrub** — measure signal-to-noise ratio in bytes. Implement `-compact` mode for tool outputs that strips boilerplate. Target: >70% signal.
3. **Tool-Call Efficiency** — identify redundant discovery calls (e.g. agent re-reading the same file). Target: 0 redundant reads per session.
4. **Payload Trimming** — gRPC field selection to avoid sending unused data. Request only needed fields in CASS queries.
5. **Persona Compaction** — measure identity density in boot context. Identity + directives should deliver maximum behavioral alignment per token. Target: <2,000 tokens for Tier 1.

Token Audit runs are logged and tracked — each audit produces a report with before/after token counts and recommendations.

### Mermaid Visualization Mandate

Per the Documentation Manifesto: *"All major architectural components MUST have a corresponding Mermaid.js visualization."*

Before any code work begins on a new component:

1. Load the relevant architecture doc into context
2. Create or update the Mermaid diagram for the proposed change
3. Review the diagram against the Top-Level architecture
4. Only upon approval of the *diagram* does code begin

This applies to all Citadel, CASS, and Agent Chassis components. Diagrams live alongside their architecture docs in `.koad-os/docs/`.

### Integrity Audit Protocol

Per the INTEGRITY_AUDIT protocol — a testing quality standard that catches false-positive test passes:

- Tests that can't fail are liabilities — every happy-path test needs a failure counterpart
- No silent `let _ =` — all `Result` types must be explicitly handled
- Session integrity checks: verify 1:1 mapping between registered agent and active session; confirm assertions fail when duplicates are injected
- **Review cadence:** PR merge (Part 2 check), config change (Part 1 scan), incident post-mortem (mandatory coverage PR), weekly full sweep
- All KSRP code reviews must include an Integrity Audit pass

---

## Notes & Brainstorm Scratchpad

*Use this section for raw ideas, edge cases, and anything that doesn't fit cleanly above yet.*

[Agent Boot Research — CLI Context Injection Patterns](https://www.notion.so/Agent-Boot-Research-CLI-Context-Injection-Patterns-0f4b35fc93b54fdca9d1bd63537502ae?pvs=21)

[Context Hydration Architecture — Research & Design](https://www.notion.so/Context-Hydration-Architecture-Research-Design-721fbb7bc1274186b4c49b1678ee8905?pvs=21)

[Docs Gap Analysis — .koad-os/docs vs Refactor Plan](https://www.notion.so/Docs-Gap-Analysis-koad-os-docs-vs-Refactor-Plan-8f937a7abc9248308dad10fdb52d7d8c?pvs=21)

[KoadOS — Spine Retirement Record](https://www.notion.so/KoadOS-Spine-Retirement-Record-321fe8ecae8f818f87baf4a4c30b413d?pvs=21)