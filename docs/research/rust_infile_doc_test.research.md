<aside>
📋

**Research Report — Citadel Library** · Deep dive into Rust in-file documentation (rustdoc) and testing patterns for KoadOS/Citadel development.

Prepared by Noti · March 2026

</aside>

---

# Part 1 — In-File Documentation (Rustdoc)

Rust's documentation system is unique: doc comments are **compiled, tested, and rendered as HTML** via `rustdoc`. Documentation is a first-class artifact, not an afterthought.

---

## 1.1 Comment Types

| **Syntax** | **Name** | **Attaches to** | **Use for** |
| --- | --- | --- | --- |
| `///` | Outer doc comment | The item **below** the comment | Functions, structs, enums, traits, constants, type aliases |
| `//!` | Inner doc comment | The **enclosing** item (parent) | Crate root (`lib.rs`), module top (`mod.rs` or inline modules) |
| `//` | Line comment | Nothing (ignored by rustdoc) | Implementation notes, TODOs, internal reasoning |

### Key Distinction

- `///` = "document the thing below me"
- `//!` = "document the thing I'm inside of"

```rust
//! # koad-core
//!
//! Core types, traits, and configuration for KoadOS.
//! This crate provides the foundational abstractions
//! shared across all KoadOS binaries and libraries.

/// A unique identifier for a KoadOS agent.
///
/// Wraps a `String` to enforce type safety at API boundaries.
pub struct AgentId(String);
```

---

## 1.2 Standard Doc Sections (Conventions)

The Rust API Guidelines (RFC 505) define standard sections. **Use these headers consistently in KoadOS**:

### Summary Line (required)

The first paragraph is the **summary**. It appears in module-level listings and search results. Make it a single, clear sentence.

```rust
/// Loads the KoadOS configuration from disk.
```

### Extended Description (optional)

Additional paragraphs after the summary provide deeper explanation.

```rust
/// Loads the KoadOS configuration from disk.
///
/// Reads `~/.koad-os/koad.json`, validates the schema,
/// and deserializes it into a [`KoadConfig`] struct.
/// If the file doesn't exist, returns a default config.
```

### `# Examples` (strongly recommended)

Code examples that **compile and run as tests**. This is rustdoc's killer feature.

```rust
/// # Examples
///
/// ```
/// use koad_core::AgentId;
///
/// let id = AgentId::new("tyr");
/// assert_eq!(id.as_str(), "tyr");
/// ```
```

### `# Errors` (required when returning `Result`)

Document every error variant the function can return.

```rust
/// # Errors
///
/// Returns [`KoadError::Config`] if the file is missing or malformed.
/// Returns [`KoadError::Database`] if the memory DB cannot be opened.
```

### `# Panics` (required when function can panic)

Document any conditions that cause a panic.

```rust
/// # Panics
///
/// Panics if `agent_id` is an empty string.
```

### `# Safety` (required for `unsafe` functions)

Explain invariants the caller must uphold.

```rust
/// # Safety
///
/// The caller must ensure `ptr` points to a valid,
/// aligned `KoadConfig` that outlives the returned reference.
```

### Canon Ordering

Use this section order in all KoadOS doc comments:

1. Summary line
2. Extended description
3. `# Examples`
4. `# Errors`
5. `# Panics`
6. `# Safety`
7. `# Notes` (misc implementation notes, if needed)

---

## 1.3 Intra-Doc Links

Rustdoc supports linking to other items **by path**, not URL. This is the preferred linking method:

```rust
/// Converts this [`AgentId`] into a [`StreamId`] using
/// the configured [`KoadConfig::stream_id`] field.
///
/// See also: [`crate::stream::publish()`]
pub fn to_stream_id(&self, config: &KoadConfig) -> StreamId { ... }
```

### Syntax Reference

