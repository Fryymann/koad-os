# Koad Spine: Diagnostic & Resilience Suite

This document defines the granular testing protocols for the KoadOS Spine. These tests are designed to verify the kernel's behavior under stress and environment-specific failures.

## 1. Service Probing (`koad dood spine-check`)
A dedicated diagnostic tool that performs "Deep Probing" of the Service Gateway.

### A. Reachability Matrix
- [ ] **WSL -> UDS**: Can local tools talk to `koad.sock`?
- [ ] **WSL -> TCP**: Can local tools talk to `0.0.0.0:50051`?
- [ ] **Windows -> TCP**: Can a Windows-native probe reach the WSL bridge?
- [ ] **Chrome -> WebDeck**: Is the Axum WebSocket responding with a valid `SYSTEM_HEARTBEAT`?

### B. Port Resilience (The "Zombie" Test)
- [ ] **Scenario**: Start Spine, kill it with `SIGKILL`, and immediately attempt a restart.
- [ ] **Expected**: Spine must identify the orphaned port, clear it, and bind successfully without "Address already in use."

## 2. Edge-Case Scenarios

### C. The "Buffer Bloat" Test
- [ ] **Scenario**: Run a task that outputs 10MB of text to `stdout`.
- [ ] **Expected**: Spine must stream the output to Redis without blocking the main event loop or exceeding memory limits.

### D. The "Sync Drift" Test
- [ ] **Scenario**: Disconnect the SQLite persistence layer while tasks are running in Redis.
- [ ] **Expected**: Spine must queue the "Write-Behind" operations and flush them once SQLite is restored, ensuring zero state loss.

### E. The "Identity Collision" Test
- [ ] **Scenario**: Register two components with the same name (e.g., two TUI decks).
- [ ] **Expected**: Spine must issue a unique `session_id` to each and track them as distinct active components.

## 3. Stress & Performance Limits
- [ ] **Concurrent Dispatch**: 100 tasks/second for 10 seconds.
- [ ] **Redis Stream TTL**: Verify that the `koad:events:stream` is trimmed to prevent memory exhaustion over long uptimes.
- [ ] **UDS Latency**: Measure IPC overhead to ensure gRPC over UDS remains < 1ms.
