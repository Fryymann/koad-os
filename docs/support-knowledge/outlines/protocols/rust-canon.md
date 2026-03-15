# RUST_CANON: Rust Development Standards

## Metadata
- Category: PROTOCOLS & GOVERNANCE
- Complexity: intermediate
- Related Topics: contributor-canon, testing-tiers
- Key Source Files: `docs/protocols/RUST_CANON.md`
- Key Canon/Doc References: `docs/protocols/RUST_CANON.md`

## Summary
The `RUST_CANON.md` is the authoritative guide to writing Rust code within the KoadOS project. It is a mandatory protocol for all contributors, including AI agents and humans, designed to ensure the codebase is performant, secure, maintainable, and consistent. It covers everything from project structure and error handling to documentation and testing standards.

## How It Works
The `RUST_CANON` is not a single tool, but a set of rules and best practices enforced through a combination of automated checks and manual code review (specifically, the KSRP).

The key pillars of the canon are:
1.  **Structural Standards**:
    - All crates must belong to the root `Cargo.toml` workspace.
    - Shared dependencies like `serde` and `tokio` are inherited from `[workspace.dependencies]` to ensure version consistency.

2.  **Error Handling (Zero-Panic Policy)**:
    - `.unwrap()` and `.expect()` are strictly forbidden in production code.
    - `anyhow::Result` must be used for application-level (binary) error handling to provide rich context.
    - `thiserror` should be used for library crates to create specific, structured error types.

3.  **Concurrency (Async-First)**:
    - `tokio` is the standard runtime for all asynchronous I/O.
    - Blocking operations are never allowed on the main async runtime; they must be moved to a dedicated thread with `tokio::spawn_blocking`.

4.  **Observability (`tracing`)**:
    - All logging must use the `tracing` crate with structured key-value pairs (e.g., `info!(key = "value")`) instead of simple string formatting. This makes logs machine-parseable.

5.  **Mandatory Documentation (Rustdoc)**:
    - This is one of the strictest rules. All public items (`struct`, `fn`, `trait`, etc.) MUST have `///` doc comments.
    - Every `.rs` file MUST begin with a `//!` module-level comment explaining its purpose.
    - The lint flags `#![warn(missing_docs)]` and `#![warn(rustdoc::broken_intra_doc_links)]` are required in all library crates.

6.  **Mandatory Testing (The Tier System)**:
    - All non-trivial code must be covered by tests.
    - The canon defines a "Tier System" for testing: Doc Tests (Tier 1), Unit Tests (Tier 2), Snapshot Tests (Tier 3), and Property Tests (Tier 4).

7.  **Quality Gates**:
    - Before a PR can be merged, it must pass `cargo clippy -- -D warnings` (all warnings treated as errors) and `cargo fmt`.
    - `cargo audit` is used to check for security vulnerabilities in third-party dependencies.

## Key Code References
- **File**: `docs/protocols/RUST_CANON.md`
  - **Element**: The entire document.
  - **Purpose**: The canonical source of truth for all Rust coding standards in the project.
- **File**: Root `Cargo.toml`
  - **Element**: `[workspace.dependencies]`
  - **Purpose**: Enforces the dependency inheritance rule.
- **File**: Any `lib.rs` in a library crate (e.g., `crates/koad-core/src/lib.rs`)
  - **Element**: `#![warn(missing_docs)]`
  - **Purpose**: The compiler directive that enforces the documentation rule.

## Configuration & Environment
- This is a development-time protocol; it does not have runtime configuration. Its enforcement relies on the CI pipeline and the discipline of the contributing agents/humans.

## Common Questions a Human Would Ask
- "What are the coding standards for this project?"
- "Why did my build fail on `missing_docs`?"
- "Can I use `.unwrap()` in my test code?"
- "How should I handle errors in a new library I'm writing?"
- "What's the difference between `anyhow` and `thiserror` and when should I use each?"

## Raw Technical Notes
- The `RUST_CANON` was established in response to technical debt and inconsistent code quality in the legacy "Spine" architecture.
- Its strictness, especially around documentation and error handling, is a deliberate choice to build a "glass box" system that is easy to understand, debug, and maintain, which is especially important when AI agents are co-authoring the code.
- Adherence to `RUST_CANON` is a major factor in the KSRP (Koad Self-Review Protocol) and directly impacts an agent's performance evaluation.
