# KoadOS Rust Development Canon (v1.0)
**Status:** MANDATORY (Citadel Phase 2+)
**Objective:** Ensure high-performance, secure, and maintainable Rust code across the KoadOS ecosystem.

## Ⅰ. Structural Standards
- **Workspace-Centric:** All new crates MUST be registered in the root `Cargo.toml`.
- **Dependency Inheritance:** Use `[workspace.dependencies]` for all shared crates (Serde, Tokio, Anyhow, Tracing).
- **Crate Boundaries:** 
    - `koad-core`: Core logic, models, and shared utilities.
    - `koad-proto`: gRPC and Protobuf definitions.
    - `koad-cli` / `koad-agent`: User-facing CLI binaries.
    - `koad-citadel`: OS-level persistent services.

## Ⅱ. Error Handling & Type Safety
- **Zero-Panic Policy:** No `.unwrap()` or `.expect()` in production code. 
- **Propagation:**
    - **Binaries:** Use `anyhow::Result` with `.context()` for ergonomic error bubbling.
    - **Libraries:** Use `thiserror` for structured, matchable error enums.
- **The Newtype Pattern:** Encode domain types (e.g., `struct AgentId(String)`) to make illegal states unrepresentable.

## Ⅲ. Concurrency & Async (Tokio)
- **Async-First:** Use `tokio` for all I/O operations.
- **Non-Blocking Rule:** Never perform blocking I/O (e.g., `std::fs`) inside an async task. Use `tokio::fs` or `spawn_blocking`.
- **Channels over Shared State:** Favor `tokio::sync::mpsc` for inter-agent communication over `Arc<Mutex<T>>` whenever possible.

## Ⅳ. Observability & Logging
- **Structured Tracing:** Use the `tracing` crate. 
- **Pattern:** `info!(agent_id = %id, task_id = %tid, "Action completed")`. Never use string interpolation in log messages.
- **Instrumentation:** Annotate public async functions with `#[instrument(skip(self))]`.

## Ⅴ. Mandatory Documentation (Rustdoc)
- **Strict Requirement:** All public items (structs, enums, traits, functions) MUST have `///` doc comments.
- **Module Headers:** Every file MUST start with a `//!` header describing its purpose and architecture.
- **Doc Structure:**
    1. **Summary:** A single, clear sentence.
    2. **Description:** Deeper technical context (optional).
    3. **`# Examples`:** Strongly recommended; examples are executed as tests.
    4. **`# Errors`:** MANDATORY for functions returning `Result`.
    5. **`# Panics` / `# Safety`:** MANDATORY where applicable.
- **Intra-Doc Links:** Always link to types using `[`Type`]`. Raw URLs are prohibited.
- **Lint Flags:** All library crates MUST include at the root:
    ```rust
    #![warn(missing_docs)]
    #![warn(rustdoc::broken_intra_doc_links)]
    ```

## Ⅵ. Mandatory Testing (The Tier System)
- **In-File Unit Tests:** Every source file MUST contain a `#[cfg(test)] mod tests` module.
- **The Testing Tiers (Enforced):**
    - **Tier 1 (Doc Tests):** All public API examples must compile and run.
    - **Tier 2 (Unit Tests):** All non-trivial logic and private edge cases must be covered.
    - **Tier 3 (Snapshot Tests):** Use the `insta` crate for all serializable types and complex outputs.
    - **Tier 4 (Property Tests):** Use `proptest` for parsers, encoders, and complex input domains.
- **Naming Convention:** Use spec-style `snake_case` (e.g., `test_rejects_empty_agent_id`).
- **Async Testing:** Use `#[tokio::test]` for all async logic.

## Ⅶ. Quality Gates
- **Clippy:** All code MUST pass `cargo clippy -- -D warnings`.
- **Formatting:** Standard `cargo fmt` is mandatory.
- **Audit:** Use `cargo audit` to check for security vulnerabilities in dependencies.

---
*Derived from Noti's Research (2026-03-13) | Approved by Tyr, Captain.*
