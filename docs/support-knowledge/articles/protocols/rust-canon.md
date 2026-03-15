# RUST_CANON: Rust Development Standards

> The mandatory coding standards protocol for all KoadOS contributors — human and AI — defining rules for structure, error handling, async code, documentation, and testing that make the codebase a maintainable, auditable "glass box".

**Complexity:** intermediate
**Related Articles:** [Cargo Workspace](../tooling/cargo-workspace.md)

---

## Overview

`RUST_CANON` is the authoritative Rust development protocol for the KoadOS project. It is a mandatory protocol — not a style guide or recommendation — and applies equally to human engineers and AI agents like Tyr and Claude. Every pull request is evaluated against it, and adherence is a primary criterion in the KSRP (Koad Self-Review Protocol).

The canon was created in response to the technical debt accumulated in the legacy "Spine" architecture, where inconsistent code quality, missing documentation, and fragile error handling made the codebase difficult to maintain and even harder to audit. The Spine was a "black box" — hard to understand even with the code in front of you. The RUST_CANON is a deliberate effort to build a **glass box**: a codebase that is transparent, readable, and correct by construction.

This transparency matters especially because AI agents co-author the code. An AI writing code that lacks documentation and has `.unwrap()` calls scattered throughout creates a compounding technical debt problem — each session adds more hard-to-verify code. The RUST_CANON forces every contribution to be self-documenting and explicitly safe.

The full canonical document lives at `docs/protocols/RUST_CANON.md`. This article summarizes its key pillars and explains the reasoning behind them.

## How It Works

The RUST_CANON is enforced through a combination of compiler configuration, automated tooling, and manual review.

### Pillar 1: Structural Standards

All crates must belong to the root `Cargo.toml` workspace. Shared dependencies (`tokio`, `serde`, `anyhow`, `tracing`, `tonic`, etc.) are defined once in `[workspace.dependencies]` and inherited by individual crates using `dep.workspace = true`. This prevents version drift and ensures the entire project compiles against the same dependency graph.

A new crate added to the project without being registered in the root `Cargo.toml`'s `[workspace]` members list will be invisible to `cargo` commands run from the root. See [Cargo Workspace](../tooling/cargo-workspace.md) for the full structure.

### Pillar 2: Error Handling — The Zero-Panic Policy

`.unwrap()` and `.expect()` are **strictly forbidden** in production code (non-test code). Any `unwrap` or `expect` in a code path that could be reached in production is a canon violation.

The convention is:
- **Binary crates** (`koad-citadel`, `koad-cass`, `koad-cli`): Use `anyhow::Result` for application-level error propagation. `anyhow` provides rich context via `.context("doing X")` and `.with_context(|| format!("..."))`.
- **Library crates** (`koad-core`, `koad-intelligence`, `koad-sandbox`): Use `thiserror` to define specific, typed error enums. This gives callers structured errors they can match on.

The distinction is intentional: binaries care about getting a human-readable error message. Libraries care about giving callers enough information to handle errors programmatically.

```rust
// Library crate: structured errors (koad-core/src/hierarchy.rs)
#[derive(Debug, thiserror::Error)]
pub enum HierarchyError {
    #[error("no koad.toml found in path: {0}")]
    NotFound(PathBuf),
    #[error("invalid hierarchy config: {0}")]
    InvalidConfig(String),
}

// Binary crate: rich context (koad-citadel/src/main.rs)
let config = KoadConfig::load("config/kernel.toml")
    .context("failed to load kernel config")?;
```

**In tests**, `.unwrap()` and `.expect()` are permitted — test failures are acceptable panics, and the context overhead of `anyhow` adds noise to test code.

### Pillar 3: Concurrency — The Non-Blocking Rule

`tokio` is the standard async runtime for all I/O. The Non-Blocking Rule is absolute: **blocking operations must never run on the main `tokio` async runtime**.

A "blocking operation" is anything that takes significant wall time without yielding: CPU-intensive computation (e.g., tree-sitter parsing), synchronous file I/O in a hot loop, or sleeping. These must be moved to a dedicated thread pool using `tokio::task::spawn_blocking`:

