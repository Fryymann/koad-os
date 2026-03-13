# Personal Bay Architecture (v5.0)
**Status:** DRAFT (Phase 1)
**Issue:** #159

## 1. Requirement
Define Personal Bay storage and isolation requirements.

## 2. Storage Model (Dual-Store)
- **SQLite (Persistence):** Local WAL file per bay (`~/.<agent>/memory/state.db`). Stores session history, FS maps, and health logs.
- **Redis (Transient):** Real-time status and lease heartbeats (`koad:session:<id>`).

## 3. Provisioning (The Forge)
- **Trigger:** Explicit `koad citadel provision-bay --agent <name>`.
- **Formatting:** Creates the directory hierarchy and initial SQLite schema.
- **Detection:** Citadel scans `config/identities/` for registered agents and verifies their bay existence at boot.

## 4. Isolation (Linux Host)
- **Strategy:** Bubblewrap (`bwrap`) for Linux-based isolation.
- **Fallback:** `chroot` if `bwrap` is unavailable.
- **Constraint:** Sanctuary Rule enforcement (No cross-bay access).

## 5. Admin Override (Dood Sovereignty)
- **Path:** Admin (Dood) can use a privileged Unix Domain Socket (`UDS`) for emergency recovery and full audit access to any Personal Bay.