# Domain: Rust Crates (crates/)
**Role:** Backend Infrastructure & Logic

## Ⅰ. Active Crates
- [koad-core/](koad-core/): Shared utilities, logging, and common types.
- [koad-proto/](koad-proto/): Canonical gRPC service definitions.
- [koad-citadel/](koad-citadel/): (Phase 1) The OS Kernel, sessions, and bays.
- [koad-cass/](koad-cass/): (Phase 2) The Agent Support System, memory, and cognitive tools.

## Ⅱ. Guidelines
- Follow [RUST_CANON.md](../docs/protocols/RUST_CANON.md).
- No legacy Spine code allowed in new crates.
- Use `tonic` for gRPC and `tokio` for async.
