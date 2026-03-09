# KoadOS Kernel Development Context

This is the project-level context for KoadOS internal development.

## Tech Stack & Architecture
- **Language**: Rust (workspace with multiple crates).
- **Core Crates**: `koad-core`, `koad-spine`, `koad-asm`, `koad-gateway`.
- **Primary Data Plane**: Redis (Unix Domain Socket at `~/.koad-os/koad.sock`).
- **Secondary Data Plane**: SQLite (`~/.koad-os/koad.db`).

## Kernel Engineering Rules
- All daemon components (Spine, ASM) MUST implement graceful shutdown capturing `SIGTERM` and `SIGINT`.
- Do not use raw `println!` for daemon output. Use the `tracing` crate via `koad_core::logging`.
- Always check for the presence of the Redis Unix socket before starting the Spine.