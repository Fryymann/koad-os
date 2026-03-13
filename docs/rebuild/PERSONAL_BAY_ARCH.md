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
- **Workspace Integration:** The **Workspace Manager** (Citadel Service) provisions Git Worktrees for assigned tasks and mounts them to the Bay's filesystem map at `~/.koad-os/workspaces/{agent}/{task_id}/`.

## 4. Isolation (Linux Host)
- **Primary (Logical):** The Citadel gRPC layer enforces the **Sanctuary Rule** by validating every file-operation path against the agent's assigned `KOAD_WORKSPACE_ROOT` (lookup from FS Map).
- **Secondary (Physical):** Bubblewrap (`bwrap`) is used for kernel-level namespace isolation if available on the host. 
- **Fallback:** If `bwrap` is unavailable, the system operates in "Logical Isolation" mode. `chroot` is reserved for high-privilege administrative sessions.
- **Constraint:** Zero-trust architecture assumes the agent shell may attempt to bypass logical guards; redundant server-side path validation is mandatory.

## 5. Docking State Machine (Lifecycle)
The Citadel manages the following agent states:
- **DORMANT:** Identity registered, no active lease.
- **DOCKING:** Handshake/Auth in progress.
- **HYDRATING:** Context loading (Tier 1/2) via CASS.
- **ACTIVE:** Session live, heartbeat established.
- **WORKING:** Active tool/execution phase.
- **DARK:** Heartbeat missed (>30s).
- **TEARDOWN:** Brain Drain protocol, lock release, session closure.

## 6. Admin Override (Dood Sovereignty)
- **Path:** Admin (Dood) can use a privileged Unix Domain Socket (`UDS`) for emergency recovery and full audit access to any Personal Bay.