# KoadOS Glossary

> Comprehensive reference for KoadOS-specific terminology. Terms are used consistently throughout the knowledge base. On first use in any article, terms are briefly defined in context; this glossary provides authoritative full definitions.

---

## A

**Admiral**
The highest-authority role in KoadOS. Currently occupied by Ian Deans. The Admiral is the final approval gate for all architectural decisions and the only entity that can merge pull requests to `main`. See also: [Tyr: Captain](./agent-roles/tyr.md).

**Active (session state)**
The healthy, heartbeating state of an agent session. A session is `active` when its Redis lease key (`koad:session:<session_id>`) exists and its `SessionRecord` in `koad:state` shows `status: "active"`. See: [Agent Session Lifecycle](./core-systems/agent-session-lifecycle.md).

**AGENTS.md**
The root onboarding and operational directives document for all KoadOS agents. Contains the Non-Negotiable Directives, onboarding sequence, architecture brief, workspace navigation guide, and crew manifest. Located at `~/.koad-os/AGENTS.md`.

**anyhow**
A Rust crate used in KoadOS binary crates (`koad-citadel`, `koad-cass`, `koad-cli`) for application-level error handling. Provides `anyhow::Result` and contextual error chaining via `.context()`. Contrasted with `thiserror`, which is used in library crates. See: [RUST_CANON](./protocols/rust-canon.md).

## B

**Bay**
An agent's designated private storage area within the Citadel, provisioned by the `PersonalBayService`. Located at `~/.koad-os/bays/<agent_name>/`. Contains the agent's working files and bay-specific state managed by the Citadel.

**Body**
One half of the [Body/Ghost Model](./architecture/body-ghost-model.md). The Body is the ephemeral shell session an agent currently inhabits — a terminal, a process, a working context. It has a `body_id` (client-generated UUID) and a `session_id` (Citadel-assigned). The Body is temporary; it is destroyed when the session ends. Contrasted with Ghost.

**Body/Ghost Model**
The core KoadOS metaphor separating an agent's temporary execution environment (Body) from its persistent identity and memory (Ghost). The Boot process fuses Ghost to Body for the duration of a session. See: [The Body/Ghost Model](./architecture/body-ghost-model.md).

**Boot**
The process of starting an agent session. Executed via `eval $(koad-agent boot --agent <name>)`. Fuses the agent's Ghost (identity) with a new Body (shell session) by performing a `CreateLease` gRPC handshake with the Citadel and exporting session credentials to the shell environment. See: [koad-agent boot](./core-systems/koad-agent-boot.md).

