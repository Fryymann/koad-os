# Task Manifest: 4.2 - Nightly Bridge & Release Cut (v3.2.0)
**Status:** ⚪ Draft
**Assignee:** Tyr (Captain)
**Reviewer:** Admiral (Ian)
**Priority:** Critical

---

## 🎯 Objective
Formalize the transition from the `nightly` development branch to a stable `main` release. This task represents the final "Gate" for KoadOS v3.2.0 "Citadel Integrity."

## 🧱 Technical Requirements

### 1. Release Documentation
- **Changelog:** Create `CHANGELOG.md` by aggregating session logs and update posts from the v3.2.0 cycle.
- **Onboarding Pass:** Ensure `AGENTS.md` and `MISSION.md` are accurately synchronized with the new `koad map` and `koad vault skill` commands.

### 2. Version Orchestration
- **Root Cargo.toml:** Bump `version = "3.2.0"`.
- **Crate Propagation:** Ensure all workspace crates reflect the new version number.
- **Metadata:** Update version strings in CLI help text and MOTD templates.

### 3. Stability Verification (The Great Test)
- **Workspace-Wide Tests:** Execute `cargo test --workspace`. All tests must pass.
- **E2E Validation:** Perform a fresh `koad system init` and `agent-boot` sequence to verify the "Out of Box" experience.

### 4. Git Orchestration
- **The Bridge Merge:** Merge `nightly` into `main`.
- **Tagging:** Create a signed git tag `v3.2.0` on the `main` branch.
- **Origin Push:** Synchronize `main` and tags with the remote repository.

## ✅ Verification Strategy
1.  **Version Check:** `koad --version` returns exactly `3.2.0`.
2.  **Tag Verification:** `git describe --tags` returns `v3.2.0`.
3.  **Distribution Test:** Clone the repo into a fresh temporary directory, run `install/bootstrap.sh`, and verify a successful boot.
