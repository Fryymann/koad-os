# Task Manifest: 2.2 - Graceful Service Lifecycle
**Status:** ⚪ Draft
**Assignee:** [Engineer-Agent (Cid/Clyde)]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `feature/graceful-shutdown`

---

## 🎯 Objective
Implement a robust, coordinated shutdown sequence for all KoadOS services (`koad-citadel`, `koad-cass`). Ensure that data is drained from the hot path (Redis) to the cold path (SQLite) and that all system resources (sockets, locks) are cleanly released.

## 🧱 Context
Currently, the Citadel kernel is simply dropped when the main process receives a SIGINT (Ctrl-C). This risks data loss if the storage drain loop hasn't finished and can leave orphaned socket files in the `run/` directory. A shareable release must be "Good Citizens" of the OS.

## 🛠️ Technical Requirements

### 1. Signal Trapping (Main Entry Points)
- **Files:** `crates/koad-citadel/src/main.rs`, `crates/koad-cass/src/main.rs`
- **Requirement:** Update the `tokio::signal::ctrl_c().await` logic to call `kernel.shutdown().await` (or the equivalent for CASS) before the process exits.
- **Requirement:** Capture `SIGTERM` (via `tokio::signal::unix::signal`) to ensure clean shutdown when managed by systemd or Docker.

### 2. Coordinated Shutdown Sequence (`Kernel`)
- **File:** `crates/koad-citadel/src/kernel.rs`
- **Requirement:** Refactor `Kernel::shutdown()` to be a blocking async function that performs the following steps in order:
    1.  **Signal Stop:** Notify all listeners (via `shutdown_tx`) to stop accepting new requests.
    2.  **Grace Period:** Allow active gRPC calls 100-200ms to complete.
    3.  **Final Drain:** Explicitly call `storage.drain_all().await?` to sync all Redis state to SQLite.
    4.  **Resource Cleanup:** Delete the Admin UDS socket (`run/kadmin.sock`) and any other UDS sockets managed by the kernel.
    5.  **Telemetry:** Log a "Graceful Shutdown Complete" event.

### 3. Storage Drain Verification
- **File:** `crates/koad-core/src/storage.rs` (and implementations)
- **Requirement:** Ensure `drain_all()` is idempotent and safe to call even if the background drain loop is already running.

### 4. CASS Shutdown Parity
- **File:** `crates/koad-cass/src/server.rs` (or equivalent)
- **Requirement:** Implement a similar shutdown logic for `koad-cass` to ensure cognitive memories are flushed to disk before exit.

## ✅ Verification Strategy
1.  **Mock Data Drain:** Insert a value into Redis via a gRPC call. Immediately Ctrl-C the Citadel. Verify the value exists in `citadel.db` (SQLite) after exit.
2.  **Socket Cleanup:** Start Citadel, verify `run/kadmin.sock` exists. Ctrl-C, verify `run/kadmin.sock` is deleted.
3.  **SIGTERM Test:** Send `kill <pid>` to a running Citadel and verify the log shows "Initiating graceful shutdown."

## 🚫 Constraints
- **MAX** shutdown time: 3 seconds. If some tasks hang, force exit after the timeout.
- **NO** panic during shutdown—log errors but continue the teardown.

---

## 🛰️ Sovereign Review (Tyr)
- Confirm that `drain_all` covers all volatile namespaces (Sessions, Signals, Health).
- Verify that the gRPC servers stop receiving *before* the storage is drained.
