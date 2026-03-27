# Project Brief: The Citadel (KoadOS Rebuild)
**Status:** Phase 1 — Citadel MVP Construction
**Architecture:** Tri-Tier Model (Body / Brain / Link)

## I. Definition
The **Citadel** is the "Body" (Infrastructure) layer of KoadOS. it serves as the central operating system responsible for session brokering, Personal Bay provisioning, shared state management, and architectural jailing. It replaces the retired "Koad Spine."

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

## IV. Build Phases
1. **Cleanup & Spec:** Legacy extraction, Canon locking, gRPC proto definition.
2. **Citadel MVP:** Session brokering, Personal Bays, Signal Corps (Redis Streams).
3. **CASS:** Memory stack integration (Mem0, Qdrant), MCP server.
4. **koad-agent:** Boot tool and identity hydration.
5. **Integration:** Full terminology scrub and documentation.

## V. Key References
- `@~/.koad-os/AGENTS.md` (New World Canon)
- `@~/.koad-os/new_world/tyr_plan_review.md` (Strategic Review)
- `github.com/Fryymann/koad-os/projects/6` (Active Rebuild Board)