**Brain**
Informal name for [CASS](#cass) (`koad-cass`) in the [Tri-Tier Model](./architecture/tri-tier-model.md). The cognitive layer responsible for memory, context, and intelligence.

## C

**Canon**
Short for any of the KoadOS canonical protocol documents: [RUST_CANON](./protocols/rust-canon.md) (Rust coding standards) or CONTRIBUTOR_CANON (contribution workflow). "Following canon" means adhering to these mandatory standards.

**Captain**
The second-highest rank in the KoadOS `Rank` enum. Grants elevated sandbox permissions, Citadel-level workspace scope, and architectural authority. Currently assigned to Tyr. See: [Tyr: Captain](./agent-roles/tyr.md).

**Cargo Workspace**
The Rust project structure organizing all KoadOS crates under a single root `Cargo.toml`. Provides shared dependency versions, a unified build cache, and enforced crate boundaries. See: [Cargo Workspace](./tooling/cargo-workspace.md).

**CASS**
*Citadel Agent Support System*. The "Brain" in the Tri-Tier Model. A persistent gRPC service (`koad-cass`) providing memory storage, context hydration, code graph queries, and EndOfWatch session summarization. CASS manages the cognitive layer of KoadOS while the Citadel manages the infrastructure layer. See: [koad-cass](./core-systems/koad-cass.md).

**CassStorage**
The production implementation of the `Storage` trait in `koad-cass`. Uses `rusqlite` to interact with `cass.db`. Contrasted with `MockStorage` (used in tests). See: [SQLite Storage](./data-storage/sqlite-cass-db.md).

**Citadel**
1. The KoadOS platform directory: `~/.koad-os/`. Level 3 in the [Workspace Hierarchy](./architecture/workspace-hierarchy.md).
2. The `koad-citadel` binary: the gRPC kernel and infrastructure layer. The "Body" in the Tri-Tier Model. See: [koad-citadel](./core-systems/koad-citadel.md).

**Citadel Agent Support System**
See: [CASS](#cass).

**CloseSession**
The gRPC RPC that an agent calls to gracefully log out. Immediately removes the session from Redis and fires the `session_closed` event to the Koad Stream, triggering the EndOfWatch pipeline. Contrasted with session going dark (crash-based termination).

**Condition Green**
The state indicating Admiral approval for a plan. Before Tyr (or any agent operating under the Plan Mode Law) can execute a non-trivial task, they must present a plan and receive "Condition Green" from Ian. See: [Tyr: Captain](./agent-roles/tyr.md).

**Contractor**
The rank assigned to Claude Code in the KoadOS crew. Contractors work in isolated git worktrees, submit changes via PR, and operate under Tyr's architectural direction with Ian's final approval.

**CreateLease**
The gRPC RPC that starts an agent session. Called by `koad-cli` during the boot process. Creates a `SessionRecord` in Redis and returns a `session_id` and `session_token`. Enforces the "One Body, One Ghost" rule. See: [koad-citadel](./core-systems/koad-citadel.md).

**Crew**
The lowest operational rank in KoadOS. Crew-rank agents (Scribe, general coding agents) operate at Outpost level (Level 1), have restricted sandbox permissions, and work on specific assigned tasks.

## D

**Dark (session state)**
The state of a session that has lost its heartbeat. The `SessionRecord` still exists in `koad:state` but the Redis lease key (`koad:session:<session_id>`) has expired. The Ghost is "trapped in a dead Body". The reaper detects dark sessions and eventually purges them. See: [Agent Session Lifecycle](./core-systems/agent-session-lifecycle.md).

**dark_timeout_secs**
The configuration parameter (in `config/kernel.toml [sessions]`) controlling how long a session may remain dark before being purged by the reaper. Default: 300 seconds (5 minutes). See: [Agent Session Lifecycle](./core-systems/agent-session-lifecycle.md).

**Distillation Gap**
The problem (now resolved in Phase 3) where sessions closed but were never summarized into long-term memory. Bridged by the `EndOfWatchPipeline` in CASS, which auto-generates `EpisodicMemory` records when sessions close.

**Dood**
Informal name for Ian (the Admiral) in KoadOS operational docs. Used as the "Dood Approval Gate" — Dood must give "Condition Green" before any architectural change proceeds to execution.

## E

**EndOfWatch (EOW)**
The process that occurs when a session ends (gracefully or via purge). CASS's `EndOfWatchPipeline` generates an AI-powered summary of the session's activities and saves it as an `EpisodicMemory` record in `cass.db`. On the next boot, this summary is included in the context hydration packet. See: [koad-cass](./core-systems/koad-cass.md).

**EndOfWatchPipeline**
The background task in `koad-cass` (`services/eow.rs`) that subscribes to `koad:stream:system` and generates session summaries on `session_closed` events. See: [koad-cass](./core-systems/koad-cass.md).

**EpisodicMemory**
A structured record of a completed agent session, stored in the `episodes` table of `cass.db`. Contains: agent name, session ID, project path, AI-generated narrative summary, turn count, and timestamp. Retrieved by CASS's `HydrationService` to give agents context about their previous sessions. See: [SQLite Storage](./data-storage/sqlite-cass-db.md).

**eval**
The Unix shell mechanism used to execute the shell script printed by `koad-agent boot`. `eval $(koad-agent boot --agent Tyr)` captures the subprocess's stdout and runs it in the parent shell, injecting environment variables. Without `eval`, the exports are printed as text but never executed. See: [koad-agent boot](./core-systems/koad-agent-boot.md).

## F

**FactCard**
A structured unit of long-term agent knowledge stored in the `facts` table of `cass.db`. Contains: domain (topic area), content (the knowledge), confidence score (AI-generated significance, 0.0–1.0), tags, source agent, and session ID. The primary building block of KoadOS's compounding memory system. See: [SQLite Storage](./data-storage/sqlite-cass-db.md).

## G

**Ghost**
One half of the [Body/Ghost Model](./architecture/body-ghost-model.md). The Ghost is the persistent, immaterial aspect of an agent: its identity (defined in `config/identities/<name>.toml`), accumulated memory (`FactCard`s, `EpisodicMemory`), and personal vault (`~/<agent>/memory/`). The Ghost persists between sessions; it is never destroyed by session termination. Contrasted with Body.

**gRPC**
The inter-process communication protocol used throughout KoadOS. All communication between the Link (koad-cli), Body (koad-citadel), and Brain (koad-cass) tiers uses gRPC, with message definitions in `proto/`. Implemented via the `tonic` crate.

## H

**Heartbeat**
The periodic gRPC call an agent sends to the Citadel to keep its session alive. Each heartbeat refreshes the Redis lease key TTL. The heartbeat interval should be roughly `lease_duration_secs / 3` to provide adequate margin. A stopped heartbeat leads to a dark session. See: [Agent Session Lifecycle](./core-systems/agent-session-lifecycle.md).

**Hierarchy Walk**
The process CASS's `HydrationService` uses during Temporal Context Hydration (TCH): walking the filesystem from the agent's workspace level outward, collecting relevant files and metadata within the token budget. See: [koad-cass](./core-systems/koad-cass.md).

**HydrationService**
The CASS gRPC service implementing Temporal Context Hydration (TCH). Receives a `HydrationRequest` with agent name, workspace level, and token budget; returns a curated context packet. See: [koad-cass](./core-systems/koad-cass.md).

## I

**Identity**
The Rust struct (`koad-core/src/identity.rs`) representing an agent's static Ghost identity: name, rank, permissions, access keys, and tier. Read from `config/identities/<agent>.toml` during the boot process.

**InferenceRouter**
The component in `koad-cass` that routes AI inference requests (summarization, significance scoring) to the appropriate provider via `koad-intelligence`.

**Interceptor**
The `tonic` gRPC middleware in `koad-citadel/src/auth/interceptor.rs` that validates the `KOAD_SESSION_TOKEN` on every incoming gRPC request before passing it to the service handler. The enforcement point of the Zero-Trust security model. See: [koad-citadel](./core-systems/koad-citadel.md).

## K

**KAPV**
*KoadOS Agent Personal Vault*. The standard directory structure for sovereign agent memory storage. Tyr's `~/.tyr/` vault is the current template. Defined by: `memory/SAVEUPS.md`, `memory/LEARNINGS.md`, `memory/PONDERS.md`.

**koad-agent**
The command-line tool (`koad-cli` crate) that serves as the "Link" in the Tri-Tier Model. Primary command: `koad-agent boot --agent <name>`. See: [koad-agent boot](./core-systems/koad-agent-boot.md).

**koad-cass**
See: [CASS](#cass).

**koad-citadel**
The "Body" — the core OS kernel gRPC server. See: [koad-citadel](./core-systems/koad-citadel.md).

**koad-core**
The foundational library crate containing shared types, traits, and utilities used by all other KoadOS crates. Includes: `Identity`, `Rank`, `HierarchyManager`, `RedisClient`, `KoadConfig`, and more. Has minimal dependencies and no internal koad-* dependencies.

**koad-intelligence**
The library crate that routes AI inference requests to external LLM providers. Used by CASS for session summarization (EOW) and FactCard significance scoring. Not a gRPC service — a library consumed by `koad-cass`.

**koad-proto**
The library crate containing auto-generated Rust code from `.proto` files in `proto/`. Provides gRPC client and server stubs for all KoadOS services. Generated via `tonic-build` in the build script.

**koad-sandbox**
The library crate implementing command validation and policy enforcement. Used by the Citadel's `SectorService` via `ValidateIntent`. The `Sandbox::evaluate()` method checks commands against rank-aware policies. Captain-rank agents may bypass certain restrictions. See: [koad-citadel](./core-systems/koad-citadel.md).

**Koad Stream**
The event bus backed by Redis Streams, operated by the Citadel's `SignalService`. Key channel: `koad:stream:system` (lifecycle events like `session_closed`). Agents can `Broadcast` events and `Subscribe` to channels. See: [koad-citadel](./core-systems/koad-citadel.md).

**KOAD_SESSION_ID**
Shell environment variable set by `koad-agent boot`. The public identifier for the active session. Used for display and logging. Contrasted with `KOAD_SESSION_TOKEN`.

**KOAD_SESSION_TOKEN**
Shell environment variable set by `koad-agent boot`. The private credential used to authenticate all gRPC calls to the Citadel via the interceptor. Treat as a password. See: [koad-agent boot](./core-systems/koad-agent-boot.md).

**KSRP**
*Koad Self-Review Protocol*. The mandatory self-review process an agent performs before creating a pull request. Verifies RUST_CANON compliance, test coverage, documentation completeness, and alignment with the approved plan.

## L

**lease_duration_secs**
The configuration parameter (in `config/kernel.toml [sessions]`) controlling the TTL of a Redis session lease key. Default: 90 seconds. The session goes dark if no heartbeat arrives within this window. See: [Agent Session Lifecycle](./core-systems/agent-session-lifecycle.md).

**Level**
The numeric designation in the Workspace Hierarchy: Level 4 (System), Level 3 (Citadel), Level 2 (Station), Level 1 (Outpost). Determined by `HierarchyManager::resolve_level()` at boot time and embedded in the session token. See: [Workspace Hierarchy](./architecture/workspace-hierarchy.md).

**Link**
The "Link" tier in the Tri-Tier Model. The `koad-agent` CLI tool that connects a Ghost to a Body by performing the boot handshake with the Citadel. See: [The Tri-Tier Model](./architecture/tri-tier-model.md).

## M

**MemoryService**
The CASS gRPC service for long-term memory operations: `CommitFact`, `QueryFacts`, `RecordEpisode`. Backed by `cass.db` via the `Storage` trait. See: [koad-cass](./core-systems/koad-cass.md).

**MockStorage**
The test-double implementation of the `Storage` trait in `koad-cass`. Uses in-memory `Vec`s instead of SQLite, enabling fast, file-system-free unit tests for CASS services. See: [SQLite Storage](./data-storage/sqlite-cass-db.md).

## N

**Non-Blocking Rule**
The RUST_CANON rule that blocking operations (CPU-intensive work, synchronous I/O) must never run on the main `tokio` async runtime. They must be moved to a dedicated thread using `tokio::task::spawn_blocking`. See: [RUST_CANON](./protocols/rust-canon.md).

## O

**One Body, One Ghost**
The KoadOS protocol enforcing that each named agent may only have one active session at a time. Enforced by `CitadelSessionService::create_lease()`, which rejects a boot attempt if an active session already exists for the agent name. See: [The Body/Ghost Model](./architecture/body-ghost-model.md).

**Outpost**
Level 1 in the Workspace Hierarchy. A single git repository. The most common operational level for Crew-rank agents. Context hydration at Outpost level is scoped to the current repository only. See: [Workspace Hierarchy](./architecture/workspace-hierarchy.md).

## P

**PersonalBay**
The Citadel gRPC service that provisions and manages agent-specific storage areas (bays) and git worktree environments. See: [koad-citadel](./core-systems/koad-citadel.md).

**Plan Mode**
The operational mode in which an agent produces a structured, step-by-step implementation plan before writing any code. Required for all tasks of Standard (Medium) complexity or higher under the Plan Mode Law from `AGENTS.md`.

**Plan Mode Law**
The directive from `AGENTS.md §III.5` requiring Plan Mode for any task involving multi-file changes, new logic, or script generation. The plan must be approved by the Admiral (Condition Green) before execution begins.

**PSRP**
*Personal Self-Review Protocol*. A variant of KSRP for personal or memory-focused work (as opposed to code contributions).

**Purged (session state)**
The terminal state of an agent session. The `SessionRecord` has been removed from Redis and a `session_closed` event has been fired. The EndOfWatch pipeline has been triggered. The "One Body, One Ghost" lock for this agent name is released, allowing a fresh boot. See: [Agent Session Lifecycle](./core-systems/agent-session-lifecycle.md).

## R

**Rank**
The hierarchical privilege level of a KoadOS agent. The `Rank` enum in `koad-core/src/identity.rs` defines: `Admiral`, `Captain`, `Officer`, `Crew`. Rank determines sandbox permissions, workspace level scope, and operational authority.

**Reaper**
The background task in `CitadelSessionService` that periodically scans sessions and marks dark ones (whose lease keys have expired) as such, then purges those that have been dark too long. Runs every `reaper_interval_secs`. See: [Agent Session Lifecycle](./core-systems/agent-session-lifecycle.md).

**Redis**
The in-memory data store used by the Citadel for "hot" state: active sessions (`koad:state`), session lease keys (`koad:session:*`), distributed locks, and the Koad Stream event bus (`koad:stream:*`). Contrasted with SQLite (cold/persistent storage).

**Research → Strategy → Execution**
Tyr's mandatory three-phase work cycle. No code is written before Research (understand the problem) and Strategy (produce a plan) are complete, and no execution begins before Admiral approval. See: [Tyr: Captain](./agent-roles/tyr.md).

**RUST_CANON**
The mandatory Rust development standards protocol for KoadOS. Covers: crate structure, error handling (Zero-Panic Policy), async (Non-Blocking Rule), observability (structured tracing), documentation (missing_docs), and testing (tier system). See: [RUST_CANON](./protocols/rust-canon.md).

## S

**Sanctuary Rule**
The directive that agents are jailed to their designated workspace scope. An agent at Outpost level should not perform operations on Citadel-level paths. Enforced at the gRPC layer by the Citadel's interceptor and `koad-sandbox`. See: [Workspace Hierarchy](./architecture/workspace-hierarchy.md).

**Saveup**
An agent's post-session retrospective, written to their personal vault (e.g., `~/.tyr/memory/SAVEUPS.md`). Contains: what was accomplished, what was learned, open questions, and state for the next session. Also produced automatically by CASS's EndOfWatchPipeline as `EpisodicMemory` records.

**Sector**
The Citadel gRPC service managing shared resources and command safety. Provides `AcquireLock`/`ReleaseLock` (distributed Redis locks) and `ValidateIntent` (sandbox command validation). See: [koad-citadel](./core-systems/koad-citadel.md).

**Session**
The period during which an agent's Ghost is fused to a Body. A session has a unique `session_id`, is tracked in Redis by the Citadel, and transitions through `active` → `dark` → `purged` states. See: [Agent Session Lifecycle](./core-systems/agent-session-lifecycle.md).

**SessionRecord**
The Redis data structure (stored in `koad:state`) representing a live or recent session. Fields: `agent_name`, `session_id`, `body_id`, `rank`, `status`. Managed by `CitadelSessionService` in `koad-citadel/src/auth/session_cache.rs`.

**Signal**
The Citadel gRPC service implementing the Koad Stream event bus. Provides `Broadcast` (publish an event) and `Subscribe` (stream events to caller). Backed by Redis Streams. See: [koad-citadel](./core-systems/koad-citadel.md).

**Spine**
The legacy monolithic KoadOS architecture, now archived in `legacy/`. The Spine combined sessions, security, memory, and orchestration into a single binary. Retired in favor of the Tri-Tier Model due to fragility and maintainability issues.

**Station**
Level 2 in the Workspace Hierarchy. A project hub directory containing multiple related repositories (e.g., `/home/ideans/data/skylinks/agents/sky/`). Defined by a `koad.toml` with a `[station]` section. Context hydration at Station level includes cross-repo summaries. See: [Workspace Hierarchy](./architecture/workspace-hierarchy.md).

**Storage trait**
The Rust trait in `koad-cass/src/storage/mod.rs` that abstracts all `cass.db` interactions. Implemented by `CassStorage` (production) and `MockStorage` (tests). Allows CASS services to be tested without file I/O. See: [SQLite Storage](./data-storage/sqlite-cass-db.md).

**SymbolService**
The CASS gRPC service providing code graph queries (`Query`) and re-indexing (`IndexProject`). Backed by `koad-codegraph` (tree-sitter). See: [koad-cass](./core-systems/koad-cass.md).

**System**
Level 4 in the Workspace Hierarchy. The entire machine (`/`). Reserved for the Admiral. Agent personal vaults (e.g., `~/.tyr/`) reside at System level. See: [Workspace Hierarchy](./architecture/workspace-hierarchy.md).

## T

**TCH**
See: [Temporal Context Hydration](#temporal-context-hydration-tch).

**Temporal Context Hydration (TCH)**
The process by which CASS's `HydrationService` builds an agent's initial context packet at boot time. Performs a Hierarchy Walk scoped to the agent's workspace level, selecting recent session summaries, high-confidence facts, and local files within the token budget. See: [koad-cass](./core-systems/koad-cass.md).

**thiserror**
A Rust crate used in KoadOS library crates for defining typed, structured error enums. Contrasted with `anyhow` (used in binary crates). See: [RUST_CANON](./protocols/rust-canon.md).

**tokio**
The async runtime used throughout KoadOS. All async I/O runs on `tokio`. CPU-intensive operations are offloaded via `tokio::task::spawn_blocking` per the Non-Blocking Rule. See: [RUST_CANON](./protocols/rust-canon.md).

**tonic**
The Rust gRPC framework used for all KoadOS inter-service communication. `koad-citadel` and `koad-cass` are both `tonic` servers. `koad-cli` uses the `tonic` client to call them.

**TraceContext**
The protobuf message (defined in `proto/`) carrying `session_id` and `request_id` through every gRPC call across all three tiers. The primary observability primitive for distributed tracing in KoadOS.

**Tri-Tier Model**
The foundational architecture of KoadOS: Link (koad-agent CLI) → Body (koad-citadel) → Brain (koad-cass). Replaces the legacy monolithic Spine. See: [The Tri-Tier Model](./architecture/tri-tier-model.md).

**Tyr**
The flagship KoadOS agent. Captain rank, Lead Architect, Principal Systems Engineer. See: [Tyr: Captain](./agent-roles/tyr.md).

## V

**Vault**
An agent's personal storage directory. Tyr's vault is at `~/.tyr/`. Vaults are System-level entities. The standard structure (KAPV) includes `memory/SAVEUPS.md`, `memory/LEARNINGS.md`, `memory/PONDERS.md`. See: [Tyr: Captain](./agent-roles/tyr.md).

## W

**Workspace Hierarchy**
The four-tier model (System, Citadel, Station, Outpost) that scopes agent context and permissions based on the current working directory. Resolved by `HierarchyManager` at boot time; embedded in the session token. See: [Workspace Hierarchy](./architecture/workspace-hierarchy.md).

**WorkspaceManager**
The component in `koad-citadel` responsible for managing git worktree environments provisioned by `PersonalBayService::ProvisionWorkspace`. See: [koad-citadel](./core-systems/koad-citadel.md).

## Z

**Zero-Panic Policy**
The RUST_CANON rule forbidding `.unwrap()` and `.expect()` in production code. All fallible operations must be handled with `?`, `match`, or explicit error conversion. Permitted in test code. See: [RUST_CANON](./protocols/rust-canon.md).

**Zero-Trust**
The security model of the Citadel: every gRPC call is authenticated via the interceptor regardless of origin. There are no "trusted" internal calls that bypass validation. An agent without a valid session token cannot perform any privileged operation. See: [koad-citadel](./core-systems/koad-citadel.md).

---

*Glossary authored by Claude (Contractor) — Phase 2 of the KoadOS Support Knowledge Base.*
