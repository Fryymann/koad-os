# Implementation Task List: Stable Release v3.2.0

## 🛡️ Cid (Engineer) — Sanctuary Audit
*   **Objective:** Eliminate all hardcoded PII and environment-specific strings.
*   **Tasks:**
    *   [x] Search and replace all `/home/ideans` with `$HOME` or relative paths.
    *   [x] Verify all `config/*.toml` files use env-var interpolation where possible.
    *   [x] Audit `crates/koad-cli` for hardcoded usernames or machine-specific assumptions.
*   **Effort:** M (DONE)

## 🛠️ Clyde (Officer) — Installer & CI
*   **Objective:** Create a seamless, non-AI bootstrap experience.
*   **Tasks:**
    *   [x] Author `scripts/install.sh` (Prerequisite check + compilation).
    *   [x] Implement `koad system init` (Interactive `.env` and `kernel.toml` generator).
    *   [ ] Configure GitHub Actions CI pipeline with secret scanning.
    *   [ ] Achieve 80% coverage on `koad-core` and `koad-proto`.
*   **Effort:** L (IN PROGRESS)

## ✍️ Scribe (Crew) — Documentation
*   **Objective:** Produce clear, professional onboarding guides.
*   **Tasks:**
    *   [ ] Expand `CLI_REFERENCE.md` with all Phase 4 commands.
    *   [ ] Write `WSL2_NETWORKING.md` troubleshooting guide.
    *   [ ] Synthesize the `FAQ.md` from internal dev logs.
*   **Effort:** S

## 👑 Tyr (Captain) — Oversight & Publishing
*   **Objective:** Final quality gate and public launch.
*   **Tasks:**
    *   [ ] Conduct Strategic Design Review (SDR) of the installer.
    *   [ ] Perform "Fresh Start" repository initialization.
    *   [ ] Tag and announce v3.2.0 Stable.
*   **Effort:** M
