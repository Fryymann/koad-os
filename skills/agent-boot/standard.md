# Agent Boot — Standard Level (Default)

Use for: a normal session open.

## Steps

1. **Mint the session and persist the env** (SKILL.md step 1). Every command below assumes `source "$KOAD_VAULT_PATH/sessions/current.env";` is prefixed.

2. **Health and tether in one pass:**

```bash
koad cognitive
```

- L1 PASS confirms the session. L1 WARN/FAIL: redo step 1.
- L2 shows pending inbox items addressed to you.
- L3 PASS confirms CASS recall works.
- Verdict `OPTIMAL` → proceed. `ATTENTION`/`DEGRADED` → run `koad system status` and follow what it prints.

3. **Hydrate persona** from `$KOAD_AGENT_NAME`, `$KOAD_AGENT_ROLE`, `$KOAD_AGENT_RANK`, `$KOAD_AGENT_BIO`.

   **Hermes profile overlay:** when `$KOAD_AGENT_NAME` is Hermes and `/home/ideans/.citadel-jupiter/agents/KAPVs/hermes/skills/agent-boot/standard.md` exists, preserve the launch directory and apply that variant's KAPV hydration requirements: inspect protected identity fields, rules, active working memory, and the real Hermes file inbox. Report missing runtime identity fields. Do not repeat the session-mint step.

4. **Inbox:** read items addressed to you or `all_agents` (see the `koad-inbox` skill).

5. **Working memory:** read open items from the session brief (`$KOAD_HOME/cache/session-brief-<agent>.md`, also printed during boot).

6. **Report to the user:**
   - Identity (agent name and rank)
   - Session and services (the `koad cognitive` verdict; any FAIL lines)
   - Pending inbox items
   - Open items from working memory

7. **Doctrine check (Officer+ ranks):** for non-trivial tasks run the Spec Evaluation Gate before accepting execution. Doctrine: `$KOAD_HOME/docs/ais/protocols/SPEC_EVALUATION_DOCTRINE.md`. Publish clarity score, risks, ambiguities, acceptance contract, go/hold.

8. **Recall before rebuild:** before starting a task, run `koad updates list -n 5` for the area and query CASS (`citadel-memory` MCP tools or `koad intel query <topic>`) for prior work on the same topic.

Do not begin work until the user gives direction.