```rust
// Correct: CPU-intensive tree-sitter parsing moved off the async runtime
let symbols = tokio::task::spawn_blocking(move || {
    parse_source_file(&source_code) // heavy CPU work
}).await?;
```

Violating this rule causes "async starvation" — the entire `tokio` runtime blocks, degrading latency for all concurrent tasks. In a multi-agent gRPC server like `koad-citadel`, a single blocking call can delay every other agent's requests.

### Pillar 4: Observability — Structured Tracing

All logging must use the `tracing` crate with **structured key-value pairs**. No `println!`, no `eprintln!`, no `log::info!` with raw string interpolation.

```rust
// Wrong: unstructured string
info!("Starting session for agent {}", agent_name);

// Correct: structured key-value
info!(agent_name = %agent_name, session_id = %session_id, "session started");
```

The structured format makes logs machine-parseable. KoadOS's observability tooling expects structured events; unstructured strings break log aggregation and filtering. The `%` sigil formats a value using `Display`; `?` uses `Debug`.

### Pillar 5: Mandatory Documentation

This is one of the strictest rules in the canon. **Every public item must have a `///` doc comment.** Every `.rs` file must begin with a `//!` module-level doc comment.

Library crates enforce this at the compiler level:

```rust
// In lib.rs of every library crate
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
```

`missing_docs` causes a warning (promoted to error in CI via `cargo clippy -- -D warnings`) for any public item without a doc comment. This means a PR that adds an undocumented public function will fail the quality gate.

The rationale: documentation is not optional in a codebase where AI agents are co-authors. Without docs, future agents (and humans) have to read and interpret the code to understand intent. With docs, intent is explicit and verifiable.

Note that `koad-citadel` uses a workaround for a Rust compiler issue (ICE — Internal Compiler Error) with this lint: `#![cfg_attr(not(test), warn(missing_docs))]`. This applies the lint only in non-test builds. This is a known workaround, not a canon exception.

### Pillar 6: Testing — The Tier System

All non-trivial code must be covered by tests. The canon defines four testing tiers:

| Tier | Type | Purpose | Tool |
|------|------|---------|------|
| 1 | **Doc Tests** | Verify that code examples in `///` comments compile and run | `rustdoc` |
| 2 | **Unit Tests** | Verify individual functions and structs in isolation | `#[test]` in `mod tests {}` |
| 3 | **Snapshot Tests** | Verify serialized output (JSON, TOML, proto) doesn't change unexpectedly | `insta` |
| 4 | **Property Tests** | Verify invariants hold across a large randomized input space | `proptest` |

Most contributions require Tier 1 and Tier 2 tests at minimum. Tier 3 is required for any code that serializes to a stable external format. Tier 4 is recommended for core algorithms (e.g., hierarchy resolution, token budgeting).

### Pillar 7: Quality Gates

Before a PR can be merged, it must pass:
1. `cargo clippy -- -D warnings` — all clippy lints treated as errors
2. `cargo fmt` — code must be formatted with `rustfmt` (the default settings)
3. `cargo audit` — no known security vulnerabilities in dependencies
4. `cargo test` — all tests pass

These gates are enforced in CI and are the minimum bar for KSRP sign-off.

## Configuration

| Tool | Config | Description |
|------|--------|-------------|
| `cargo clippy` | `Cargo.toml` `[lints]` section or `.cargo/config.toml` | Additional lint configuration (warnings promoted to errors) |
| `cargo fmt` | `rustfmt.toml` (if present) | Formatting overrides; defaults are used if absent |
| `cargo audit` | `deny.toml` (if present) | Vulnerability database configuration |
| `#![warn(missing_docs)]` | Each library crate's `lib.rs` | Enforces documentation requirement at compile time |

The RUST_CANON is a **development-time** protocol. It has no runtime configuration or environment variables.

## Failure Modes & Edge Cases

