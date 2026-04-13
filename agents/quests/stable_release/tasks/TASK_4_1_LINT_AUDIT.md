# Task Manifest: 4.1 - Workspace Audit & Canon Compliance
**Status:** 🟢 Active
**Assignee:** Cid (Engineer)
**Reviewer:** Tyr (Captain)
**Priority:** High

---

## 🎯 Objective
Eliminate technical debt and ensure the entire KoadOS workspace adheres to the strict standards defined in `RUST_CANON` and `CONTRIBUTOR_CANON`. Produce a warning-free, optimized build.

## 🧱 Technical Requirements

### 1. Zero-Warning Mandate
- **Clippy Audit:** Execute `cargo clippy --workspace --all-targets -- -D warnings`.
- **Resolution:** Resolve every lint violation. Do NOT use `#[allow(...)]` unless explicitly approved by the Captain for specific architectural reasons (e.g., FFI).
- **Specific Targets:** 
    - Fix unused imports and variables in `koad-bridge-notion`.
    - Fix test-related warnings in `koad-cli`.

### 2. Dead Code Elimination
- **Graph Analysis:** Use the `code-review-graph` to identify "Orphan Nodes" (functions, structs, or traits with zero incoming edges).
- **Verification:** Manually verify if the code is truly dead or intended for future WASM plugin hooks.
- **Action:** Remove all verified dead code from the distribution.

### 3. Dependency Optimization
- **Version Sync:** Ensure all workspace crates use consistent versions of shared dependencies (serde, tokio, etc.) via the workspace manifest.
- **Unused Deps:** Remove any dependencies listed in `Cargo.toml` files that are not actually imported in the source.

### 4. Canon Documentation
- **Missing Docs:** Ensure all public structs and functions in the core crates (`koad-core`, `koad-citadel`, `koad-proto`) have proper doc comments (`///`).

## ✅ Verification Strategy
1.  **Build Pass:** `cargo build --workspace` produces zero output to stderr.
2.  **Lint Pass:** `cargo clippy --workspace` produces zero warnings.
3.  **Graph Pass:** `code-review-graph status` shows a lean node-to-edge ratio.
