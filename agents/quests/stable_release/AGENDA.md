# Mission: KoadOS v3.2.0 "Citadel Integrity"
**Objective:** Transition KoadOS from Developer-Mode to a Stable, Shareable Distribution.
**Commander:** Tyr (Project Manager)
**Status:** 🟢 Active

---

## 📋 The Stable Release Agenda

### 1. Robustness & Stability (Zero-Ghost Policy)
- [x] **Service Lifecycle:** Refine `koad-citadel` and `koad-cass` to handle graceful shutdowns and signal trapping. (COMPLETE)
- [x] **Autonomic Recovery:** Implement "Fix" logic in `koad doctor` for stale Redis keys, missing sockets, and orphaned PID-like state. (COMPLETE)
- [x] **Error Boundaries (CP-03-GRPC):** Transition gRPC errors to stylized, human-actionable prompts. (COMPLETE)
- [x] **Fix Build Blockers (CP-05-FIX):** Resolve compile errors in `status.rs` related to fred streams. (COMPLETE)
- [x] **Knowledge Graph (CP-11-SYNC):** Implement **Atlas Pivot**: Dynamic System Map via `code-review-graph` and redesigned `koad map`. (COMPLETE)

### 2. Vault Phase 3: Skill & Blueprint Standardization
- [ ] **Blueprint Model (CP-04-VAULT):** Formalize the Skill Blueprint vs. Instance architecture. (DELEGATED to Clyde - TASK 3.1)
- [ ] **Vault Skill CLI (CP-04-VAULT):** Implement `koad vault skill` to allow listing and inspecting available skills. (DELEGATED to Clyde - TASK 3.2)

### 3. Distribution & Onboarding (Shareability)
- [x] **Sanctuary Audit (COMPLETE):** Scrub all hardcoded paths (e.g., `/home/ideans/`) from `distribution` crates. Absolute reliance on `KOAD_HOME` and `resolve_vault_path` is now enforced.
- [x] **Hierarchical Scaffolding (COMPLETE):** Implemented `koad deploy station/outpost` to automate the creation of project hubs and mission sectors with localized support folders.
- [ ] **Bootstrap Verification:** Audit `install/bootstrap.sh` for idempotency and compatibility on fresh WSL2/Linux environments. Interactive Captain creation is now integrated into `koad system init`.
- [ ] **The Admiral's Guide:** Update `MISSION.md` and `AGENTS.md` for a clean "Quick Start" onboarding flow.

### 4. Technical Debt & Final Polish
- [ ] **Workspace Audit:** Run `cargo clippy --workspace` and resolve all warnings.
- [ ] **Nightly Bridge:** Prepare and verify the final diff for merging `nightly` into `main`.
- [ ] **AIS Refactor (Postponed):** Moved to [Side Quest SQ-01-AIS](../side_quests/SQ-01-AIS.md).

---

## 🛰️ Current Focus
- **Task:** [TASK 3.1 & 3.2] — Vault Phase 3: Skill Standardization & CLI
- **Priority:** Alpha (Pluggable Architecture)
* Alpha (Pluggable Architecture)
