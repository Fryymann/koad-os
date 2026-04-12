# Task Manifest: 1.1 - The Great Path Scrub
**Status:** 🟢 Complete
**Assignee:** Clyde
**Reviewer:** Tyr (Captain/PM)
**Branch:** `refactor/path-scrub`

---

## 🎯 Objective
Eliminate all hardcoded absolute paths (e.g., `/home/ideans/`) from the KoadOS codebase and configuration. Transition to a strictly dynamic path resolution model anchored to the `KOAD_HOME` (or `KOADOS_HOME`) environment variable.

## 🧱 Context
Currently, several critical components (Redis config, Plugin registration) rely on hardcoded paths specific to the developer's local environment. This blocks KoadOS from being shareable or portable to any other machine. We must achieve "Sanctuary Compliance"—where the codebase is environment-agnostic.

## 🛠️ Technical Requirements

### 1. Audit & Search
- Use `grep` and `find` to locate all occurrences of `/home/ideans/` in:
    - `crates/**/*.rs`
    - `config/**/*.conf`
    - `scripts/**/*.sh`
    - `bin/koad-functions.sh`  <-- [CRITICAL]
    - `templates/**/*.md`

### 2. Implementation: `koad-core` Config Logic
- Ensure `KoadConfig::load()` correctly detects the project root.
- Use `dirs::home_dir()` as a fallback for `~/.koad-os` if no environment variable is set.
- **Goal:** All downstream crates must use `config.home.join("path/to/resource")` instead of literal strings.

### 3. Implementation: `register-tool.rs`
- **File:** `crates/koad-cli/src/bin/register-tool.rs`
- **Change:** Replace the hardcoded `component_path` with a resolution that uses `KoadConfig` or `std::env::var("KOADOS_HOME")`.
- **Logic:** `PathBuf::from(env::var("KOADOS_HOME")?).join("crates/koad-plugins/wit/hello-plugin.component.wasm")`

### 4. Implementation: `redis.conf`
- **File:** `config/redis.conf`
- **Problem:** Redis requires absolute paths for its socket and pidfile, but these are currently hardcoded to `/home/ideans`.
- **Strategy:** Transition `redis.conf` to a **template** (e.g., `config/redis.conf.template`).
- **Logic:** The `Kernel` (`koad-citadel`) or a start script should generate a temporary `run/redis.active.conf` at runtime with the correct absolute paths injected.

## ✅ Verification Strategy
1.  **Dry Run:** Change the `KOADOS_HOME` env var to a temporary directory and verify that the system attempts to look for files there.
2.  **Compilation:** Ensure all crates build after the refactor.
3.  **Bootstrap Test:** Run `install/bootstrap.sh` and verify that the generated `.env` and configs are correct.
4.  **CLI Check:** Run `koad status` and `koad doctor` to ensure they still resolve sockets correctly.

## 🚫 Constraints
- **NO** hardcoded strings starting with `/home/`.
- **NO** platform-specific path separators (use `PathBuf::join`).
- **RETAIN** support for `~/.koad-os` as the default location.

---

## 🛰️ Sovereign Review (Tyr)
- Ensure no regressions in `KoadConfig` path resolution.
- Verify that `koad-agent` still boots correctly after changes.
