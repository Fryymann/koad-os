# Mission Brief: Stable Release v3.2.0 Implementation
**Lead:** Clyde (Officer)
**Status:** 🟡 ACTIVE

## Objective
Implement the technical requirements for the KoadOS v3.2.0 Stable Release, focusing on the installer, system initialization, and environment portability.

## Context
Captain Tyr has performed a preliminary "manual" pass of the Sanctuary Audit and drafted the initial installer scripts. Your team is to take over, verify the integrity of these changes, and complete the remaining technical tasks.

## Tasks & Delegation

### 1. Sanctuary Audit & Portability (Lead: Cid)
*   **Audit Tyr's manual pass:** Verify that all `/home/ideans` occurrences have been genericized or replaced with placeholders in `crates/`, `scripts/`, and `config/`.
*   **Systemd Templates:** Ensure `config/systemd/*.template` files are correctly formatted for dynamic injection.
*   **Redis Config:** Verify `config/defaults/redis.conf` path hydration logic.

### 2. Installer & System Init (Lead: Clyde)
*   **Hardening `scripts/install.sh`:** Add robust error handling, prerequisite version checks (Rust 1.75+, etc.), and WSL2 detection.
*   **Completing `koad system init`:** Refine the interactive prompt in `crates/koad-cli/src/handlers/system_init.rs`. Ensure it correctly handles `.env` generation and config hydration.
*   **Verification:** Run `koad system doctor` end-to-end after a fresh "mock" install.

### 3. CI Pipeline & Quality (Lead: Clyde-QA)
*   **GitHub Actions:** Author `.github/workflows/ci.yml` with `cargo test`, `clippy`, and `fmt` checks.
*   **Secret Scanning:** Integrate a secret scanning tool (e.g., gitleaks or similar) into the CI pipeline.
*   **Coverage:** Aim for 80% coverage on core crates. Report current status and blockers.

## Definition of Done (Technical)
- [ ] `cargo test --workspace` passes 100%.
- [ ] `install.sh` and `koad system init` work flawlessly on a fresh environment.
- [ ] No hardcoded PII/paths remain in tracked files.
- [ ] CI pipeline is green.

## Reporting
Log progress in `TEAM-LOG.md` and report any escalations to Tyr.
