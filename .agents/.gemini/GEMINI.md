# KoadOS Citadel v3 — Development Context

This is the project-level context for KoadOS internal development on **Jupiter** (Citadel v3 era).

## Tech Stack & Architecture

- **Language**: Rust (workspace, multiple crates).
- **Core Crates**: `koad-core`, `koad-citadel`, `koad-cass`, `koad-plugins`, `koad-cli`, `koad-proto`.
- **Primary Data Plane**: Redis (Unix Domain Socket at `~/.koad-os/koad.sock`).
- **Secondary Data Plane**: SQLite (`~/.koad-os/koad.db` and per-bay `bays/<agent>/state.db`).
- **Service Layer**: gRPC — Citadel on `:50051`, CASS on `:50052`.

## Jupiter Service State (Dark Mode)

Services are built but **not currently running** on Jupiter. All agents operate in dark mode:
- Citadel gRPC (`:50051`): DARK
- CASS gRPC (`:50052`): DARK
- Qdrant (vector search): OFFLINE — requires Docker Desktop WSL integration
- Docker: OFFLINE — requires Docker Desktop WSL backend

Graceful dark-mode degradation is required in all service-dependent code paths.

## Kernel Engineering Rules (RUST_CANON v1.0)

- All source files MUST have a `//!` module-level doc comment.
- All public functions and types MUST have `///` doc comments.
- All source files MUST have a `#[cfg(test)] mod tests {}` block.
- All public `async fn` MUST use `#[instrument]` (tracing crate).
- No `.unwrap()` or `.expect()` in production paths — use `?` with `anyhow::Context`.
- `std::fs` is prohibited in async code — use `tokio::fs`.
- All daemon components MUST implement graceful shutdown capturing `SIGTERM` and `SIGINT`.
- Do not use raw `println!` for daemon output — use the `tracing` crate via `koad_core::logging`.
- All public enums MUST derive `Debug`.

## Agent Identity System

- **KAI (KoadOS Agent Identity)**: tri-layer — `config/identities/<key>.toml` + KAPV vault + crew docs.
- **KAPV (KoadOS Agent Personal Vault)**: at `.agents/<key>/` — standard dirs: bank/, config/, identity/, instructions/, memory/, reports/, sessions/, skills/, tasks/, templates/.
- **Booting**: `eval $(KOAD_RUNTIME=<runtime> koad-agent boot <name>)`
- **Identity TOMLs**: `~/.koad-os/config/identities/` — scanned by Citadel on startup for bay auto-provisioning.
