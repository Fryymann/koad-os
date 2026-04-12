# Task Manifest: 2.1 - Autonomic Recovery (The Doctor is In)
**Status:** ⚪ Draft
**Assignee:** [Engineer-Agent (Cid/Clyde)]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `feature/autonomic-recovery`

---

## 🎯 Objective
Transform the `koad doctor --fix` command into a fully functional "Autonomic Recovery" engine. It must not only detect system failures but also perform safe, automated repairs to restore the Citadel's operational integrity.

## 🧱 Context
Currently, `koad doctor` is primarily diagnostic. For a stable release, the system must be capable of self-healing common environmental issues (stale sockets, orphaned Redis keys, crashed containers) to minimize manual intervention by the Admiral.

## 🛠️ Technical Requirements

### 1. Stale Socket Recovery
- **Issue:** Abandoned `.sock` files in `run/` prevent services from restarting.
- **Fix:** If a socket file exists but the corresponding process (Citadel, Cass, Redis) is not running, the Doctor must safely remove the orphaned socket.

### 2. Redis State Reconciliation
- **Issue:** Stale health registry or session keys in Redis cause "Ghost" status reports.
- **Fix:** Implement a `koad:state` cleanup routine. If the Citadel is confirmed offline, the Doctor should purge volatile keys in the `koad:state` hash to ensure a clean slate upon next boot.

### 3. Docker Stack Resurrection
- **Issue:** Redis or Qdrant containers are stopped or paused.
- **Fix:** Use `docker compose up -d` within the Fix logic to restart missing infrastructure components if Docker is available.

### 4. Database Integrity Sweep
- **Issue:** SQLite WAL files left in a "busy" state after a crash.
- **Fix:** Perform a `PRAGMA integrity_check;` and a vacuum/re-index if necessary to ensure the Memory Bank is consistent.

### 5. CLI Implementation (`koad-cli/src/handlers/status.rs`)
- **Requirement:** Implement the logic inside the `if fix { ... }` block.
- **Requirement:** Provide high-signal output for each fix attempted (e.g., `[HEAL] Removed orphaned kcitadel.sock`).
- **Requirement:** Require a `--confirm` or `-y` flag for destructive fixes (like purging Redis keys).

## ✅ Verification Strategy
1.  **Orphaned Socket Test:** Manually `touch run/kcitadel.sock` while Citadel is offline. Run `koad doctor --fix` and verify the file is removed.
2.  **Stale Redis Test:** Insert a dummy key into `koad:state`. Run `koad doctor --fix` and verify it's purged.
3.  **Container Recovery:** Stop the `koad-redis-stack` container. Run `koad doctor --fix` and verify it is restarted.

## 🚫 Constraints
- **NEVER** delete data files (`.db`, `.rdb`) unless explicitly requested via an advanced flag.
- **NEVER** kill a running process unless it's confirmed to be a "Ghost" (unresponsive).
- **MUST** provide a summary of all "Heal" actions performed at the end of the run.

---

## 🛰️ Sovereign Review (Tyr)
- Verify that the recovery logic is non-destructive and prioritizes system safety.
- Ensure the Doctor's output remains professional and informative.
