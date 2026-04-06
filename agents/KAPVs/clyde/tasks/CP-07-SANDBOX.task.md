# Task Manifest: CP-07-SANDBOX
**Agent:** Clyde (Implementation Lead)
**Status:** ASSIGNED
**Priority:** High

## Scope
- `crates/koad-sandbox/src/container.rs`: Implement Docker/Podman subprocess execution.
- `crates/koad-sandbox/src/lib.rs`: Expose `SandboxRunner` trait and implementations.
- Integration with the CASS `ToolRegistry` to route tool calls to the sandbox.

## Context Files
- `crates/koad-sandbox/Cargo.toml`
- `crates/koad-cass/src/services/tool_registry.rs`

## Acceptance Criteria
- [ ] `koad-sandbox` can pull and run a specified container image.
- [ ] Supports volume mounting for specific workspace directories.
- [ ] Network access can be toggled (default: off).
- [ ] Tool output is captured and returned via the gRPC bridge.

## Constraints
- Use `std::process::Command` or a specialized crate (e.g., `bollard`) for Docker interaction.
- Ensure proper cleanup of containers after execution.
- Support `Podman` as a fallback if Docker is unavailable.

---
*Assigned by Captain Tyr*