| **Link to** | **Syntax** |
| --- | --- |
| Type in same crate | `[`MyStruct`]` |
| Method on a type | `[`MyStruct::my_method()`]` |
| Type in another crate | `[`other_crate::MyType`]` |
| Module | `[`crate::stream`]` |
| Trait | `[`MyTrait`]` |
| Enum variant | `[`MyEnum::VariantA`]` |
| Disambiguate function | `[`my_fn()`]` (trailing `()`) |
| Disambiguate type vs value | `[`type@MyType`]` or `[`value@MY_CONST`]` |

### Canon Rule

**Always use intra-doc links** instead of raw URLs or plain text type names. They're checked at build time — broken links produce warnings with `#![warn(rustdoc::broken_intra_doc_links)]`.

---

## 1.4 Module-Level Documentation

Every module in KoadOS should have a `//!` header block at the top:

```rust
//! # Stream Management
//!
//! This module handles the Koad Stream message bus —
//! publishing, subscribing, and message serialization.
//!
//! ## Architecture
//!
//! Messages flow through [`StreamPublisher`] and are
//! consumed by [`StreamSubscriber`] instances. See
//! [`crate::config::KoadConfig`] for connection settings.
//!
//! ## Examples
//!
//! ```no_run
//! use koad_stream::{StreamPublisher, Message};
//!
//! let pub_ = StreamPublisher::connect("localhost:6379").await?;
//! pub_.publish(Message::log("Sync complete")).await?;
//! ```
```

### Crate Root (`lib.rs`)

The `lib.rs` `//!` block is the **crate landing page**. It should include:

- One-sentence crate purpose
- Feature overview
- Quick start example
- Links to key modules
- Feature flag documentation

---

## 1.5 Attribute Annotations for Docs

| **Attribute** | **Effect** |
| --- | --- |
| `#[doc(alias = "name")]` | Adds search aliases (e.g., alias "KAI" for `AgentId`) |
| `#[doc(hidden)]` | Hides from generated docs (internal helpers) |
| `#[must_use = "reason"]` | Compiler warns if return value is unused |
| `#[deprecated(since = "0.2.0", note = "Use X instead")]` | Marks item as deprecated with migration guidance |
| `#![warn(missing_docs)]` | Warn on any public item without docs (add to `lib.rs`) |
| `#![warn(rustdoc::broken_intra_doc_links)]` | Warn on broken doc links |

### Canon Rule for KoadOS

All lib crates must have these in `lib.rs`:

```rust
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
```

---

# Part 2 — Documentation Tests (Doctests)

Doc tests are Rust's secret weapon: examples in documentation that **compile and execute as tests**. They guarantee your docs never go stale.

---

## 2.1 How Doctests Work

- Rustdoc extracts every fenced code block from `///` and `//!` comments.
- Each block is compiled as a **standalone mini-program** with an implicit `fn main() { ... }` wrapper.
- They run during `cargo test` (specifically the "Doc-tests" section).
- Since Rust 2024 edition, doctests are **combined into a single binary** for dramatically faster execution.

### Implicit Wrapping

You write:

```rust
/// ```
/// let x = 5;
/// assert_eq!(x, 5);
/// ```
```

Rustdoc compiles:

```rust
fn main() {
    let x = 5;
    assert_eq!(x, 5);
}
```

---

## 2.2 Hidden Lines (`#`)

Prefix a line with `#`  to **hide it from rendered docs but include it in the test**. This is critical for keeping examples clean:

