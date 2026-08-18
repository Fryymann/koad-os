# Agent Boot — Standard Level (Default)

Use for: normal session open.

## Steps

1. **Mint session + persist env** (see SKILL.md — env does not survive between tool calls):

```bash
SESSFILE="$KOAD_VAULT_PATH/sessions/current.env"
"$KOAD_BIN/koad-agent" boot "$KOAD_AGENT_NAME" 2>/dev/null | grep -E '^export ' | sed 's/;$//' > "$SESSFILE"
chmod 600 "$SESSFILE"
```

Every command below assumes `source "$KOAD_VAULT_PATH/sessions/current.env";` is prefixed.

2. **Verify tether:**

```bash
koad signal list
```

`Unauthenticated: Missing x-session-id header` → step 1 failed, re-run it before continuing. Any other output (including `No pending signals`) is a pass, and doubles as the inbox check.

3. **Hydrate persona** from `$KOAD_AGENT_NAME`, `$KOAD_AGENT_ROLE`, `$KOAD_AGENT_RANK`, `$KOAD_AGENT_BIO`. These are the source of truth. Do not hand-edit the generated `CLAUDE.md` / `GEMINI.md` / `AGENTS.md` anchors — boot regenerates them.

4. Situational awareness:

```bash
koad map look
```

5. Service health:

```bash
koad system status
```

- All **[PASS]** → proceed.
- Any **[FAIL]** / **[WARN]** → `koad doctor -f` to self-heal.
- Still broken → `koad system start`.
- Ignore `koad whoami` reporting `[NOT_TETHERED]`; it is a known false alarm (see SKILL.md traps).

6. Read working memory open items from the session brief (`$KOAD_HOME/cache/session-brief-<agent>.md`, also printed during boot).

7. Report to user:
   - Identity confirmed (agent name + rank)
   - Service state (Redis / Citadel / SQLite / CASS — PASS or OFFLINE)
   - Pending signals, if any
   - Open items surfaced from working memory

8. Doctrine check (Officer+ ranks): for non-trivial tasks run the Spec Evaluation Gate before accepting execution. Doctrine: `$KOAD_HOME/docs/ais/protocols/SPEC_EVALUATION_DOCTRINE.md`. Publish clarity score, risks, ambiguities, acceptance contract, go/hold.

9. **Recall before rebuild:** before starting any task, run `koad updates list -n 5` and query CASS (`koad intel query <topic>` or the `citadel-memory` MCP tools) for prior work on the same topic.

Do not begin work until the user gives direction.
