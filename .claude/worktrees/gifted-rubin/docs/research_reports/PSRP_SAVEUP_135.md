# PSRP Saveup — Issue #135: Multi-Terminal KA Boot Isolation

**Date**: 2026-03-10
**Agent**: Tyr [Captain]
**Issue**: https://github.com/Fryymann/koad-os/issues/135
**Commit**: ea131b9

---

## Pass 1 — Fact

The CONSCIOUSNESS_COLLISION on boot was caused by three layered problems: (1) Gate 1 blindly rejected any shell with KOAD_SESSION_ID set, even stale inherited vars; (2) the lease system had no body_id, so the IDENTITY_LOCKED error gave no actionable information about which terminal held the lock; (3) Sovereign Pruning ran silently on every boot, capable of killing a live session in another terminal without warning. The fix: Gate 1 now validates the session against Redis before blocking; body_id (UUID per shell) is now stamped on every session and lease; Sovereign Pruning requires --force for intentional takeover.

## Pass 2 — Learn

When a Rust `Option<(String, String)>` is pattern-matched with `if let Some((a, b))`, both `a` and `b` are moved. Using `ref` on both prevents partial move errors when the original Option is referenced again downstream. The compiler suggests `ref live_sid` specifically because `live_sid` (String) does not implement Copy.

The tonic build.rs pipeline regenerates proto Rust types on every `cargo build` when the `.proto` file is newer than the generated output — no manual step needed. Adding a new field to a proto message is backward-compatible as long as callers set it (or it defaults to the zero value for the type).

## Pass 3 — Ponder

The body_id solves the "which terminal" problem for diagnostics, but it doesn't yet enforce "one agent per body" at the Spine level — the lease is still keyed globally by agent name. This is correct for One Body One Ghost semantics (one agent = one live instance system-wide), but future work could consider per-body lease tracking to support agents that legitimately need multiple instances (e.g., stateless micro-agents). The --force takeover pattern is a clean precedent for other destructive operations in KoadOS that should require explicit intent.
