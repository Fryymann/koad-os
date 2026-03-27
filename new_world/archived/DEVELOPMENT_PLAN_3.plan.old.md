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
2. **Phase 1: Citadel Core (Control Plane)** → [COMPLETE] Stabilize the kernel, gRPC service, and **Level-Aware Authorization**.
3. **Phase 2: CASS Core (Agent Support)** → [COMPLETE] Build memory hydration, context management, and **Hierarchy-Aware TCH**.
4. **Phase 3: Agent Tools Layer** → [COMPLETE] Advanced code knowledge graphs and execution sandboxes.
5. **Phase 4: Dynamic Tools & Containerized Sandboxes** → [NEXT] MCP Registry and isolated tool runs.

---

## The Core Proposal: Workspace Levels

KoadOS adoption of the **Game Map Metaphor** for information topology:

| **Level** | **Scope** | **Contains** | **Agents** |
| --- | --- | --- | --- |
| **System** | Full Machine | Global configs, Officer vaults (~/.tyr) | Tyr (Captain) |
| **Citadel** | Platform Core | Core protocols, platform-level data (~/.koad-os) | Scribe, Vigil |
| **Station** | Project Hub | Shared domain resources (e.g., ~/skylinks) | Sky (Specialist) |
| **Outpost** | Single Repo | Local code, task-specific state (agents/) | Crew / Scouts |

---

## Refactor Sequencing (v3.2 — Prioritized Roadmap)

### Phase 0 — Diagnostic Harness & Documentation Shield (RESULTS)
- **Status**: 🟢 COMPLETE (2026-03-14)

### Phase 1 — Citadel Core (Control Plane) — [COMPLETE]
- **Status**: 🟢 COMPLETE (2026-03-14)
- **Outcome**: Level-aware kernel, Zero-Trust Interceptor, and Proto v5.1 active.

### Phase 2 — CASS Core (Agent Support Layer) — [COMPLETE]
- **Status**: 🟢 COMPLETE (2026-03-14)
- **Outcome**: Memory services, TCH, and EoW pipeline active.

### Phase 3 — Agent Tools Layer (Stability & HARDENING) — [COMPLETE]
- **Status**: 🟢 COMPLETE (2026-03-15)
- **Outcome**: Local Ollama distillation, Config-driven Sandbox, and AST CodeGraph active.

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
**Revision:** v3.2 (2026-03-15)
**Approved by:** Ian [APPROVED]
