---
name: koad-cognitive
description: Use when checking whether an agent's continuity systems are healthy - a live Citadel session, hot context, the agent inbox, and CASS memory recall.
license: MIT
compatibility: Requires a KoadOS Citadel install (koad CLI, $KOAD_HOME, running Citadel and CASS services).
metadata:
  author: koados
  version: "2.0.0"
---

# koad cognitive

One command that checks what an agent's continuity depends on, and reports a verdict derived from the results.

```bash
koad cognitive
```

| Check | What it verifies |
|---|---|
| L1 Session | The session in `KOAD_SESSION_ID` is accepted by the Citadel (a heartbeat, which also keeps it alive) |
| L2 Hot context | Redis is reachable; number of hot-context chunks for the session |
| L2 Inbox | Items addressed to this agent in `$KOAD_HOME/agents/inbox/` |
| L3 CASS | A recall query in this agent's partition returns memory |

Verdict: any **FAIL** → `DEGRADED`, any **WARN** → `ATTENTION`, otherwise `OPTIMAL`.

## Acting on results

| Result | Action |
|---|---|
| L1 WARN "No session loaded" / FAIL "rejected" | Run the `agent-boot` skill to mint a session |
| L1 FAIL "Citadel unreachable" | `koad system status`, then the restart command it prints |
| L2 FAIL Redis | `koad doctor -f` |
| L2 inbox items | Read them (`koad-inbox` skill) before starting new work |
| L3 FAIL | `koad system status`; CASS may need a restart |
| L3 WARN no memories | Normal for a new agent; otherwise check the partition name |

`koad system status` covers infrastructure; `koad cognitive` covers this agent's session and memory.
