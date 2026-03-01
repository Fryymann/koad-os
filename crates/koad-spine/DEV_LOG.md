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

## [2026-03-01] Sprint 4: Systemd Environment & Execution Integrity

### Overview
Hardened the CommandProcessor to ensure absolute reliability when running as a systemd user service and improved kernel concurrency.

### Milestones
- **Environment Integrity**: 
    - Implemented explicit PATH injection in `commands.rs`, including verified paths for `.cargo/bin`, `.nvm`, and `.koad-os/bin`.
    - Resolved persistent `os error 2` failures seen in background service boot logs.
- **Kernel Concurrency**: 
    - Refactored `execute_task` to use `tokio::process::Command` (async) instead of blocking standard library calls.
    - This ensures the Spine remains responsive during long-running tasks.
- **Automated Validation**: 
    - Added `test_path_integrity` to engine tests to probe for non-standard binary reachability.

### Rationale
A true operating system must provide a stable execution environment regardless of how it is launched. Explicitly managing the PATH eliminates the most common cause of background service failure.
