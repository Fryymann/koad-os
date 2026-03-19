## Overview

This page is the planning and research ground for a major KoadOS architectural refactor. The core proposal: **retire The Spine** as a monolithic concept and replace it with a three-tier model — **The Citadel**, **CASS (Citadel Agent Support System)**, and **Agent-Level Cognition**, all operating within a **Level-Aware Workspace Hierarchy**.

---

## 🎯 Primary Build Goal (v3.2 — Level-Aware Support)

<aside>
🎯

**Phase Priority — Officer Agent Support First.** The first goal of this rebuild is getting the Citadel built and stable enough to support the three Officer agents: **Tyr, Sky, and Vigil**. This v3.2 plan accelerates agent-support tools and implements the **Citadel → Station → Outpost** hierarchy to immediately reduce token burn and cognitive load.

</aside>

**Build sequence (top-level):**

1. **Phase 0: Diagnostic Harness & Documentation Shield** → [COMPLETE] High-ROI pre-Citadel wins.
2. **Phase 1: Citadel Core (Control Plane)** → Stabilize the kernel, gRPC service, and **Level-Aware Authorization**.
3. **Phase 2: CASS Core (Agent Support)** → Build memory hydration, context management, and **Hierarchy-Aware TCH**.
4. **Phase 3: Agent Tools Layer** → Advanced code knowledge graphs and execution sandboxes.

---

## The Core Proposal: Workspace Levels

KoadOS adoption of the **Game Map Metaphor** for information topology:

| **Level** | **Scope** | **Contains** | **Agents** |
| --- | --- | --- | --- |
| **System** | Full Machine | Global configs, Officer vaults (~/.tyr) | Tyr (Captain) |
| **Citadel** | Platform Core | Core protocols, platform-level data (~/.koad-os) | Scribe, Vigil |
| **Station** | Project Hub | Shared domain resources (e.g., ~/skylinks) | Sky (Specialist) |
| **Outpost** | Single Repo | Local code, task-specific state (.agents/) | Crew / Scouts |

### Key Principles
- **Locality of Reference**: Most work is local (Outpost).
- **Inheritance vs. Isolation**: Lower levels benefit from higher standards but remain jail-safe.
- **The .agents/ Interface**: Universal entry point for agent data at every level.

---

## Proposed Architecture

### 🏰 The Citadel (OS Layer)
- **Hierarchy Manager**: Detects and validates current Workspace Level (Outpost vs Station).
- Agent connectivity and session brokering.
- **Zero-Trust gRPC Enforcement**: Mandatory security at the Control Plane from day one.

### 🧬 CASS (Agent Support Layer)
- **Temporal Context Hydrator (TCH)**: Selective loading based on level (Outpost Local + Station Pointers).
- **Context Compactor**: Flash-Lite distillation service.

---

## Refactor Sequencing (v3.2 — Prioritized Roadmap)

### Phase 0 — Diagnostic Harness & Documentation Shield (RESULTS)
- **Token ROI**: 30-40% reduction achieved via Domain Indices.
- **Observability**: Telemetry active at `~/.koad-os/logs/telemetry.log`.
- **Protocol**: EoW Schema locked at `~/.koad-os/docs/protocols/EOW_SCHEMA.toml`.
- **Status**: 🟢 COMPLETE (2026-03-14)

### Phase 1 — Citadel Core (Control Plane) — [COMPLETE]
<aside>
🔵 **Goal: Replace the Spine with a stable gRPC kernel.**
- **Status**: 🟢 COMPLETE (2026-03-14)
- **Outcome**: Level-aware kernel, Zero-Trust Interceptor, and Proto v5.1 active.

</aside>

1. **New `koad-citadel` crate** — Design `citadel.proto` with level-awareness.
2. **Configuration-First Architecture** — Zero hardcoded values. 
3. **Hierarchy-Based Authorization (Sanctuary Rule)** — Enforce gRPC-layer auth based on level depth.
4. **Stable Session Lifecycle** — Hardened boot/heartbeat/drain/purge cycle.

### Phase 2 — CASS Core (Agent Support Layer) — [COMPLETE]
<aside>
🟣 **Goal: Stand up cognitive support infrastructure.**
- **Status**: 🟢 COMPLETE (2026-03-14)
- **Outcome**: Memory services, TCH, and EoW pipeline active.

</aside>

1. **Memory Query Interface** — RPC interface for FactCard retrieval.
2. **EndOfWatch Pipeline** — Automated EoW generation on session close.
3. **Hierarchy-Aware TCH** — Load context according to Citadel → Station → Outpost depth.
4. **Dark Mode Persistence** — Standardized offline-to-online reconciliation format.

### Phase 3 — Agent Tools Layer (Stability & HARDENING) — [COMPLETE]
<aside>
🛡️ **Goal: Advanced intelligence and security tools.**
- **Status**: 🟢 COMPLETE (2026-03-15)
- **Outcome**: Local Ollama distillation, Config-driven Sandbox, and AST CodeGraph active.

</aside>

1. **Intelligence Layer (`koad-intelligence`)** — Unified `InferenceRouter` for task-based model selection.
2. **Security Sandbox (`koad-sandbox`)** — Config-driven sanctuary and blacklist enforcement.
3. **Code Knowledge Graph (`koad-codegraph`)** — AST-based symbol indexing using `tree-sitter`.

### Phase 4 — Dynamic Tools & Containerized Sandboxes
<aside>
🚀 **Goal: Externalize tool execution and dynamic loading.**

</aside>

1. **MCP Tool Registry** — CASS service for registering and invoking Model Context Protocol (MCP) tools. [ACTIVE]
2. **Filesystem MCP Server** — Integrate the standard MCP Filesystem server into the containerized toolbox. [APPROVED]
3. **Code Execution Sandbox** — Docker/Podman isolation for running arbitrary agent code.
4. **Dynamic Library Loading** — Allow CASS to load custom tool implementations at runtime.

---

**Prepared by:** Tyr, Captain (KAI Officer)
**Revision:** v3.2 (2026-03-14)
**Approved by:** Ian [APPROVED]
