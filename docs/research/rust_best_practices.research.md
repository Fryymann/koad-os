<aside>
📋

**Research Report for Tyr** — Deep-dive into Rust idioms, Cargo ecosystem features, and production patterns. All findings should be adopted into KoadOS/Citadel development canon.

Prepared by Noti · March 2026

</aside>

---

## 1. Project Structure & Cargo Workspaces

KoadOS/Citadel should use a **Cargo workspace** as the top-level organizational unit. This is the idiomatic Rust approach for multi-crate projects and gives us unified dependency resolution, a single `Cargo.lock`, and shared build artifacts.

### Recommended Layout

```
koad-os/
├── Cargo.toml          # [workspace] root
├── Cargo.lock
├── apps/               # Binary crates (executables)
│   ├── koad/           # Main CLI binary
│   ├── citadel/        # Citadel core binary
│   └── tyr/            # Tyr agent binary (if applicable)
├── libs/               # Library crates (shared code)
│   ├── koad-core/      # Core types, traits, config
│   ├── koad-memory/    # SQLite/Qdrant memory stack
│   ├── koad-stream/    # Koad Stream message bus
│   ├── koad-telemetry/ # Logging/tracing setup
│   └── koad-utils/     # Small shared utilities
├── tests/              # Workspace-level integration tests
└── .cargo/
    └── config.toml     # Workspace-wide Cargo config
```

### Workspace Cargo.toml Pattern

```toml
[workspace]
members = ["apps/*", "libs/*"]
resolver = "2"

[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "MIT"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
clap = { version = "4", features = ["derive"] }
rusqlite = { version = "0.32", features = ["bundled"] }
```

### Key Rules

- **Always define shared dependencies in `[workspace.dependencies]`** and inherit them in member crates with `dep.workspace = true`. This centralizes version management and prevents drift.
- **Use `resolver = "2"`** — it's the modern feature resolver that handles feature unification correctly for dev-dependencies and platform-specific deps.
- **Split crates by domain boundary**, not by file size. Each crate should own a clear responsibility. If two apps share memory logic, that's a `koad-memory` lib crate.
- **Avoid a single monolithic `shared` crate** that grows unbounded. Prefer multiple focused lib crates.

---

## 2. Error Handling

Rust's `Result<T, E>` and `Option<T>` are the foundation. Never panic in library code.

### Strategy: `thiserror` for libraries, `anyhow` for binaries

- **`thiserror`** — Use in all lib crates. Derive structured error enums so callers can match on variants.
- **`anyhow`** — Use in binary/app crates for ergonomic error propagation when you just need to report, not match.

### Patterns

```rust
// libs/koad-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KoadError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("config parse error: {0}")]
    Config(#[from] serde_json::Error),
    #[error("stream unavailable: {reason}")]
    StreamUnavailable { reason: String },
}
```

```rust
// apps/koad/src/main.rs
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("failed to load koad.json")?;
    // ...
    Ok(())
}
```

### Canon Rules

- **Never use `.unwrap()` or `.expect()` in library code** — propagate with `?` instead.
- **Reserve `.expect("reason")` for truly impossible states** in binary code, always with a descriptive message.
- **Add context as errors propagate** — use `anyhow`'s `.context()` or `.with_context(||)` so the final error message tells a clear story.
- **Implement `From` traits** (via `#[from]`) to enable seamless `?` conversion between error types.

---

## 3. Ownership, Borrowing & Idiomatic Patterns

### Core Principles

- **Prefer borrowing (`&T`, `&mut T`) over cloning** unless ownership transfer is genuinely needed.
- **Use `Cow<'a, str>`** when a function might need to either borrow or own a string.
- **Prefer `&str` over `&String`** in function parameters — it's more general.
- **Use `impl Into<String>` or `AsRef<str>`** for ergonomic APIs that accept multiple string types.
- **Prefer iterators over indexing** — `for item in &collection` is idiomatic; avoid `collection[i]` loops.

### Newtype Pattern

Encode domain invariants in the type system. Make illegal states unrepresentable:

