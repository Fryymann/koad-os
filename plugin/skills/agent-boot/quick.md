# Agent Boot — Quick Level

Use for: mid-session re-hydration, scoped subagent spawn, CI context.

## Steps

1. Mint session + persist env:

```bash
SESSFILE="$KOAD_VAULT_PATH/sessions/current.env"
"$KOAD_BIN/koad-agent" boot "$KOAD_AGENT_NAME" 2>/dev/null | grep -E '^export ' | sed 's/;$//' > "$SESSFILE"
chmod 600 "$SESSFILE"
grep SESSION_ID "$SESSFILE"
```

2. Confirm a real session ID printed — `SID-<agent>-<hash>`. A `local-fallback-<uuid>` ID means no Citadel tether; re-run step 1.

3. Stop. Await user direction — do not orient or summarize.
