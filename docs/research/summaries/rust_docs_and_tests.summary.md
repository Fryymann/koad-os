<aside>
📜

**Scribe Briefing** — Condensed summary of the Rust In-File Documentation & Testing deep dive. All agents writing or reviewing Rust code should read this first.

For full detail, code examples, and patterns → [Rust In-File Documentation & Testing — Deep Dive](https://www.notion.so/Rust-In-File-Documentation-Testing-Deep-Dive-3bff2b18b9b24836bd73268187c2ee53?pvs=21)

</aside>

---

## How to Use This Briefing

- **Read this page first** before writing docs or tests in any KoadOS/Citadel Rust crate.
- **If you need code templates, annotation tables, or pattern examples**, load the full source report linked above.
- **Do not deviate from these rules** without explicit approval from Ian or an updated canon entry.

---

## Documentation (Rustdoc)

### Comment Types

- `///` documents the item **below** it (functions, structs, enums, traits).
- `//!` documents the **enclosing** item (crate root `lib.rs`, module headers).
- `//` is a regular comment — ignored by rustdoc.

### Required Doc Sections (in this order)

1. **Summary line** — single clear sentence, always required.
2. **Extended description** — optional deeper explanation.
3. **`# Examples`** — strongly recommended; code blocks compile and run as tests.
4. **`# Errors`** — required for any function returning `Result`.
5. **`# Panics`** — required if the function can panic.
6. **`# Safety`** — required for `unsafe` functions.

### Intra-Doc Links

- Always link to types/functions **by path**: `[`AgentId`]`, `[`KoadConfig::stream_id`]`.
- Never use raw URLs in doc comments. Intra-doc links are checked at build time.

### Module & Crate Docs

- Every module needs a `//!` header block describing its purpose and architecture.
- `lib.rs` must include: purpose, feature overview, quick start example, links to key modules.

### Mandatory Lint Flags

All lib crate roots must include:

- `#![warn(missing_docs)]`
- `#![warn(rustdoc::broken_intra_doc_links)]`

---

## Documentation Tests (Doctests)

- Code blocks in `///` comments are **compiled and executed** during `cargo test`.
- Use **hidden lines** (`#` ) to hide imports and `fn main()` wrappers from rendered docs while keeping the test complete.
- Use `?` in doctests by wrapping in a hidden `fn main() -> Result<(), ...>`.

### Code Block Annotations

- **(default)** — compiles and runs.
- **`no_run`** — compiles but doesn't execute (for network/file examples).
- **`compile_fail`** — must fail to compile (for showing API misuse).
- **`should_panic`** — must panic when run.
- **`ignore`** — skipped entirely.
- **`text`** — non-Rust content (shell commands, config output).

---

## Testing

### Test Organization

- **Unit tests** → `#[cfg(test)] mod tests` in each source file. Can test private functions.
- **Integration tests** → `tests/` directory at crate root. Public API only.
- **Shared fixtures** → `tests/common/mod.rs` (avoids Cargo treating it as a test binary).

### Async Tests

- Use `#[tokio::test]` for async functions. Default is `current_thread`; use `flavor = "multi_thread"` when testing concurrency.

### Mocking Strategy

- **Simple deps** (Clock, Config) → manual fakes via trait + test struct.
- **Complex deps** → `mockall` with `#[automock]`.
- **Never mock types you own** — test the real implementation.

### Snapshot Testing (`insta`)

- Use `assert_json_snapshot!` for serializable structs, `assert_snapshot!` for strings.
- Redact volatile fields (timestamps, UUIDs) with `insta::Settings`.
- Snapshots are committed to git; use `cargo insta review` for interactive approval.

### Property-Based Testing (`proptest`)

- Use for serde roundtrips, parser validation, and wide-input-domain functions.
- Proptest auto-shrinks failing inputs to minimal reproducible cases.

### Testing Tier System

- **Tier 1 (Required):** Doc tests on all public API.
- **Tier 2 (Required):** Unit tests for all non-trivial logic.
- **Tier 3 (Required):** Snapshot tests for all serializable types.
- **Tier 4 (Recommended):** Property tests for parsers/encoders.
- **Tier 5 (Recommended):** Integration tests per crate.

### Test Naming

- `snake_case` names that read as specs: `loads_config_from_default_path`, `rejects_empty_agent_id`.
- Always include context messages in assertions.

### Test Crate Stack

`insta` · `proptest` · `mockall` · `test-context` · `assert_cmd` · `predicates` · `tempfile` · `fake` · `cargo-nextest`

---

<aside>
🔗

**Need more?** Load the full report: [Rust In-File Documentation & Testing — Deep Dive](https://www.notion.so/Rust-In-File-Documentation-Testing-Deep-Dive-3bff2b18b9b24836bd73268187c2ee53?pvs=21)

It contains complete code examples, annotation reference tables, mocking patterns, and the implementation checklist.

</aside>