```rust
pub struct AgentId(String);
pub struct StreamId(uuid::Uuid);

impl AgentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### Builder Pattern

Use builders for complex struct construction. The `typed-builder` or `bon` crate can derive these automatically.

---

## 4. Async Runtime — Tokio

Tokio is the de facto async runtime for Rust. KoadOS should standardize on it.

### Setup

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init tracing, config, then run
    Ok(())
}
```

### Best Practices

- **Never block the async runtime** — use `tokio::task::spawn_blocking()` for CPU-heavy or blocking I/O work (e.g., SQLite queries via rusqlite).
- **Use channels for inter-task communication** — `tokio::sync::mpsc` for multi-producer, `broadcast` for pub/sub, `oneshot` for request/response.
- **Use `tokio::select!`** for racing multiple futures (e.g., graceful shutdown signals).
- **Instrument async tasks** — always use `.instrument(tracing::info_span!("task_name"))` when spawning tasks so traces propagate correctly.
- **Use `tokio::time::timeout()`** to guard against hung operations.
- **Prefer `Arc<T>` over cloning data** when sharing across tasks. Combine with `Mutex` or `RwLock` from `tokio::sync` (not `std::sync`) for async-safe locking.

---

## 5. Serialization — Serde

Serde is non-negotiable for any Rust project dealing with config, APIs, or persistence.

### Patterns

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KoadConfig {
    pub binary_path: String,
    pub memory_db: String,
    pub stream_id: String,
    #[serde(default)]
    pub debug_mode: bool,
}
```

### Canon Rules

- **Use `#[serde(rename_all = "camelCase")]`** or `snake_case` consistently per format.
- **Use `#[serde(default)]`** for optional fields with sensible defaults.
- **Use `#[serde(skip_serializing_if = "Option::is_none")]`** to keep output clean.
- **Use `config-rs`** crate for layered configuration (file → env vars → CLI args) with type-safe deserialization.
- **Gate serde behind a feature flag in library crates** — `serde = { version = "1", optional = true }` with a `serde` feature, so downstream users opt in.

---

## 6. Logging & Observability — `tracing`

The `tracing` crate is the Rust ecosystem standard for structured, async-aware logging.

### Setup

```rust
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(true))
        .init();
}
```

### Best Practices

- **Use structured fields, not string interpolation**: `tracing::info!(agent_id = %id, "agent started")` instead of `info!("agent {} started", id)`.
- **Use `#[instrument]`** on functions to automatically create spans with argument capture.
- **Choose levels wisely**: `TRACE`/`DEBUG` for dev, `INFO`+ for production.
- **Use `RUST_LOG` env var** for runtime filtering (e.g., `RUST_LOG=koad_core=debug,koad=info`).
- **Initialize tracing before any business logic** — first thing in `main()`.
- **For JSON structured output** (production): enable `tracing-subscriber`'s `json` feature.

---

## 7. Testing

Cargo has first-class testing support. Lean into it heavily.

### Test Organization

- **Unit tests**: In the same file, in a `#[cfg(test)] mod tests { ... }` block. Test private functions here.
- **Integration tests**: In `tests/` directory at the crate root. Test the public API.
- **Doc tests**: In `///` doc comments with code examples. They run as tests automatically with `cargo test`.

### Cargo Commands

```bash
cargo test                     # Run all tests
cargo test -p koad-core        # Test a specific crate
cargo test -- --nocapture      # Show println! output
cargo test my_func             # Run tests matching name
```

### `cargo-nextest` (Recommended)

Install `cargo-nextest` as the upgraded test runner:

```bash
cargo install cargo-nextest
cargo nextest run
```

Benefits over `cargo test`:

- **Faster** — runs each test in its own process, better parallelism.
- **Cleaner output** — pass/fail at a glance, debug logs only on failure.
- **Retries and flaky test detection** built in.
- **JUnit XML output** for CI integration.

### Canon Rules

- Every public function in a lib crate must have at least one unit test.
- Use `#[should_panic]` sparingly — prefer testing `Result::Err` variants.
- Use `assert_eq!` and `assert_ne!` with descriptive messages: `assert_eq!(result, expected, "config should parse debug_mode");`
- Use `proptest` or `quickcheck` for property-based testing of parsing/encoding logic.

