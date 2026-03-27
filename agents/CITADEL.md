# Project Brief: The Citadel (KoadOS Rebuild)
**Status:** Phase 4 — Dynamic Tools & Containerized Sandboxes
**Architecture:** Tri-Tier Model (Body / Brain / Link)

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

## IV. Build Phases (The 10-Phase Plan)
1. **Cleanup & Spec:** [COMPLETE] Legacy extraction, Canon locking, gRPC proto definition.
2. **Citadel Core:** [COMPLETE] Session brokering, Personal Bays, Signal Corps (Redis Streams).
3. **CASS Core:** [COMPLETE] Memory stack integration, TCH (Temporal Context Hydration), EoW pipeline.
4. **Agent Tools & Sandbox:** [ACTIVE] WASM plugin runtime (MCP), config-driven sandbox, containerized execution.
5. **Knowledge Graph:** [PLANNED] AST-based symbol indexing and static analysis integration.
6. **Intelligence Layer:** [PLANNED] Unified InferenceRouter and local model distillation.
7. **Multi-Project Hub:** [PLANNED] Station-level coordination and cross-repo context.
8. **Autonomous Fleet:** [PLANNED] Inter-agent delegation and task orchestration.
9. **Minion Swarm:** [PLANNED] High-density, low-cost sub-agents for specialized tasks.
10. **Sovereignty:** [PLANNED] Full self-governance and automated system evolution.

## V. Key References
- `AGENTS.md`: Root onboarding portal and New World Canon.
- `SYSTEM_MAP.md`: Canonical workspace index and directory tree.
- `MISSION.md`: Core mission, vision, and architectural philosophy.
- `docs/protocols/`: Engineering standards and RUST_CANON.
- `updates/`: Real-time chronological board of system updates.
