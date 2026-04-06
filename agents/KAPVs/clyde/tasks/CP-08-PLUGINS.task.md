# Task Manifest: CP-08-PLUGINS
**Agent:** Clyde (Implementation Lead)
**Status:** ASSIGNED
**Priority:** Medium

## Scope
- `crates/koad-plugins/src/registry.rs`: Finalize the plugin registry.
- Support for WASM-based plugins (using `wasmtime`).
- Dynamic library loading for `.so`/`.dll` plugin components.

## Context Files
- `crates/koad-plugins/Cargo.toml`
- `crates/koad-plugins/src/lib.rs`

## Acceptance Criteria
- [ ] Plugins can be registered with specific permissions (read/write/net).
- [ ] The registry supports hot-reloading (detecting changes on disk).
- [ ] Integration with `ToolRegistryServiceClient` to expose plugins via gRPC.

## Constraints
- WASM plugins must adhere to the KoadOS tool interface.
- No memory leaks during plugin unloading.
- Secure by default: no ambient access to the host filesystem.

---
*Assigned by Captain Tyr*