---

## 8. Cargo Tools — Offloading Work

One of Cargo's greatest strengths is its plugin ecosystem. Use these to automate quality gates.

### Essential Cargo Subcommands

| **Tool** | **Command** | **Purpose** |
| --- | --- | --- |
| **Clippy** | `cargo clippy` | Official linter. Catches common mistakes, suggests idiomatic patterns. Run on every commit. |
| **Rustfmt** | `cargo fmt` | Auto-formatter. Enforces consistent style. Run on every save. Configure via `rustfmt.toml`. |
| **cargo-nextest** | `cargo nextest run` | Next-gen test runner. Faster, cleaner output, retry support. |
| **cargo-llvm-cov** | `cargo llvm-cov` | Code coverage via LLVM instrumentation. Already in KoadOS prime directives. |
| **cargo-doc** | `cargo doc --open` | Generate & browse HTML documentation from doc comments. |
| **cargo-deadlinks** | `cargo deadlinks` | Find broken links in generated docs. |
| **cargo-deny** | `cargo deny check` | Audit dependencies for licenses, vulnerabilities, duplicates, and banned crates. |
| **cargo-audit** | `cargo audit` | Check dependencies against the RustSec advisory database. |
| **cargo-outdated** | `cargo outdated` | Show which dependencies have newer versions available. |
| **cargo-watch** | `cargo watch -x check` | Auto-rerun commands on file change. Great for dev loop: `cargo watch -x 'clippy -- -D warnings'`. |
| **cargo-expand** | `cargo expand` | Show macro-expanded code. Essential for debugging derive macros. |
| **cargo-bloat** | `cargo bloat` | Identify what's taking up space in your binary. |
| **cargo-udeps** | `cargo udeps` | Find unused dependencies in Cargo.toml. |

### Install Script

```bash
# One-liner to install all recommended cargo tools
cargo install cargo-nextest cargo-llvm-cov cargo-deny cargo-audit \
  cargo-outdated cargo-watch cargo-expand cargo-bloat cargo-udeps
```

---

## 9. Build Optimization

### Dev Build Speed

Add to `.cargo/config.toml` for faster dev builds:

```toml
[profile.dev]
opt-level = 1            # Small optimization for dev

[profile.dev.package."*"]
opt-level = 3            # Full optimization for dependencies

[build]
jobs = 8                 # Parallel compilation jobs
```

This gives you fast recompiles of your own code while dependencies stay fully optimized — best of both worlds for runtime performance during development.

### Release Profile

```toml
[profile.release]
opt-level = 3
lto = "thin"              # Link-time optimization (good balance)
codegen-units = 1         # Better optimization, slower compile
strip = true              # Strip debug symbols from binary
panic = "abort"           # Smaller binary, no unwinding overhead
```

### Linker Optimization

Use `mold` (Linux) or `lld` for dramatically faster linking:

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

---

## 10. Concurrency & Memory Safety Patterns

### Choosing the Right Primitive

| **Scenario** | **Use** |
| --- | --- |
| Share read-only data across threads | `Arc<T>` |
| Shared mutable state, mostly reads | `Arc<RwLock<T>>` (tokio) |
| Shared mutable state, frequent writes | `Arc<Mutex<T>>` (tokio) |
| Message passing between tasks | `tokio::sync::mpsc` / `broadcast` |
| One-shot response | `tokio::sync::oneshot` |
| Lock-free counters/flags | `std::sync::atomic::*` |

### Canon Rules

