# Crate: koad-citadel
**Status:** Complete (Core Orchestrator)
**Port:** :50051

## Purpose
The central nervous system of KoadOS. This crate implements the Citadel gRPC service, responsible for session management, agent bay provisioning, inter-agent signaling, and architectural jailing.

## Source Map
- `main.rs`: Entry point; initializes the `Kernel`.
- `kernel.rs`: Orchestration engine; manages listeners, reapers, and service lifecycles.
- `auth/`: Zero-trust security layer and Sanctuary Rule enforcement.
- `services/`:
    - `session.rs`: Lease management, heartbeats, and session state.
    - `bay.rs`: Provisioning and management of agent-specific workspace bays.
    - `signal.rs`: High-speed message routing via the Signal Corps.
    - `xp.rs`: Experience point tracking and persistent ledger management.
- `state/`:
    - `storage_bridge.rs`: SQLite/Redis hybrid persistence layer.
    - `bay_store.rs`: Filesystem-level bay management and auto-provisioning.
- `signal_corps/`: Low-level Redis stream orchestration.

## Public API (gRPC)
- `CitadelSessionServer`: Boot, lease, and heartbeat protocols.
- `PersonalBayServer`: Bay discovery and workspace registration.
- `SignalServer`: A2A-S message publishing and subscription.
- `XpServiceServer`: XP status queries and awards.

## Dependencies
- `koad-core`: Shared types and config.
- `koad-proto`: gRPC service definitions.
- `koad-sandbox`: Security evaluation logic.