```rust
/// ```
/// # use koad_core::{KoadConfig, KoadError};
/// # fn main() -> Result<(), KoadError> {
/// let config = KoadConfig::load()?;
/// assert!(!config.binary_path.is_empty());
/// # Ok(())
/// # }
/// ```
```

The reader sees only:

```rust
let config = KoadConfig::load()?;
assert!(!config.binary_path.is_empty());
```

But the full program compiles and runs.

### When to Use Hidden Lines

- `use` imports (almost always hide these)
- `fn main()` wrappers for `Result`-returning examples
- Setup/teardown code
- Error type aliases

---

## 2.3 Code Block Annotations

| **Annotation** | **Compiles?** | **Runs?** | **Use when** |
| --- | --- | --- | --- |
| (none) or `rust` | ✅ | ✅ | Normal example — default |
| `no_run` | ✅ | ❌ | Example requires runtime resources (network, files, hardware) |
| `should_panic` | ✅ | ✅ (must panic) | Demonstrating panic behavior |
| `compile_fail` | ❌ (must fail) | ❌ | Showing what does NOT compile (borrow checker demos, API misuse) |
| `ignore` | ❌ | ❌ | Placeholder, pseudo-code, or known broken example |
| `text` | ❌ | ❌ | Non-Rust content (shell commands, config files, output) |

### Examples

```rust
/// Network example — compiles but doesn't run in tests:
/// ```no_run
/// let stream = koad_stream::connect("localhost:6379").await?;
/// ```
///
/// This should NOT compile (enforces API safety):
/// ```compile_fail
/// let id: AgentId = "raw string".into(); // no From<&str> impl
/// ```
///
/// Shell command (not Rust code):
/// ```text
/// $ cargo run -- stream post "Hello" "World"
/// ```
```

---

## 2.4 Doctest Patterns for `?` and `Result`

The most common doctest pain point. The hidden `main()` wrapper returns `()`, so `?` doesn't work by default. Solutions:

### Pattern A: Hidden `main` with `Result` return (preferred)

```rust
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = koad_core::KoadConfig::load()?;
/// assert!(config.debug_mode == false);
/// # Ok(())
/// # }
/// ```
```

### Pattern B: Use `anyhow::Result` for shorter hidden boilerplate

```rust
/// ```
/// # fn main() -> anyhow::Result<()> {
/// let agent = koad_core::AgentId::new("tyr");
/// # Ok(())
/// # }
/// ```
```

### Pattern C: Unwrap (acceptable for infallible examples)

```rust
/// ```
/// let parsed: u32 = "42".parse().unwrap();
/// assert_eq!(parsed, 42);
/// ```
```

---

# Part 3 — Testing

---

## 3.1 Test Organization

### Unit Tests

Live **inside the source file** in a `#[cfg(test)]` module:

```rust
// src/config.rs

pub fn load_config() -> Result<KoadConfig, KoadError> {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_default_config() {
        let config = load_config().expect("default config should load");
        assert!(!config.binary_path.is_empty());
    }

    #[test]
    fn rejects_malformed_json() {
        // test with bad input...
        let result = parse_config("{invalid");
        assert!(result.is_err());
    }
}
```

**Key points:**

- `#[cfg(test)]` means this module is **compiled only during `cargo test`** — zero runtime overhead.
- `use super::*` imports everything from the parent module, including private items.
- Unit tests **can test private functions** — this is intentional and idiomatic in Rust.

### Integration Tests

Live in `tests/` at the crate root. They can only access the **public API**:

```
libs/koad-core/
├── src/
│   └── lib.rs
└── tests/
    ├── config_tests.rs
    └── common/
        └── mod.rs       # Shared test utilities
```

```rust
// tests/config_tests.rs
use koad_core::KoadConfig;

#[test]
fn config_round_trips_through_serde() {
    let original = KoadConfig::default();
    let json = serde_json::to_string(&original).unwrap();
    let restored: KoadConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}
```

### Shared Test Utilities

Place shared helpers in `tests/common/mod.rs`. Import with `mod common;` from test files. This pattern avoids Cargo treating `common.rs` as its own test binary.

---

## 3.2 Async Testing with Tokio

For async code, use `#[tokio::test]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stream_publishes_message() {
        let publisher = StreamPublisher::connect_test().await.unwrap();
        let result = publisher.publish(Message::log("test")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn timeout_on_unreachable_host() {
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            StreamPublisher::connect("192.0.2.1:6379"),
        ).await;
        assert!(result.is_err(), "should timeout");
    }
}
```

