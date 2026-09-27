---
name: koad-signal
description: Use when sending a message to another KoadOS agent, checking for incoming signals at session start, or coordinating handoffs between agents asynchronously.
---

# koad signal

Asynchronous agent-to-agent messaging (A2A-S). Use for cross-agent coordination without requiring both agents to be online simultaneously.

## Commands

### Send a signal
```bash
koad signal send <target> --message "<content>"
koad signal send clyde --message "CASS migration complete. Ready for Phase 4."
koad signal send tyr --message "Rook stack is live on skylinks." --priority high
```

Priorities: `low` | `standard` (default) | `high` | `critical`

Target is the agent name (lowercase): `clyde`, `tyr`, `rook`, etc.

### Check incoming signals
```bash
koad signal list          # pending signals only
koad signal list --all    # all including read and archived
```

Run at session start after `agent-boot` to check for pending messages.

### Read a signal
```bash
koad signal read <ID>
```

### Archive after acting
```bash
koad signal archive <ID>
```

## When to Use

- Handing off completed work to another agent
- Notifying an agent of a state change they need to act on
- Coordinating phase transitions in multi-agent tasks
- Leaving context for an agent that will pick up work later

## Protocol

1. Send signal with enough context to act without re-deriving
2. Include: what happened, what's next, any blockers
3. Archive signals after acting on them — don't leave inbox cluttered

## Deployed Jupiter limitation (verified 2026-08-18)

The current Citadel Jupiter signal gRPC service is still stubbed in `crates/koad-citadel/src/services/signal.rs`:

- `send_signal` reports `Signal sent (stub)` without persisting delivery;
- `get_signals` returns an empty signal list;
- status updates are also stubbed.

A sender can therefore receive a success-looking response while the recipient sees `No pending signals`. When diagnosing a claimed missing signal:

1. Mint a fresh Citadel session; an expired lease can produce `Unauthenticated`.
2. Run both `koad signal list` and `koad signal list --all`.
3. Inspect Redis stream keys through the configured Redis endpoint; absence of a recipient/signal stream corroborates non-delivery.
4. Check the direct source implementation before assuming the CLI is durable.
5. Use a shared inbox artifact, CASS handoff card, or direct user relay until the service implementation is live. Do not repeatedly resend through the stub.

## Common Mistakes

| Mistake | Fix |
|---|---|
| Vague messages ("done, check it") | Include what was done, where, and what's next |
| Never checking inbox at session start | `koad signal list` is part of the boot sequence |
| Leaving read signals unarchived | Archive after acting to keep list clean |
| Trusting a success-looking send response on stubbed Jupiter | Verify recipient retrieval and use a durable alternate handoff path |
