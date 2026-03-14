## Overview

This page is the planning and research ground for a major KoadOS architectural refactor. The core proposal: **retire The Spine** as a monolithic concept and replace it with a three-tier model — **The Citadel**, **CASS (Citadel Agent Support System)**, and **Agent-Level Cognition**.

---

## 🎯 Primary Build Goal (v3 — Support-First)

<aside>
🎯

**Phase Priority — Officer Agent Support First.** The first goal of this rebuild is getting the Citadel built and stable enough to support the three Officer agents: **Tyr, Sky, and Vigil**. This v3 plan accelerates agent-support tools (Context Hydration, CASS MCP, and koad-agent) to immediately reduce agent token burn and cognitive load during the rebuild process.

</aside>

**Build sequence (top-level):**

1. **Phase 1: Lock Canon & Bootstrap Bridge** → Stabilize the blueprint and build a minimal `koad-agent` for identity loading.
2. **Phase 2: Citadel Core + MCP Bridge** → Build the "Body" (Citadel) and the "Tool Surface" (CASS MCP) concurrently for early cognitive offload.
3. **Phase 3: CASS Memory & Context Shield** → Bring cognitive continuity and token-efficient context hydration online.
4. **Continued Citadel + full KoadOS development** → All agents contributing with full CASS support.

---

## The Core Proposal

### What's changing

- **The Spine** → decomposed and retired
- **The Citadel** → central OS layer (Tyr's domain)
- **CASS** → active cognitive support layer (Memory + Tools)
- **Agent-Level Cognition** → formalized as each agent's own cognitive domain
- **"Koados" pseudonym** → retired. KoadOS is the canonical name.

### What's changing (scope update — from-scratch rewrite)

<aside>
🔴

**Scope escalation (2026-03-11):** This refactor is now a **from-scratch rewrite**, not a decomposition/rename of the existing Spine codebase. The old `koad-spine` crate and its dependents are retired wholesale — not refactored in place.

</aside>

- The old Spine codebase is **archived, not migrated**.
- The Citadel, CASS, and `koad-agent` are built as **new crates/services** with clean APIs and proto definitions.
- Config migration is replaced by **fresh `koad install`**.
- Terminology scrub is a **documentation-only pass**.

### What's NOT changing

- The Prime Directives
- Agent identities (Sky, Tyr, Vigil, Scribe)
- Tyr as Captain
- Validated architectural concepts (Sector Locking, CIP, Atomic Lease, Watchdog)
- The KoadConfig / TOML config philosophy

---

## Proposed Architecture

### 🏰 The Citadel (OS Layer)
- Agent connectivity and session brokering
- Shared state management (Redis, Sector Locking)
- Watchdog and Sentinel (Self-healing)
- **Signal Corps** — real-time observability bus
- **Trace ID chain** — E2E auditability
- **Workspace Manager** — Git Worktree orchestration
- **Personal Bays** — dedicated agent slots with scoped credentials and persistent filesystem maps
- **Zero-Trust gRPC Enforcement** — Mandatory security at the Control Plane from day one.

### 🧬 CASS (Agent Support Layer)

**1. Cognitive continuity — memory and sense of self**
- **4-layer memory stack** (L1-L4): Redis, SQLite, Qdrant
- **Mem0 middleware** for extraction, scoring, and decay
- **Memory isolation model**: private collections + shared `koados_knowledge`

**2. Cognitive offload — tools agents can invoke**
- **CASS MCP Server**: Native model tools for `intel_commit`, `intel_query`, `memory_hydrate`, `signal_send`, etc.
- **Context Compactor**: A Flash-Lite-powered service (leveraging Scribe) that distills verbose tool outputs into 1-line summaries to save tokens.

**3. Context Hydration Architecture — Three-Tier Model**
- **Tier 1 — Core**: Identity + Prime Directives (Always-loaded)
- **Tier 2 — Task-Relevant**: Station/Project context (Hydrated on demand)
- **Tier 3 — Deep Context**: Historical outcomes / Knowledge pool (On-demand via MCP)

### 🧠 Agent-Level Cognition (Local Layer)
- Independent local task state
- **Dark Mode persistence**: Standardized `.md` save format for offline-to-online reconciliation
- **Scribe as "Token Shield"**: Scribe (Crew-tier) handles cheap reading and distillation to protect Tier 1 agent token budgets.

---

## Refactor Sequencing (v3 — Prioritized Roadmap)

<aside>
🔴

**Gate rule:** Each phase requires explicit Ian approval. No code is written until canon is locked and all 🔴 items are resolved.

</aside>

### Phase 0 — Legacy Knowledge Extraction & Archive
1. **Legacy state extraction** — Dump Redis and SQLite memory banks.
2. **Archive old codebase** — Move legacy crates to `archive/`.
3. **Pre-archive cleanup** — Remove stale files and redundancy.

### Phase 1 — Lock Canon & Bootstrap Bridge
1. **Lock canon** — Define the Tri-Tier Model in the Core Contract.
2. **Resolve all 🔴 items** — Dark Mode format, Zero-Trust gRPC, Migration protocol.
3. **Phase 1.5: Minimal `koad-agent`** — Build the shell-prep tool for identity loading and env variable export. **Resolves the "Bootstrap Gap."**

### Phase 2 — Build the Citadel & MCP Bridge
1. **New `koad-citadel` crate** — Session brokering + Personal Bays. **Zero-Trust enforced from day one.**
2. **Phase 2.5: CASS MCP Bridge** — Deploy the MCP tool surface for early cognitive offload (status, heartbeats, minimal memory commits).
3. **Phase 2.6: Context Hydration (v1)** — Implement the 3-tier dynamic context generation to stop token-burn from static directives.
4. **Signal Corps & Trace ID** — Observability bus and audit trail.

### Phase 3 — Build CASS Memory & Continuity
1. **Memory Stack (L1/L2 Priority)** — Deploy Redis Stack and SQLite Episodic memory first.
2. **Memory Stack (L3/L4)** — Deploy Qdrant Semantic memory and Procedural stores.
3. **Context Compactor** — Deploy the Flash-Lite distillation service to keep active context windows lean.
4. **Data migration tool** — `koad system migrate-v5`.

### Phase 4 — Full `koad-agent` & Ghost Model
1. **Full `koad-agent` CLI** — Preflight validation, session restoration, and ghost config loading.
2. **Ghost config system** — Identity TOML integration.

### Phase 5 — Integration & Documentation
1. **Scrub Spine Terminology** — Systematic documentation-only pass.
2. **Core Contract version bump** — v2.4.
3. **Announce to KAI** — Final agent-level briefings.

### Phase 6 & 7 — Advanced Memory & Future
- Mem0 integration hooks, A2A-S (Agent-to-Agent Signals), and future Swarm Hangar.

---

## Terminology Shift

| **Old term** | **Replace with** |
| --- | --- |
| The Koad Spine / The Spine | The Citadel (Infra) / CASS (Support) |
| k-spine | citadel (lowercase for CLI/code) |
| Spine session | Citadel session |
| Sentinel / Preflight | `koad-agent` preparation |

---

**Prepared by:** Tyr, Captain (KAI Officer)
**Revision:** v3.0 (2026-03-13)
**Approved by:** [Awaiting Dood Signature]
