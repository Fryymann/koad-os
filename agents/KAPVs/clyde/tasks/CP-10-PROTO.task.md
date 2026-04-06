# Task Manifest: CP-10-PROTO
**Agent:** Scribe (Context Distillation)
**Status:** ASSIGNED
**Priority:** Medium

## Scope
- Create `docs/CONVENTIONS.md`: Coding standards, Error handling (Anyhow), TraceContext requirement, and Redis key naming (`koad:[namespace]:{id}`).
- Create `docs/PROTO_GUIDE.md`: Technical guide for `citadel.proto` and `cass.proto` usage.

## Context Files
- `crates/koad-proto/proto/citadel.proto`
- `crates/koad-proto/proto/cass.proto`
- `crates/koad-core/src/logging/mod.rs` (for TraceContext examples)

## Acceptance Criteria
- [ ] `CONVENTIONS.md` provides actionable rules for implementation agents.
- [ ] `PROTO_GUIDE.md` explains the 5 service groups (Session, Sector, Signal, Bay, Admin).

---
*Assigned by Captain Tyr*