### Multi-threaded vs Current-thread

```rust
#[tokio::test]                            // default: current_thread
#[tokio::test(flavor = "multi_thread")]   // use when testing concurrency
```

---

## 3.3 Test Fixtures & Setup/Teardown

Rust has no built-in fixtures. Patterns:

### Pattern A: Helper functions (simplest, preferred)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> KoadConfig {
        KoadConfig {
            binary_path: "/tmp/koad".into(),
            memory_db: ":memory:".into(),
            stream_id: "test-stream".into(),
            debug_mode: true,
        }
    }

    #[test]
    fn config_has_debug_enabled() {
        let config = test_config();
        assert!(config.debug_mode);
    }
}
```

### Pattern B: `test-context` crate (for setup/teardown)

```rust
use test_context::{test_context, TestContext};

struct DbContext {
    conn: rusqlite::Connection,
}

impl TestContext for DbContext {
    fn setup() -> Self {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE agents (id TEXT)").unwrap();
        DbContext { conn }
    }
    fn teardown(self) {
        // Connection drops automatically
    }
}

#[test_context(DbContext)]
#[test]
fn inserts_agent(ctx: &mut DbContext) {
    ctx.conn.execute("INSERT INTO agents VALUES (?)", ["tyr"]).unwrap();
    // ...
}
```

---

## 3.4 Mocking with Traits & Mockall

Rust's approach to mocking: **define a trait for the dependency, then swap implementations in tests.**

### Manual Mock (preferred for simple cases)

```rust
pub trait Clock {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
}

pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeClock(chrono::DateTime<chrono::Utc>);
    impl Clock for FakeClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            self.0
        }
    }

    #[test]
    fn formats_timestamp() {
        let fixed = chrono::Utc::now();
        let clock = FakeClock(fixed);
        let result = format_log_entry(&clock, "test message");
        assert!(result.contains("test message"));
    }
}
```

### Mockall (for complex trait mocking)

```rust
use mockall::automock;

#[automock]
pub trait EmailSender {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), SendError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notifier_sends_email_on_alert() {
        let mut mock = MockEmailSender::new();
        mock.expect_send()
            .withf(|to, subject, _| to == "ian@koad.os" && subject.contains("Alert"))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let notifier = Notifier::new(mock);
        notifier.alert("System overload").unwrap();
    }
}
```

### Canon Rule

- **Prefer manual fakes** for simple dependencies (Clock, Config, Logger).
- **Use `mockall`** only when the trait has many methods or complex expectations.
- **Never mock types you own** — test the real implementation directly.

---

## 3.5 Snapshot Testing with `insta`

Snapshot tests capture output once, then assert against it on future runs. Ideal for:

- CLI output
- Serialized data structures (JSON, TOML)
- Error messages
- TUI rendering (works great with `ratatui`)

### Setup

```toml
[dev-dependencies]
insta = { version = "1", features = ["json", "yaml"] }
```

### Usage

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn config_serializes_correctly() {
        let config = KoadConfig::default();
        assert_json_snapshot!(config);
    }

    #[test]
    fn error_message_is_human_readable() {
        let err = KoadError::StreamUnavailable {
            reason: "connection refused".into(),
        };
        insta::assert_snapshot!(err.to_string());
    }
}
```

### Workflow

```
cargo install cargo-insta         # Install CLI
cargo insta test                  # Run tests, generate new snapshots
cargo insta review                # Interactively accept/reject changes
```

Snapshots are stored in `snapshots/` directories and **committed to git**. Diffs show up in PRs for easy review.

### Canon Rules for Snapshots

- Use `insta::assert_json_snapshot!` for serializable structs.
- Use `insta::assert_snapshot!` for string output.
- **Redact volatile fields** (timestamps, UUIDs) with `insta::Settings`:

