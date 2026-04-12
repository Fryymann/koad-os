# Domain: Rust Workspace (crates/)
**Role:** Backend Infrastructure & Agent Logic

## Ⅰ. Workspace Crate Index

| Crate | Status | Purpose |
| :--- | :--- | :--- |
| `koad-core` | Complete | Shared primitives, configuration, session management, and logging. |
| `koad-proto` | Complete | gRPC bindings (tonic), auto-generated from `proto/`. |
| `koad-citadel` | Complete | Citadel gRPC service (:50051) — sessions, bays, signal corps, auth, and state. |
| `koad-cass` | Complete | CASS gRPC service (:50052) — memory, TCH, and EoW pipeline. |
| `koad-cli` | Complete | `koad` and `koad-agent` binaries, all CLI subcommands. |
| `koad-intelligence` | Complete | AI inference routing and local model distillation. |
| `koad-codegraph` | Complete | AST-based symbol indexing using `tree-sitter`. |
| `koad-board` | Complete | Updates board service and event tracking. |
| `koad-bridge-notion` | Complete | Notion MCP bridge (Noti remote agent integration). |
| `koad-sandbox` | Complete | Config-driven security jailing; containerized execution active. |
| `koad-plugins` | Complete | WASM plugin runtime (wasmtime); dynamic loading active. |

## Ⅱ. Engineering Guidelines

- **Standard:** Follow [RUST_CANON.md](../docs/protocols/RUST_CANON.md) for all logic and I/O.
- **Purity:** No legacy terminology (Spine, ASM) allowed in active crates.
- **Async:** Use `tokio` for the runtime and `tonic` for all gRPC communication.
- **Diagnostics:** Every major operation must emit tracing events.
- **Documentation:** Every crate must contain an `AGENTS.md` file describing its API surface.
