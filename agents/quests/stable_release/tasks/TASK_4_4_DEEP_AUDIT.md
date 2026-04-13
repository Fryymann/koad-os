# Task Manifest: 4.4 - Deep Audit & Dead Code Elimination
**Status:** ⚪ Draft
**Assignee:** [Engineer-Agent (Cid)]
**Reviewer:** Tyr (Captain)
**Priority:** Medium

---

## 🎯 Objective
Complete the technical debt reduction initiated in Task 4.1. This task focuses on high-fidelity codebase optimization: removing unused logic, synchronizing the dependency mesh, and ensuring full rustdoc compliance.

## 🧱 Technical Requirements

### 1. Dead Code Purge
- **Requirement:** Triage the 315+ dead-code candidates identified by the `code-review-graph`.
- **Requirement:** Remove verified orphan functions, structs, and modules that are not intended for external integration or WASM hooks.

### 2. Dependency Mesh Synchronization
- **Requirement:** Transition all workspace crates to use `dep.workspace = true` for shared libraries (serde, tokio, anyhow, etc.).
- **Requirement:** Remove unused dependencies from all `Cargo.toml` files.

### 3. Canon Documentation Sweep
- **Requirement:** Ensure all public APIs in `koad-core`, `koad-citadel`, and `koad-proto` have complete `///` documentation.
- **Requirement:** Enable crate-level documentation lints in `koad-proto/src/lib.rs`.

## ✅ Verification Strategy
1. **Graph Audit:** `code-review-graph status` should show a significant reduction in orphan nodes.
2. **Build Test:** `cargo build --workspace` remains clean.
3. **Doc Test:** `cargo doc --workspace --no-deps` generates a complete and warning-free documentation set.
