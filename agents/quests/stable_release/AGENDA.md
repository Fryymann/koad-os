# Mission: KoadOS v3.2.0 "Citadel Integrity"
**Objective:** Transition KoadOS from Developer-Mode to a Stable, Shareable Distribution.
**Commander:** Tyr (Project Manager)
**Status:** 🟢 Active

---

## 📋 The Stable Release Agenda

### 1. Robustness & Stability (Zero-Ghost Policy)
- [ ] **Service Lifecycle:** Refine `koad-citadel` and `koad-cass` to handle graceful shutdowns and signal trapping.
- [ ] **Autonomic Recovery:** Implement "Fix" logic in `koad doctor` for stale Redis keys, missing sockets, and orphaned PID-like state.
- [ ] **Error Boundaries:** Update gRPC error propagation to provide user-facing "How to fix" messages.

### 2. Vault Phase 3: Skill & Blueprint Standardization
- [ ] **Blueprint Model:** Formalize the Skill Blueprint vs. Instance architecture.
- [ ] **Vault Skill CLI:** Implement `koad vault skill` to allow listing and inspecting available skills.

### 3. Distribution & Onboarding (Shareability)
- [x] **Sanctuary Audit (COMPLETE):** Scrub all hardcoded paths (e.g., `/home/ideans/`) from `distribution` crates. Absolute reliance on `KOAD_HOME` and `resolve_vault_path` is now enforced.
- [ ] **Bootstrap Verification:** Audit `install/bootstrap.sh` for idempotency and compatibility on fresh WSL2/Linux environments.
- [ ] **The Admiral's Guide:** Update `MISSION.md` and `AGENTS.md` for a clean "Quick Start" onboarding flow.

### 4. Technical Debt & Final Polish
- [ ] **Workspace Audit:** Run `cargo clippy --workspace` and resolve all warnings.
- [ ] **Nightly Bridge:** Prepare and verify the final diff for merging `nightly` into `main`.

---

## 🛰️ Current Focus
- **Task:** 3.2 Bootstrap Verification (scripts/install.sh audit)
- **Priority:** Alpha (Blocker for Shareability)
