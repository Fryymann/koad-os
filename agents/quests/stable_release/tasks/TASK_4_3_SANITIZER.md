# Task Manifest: 4.3 - The Distribution Sanitizer (`koad-scrub`)
**Status:** ✅ Complete
**Assignee:** Clyde (+ clyde-dev / clyde-qa)
**Reviewer:** Tyr (Captain)
**Priority:** High

---

## 🎯 Objective
Create a reliable "Citadel-to-Distribution" bridge tool. This ensures the KoadOS repository can be shared, cloned, and distributed without leaking local agent data, private logs, or sensitive database state.

## 🧱 Technical Requirements

### 1. Tool Implementation
- **Component:** Implement as a new module in `koad-cli` or a dedicated shell script `bin/koad-scrub`.
- **Interface:** `koad system scrub [--force]`

### 2. Scrub Targets (The "Clean Slate" List)
- **Databases:** Delete all SQLite files in `data/db/` (except tracked templates).
- **Logs:** Truncate or remove all files in `logs/`.
- **Runtime:** Remove all sockets (`.sock`) and PID files (`.pid`) in `run/`.
- **Bays:** Wipe all agent state directories in `agents/bays/`.
- **KAPVs:** Remove all agent-specific vaults in `agents/KAPVs/` (preserving `TEMPLATE.md`).
- **History:** Truncate `SESSIONS_LOG.md` and reset `TEAM-LOG.md` to the release header.
- **Cache:** Clear `~/.koad-os/cache/`.

### 3. Safety Enforcement
- **Confirmation:** The tool MUST prompt the user for confirmation unless `--force` is provided.
- **Git Check:** Check for uncommitted changes in the `distribution` layer before scrubbing.

## ✅ Verification Strategy
1.  **Dry Run:** Verify that the tool correctly identifies all target paths without deleting them.
2.  **Full Scrub:** Run the tool on a working Citadel and verify the resulting directory tree matches the "Pure Distribution" specification in `MISSION.md`.
3.  **Bootstrap Test:** Verify that a "Scrubbed" Citadel can be successfully re-initialized using `koad system init`.
