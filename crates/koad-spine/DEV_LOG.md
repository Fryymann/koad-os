# Koad Spine: Development History

## [2026-03-01] Sprint 0: Backbone Stabilization

### Overview
Initiated the Spine Mini-Project to resolve cross-environment (WSL/Windows) instability, environment pathing errors (`os error 2`), and diagnostic invisibility.

### Architectural Milestones
- **Schema Stabilization (Phase 1)**: 
    - Defined `proto/spine.proto` with `EnvironmentType` and `ServiceEntry` for cross-env discovery.
    - Locked Redis Hot-Path JSON schemas for tasks, events, and services.
- **Diagnostic Visibility (Phase 2)**:
    - Upgraded `ShipDiagnostics` to emit `SYSTEM_HEARTBEAT` to a persistent Redis Stream (`koad:events:stream`).
    - Implemented `spine-logs.py` for real-time, color-coded event tailing.
    - Implemented `spine-check.py` for granular service reachability probing.

### Diagnostic Results (Baseline)
- **Redis UDS**: PASS (Reachable via `koad.sock`)
- **gRPC Backbone**: FAIL (Connection refused on 50051)
- **Web Deck**: PASS (Bound on 3000)
- **Service Inventory**: WARN (Empty/Unpopulated)

### Phase 3: Robust Execution (Sprint 1)
- **Status**: Code Complete; Validation in Progress.
- **Milestones**:
    - Implemented **Dual gRPC Servers** (UDS + TCP 50051) for cross-environment support.
    - Updated `main.rs` to initialize background listeners for both local WSL and Windows 11 bridge.
    - Resolved `os error 98` (Address already in use) by implementing a "Zombie" cleanup strategy in the diagnostic tools.
- **Challenge**: Port 50051 binding is delayed due to Spine initialization order. Future fix: Move gRPC binding to a dedicated early-boot module.
- **Identity**: Confirmed the Spine is correctly scanning `skills` and `doodskills` directories and populating the `Morning Report`.

## [2026-03-01] Sprint 3: Full gRPC Implementation & 0.0.0.0 Parity

### Overview
Successfully transitioned the Spine from mock listeners to a full **Tonic gRPC Server** implementation and achieved `0.0.0.0` binding across all Edge Gateway protocols.

### Milestones
- **Schema Unification**: Consolidated all protos into the root `proto/` directory and updated `koad-proto` to generate `koad.spine.v1` traits.
- **Tonic Integration**: 
    - Replaced mock listeners in `main.rs` with dual `tonic::transport::Server` instances (UDS + TCP 50051).
    - Implemented the `SpineService` trait in `rpc/mod.rs` with intent-based task dispatch.
- **Connectivity Lockdown**: 
    - Confirmed via `ss` that ports `3000` (Web) and `50051` (gRPC) are bound to `0.0.0.0`.
    - Verified cross-environment reachability using `spine-check.py`.

### Rationale
Full gRPC support on `0.0.0.0` allows Windows-native tools (like a dedicated TUI or Dashboard backend) to interact with the KoadOS kernel with the same performance and security as local WSL tools.
