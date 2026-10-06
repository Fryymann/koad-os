# Agent Boot — Full Level

Use for: the start of a major session, post-incident recovery, inter-agent handoff.

## Steps

1. Run every step of `standard.md`. All commands below assume `source "<SESSFILE>";` is prefixed, using the absolute path SKILL.md step 1 printed.

2. Read open tasks from the agent vault:

```bash
ls "$KOAD_VAULT_PATH/tasks/"
head -40 "$KOAD_VAULT_PATH"/tasks/*.md 2>/dev/null || echo "No open task files."
```

Task subdirectories hold multi-part workstreams: list them and read only what the mission needs.

3. Recent KoadOS changes:

```bash
koad updates list -n 5 -l citadel
```

4. Assert Condition Green:
   - `koad system status`: Redis, Citadel, CASS and SQLite all **[PASS]**
   - `koad cognitive` verdict `OPTIMAL`
   - Flag anything failing explicitly. Do not start implementation work on degraded services unless Dood approves.

5. Deliver the full situational report to Dood:
   - Identity (name and rank)
   - Service state (GREEN / DEGRADED, listing failures)
   - Session ID
   - Pending inbox items
   - Open tasks (titles)
   - Blockers to Condition Green
   - Ready for orders
