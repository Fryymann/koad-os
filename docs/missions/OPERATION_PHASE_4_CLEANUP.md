# Mission Brief: Operation Phase 4 Cleanup
**Status:** ACTIVE
**Lead:** Clyde (Implementation Lead)
**Captain:** Tyr (Strategic Oversight)

## 1. Objective
To finalize the remaining "Security & Isolation" deliverables for Phase 4. This mission focuses on moving from mock/local execution to full containerized sandboxing and stable dynamic plugin loading.

## 2. Background
While the MCP registry and gRPC bridge are live, agents still run tools directly in the host environment in some cases. Phase 4 requires strict environment isolation to prevent workspace corruption and enable secure multi-agent execution.

## 3. Scope
- **koad-sandbox:** Implement full Docker/Podman subprocess execution with volume mounting and network isolation.
- **koad-plugins:** Finalize the WASM component model registry and dynamic library loading (`.so`/`.dll` support).

## 4. Key Deliverables
- [ ] `koad sandbox run` command for isolated tool execution.
- [ ] Integration of the sandbox into the CASS tool registry.
- [ ] Stable WASM plugin registry with lifecycle management (load/unload).

## 5. Success Criteria
- [ ] A tool can be registered in CASS and executed inside a Docker container via a gRPC call.
- [ ] WASM plugins can be dynamically loaded without restarting CASS.
- [ ] `cargo build` and `cargo test` pass across the entire workspace.

---
*Signed,*
**Captain Tyr**
*Citadel Jupiter*