```rust
let mut settings = insta::Settings::clone_current();
settings.add_redaction(".created_at", "[timestamp]");
settings.bind(|| {
    assert_json_snapshot!(event);
});
```

---

## 3.6 Property-Based Testing

Generate random inputs and verify invariants hold for all of them.

### `proptest` (recommended)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn agent_id_roundtrips(id in "[a-z][a-z0-9_]{2,20}") {
        let agent = AgentId::new(&id);
        assert_eq!(agent.as_str(), id);
    }

    #[test]
    fn config_survives_serde_roundtrip(debug in any::<bool>()) {
        let config = KoadConfig {
            debug_mode: debug,
            ..KoadConfig::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: KoadConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.debug_mode, restored.debug_mode);
    }
}
```

### When to Use

- Serialization/deserialization roundtrips
- Parser input validation
- Encoding/decoding functions
- Any function with a wide input domain

---

## 3.7 Test Naming & Assertions

### Naming Convention

Use `snake_case` names that read as **specifications**:

```rust
#[test]
fn loads_config_from_default_path() { ... }

#[test]
fn returns_error_on_missing_file() { ... }

#[test]
fn rejects_empty_agent_id() { ... }

#[test]
fn stream_id_formats_as_uuid() { ... }
```

### Assertion Best Practices

- Always include context messages:

```rust
assert_eq!(result.len(), 3, "expected 3 agents, got {}", result.len());
assert!(config.debug_mode, "debug_mode should be true in test config");
```

- Use `matches!` for enum variant checks:

```rust
assert!(matches!(err, KoadError::StreamUnavailable { .. }));
```

- Use `assert!(result.is_ok())` and `assert!(result.is_err())` for Result checks. For extracting values:

```rust
let config = load_config().expect("test setup: config should load");
```

---

## 3.8 Canon: KoadOS Testing Tier System

| **Tier** | **Type** | **Scope** | **Required?** |
| --- | --- | --- | --- |
| 1 | Doc tests | Every public function with `# Examples` | ✅ Required for all public API |
| 2 | Unit tests | `#[cfg(test)] mod tests` in each source file | ✅ Required for all non-trivial logic |
| 3 | Snapshot tests | Serialization output, error messages, CLI output | ✅ Required for all serializable types |
| 4 | Property tests | Parsers, encoders, validators | ⬜ Recommended for wide-input functions |
| 5 | Integration tests | `tests/` directory, public API only | ⬜ Recommended per crate |

---

## 3.9 Essential Test Crate Stack

| **Crate** | **Purpose** |
| --- | --- |
| `insta` | Snapshot testing (JSON, YAML, string output) |
| `proptest` | Property-based testing with shrinking |
| `mockall` | Automated trait mocking |
| `test-context` | Setup/teardown fixtures |
| `assert_cmd` | CLI binary integration testing |
| `predicates` | Fluent assertions for `assert_cmd` |
| `tempfile` | Temporary files/dirs that auto-cleanup |
| `fake` | Generate realistic fake data for tests |
| `cargo-nextest` | Faster test runner with better output |

---

## Implementation Checklist

- [ ]  Add `#![warn(missing_docs)]` and `#![warn(rustdoc::broken_intra_doc_links)]` to all lib crate roots
- [ ]  Write `//!` module docs for every module in every crate
- [ ]  Add `# Examples` doc sections to all public functions
- [ ]  Add `# Errors` sections to all public functions returning `Result`
- [ ]  Use hidden lines (`#`) in doc examples for clean rendering
- [ ]  Use intra-doc links exclusively (no raw URLs in docs)
- [ ]  Set up `insta` for snapshot testing of all serializable types
- [ ]  Add `proptest` roundtrip tests for all serde types
- [ ]  Establish shared test utilities in `tests/common/mod.rs` per crate
- [ ]  Define trait boundaries for external dependencies to enable mocking