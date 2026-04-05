# KoadOS Stable Release Spec (v1.0)
**Version:** 3.2.0 (Stable)
**Date:** 2026-04-04
**Lead:** Tyr (Captain)

## A) Release Definition
*   **Guarantee:** An external developer can clone the repo into a fresh Win11 + WSL2 + Ubuntu environment, run `install.sh`, and boot a functioning "Captain" agent within 5 minutes.
*   **In-Scope:** Citadel Control Plane, CASS Tiered Memory, Koad CLI, MCP Tool Registry, Big Three Providers (Gemini, Claude, Codex).
*   **Out-of-Scope:** Native Windows (non-WSL), macOS support, local models for Captain-rank agents, web-based UI (TUI only).
*   **Toolchain:** Rust 1.75+, Redis 7.2+, Docker 24+, Ollama 0.1.30+.

## B) Privacy & Redaction Spec (Zero Personal Data)
*   **Strategy:** **Fresh Start.** We will initialize a new public repository and push a squashed "v3.2.0 stable" commit to ensure no PII exists in the history.
*   **Sanctuary Checklist:**
    *   Redact all `/home/ideans/` paths to `$KOAD_HOME` or `~/.koad-os`.
    *   Ensure `.env` is globally gitignored and `.env.template` contains only dummy values (e.g., `YOUR_KEY_HERE`).
    *   Scrub all Notion DB IDs and Airtable keys from `config/`.
    *   Scan `logs/` and `data/db/` to ensure no active databases are tracked.
*   **DoD:** A `grep -r "ideans"` returns zero matches in the tracked files.

## C) Install & Bootstrap Spec (The Installer)
*   **UX Flow:** 
    1.  User runs `curl -sSL https://koad.os/install.sh | bash`.
    2.  Script detects WSL2/Ubuntu.
    3.  Script checks for `rustc`, `docker`, `redis-server`, `ollama`.
    4.  Script prompts for one Provider Key (Gemini/Claude/Codex).
    5.  Script generates `.env` and `config/kernel.toml`.
    6.  Script runs `cargo build --release`.
*   **Failure Handling:** If a prerequisite is missing, the script provides the exact `sudo apt install` command and exits gracefully.

## D) Captain Provider Requirements
*   **Auth:** Handled via gitignored `.env` file.
*   **Abstraction:** All provider calls must go through the `koad-intelligence` InferenceRouter to allow future local-model swap-ins.
*   **Validation:** `koad system doctor` will attempt a minimal "Hello" chat with the configured provider to verify the key on first run.

## E) Testing & CI Spec
*   **Target Coverage:** 
    *   100% Path Coverage for `install.sh` and `koad system init`.
    *   80% Line Coverage for `koad-core`, `koad-proto`, and `koad-cass/storage`.
*   **CI Gates:** GitHub Actions must pass: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, and a `secret-scan` job.

## F) Documentation & Developer Guides
*   **Priority 1 (Tyr):** `README.md` (Quick Start), `ARCHITECTURE.md` (Tri-Tier Model).
*   **Priority 2 (Scribe):** `CONTRIBUTING.md`, `WSL2_NETWORKING.md`, `CLI_REFERENCE.md`, `FAQ.md`.

## G) Release Engineering Spec
*   **Versioning:** SemVer 2.0.0.
*   **Tagging:** `v3.2.0-stable` on the `main` branch.
*   **Rollback:** Keep the `v3.1.x` branch active as `legacy-spine`.

## H) Execution Plan
1.  **Cid:** Perform Sanctuary Audit (48h).
2.  **Clyde:** Build `install.sh` and `koad system init` (72h).
3.  **Scribe:** Generate Priority 2 docs (48h).
4.  **Tyr:** Final SDR and Fresh Start push.
