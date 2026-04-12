# Task Manifest: 4.1 - Workspace Lint & Audit
**Status:** ⚪ Draft
**Assignee:** [Engineer-Agent (Cid/Clyde)]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `refactor/workspace-lint-v3.2.0`

---

## 🎯 Objective
Perform a comprehensive technical debt audit and cleanup of the entire KoadOS workspace. Resolve all `clippy` warnings, ensure consistent formatting, and enforce strict adherence to the KoadOS Rust Canon.

## 🧱 Context
As we move toward a stable release, the codebase must be clean, idiomatic, and free of low-level "code smells" (e.g., unused variables, empty doc lines, redundant clones). A clean workspace ensures long-term maintainability and professionalism.

## 🛠️ Technical Requirements

### 1. Clippy Cleanup
- **Command:** `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- **Requirement:** Resolve all errors and warnings detected by Clippy across all 11 crates.
- **Specific Fixes Needed (Identified):**
    - `koad-core/src/utils/lock.rs`: Fix empty lines after doc comments (Macro L81).
    - `koad-bridge-notion/src/mcp.rs`: Remove unused imports and variables.
    - `koad-agent.rs`: Resolve unused variables and assignments.

### 2. Formatting Audit
- **Command:** `cargo fmt --workspace --all -- --check`
- **Requirement:** Ensure all files adhere to the project's `rustfmt` standard.

### 3. Redundant Dependency Check
- **Requirement:** Audit `Cargo.toml` files for unused or redundant dependencies.
- **Requirement:** Verify that all crates are correctly using workspace-level dependencies for unified versioning.

### 4. Dead Code Removal (Final Pass)
- **Requirement:** Audit the remaining `pub` items in `koad-core` and `koad-proto` that are not used by any internal binary or library.

### 5. Documentation Integrity
- **Requirement:** Ensure all public structs and functions in `koad-core` have descriptive doc comments (`///`).
- **Requirement:** Verify that `//!` doc comments are present at the top of all `lib.rs` and `main.rs` files.

## ✅ Verification Strategy
1.  **Strict Compilation:** Run `cargo clippy --workspace -- -D warnings` and verify it exits with `0`.
2.  **Formatting Check:** Run `cargo fmt --workspace -- --check` and verify it exits with `0`.
3.  **Test Suite:** Run `cargo test --workspace` and verify all tests pass after the cleanup.

## 🚫 Constraints
- **NEVER** use `#[allow(clippy::...)]` unless there is a strictly documented architectural reason.
- **NEVER** disable a lint to bypass a fix; find the idiomatic Rust solution.
- **MUST** be performed across all crates simultaneously.

---

## 🛰️ Sovereign Review (Tyr)
- Confirm that the codebase feels "idiomatic" and follows the KoadOS aesthetic.
- Verify that the doc comments provide high-signal value.
