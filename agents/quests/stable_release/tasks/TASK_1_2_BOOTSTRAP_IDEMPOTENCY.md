# Task Manifest: 1.2 - Bootstrap Idempotency
**Status:** ⚪ Draft
**Assignee:** [Engineer-Agent (Clyde/Cid)]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `feature/bootstrap-idempotency`

---

## 🎯 Objective
Refactor the `install/bootstrap.sh` script to be fully idempotent, environment-agnostic (no hardcoded `~/.koad-os`), and resilient to repeated executions.

## 🧱 Context
A shareable system must have a "one-command" setup that works whether it's the first time or the hundredth time the script is run. It must handle missing directories, existing configurations, and partial failures without leaving the system in a broken state.

## 🛠️ Technical Requirements

### 1. Dynamic Path Resolution
- **Current Issue:** Shell integration (Step 8) hardcodes `~/.koad-os/bin`.
- **Requirement:** The script must use `$KOAD_HOME` (calculated from the script location) for all internal pathing. For shell integration, it should use the absolute path of the current installation directory, allowing KoadOS to be installed anywhere (e.g., `/opt/koados`).

### 2. Idempotent Configuration Management
- **Environment:** If `.env` exists, do NOT overwrite it. Instead, provide a "diff" check or simply notify the user. 
- **Templates:** When copying from `config/defaults/`, ensure existing local configs are preserved. 
- **Requirement:** Implement a `sync_config` function that only copies if the target doesn't exist.

### 3. Directory Scaffolding
- Ensure the following directory tree is present:
    - `bin/`, `logs/`, `cache/`, `run/`
    - `data/db/`, `data/redis/`
    - `agents/bays/`, `agents/crews/`
    - `config/identities/`, `config/interfaces/`, `config/integrations/`

### 4. Build System Hardening
- **Requirement:** Explicitly handle `PROTOC` and `PROTOC_INCLUDE` detection.
- **Requirement:** Add a check for `cargo` workspace integrity before building.
- **Optimisation:** Only rebuild and copy binaries if the source has changed (or if a `--force` flag is provided).

### 5. Docker Compose Resilience
- **Current Issue:** The script fails silently if Docker isn't running.
- **Requirement:** If `docker info` fails, provide a clear warning that the Redis/Qdrant stack must be started manually later, but don't exit with an error (non-blocking).
- **Requirement:** Use `--wait` with `docker compose up` to ensure services are ready before proceeding to DB initialization.

### 6. Shell Integration Refactor
- **Requirement:** Add a check to prevent duplicate entries in `.bashrc`.
- **Requirement:** Use a "sentinel" block in `.bashrc` (e.g., `# >>> KoadOS Initialize >>> ... # <<< KoadOS Initialize <<<`) to make the integration easy to find and update.

### 7. Non-Interactive Mode
- **Requirement:** Support a `-y` or `--yes` flag to bypass all prompts (useful for CI or automated deployment).

## ✅ Verification Strategy
1.  **Fresh Install:** Run on a new clone in a non-standard directory (e.g., `/tmp/koad-test`).
2.  **Re-run:** Run the script again immediately; it should finish significantly faster and change nothing.
3.  **Path Verification:** Ensure `koad` and `koad-agent` in the `bin/` directory point to the correct paths.
4.  **Integration Check:** Verify `.bashrc` has exactly one KoadOS block.

## 🚫 Constraints
- **NO** hardcoded `/home/ideans/` or `~/.koad-os/` in script logic.
- **NO** destructive overwrites of user configuration.
- **MUST** be compatible with standard Bash 4.x.

---

## 🛰️ Sovereign Review (Tyr)
- Verify that the shell integration logic correctly handles the `$KOAD_HOME` variable.
- Ensure the script provides high-signal output during the build phase.
