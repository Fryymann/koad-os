# Task Manifest: 4.3 - The Distribution Sanitizer
**Status:** ⚪ Draft
**Assignee:** [Engineer-Agent (Cid/Clyde)]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `ops/dist-sanitizer`

---

## 🎯 Objective
Create a reliable "Scrub" utility to transition a local Citadel (with months of history and data) into a "Pure Distribution" state. This ensures that no personal data, session history, or instance-specific databases are leaked during a public release.

## 🧱 Context
KoadOS naturally accumulates a large amount of "Instance" data (bay state, CASS cognition, command history). For a stable, shareable release, we must have a one-command way to "Sanitize" the repository, leaving only the "Distribution" (the code, templates, and protocols).

## 🛠️ Technical Requirements

### 1. Script Implementation (`scripts/koad-sanitize.sh`)
- **Requirement:** Implement a robust bash script that performs a "Deep Clean" of the Citadel.
- **Requirement:** **Mandatory Destructive Actions:**
    - Purge all files in `run/` (sockets, pids).
    - Purge all logs in `logs/`.
    - Purge all session briefs in `cache/`.
    - Purge all agent bay databases in `agents/bays/`.
    - Purge all command history: `~/.koad-os/agents/KAPVs/*/sessions/bash_history`.
- **Requirement:** **Optional (with --full) Actions:**
    - Purge the primary SQLite databases: `data/db/*.db`.
    - Purge the Redis persistence file: `data/redis/dump.rdb`.

### 2. Safeguard Mechanisms
- **Requirement:** The script MUST require a `--confirm` flag to execute.
- **Requirement:** The script MUST check if any Citadel services are running (`koad status`) and refuse to run if they are active (to prevent database corruption).

### 3. Integration with Bootstrap
- **Requirement:** Ensure that after a "Sanitize" run, `bash install/bootstrap.sh` can still successfully re-initialize the environment.

### 4. Git Ignore Verification
- **Requirement:** Verify that the `.gitignore` in the repo root correctly covers all the data directories targeted by the sanitizer (e.g., `data/db/`, `run/`, `logs/`).

## ✅ Verification Strategy
1.  **Sanity Run:** Populate a local Citadel with fake logs and bay data. Run `koad-sanitize --confirm`. Verify that all targeted files are deleted.
2.  **Recovery Run:** Run `bootstrap.sh` immediately after sanitization and verify the Citadel can boot again.
3.  **Grep Audit:** Run a final `grep` for any personal session data (e.g., agent names, specific task IDs) in the `crates/` and `config/` directories.

## 🚫 Constraints
- **NEVER** delete `Cargo.toml`, `.rs` files, or anything in the `templates/` or `config/defaults/` directories.
- **NEVER** modify `.git/` history (this tool handles filesystem state only).

---

## 🛰️ Sovereign Review (Tyr)
- Confirm that the script correctly handles the "Sanctuary" boundary between distribution and instance data.
- Verify that the "Deep Clean" includes the removal of the `.env` file (if it contains real secrets).
