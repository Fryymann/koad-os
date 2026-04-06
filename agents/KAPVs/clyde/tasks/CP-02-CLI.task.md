# Task Manifest: CP-02-CLI
**Agent:** Clyde / Cid (Implementation/Support)
**Status:** ASSIGNED
**Priority:** Medium

## Scope
- `crates/koad-cli/src/handlers/pulse.rs`: New `pulse` command handler.
- `crates/koad-cli/src/handlers/updates.rs`: Hook into `koad updates create` to trigger pulse.
- `crates/koad-cli/src/main.rs`: Register `pulse` subcommand.

## Context Files
- `crates/koad-cli/src/handlers/updates.rs`
- `crates/koad-cli/src/main.rs`

## Acceptance Criteria
- [ ] `koad pulse "<message>"` successfully calls CASS `AddPulse` gRPC endpoint.
- [ ] `koad updates create` prompts for a pulse message after file creation.
- [ ] Command supports `--role` filtering and `--sync` for MISSION.md.

## Constraints
- Use `anyhow::Result` for error handling.
- Gracefully handle "CASS Offline" scenarios.

---
*Assigned by Captain Tyr*
