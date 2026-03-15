# KoadOS Support Knowledge Base — Article Index

**Phase 2 Status:** Complete (11 articles)
**Generated:** 2026-03-15
**Author:** Claude (Contractor)

This index is the entry point for the KoadOS support knowledge base. Articles are written for developers familiar with Rust and systems programming but new to KoadOS. Scribe uses this index as a navigation layer for RAG-backed support queries.

---

## Start Here: Recommended Reading Order

If you're new to KoadOS, read in this order:

1. [The Tri-Tier Model](./architecture/tri-tier-model.md) — What is KoadOS and how are its three layers organized?
2. [The Body/Ghost Model](./architecture/body-ghost-model.md) — What is an agent's identity and how does it persist across sessions?
3. [koad-agent boot](./core-systems/koad-agent-boot.md) — How do you actually start an agent session?
4. [Agent Session Lifecycle](./core-systems/agent-session-lifecycle.md) — What happens to a session from boot to logout?
5. [koad-citadel](./core-systems/koad-citadel.md) — Deep dive into the infrastructure kernel
6. [koad-cass](./core-systems/koad-cass.md) — Deep dive into the cognitive layer
7. [Cargo Workspace](./tooling/cargo-workspace.md) — How is the codebase organized?
8. [RUST_CANON](./protocols/rust-canon.md) — What are the coding standards?

---

## Full Article List by Category

### Architecture & Concepts

| Title | Complexity | Summary | Link |
|-------|------------|---------|------|
| The Tri-Tier Model | basic | The foundational Body/Brain/Link architecture — Citadel, CASS, and koad-agent — and how they communicate via gRPC. | [→](./architecture/tri-tier-model.md) |
| The Body/Ghost Model | basic | The separation between an agent's ephemeral shell session (Body) and its persistent identity and memory (Ghost). | [→](./architecture/body-ghost-model.md) |
| The Workspace Hierarchy | intermediate | The four-tier filesystem scope model (System → Citadel → Station → Outpost) that controls agent context and permissions. | [→](./architecture/workspace-hierarchy.md) |

### Core Systems & Subsystems

| Title | Complexity | Summary | Link |
|-------|------------|---------|------|
| koad-citadel | advanced | The "Body" — the persistent gRPC kernel managing sessions, security, resource locks, the event bus, and agent bays. | [→](./core-systems/koad-citadel.md) |
| koad-cass | advanced | The "Brain" — the cognitive gRPC service providing memory storage, context hydration, code graph queries, and EndOfWatch summarization. | [→](./core-systems/koad-cass.md) |
| koad-agent boot | intermediate | The "Link" command that hydrates a shell with agent identity and Citadel session credentials via `eval $(koad-agent boot ...)`. | [→](./core-systems/koad-agent-boot.md) |
| Agent Session Lifecycle | intermediate | The `active` → `dark` → `purged` state machine governing agent sessions, heartbeats, and automatic cleanup. | [→](./core-systems/agent-session-lifecycle.md) |

### Protocols & Governance

| Title | Complexity | Summary | Link |
|-------|------------|---------|------|
| RUST_CANON | intermediate | The mandatory Rust coding standards — error handling, async rules, documentation requirements, and testing tiers. | [→](./protocols/rust-canon.md) |

### Agent Roles & Responsibilities

| Title | Complexity | Summary | Link |
|-------|------------|---------|------|
| Tyr: Captain & Lead Architect | basic | The flagship KoadOS agent — Captain-rank, principal architect, and governor of the Citadel rebuild. | [→](./agent-roles/tyr.md) |

### Data & Storage

| Title | Complexity | Summary | Link |
|-------|------------|---------|------|
| SQLite Storage (cass.db) | intermediate | The "cold path" persistent memory store — schema, `Storage` trait abstraction, `FactCard`/`EpisodicMemory` tables, and direct inspection. | [→](./data-storage/sqlite-cass-db.md) |

### Tooling & Developer Workflow

| Title | Complexity | Summary | Link |
|-------|------------|---------|------|
| Cargo Workspace (koad-os) | intermediate | The multi-crate workspace structure — crate roles, dependency hierarchy, shared version management, and `cargo` command patterns. | [→](./tooling/cargo-workspace.md) |

---

## Topic Relationship Map

```
The Tri-Tier Model
├── koad-citadel (The Body)
│   ├── Agent Session Lifecycle
│   ├── koad-agent boot (client side)
│   └── (enforces) Workspace Hierarchy
├── koad-cass (The Brain)
│   ├── SQLite Storage / cass.db
│   └── (uses) Agent Session Lifecycle (EndOfWatch)
└── koad-agent boot (The Link)
    └── (implements) The Body/Ghost Model

The Body/Ghost Model
└── Agent Session Lifecycle

RUST_CANON
└── Cargo Workspace (structural rules)

Tyr: Captain
├── The Body/Ghost Model (how Tyr exists as an agent)
├── koad-agent boot (how Tyr starts a session)
└── RUST_CANON (Tyr enforces this)
```

---

## Coverage Gaps

The following topics were identified in the outline INDEX.md as needing future documentation. These are gaps in Phase 2 coverage — outlines do not yet exist for them, so articles could not be written. Tyr or Scribe should produce outlines for these in a future pass.

**Sub-Crate Deep Dives:**
- `koad-intelligence` — AI inference routing, model provider abstraction, significance scoring
- `koad-sandbox` — Command policy engine, rank-based bypass logic, path validation
- `koad-codegraph` — tree-sitter integration, symbol index structure, re-indexing triggers
- `koad-watchdog` — Watchdog service purpose and operations (crate exists but no outline)
- `koad-board` — Board/notification service (crate exists but no outline)
- `koad-bridge-notion` — Notion integration (crate exists but no outline)

**Protocol Details:**
- `KSRP` (Koad Self-Review Protocol) — The self-review process Tyr and Claude follow before submitting PRs
- `PSRP` (Personal Self-Review Protocol) — Variant for personal/memory work
- Contributor Canon — Sibling protocol to RUST_CANON covering contribution workflow

**Data Primitives:**
- `FactCard` — Detailed article on the knowledge primitive: fields, scoring, lifecycle
- `EpisodicMemory` — Detailed article on session summaries: generation, storage, retrieval
- Redis usage patterns — Hot state: session keys, lock keys, stream keys, TTL strategy

**Agent Roles:**
- Sky — CASS architect and memory specialist
- Scribe — Context scout, documentation agent, RAG support provider
- Cid — Systems/infrastructure engineer, CI/CD
- Helm — (if active) Navigation and routing agent

**Tooling:**
- Shell functions — `koad-auth`, `koad-refresh`, and other injected utilities
- `koad-agent` CLI commands beyond boot — `status`, `doctor`, `logout`, etc.
- Temporal Context Hydration (TCH) — Standalone deep-dive article

---

*Knowledge base authored by Claude (Contractor), Phase 2. Scribe serves Phase 3 RAG.*
