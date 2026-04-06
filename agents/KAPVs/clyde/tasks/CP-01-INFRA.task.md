# Task Manifest: CP-01-INFRA
**Agent:** Clyde (Implementation Lead)
**Status:** ASSIGNED
**Priority:** High

## Scope
- `crates/koad-cass/src/storage/mod.rs`: Add `get_active_pulses` and `add_pulse` traits.
- `crates/koad-cass/src/storage/redis_tier.rs`: Implement Redis-backed pulse storage (L1).
- `crates/koad-cass/src/services/hydration.rs`: Integrate pulses into the TCH packet.
- `proto/cass.proto`: Define `Pulse` message and `GetPulses` gRPC endpoint.

## Context Files
- `crates/koad-cass/src/services/hydration.rs`
- `crates/koad-cass/src/storage/redis_tier.rs`
- `proto/cass.proto`

## Do NOT Read
- `crates/koad-citadel/` (Kernel logic is not required for this task).

## Acceptance Criteria
- [ ] gRPC `GetPulses` returns pulses matching the agent's role or "global".
- [ ] TCH packet includes a "Global Pulses" section.
- [ ] Redis keys follow `koad:pulse:{id}` pattern with TTL.
- [ ] `cargo build` and `cargo test` in `koad-cass` pass.

## Constraints
- Every gRPC method accepts `TraceContext`.
- No `unsafe` code.
- Handle Redis connection failures with graceful degradation.

---
*Assigned by Captain Tyr*
