# Project Brief: The Citadel (KoadOS Rebuild)
**Status:** Phase 6 — Canon Lock (ACTIVE)
**Architecture:** Multi-Citadel Distribution (v3.3)
**Source of Truth:** `new_world/MASTER_DEVELOPMENT_PLAN.md`

## I. Definition
The **Citadel** is the "Body" (Infrastructure) layer of KoadOS. It serves as the central operating system responsible for session brokering, Personal Bay provisioning, shared state management, and architectural jailing. It replaces the retired "Koad Spine."

## II. Core Architecture
- **The Citadel (Body):** Infrastructure, gRPC control plane, session management.
- **CASS (Brain):** Cognitive support, 4-layer memory stack, MCP tool hosting.
- **koad-agent (Link):** CLI preparation tool, environment hydration, identity injection.

## III. Architectural Decisions & Constraints
- **Zero-Trust:** All mutations must pass through the gRPC layer with a valid `TraceContext` (Session ID + Request ID).
- **One Body, One Ghost:** Strictly one agent per CLI session. No concurrent identities in a single body.
- **Persistence:** SQLite for long-term state (Bays, Facts, History); Redis for transient state (Leases, Heartbeats, Streams).
- **Isolation:** Agents are jailed to their assigned workspace roots (Sanctuary Rule) via gRPC-level enforcement.
- **Legacy:** The Spine architecture (v1-v5) is retired to `legacy/`. Do not migrate; build clean.

## IV. Build Phases (Revised 10-Phase Plan)
1. **Cleanup & Spec:** [COMPLETE] Legacy extraction, Canon locking, gRPC proto definition.
2. **Citadel Core:** [COMPLETE] Session brokering, Personal Bays, Signal Corps (Redis Streams).
3. **CASS Core:** [COMPLETE] Memory stack integration, TCH (Temporal Context Hydration), EoW pipeline.
4. **Dynamic Tools & Sandbox:** [COMPLETE] MCP Tool Registry, config-driven sandbox, containerized execution.
5. **koad-agent MVP:** [COMPLETE] Context generation engine (context, boot, task commands).
6. **Canon Lock:** [ACTIVE] Documentation distillation and architectural stabilization.
7. **CASS Expansion:** [COMPLETE] Memory stack (L1-L4) + CASS MCP Server.
7.5. **Citadel Pulse:** [COMPLETE] Global intelligence and agent hydration.
8. **koad-agent Full:** [PLANNED] CASS integration, End-of-Watch pipeline, knowledge migration.
9. **Minion Swarm:** [PLANNED] Hangar manager, minion proto, and task delegation.
10. **Advanced Features:** [COMPLETE] Agent signaling, Federation, Knowledge Graph (code-review-graph integration).

## V. Key References
- `AGENTS.md`: Root onboarding portal and New World Canon.
- `SYSTEM_MAP.md`: Canonical workspace index and dynamic Dynamic System Map (DSM).
- `MISSION.md`: Core mission, vision, and architectural philosophy.
- `docs/protocols/`: Engineering standards and RUST_CANON.
- `updates/`: Real-time chronological board of system updates.