**Build fails with "missing documentation" error.**
A public item in a library crate (`pub fn`, `pub struct`, `pub trait`, etc.) is missing a `///` doc comment. Add a meaningful doc comment explaining what the item does, its parameters (for functions), and any important behavior. The `missing_docs` lint is active on all library crates via `#![warn(missing_docs)]` in `lib.rs`.

**Clippy fails with "unwrap used".**
Production code contains `.unwrap()` or `.expect()` on a `Result` or `Option`. Replace with proper error propagation: `?` for `Result`, `.ok_or_else(|| ...)` for `Option`, or explicit `match` for complex cases.

**`cargo fmt` fails in CI.**
Code was not formatted before commit. Run `cargo fmt` locally from the workspace root before pushing. Set up a pre-commit hook or IDE integration to auto-format on save.

**Tests use real I/O (hitting disk or network).**
Tests should use mock implementations where possible. For CASS tests, use `MockStorage` instead of `CassStorage`. For Citadel tests, use in-memory Redis (e.g., `redis-mock`) or test fixtures. Real I/O in unit tests makes them slow and environment-dependent.

## FAQ

### Q: What are the coding standards for this project?
The RUST_CANON defines all coding standards. Key rules: no `.unwrap()` in production code (use `anyhow` for binaries, `thiserror` for libraries); use `tokio` for all async I/O; move CPU-intensive work off the async runtime with `spawn_blocking`; all logging via the `tracing` crate with structured key-value pairs; all public items must have `///` doc comments; all `.rs` files must start with `//!`; all tests must pass and clippy must be clean with `-D warnings`. The canonical reference is `docs/protocols/RUST_CANON.md`.

### Q: Why did my build fail on `missing_docs`?
You added a `pub` function, struct, enum, or trait to a library crate without a `///` doc comment. Library crates in KoadOS have `#![warn(missing_docs)]` in their `lib.rs`, which the CI pipeline promotes to an error via `cargo clippy -- -D warnings`. Add a doc comment like `/// Creates a new session record with the given agent name.` before the item. If the item is intentionally undocumented (e.g., a test helper), make it non-public (`pub(crate)` or private).

### Q: Can I use `.unwrap()` in my test code?
Yes. `.unwrap()` and `.expect()` are permitted in test code (`#[cfg(test)]` modules and integration tests). Test panics are expected and acceptable. The Zero-Panic Policy applies only to production code paths. Using `.unwrap()` in tests is often cleaner than full error propagation when the test is specifically verifying a success case.

### Q: How should I handle errors in a new library I'm writing?
Define a custom error type using `thiserror`:
```rust
#[derive(Debug, thiserror::Error)]
pub enum MyLibError {
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```
Use this error type as the `Err` variant in all your `Result` returns. This gives callers strongly-typed errors they can match on. Don't use `anyhow` in library crates — `anyhow::Error` erases the error type, preventing callers from handling specific cases.

### Q: What's the difference between `anyhow` and `thiserror` and when should I use each?
`thiserror` generates structured, typed error enums. Use it in library crates where callers need to differentiate error cases and handle them programmatically. `anyhow` provides a convenient `Error` type that wraps any error and accumulates context. Use it in binary crates (applications) where the goal is a rich, human-readable error message rather than programmatic handling. The KoadOS rule: `thiserror` for `koad-core`, `koad-intelligence`, `koad-sandbox` and other library crates; `anyhow` for `koad-citadel`, `koad-cass`, and `koad-cli` binaries.

## Source Reference

- `docs/protocols/RUST_CANON.md` — The full canonical document; authoritative source for all rules
- `Cargo.toml` (root) — `[workspace.dependencies]` table; enforces the dependency inheritance rule
- `crates/koad-core/src/lib.rs` — Example of `#![warn(missing_docs)]` and `#![warn(rustdoc::broken_intra_doc_links)]` in a library crate
- `crates/koad-citadel/src/lib.rs` — Example of the ICE workaround: `#![cfg_attr(not(test), warn(missing_docs))]`
