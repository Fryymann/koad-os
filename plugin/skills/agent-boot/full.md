# Agent Boot — Full Level

Use for: start of a new major session, post-incident recovery, inter-agent handoff.

## Steps

1. Run every step of `standard.md` (mint + persist env, verify tether, persona, map look, system status, session brief, SEG, recall-before-rebuild).

All commands below assume `source "$KOAD_VAULT_PATH/sessions/current.env";` is prefixed.

2. Read open tasks from the agent vault:

```bash
ls "$KOAD_VAULT_PATH/tasks/"
head -40 "$KOAD_VAULT_PATH"/tasks/*.md 2>/dev/null || echo "No open task files."
```

Task subdirectories (`ls -d "$KOAD_VAULT_PATH"/tasks/*/`) hold multi-part workstreams — list them, read only what the mission needs.

3. Pull recent fleet activity:

```bash
koad updates list -n 5
```

4. Assert Condition Green:
   - Redis, Citadel control plane, and SQLite memory bank must all be **[PASS]** in `koad system status`
   - CASS must answer — verify with the `citadel-memory` MCP `status_citadel` tool or `koad intel query <topic>`
   - Session tether must be verified (step 1) — `koad whoami` showing `[NOT_TETHERED]` is a known false alarm and does not break Condition Green
   - Flag any OFFLINE service explicitly. Do not start implementation work with degraded services unless Dood approves.

5. Deliver full situational report to Dood:
   - Identity confirmed (name + rank)
   - Service state (GREEN / DEGRADED — list OFFLINE services)
   - Session ID and tether status
   - Pending signals
   - Open tasks (titles)
   - Blockers preventing Condition Green
   - Ready for orders