- **Prefer message passing over shared state** when architecturally feasible (aligns with Koad Stream's message bus design).
- **Never hold a lock across an `.await` point** — use scoped blocks or redesign with channels.
- **Use `Arc::clone(&arc)`** instead of `arc.clone()` to make intent explicit.

---

## 11. Documentation

Cargo's doc system is powerful. Use it as a first-class deliverable.

### Rules

- **Every public item gets a `///` doc comment** — functions, structs, enums, traits, modules.
- **Include examples in doc comments** — they're compiled and tested with `cargo test`.
- **Use `//!` module-level docs** at the top of `lib.rs` and key modules.
- **Run `cargo doc --open --no-deps`** to preview.
- **Run `cargo deadlinks`** to catch broken intra-doc links.

### Example

```rust
/// Loads the KoadOS configuration from disk.
///
/// Reads `~/.koad-os/koad.json` and deserializes it into a
/// [`KoadConfig`] struct.
///
/// # Errors
///
/// Returns [`KoadError::Config`] if the file is missing or malformed.
///
/// # Examples
///
/// ```no_run
/// let config = koad_core::load_config()?;
/// assert!(!config.binary_path.is_empty());
/// ```
pub fn load_config() -> Result<KoadConfig, KoadError> {
    // ...
}
```

---

## 12. Essential Crate Stack

Curated crate recommendations for KoadOS/Citadel:

| **Domain** | **Crate** | **Notes** |
| --- | --- | --- |
| Async runtime | `tokio` | Full-featured async runtime. Use `features = ["full"]`. |
| Error handling (lib) | `thiserror` | Derive `Error` enums with structured variants. |
| Error handling (bin) | `anyhow` | Ergonomic error propagation with context. |
| Serialization | `serde`  • `serde_json` / `toml` | Config, API payloads, persistence. |
| CLI | `clap` (derive) | Argument parsing with auto-generated help. |
| Logging | `tracing`  • `tracing-subscriber` | Structured, async-aware, span-based telemetry. |
| HTTP client | `reqwest` | Async HTTP with TLS, JSON, cookies. |
| HTTP server | `axum` | Tokio-native, tower-based web framework. |
| Database (SQLite) | `rusqlite` | Bundled SQLite. Use `spawn_blocking` for async compat. |
| UUID | `uuid` | For Stream IDs, agent IDs, etc. |
| Date/time | `chrono` or `time` | `time` is lighter; `chrono` has broader ecosystem support. |
| TUI | `ratatui` | Already in KoadOS stack. Terminal UI framework. |
| Testing | `proptest` | Property-based testing for encoding/parsing. |

Reference: [blessed.rs](http://blessed.rs) — community-curated crate recommendations.

---

## 13. CI / Pre-Commit Quality Gates

Every commit should pass these checks. Automate via Git hooks or CI:

```bash
#!/bin/bash
set -e

# Format check
cargo fmt --all -- --check

# Lint (treat warnings as errors)
cargo clippy --workspace --all-targets -- -D warnings

# Test
cargo nextest run --workspace

# Security audit
cargo audit

# Dependency policy
cargo deny check

# Doc build (catch broken links)
cargo doc --workspace --no-deps
cargo deadlinks
```

---

## 14. Feature Flags & Conditional Compilation

Use Cargo features to keep the build modular:

```toml
# libs/koad-core/Cargo.toml
[features]
default = ["sqlite"]
sqlite = ["dep:rusqlite"]
qdrant = ["dep:qdrant-client"]
```

This lets Tyr build only the features needed for a given binary or environment. Use `#[cfg(feature = "...")]` in source code to gate implementations.

---

## 15. Summary — Tyr Implementation Checklist

- [ ]  Restructure repo as a Cargo workspace (`apps/` + `libs/`)
- [ ]  Centralize all shared deps in `[workspace.dependencies]`
- [ ]  Adopt `thiserror` in all lib crates, `anyhow` in binaries
- [ ]  Eliminate all `.unwrap()` calls from library code
- [ ]  Set up `tracing` + `tracing-subscriber` as the logging standard
- [ ]  Install and configure `cargo-nextest` as the test runner
- [ ]  Add `cargo clippy -D warnings` and `cargo fmt --check` to pre-commit
- [ ]  Configure `.cargo/config.toml` with dev build optimizations + `mold` linker
- [ ]  Set up `cargo deny` with license and advisory policies
- [ ]  Add doc comments to all public items; verify with `cargo doc` + `cargo deadlinks`
- [ ]  Gate optional backends behind feature flags
- [ ]  Establish release profile with LTO, strip, and `panic = "abort"`