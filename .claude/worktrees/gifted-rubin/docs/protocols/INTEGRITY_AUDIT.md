# KoadOS Protocol: False Pass & Hardcode Review

## 1. Purpose
This protocol exists to catch **false-positive test passes** and **hardcoded values** that mask real system failures. This protocol treats any test that cannot fail as a liability.

## 2. Part 1 — Hardcoded Value Audit

### 2.1 Scan Target
- Magic strings/numbers (ports, IDs, URLs, tokens).
- Duplicated values that should share a single source of truth.
- Config values embedded in code rather than env/config layer.
- Hardcoded entity IDs in test fixtures matching real system values.

### 2.2 Disposition
- **Promote to env/config**: Move to `.env` or typed config struct.
- **Define as constant**: Use shared `constants.rs` with documentation.
- **Flag as test-only**: Wrap in `#[cfg(test)]`.
- **Issue Tracking**: Create `[audit]` issues for unresolved findings.

## 3. Part 2 — False Pass Test Detection

### 3.1 Failure Patterns
- Assertions on mocks that always return `Ok`.
- Dead tests (function never actually called).
- Tautological assertions (`assert!(true)`).
- Swallowed `Result` (`let _ = foo()`).
- Asserting on setup rather than logic outcome.
- Mocked timeout/retry logic that never validates real timing.

### 3.2 Review Checklist
- **Can this test fail?** If not, it is a liability.
- **Is the assertion on real logic?**
- **Are all Results unwrapped?** No silent `let _ =`.
- **Is there a negative case?** Every happy path needs a failure counterpart.

## 4. Part 3 — Agent Session Integrity
- Verify session registration writes to real Redis store.
- Ensure 1:1 mapping between KAI and active session.
- Confirm session count assertions fail when duplicates are injected.

## 5. Review Cadence
- **PR Merge (Test):** Manual Part 2 check.
- **PR Merge (Config):** Part 1 scan.
- **Incident Post-Mortem:** Mandatory coverage PR.
- **Weekly:** Full System Sweep.

> A test that cannot fail is not a test. It is documentation that happens to run.
