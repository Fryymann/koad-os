# Task Manifest: CP-04-CONTEXT
**Agent:** Clyde (Implementation Lead)
**Status:** ASSIGNED
**Priority:** High

## Scope
- `crates/koad-cli/src/commands/agent/context.rs`: Implement `koad-agent context <crate>`.
- Integration with `koad-codegraph` to extract public API signatures.
- Integration with `git` to extract the last 5 relevant commit messages for a crate.
- Formatting the output as a high-signal markdown packet (similar to TCH).

## Context Files
- `crates/koad-cli/src/main.rs`
- `crates/koad-codegraph/src/lib.rs`
- `crates/koad-cass/src/services/hydration.rs` (for TCH formatting inspiration)

## Acceptance Criteria
- [ ] `koad-agent context koad-citadel` generates a `.context.md` file.
- [ ] Packet includes: Crate purpose, Public Structs/Enums, Function signatures, and recent Git activity.
- [ ] Output is optimized for minimal token count while maintaining high signal.
- [ ] Works in DEGRADED mode (no CASS or Citadel running).

## Constraints
- Use `tree-sitter` (via `koad-codegraph`) for symbol extraction.
- Every gRPC call (if any) must be optional/best-effort.
- `cargo build` and `cargo test` in `koad-cli` pass.

---
*Assigned by Captain Tyr*
