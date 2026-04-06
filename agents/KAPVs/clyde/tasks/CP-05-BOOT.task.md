# Task Manifest: CP-05-BOOT
**Agent:** Clyde (Implementation Lead)
**Status:** ASSIGNED
**Priority:** Medium

## Scope
- `crates/koad-cli/src/commands/agent/boot.rs`: Implement `koad-agent boot <identity>`.
- `koad-core/src/config/identities/`: Schema for identity TOML files.
- Environment variable generation for shell hydration (KOAD_AGENT, KOAD_RUNTIME, etc.).
- Runtime-specific config generation (e.g., `.claude/settings.json`).

## Context Files
- `crates/koad-cli/src/handlers/boot.rs` (current session-based boot for comparison)
- `koad-core/src/config/mod.rs` (KoadConfig and identity paths)

## Acceptance Criteria
- [ ] `koad-agent boot tyr` outputs `export` commands for shell hydration.
- [ ] Load `identity.toml` from `~/.koad-os/config/identities/`.
- [ ] Generates shell exports for: `KOAD_AGENT_NAME`, `KOAD_AGENT_ROLE`, `KOAD_RUNTIME`, `KOAD_VAULT_URI`.
- [ ] Pattern `eval $(koad-agent boot tyr)` works in a clean shell.

## Constraints
- Do NOT modify the `koad logout` / `koad session` logic (Citadel-dependent) in this task.
- Ensure cross-platform bash compatibility for exports.

---
*Assigned by Captain Tyr*
