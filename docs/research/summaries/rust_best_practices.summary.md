**Scribe Briefing** — Condensed summary of the Rust Best Practices & Cargo Guide for KoadOS/Citadel development. All agents working on Rust code should read this briefing first.

For full detail, code examples, and implementation patterns → [Rust Best Practices & Cargo Guide — KoadOS Development Canon](.koad-os\docs\research\rust_best_practices.research.md)
---

## How to Use This Briefing

- **Read this page first** before writing or reviewing any Rust code in KoadOS/Citadel.
- **If you need code examples, config snippets, or deeper rationale** on any topic below, load the full source report linked above.
- **Do not deviate from these rules** without explicit approval from Ian or an updated canon entry.

---

## Project Structure

- KoadOS uses a **Cargo workspace** with an `apps/` (binaries) + `libs/` (libraries) layout.
- All shared dependencies live in `[workspace.dependencies]` at the workspace root. Member crates inherit with `dep.workspace = true`.
- Use `resolver = "2"`. Split crates by **domain boundary**, not file size. No monolithic "shared" crate.

## Error Handling

- **Libraries** → `thiserror` (structured error enums, matchable variants).
- **Binaries** → `anyhow` (ergonomic propagation with `.context()`).
- **Never `.unwrap()` or `.expect()` in library code.** Propagate with `?`.
- Always add context as errors bubble up.

## Ownership & Idioms

- Borrow (`&T`) over clone. Use `&str` not `&String` in params.
- Use the **newtype pattern** to encode domain types (`AgentId`, `StreamId`) — make illegal states unrepresentable.
- Prefer **iterators** over index-based loops.

## Async (Tokio)

- Tokio is the standard runtime. Never block it — use `spawn_blocking()` for CPU/blocking work.
- Use `tokio::sync` channels (`mpsc`, `broadcast`, `oneshot`) for inter-task comms.
- Always `.instrument()` spawned tasks with tracing spans.
- Use `tokio::sync::Mutex`/`RwLock`, **not** `std::sync`, in async contexts.

## Serialization (Serde)

- Use `#[serde(rename_all = "camelCase")]` consistently. Use `#[serde(default)]` for optional fields.
- Gate serde behind a **feature flag** in lib crates.
- Use `config-rs` for layered config (file → env → CLI).

## Logging (`tracing`)

- `tracing` + `tracing-subscriber` is the standard. Initialize **before** any business logic.
- Use **structured fields** (`info!(agent_id = %id, "started")`) — never string interpolation.
- Use `#[instrument]` on functions. Filter with `RUST_LOG` env var.

## Testing

- Unit tests: `#[cfg(test)] mod tests` in-file. Integration tests: `tests/` directory.
- Use **`cargo-nextest`** as the test runner (faster, cleaner, retries).
- Every public function needs at least one test. Use `proptest` for encoding/parsing.

## Cargo Toolchain (Quality Gates)

- **Every commit must pass**: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo nextest run`, `cargo audit`, `cargo deny check`.
- Additional tools: `cargo-llvm-cov` (coverage), `cargo-deadlinks` (docs), `cargo-udeps` (unused deps), `cargo-bloat` (binary size).

## Build Optimization

- Dev: `opt-level = 1` for own code, `opt-level = 3` for deps. Use `mold` linker.
- Release: `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = true`, `panic = "abort"`.

## Concurrency

- **Prefer message passing** (channels) over shared state — aligns with Koad Stream design.
- Never hold a lock across `.await`. Use `Arc::clone(&arc)` for explicit intent.

## Feature Flags

- Gate optional backends (e.g., `sqlite`, `qdrant`) behind Cargo features.
- Use `#[cfg(feature = "...")]` for conditional compilation.

## Approved Crate Stack

`tokio` · `thiserror` · `anyhow` · `serde` · `clap` · `tracing` · `reqwest` · `axum` · `rusqlite` · `uuid` · `chrono`/`time` · `ratatui` · `proptest`

---

<aside>
🔗

**Need more?** Load the full report: [Rust Best Practices & Cargo Guide — KoadOS Development Canon](https://www.notion.so/Rust-Best-Practices-Cargo-Guide-KoadOS-Development-Canon-c7a2ebc9049f4e089744f12f24c2425a?pvs=21)

It contains code examples, config file templates, the complete Cargo tools table, and Tyr's implementation checklist.

</aside>