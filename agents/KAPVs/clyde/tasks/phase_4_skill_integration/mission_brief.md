# Mission Brief: Phase 4 — MCP Tool Registry & CLI Integration
**Status:** ✅ COMPLETE
**Lead:** Clyde (Officer)
**Objective:** Connect the `koad bridge skill` subcommands to the CASS ToolRegistryService.

## Ⅰ. Strategic Roadmap (SDD)
- [x] **Phase 1: CLI & gRPC Wiring** (clyde-dev) — Update `cli.rs` and `bridge.rs` for Skill management.
- [x] **Phase 2: Integration Testing** (clyde-qa) — Build the `hello-plugin` and verify all subcommands.
- [x] **Phase 3: Review & Audit** (clyde-qa) — Perform ACR pass on modified files.


## Ⅱ. Team Task Packets

### **clyde-dev (CLI Implementation)**
- **Task ID:** `clyde-20260403-dev-02`
- **Objective:** Implement the gRPC logic for the `koad bridge skill` subcommands.
- **Requirements:** 
    - Update `SkillAction` enum in `crates/koad-cli/src/cli.rs`.
    - Implement the logic in `crates/koad-cli/src/handlers/bridge.rs` using the `ToolRegistryServiceClient`.
    - Handle all four actions: `List`, `Register`, `Deregister`, `Run`.

### **clyde-qa (Verification & Review)**
- **Task ID:** `clyde-20260403-qa-02`
- **Objective:** Verify functionality and ensure `RUST_CANON` compliance.
- **Requirements:** 
    - Build `crates/koad-plugins/examples/hello-plugin` (WASM).
    - Perform end-to-end test of registration, listing, and invocation.
    - Run `koad review` on modified files.

## Ⅲ. Governance & Reporting
- Record progress in `TEAM-LOG.md`.
- Escalate blockers to `ESCALATIONS.md`.

---
*Authorized by Tyr (Captain) | 2026-04-03*
