# Task Manifest: 4.2 - Nightly Bridge & Release Cut
**Status:** ⚪ Draft
**Assignee:** [Tyr (Captain/PM)]
**Reviewer:** Admiral (Ian)
**Branch:** `main` (Merge target)

---

## 🎯 Objective
Execute the final transition from the development environment to the stable release. Merge the `nightly` branch into `main`, generate the official v3.2.0 changelog, and tag the release for public distribution.

## 🧱 Context
This is the final gate. The `nightly` branch contains all our Phase 1-4 stability and portability improvements. Moving to `main` signifies that KoadOS is ready for external use and adheres to the "Sanctuary" standard of excellence.

## 🛠️ Technical Requirements

### 1. Final Integration Sweep
- **Requirement:** Perform a clean build from scratch: `cargo clean && cargo build --workspace`.
- **Requirement:** Run the full test suite one last time: `cargo test --workspace`.
- **Requirement:** Run `koad doctor --full` to verify the local environment is 100% green.

### 2. The Nightly-to-Main Bridge
- **Action:** Merge `nightly` into `main`. 
- **Conflict Resolution:** If any conflicts exist with legacy code on `main`, prioritize the `nightly` implementation (the "New World" architecture).
- **Requirement:** Ensure all "PII" (Personally Identifiable Information, like hardcoded developer paths) has been scrubbed during the Phase 1 tasks before this merge.

### 3. Changelog Generation (`CHANGELOG.md`)
- **Requirement:** Create a new `CHANGELOG.md` in the repo root.
- **Content:**
    - **v3.2.0 "Citadel Integrity" (Stable)**
    - **Stability:** Graceful shutdown, autonomic recovery (`koad doctor --fix`).
    - **Portability:** Removal of all hardcoded home paths; environment-agnostic booting.
    - **Architecture:** Formalized Skill Blueprint vs. Instance model.
    - **Refinement:** Workspace-wide linting, technical debt reduction, and dead code removal.

### 4. Versioning & Tagging
- **Requirement:** Ensure `Cargo.toml` (workspace and all crates) is set to exactly `3.2.0`.
- **Action:** Create a git tag: `git tag -a v3.2.0 -m "Release v3.2.0 - Citadel Integrity"`.

### 5. Distribution Verification
- **Requirement:** Verify that the `install/bootstrap.sh` script works perfectly on the `main` branch.
- **Requirement:** Verify that `agent-boot tyr` successfully initializes a session on `main`.

## ✅ Verification Strategy
1.  **Main Build:** Verify `cargo build` succeeds on the `main` branch.
2.  **Tag Verification:** `git describe --tags` should return `v3.2.0`.
3.  **Sanctuary Check:** A final `grep` for `/home/ideans` must return ZERO results in the `main` branch source code.

## 🚫 Constraints
- **NO** merges to `main` until all Phase 1-4 tasks are marked **Complete**.
- **NO** force-pushing to `main`.
- **MUST** obtain final Admiral approval before tagging.

---

## 🛰️ Sovereign Review (Tyr)
- Confirm that the `CHANGELOG.md` accurately reflects the massive structural improvements made in v3.2.0.
- Verify that the repository is "Clean" (no untracked artifacts) before the release cut.
