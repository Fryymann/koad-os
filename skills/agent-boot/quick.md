# Agent Boot — Quick Level

Use for: mid-session re-hydration, scoped subagent spawn, CI context.

## Steps

1. Mint the session and persist the env (SKILL.md step 1). The token count must be `1`.

2. Verify the tether:

```bash
source "$KOAD_VAULT_PATH/sessions/current.env"; koad system heartbeat
```

3. Stop. Await user direction; do not orient or summarize.
