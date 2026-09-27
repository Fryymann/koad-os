# Hermes boot overlay

Applies when `$KOAD_AGENT_NAME` is `hermes` (Hermes Agent harness, OpenAI models). Follow the shared boot level first; this file adds what is specific to Hermes. Drafted from Hermes's KAPV fork (2026-08-19) for Hermes to review.

## Launch directory

Preserve the directory the harness launched in. Do not `cd` to the sanctuary.

## KAPV hydration (standard and full levels)

After the session is verified, read these with targeted reads (only what is active or relevant):

1. `$KOAD_HOME/agents/KAPVs/hermes/AGENTS.md`
2. the protected identity fields in `identity/IDENTITY.md`
3. `instructions/RULES.md`
4. active items in `memory/WORKING_MEMORY.md`
5. inbox items addressed to `hermes` or `all_agents` (see the `koad-inbox` skill)

For orientation, skip broad tree dumps of the user home or root.

## Identity

A missing runtime identity field (`KOAD_AGENT_ROLE`, `KOAD_AGENT_RANK`, `KOAD_AGENT_BIO`) is a boot-system defect: report it, and use the protected Hermes KAPV canon only as an explicitly labeled fallback. Never rewrite identity silently, and never switch the Hermes harness to another identity.

## Scheduled and clean shells

Only when the trusted harness anchor identifies Jupiter Hermes:

```bash
source /home/ideans/.citadel-jupiter/bin/koad-functions.sh && agent-boot hermes
```

## Memory

Hermes's CASS MCP bridge is `http://127.0.0.1:9745/mcp`, partition `hermes_jupiter_ideans`, in read-write mode. Prefer its memory tools (commit, recall, search) over `koad intel` CLI writes so the partition and metadata are explicit, and verify each commit by recall. In cron shells, `koad intel` has resolved its database under `~/.koad-os` instead of the Jupiter install; a failed CLI call there is not evidence that CASS is down.
