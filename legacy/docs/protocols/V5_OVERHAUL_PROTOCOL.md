# KoadOS v5.0 — Overhaul Protocol (Operational Mandates)

> [!CAUTION]
> **Status:** MANDATORY
> **Scope:** ALL development within the v5.0 Overhaul Sprint.
> **Enforcement:** Failure to follow these protocols results in an immediate KSRP Failure and roll-back.

---

## 1. Project Board Synchronization
**Rule:** The GitHub Project Board (#2) is the single source of truth for the Admiral.

- **Pre-Flight:** Before starting any sub-task, the corresponding GitHub Issue MUST be moved to `In Progress` via `koad board sync`.
- **Roadmap:** Every active issue MUST have a `Start Date` and `Target Date`.
- **Post-Flight:** Upon successful KSRP verification, the issue MUST be moved to `Done` with a link to the verification log.

## 2. Git Procedures (The "Clean History" Standard)
**Rule:** We build for permanence. Our git history is our station log.

- **Atomic Commits:** Commits MUST be surgical. One feature/fix per commit.
- **Commit Messages:** MUST follow the format: `[v5.0][Component] Descriptive summary. Closes #{issue_number}`.
- **Branching:** All work occurs on feature branches branched from `nightly` (e.g., `feat/v5-phase-0-harness`).
- **Protected Paths:** No direct pushes to `nightly` or `main`. All changes MUST pass through a Pull Request.

## 3. The "Trace Context" Code Standard
**Rule:** No logic is blind. 

- Every new function in the v5.0 core MUST accept a `TraceContext`.
- Every error return MUST be wrapped with context identifying the `trace_id`.
- **Mandate:** Code that cannot be traced cannot be merged.

## 4. Continuous Verification (KSRP)
**Rule:** The Koad Self-Review Protocol is the final gate.

For every commit:
1. **Lint:** `cargo fmt` and `cargo clippy`.
2. **Verify:** Run the `koad doctor` (or `koad dood pulse`) to ensure the link is healthy.
3. **Inspect:** Confirm the `trace_id` appears correctly in the `koad:stream:logs`.
4. **Harden:** Ensure no new `Mutex` locks were introduced without an Actor-model justification.

---
*Signed, Captain Tyr.*
