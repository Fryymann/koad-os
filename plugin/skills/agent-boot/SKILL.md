---
name: agent-boot
description: Use when starting a KoadOS agent session, re-hydrating mid-session, or booting a named agent for the first time. Accepts an agent name and optional level flag (--quick, --full). Default level is standard.
---

# Agent Boot

Boots a KoadOS agent: mints a Citadel session, hydrates identity, persists the session env, and orients.

## Usage

```bash
agent-boot                  # use current $KOAD_AGENT_NAME (recommended)
agent-boot <name>           # override with specific name
agent-boot [name] --quick   # boot only, no orientation
agent-boot [name] --full    # boot + orient + tasks + Condition Green
```

## Identity Rules

The session environment variables (`KOAD_AGENT_NAME`, `KOAD_AGENT_ROLE`, `KOAD_AGENT_RANK`, `KOAD_AGENT_BIO`) are the absolute source of truth. Establish persona from them.

- **NEVER** run `agent-prep` (or `--agentprep`), and never edit these variables to change identity.
- Always boot without a name argument — rely on `$KOAD_AGENT_NAME`.
- Do **not** hand-edit `CLAUDE.md` / `GEMINI.md` / `AGENTS.md` identity anchors. `koad-agent boot` regenerates all three (`.claude/CLAUDE.md`, `.gemini/GEMINI.md`, `.codex/AGENTS.md`) on every boot. Hand edits are overwritten.

## CRITICAL: Env Does Not Survive Between Tool Calls

Under Claude Code (and any harness that runs each Bash invocation in a fresh shell), `agent-boot` hydrates a shell that **dies when the call returns**. The `KOAD_SESSION_ID` / `KOAD_SESSION_TOKEN` it exports are lost, and every later `koad` call fails with:

```
Error: status: Unauthenticated, message: "Missing x-session-id header"
```

Boot MUST therefore persist the session env to a file and every later call MUST source it.

## How to Execute

**Step 1 — mint session and persist env:**

```bash
SESSFILE="$KOAD_VAULT_PATH/sessions/current.env"
"$KOAD_BIN/koad-agent" boot "$KOAD_AGENT_NAME" 2>/dev/null | grep -E '^export ' | sed 's/;$//' > "$SESSFILE"
chmod 600 "$SESSFILE"
grep -E 'SESSION_ID|AGENT_NAME' "$SESSFILE"
```

`koad-agent boot` normalizes the agent name, so `Clyde` and `clyde` both work here.

**Step 2 — verify the tether (do not skip):**

```bash
source "$KOAD_VAULT_PATH/sessions/current.env"; koad signal list
```

Any answer other than `Unauthenticated` means the session is live. `Unauthenticated` means step 1 failed — re-run it, do not proceed.

**Step 3 — prefix every later koad/CASS call:**

```bash
source "$KOAD_VAULT_PATH/sessions/current.env"; koad <command>
```

In an interactive terminal (not a harness) the classic form still works and needs none of the above:

```bash
source "$KOAD_HOME/bin/koad-functions.sh" && agent-boot
```

**Clean/scheduled environment exception:** cron and other non-interactive shells may have empty `KOAD_HOME` and `KOAD_AGENT_NAME`. Never expand an empty `KOAD_HOME` into `/bin/koad-functions.sh`. If the harness/project's trusted identity anchor explicitly provides the Citadel home and current identity, source that absolute `koad-functions.sh` and boot that same identity explicitly (e.g. Jupiter Hermes: `source /home/ideans/.citadel-jupiter/bin/koad-functions.sh && agent-boot hermes`). Never infer or switch identity from an unrelated repository anchor. If no trusted identity is available, stop rather than guessing.

## Known Traps (verified 2026-08-18)

| Trap | Reality |
|---|---|
| `koad boot -a <name> --export-env` | Does **not** mint a real session. Returns `local-fallback-<uuid>` with no token, plus `Session token is invalid or missing`. Use `koad-agent boot` instead. |
| `koad boot -a Clyde` | Rejects capitalized names: `Agent 'Clyde' is not a registered Sovereign KAI`. `koad-agent boot Clyde` accepts it. |
| `koad whoami` says `[NOT_TETHERED]` | Session lease (Redis hash `koad:state`, field `koad:session:<SID>`) has a 90s TTL and `koad system heartbeat` returns `[OK]` **without extending `expires_at`** — a server-side bug in the Citadel heartbeat handler. Read paths (signals, intel query, CASS recall) keep working past expiry because they validate the token only. **Writes do not** — `koad intel remember` fails with `Commit failed / Unauthenticated: Session not found or expired`. Re-run step 1 to re-mint before any memory write in a long session. |
| `koad signal inbox` | No such subcommand. Use `koad signal list`. |
| `koad signal send` / `list` | **The Citadel signal service is a stub** (`crates/koad-citadel/src/services/signal.rs:121-142`): `send_signal` discards the payload and returns `Signal sent (stub)`, which the CLI reports as `Signal dispatched to <agent>.`; `get_signals` always returns an empty vec. Nothing is ever delivered or received. Use `$KOAD_HOME/agents/inbox/<slug>.<type>.<agent>.md` files or a CASS fact card for agent-to-agent handoff. |
| `agent-boot <name> --quick` | Some deployed wrappers delegate straight to `koad-agent boot` and reject level flags as an unexpected argument. If a flag is rejected, drop it and run the level procedure manually — never swap the agent name to work around it. |
| Boot prints `[QUICK-RESTORE]` only | Normal — the export lines are consumed by `eval`, not printed. Confirm the session by checking `$SESSFILE`, not by reading boot output. |

## Boot Levels

- **`--quick`:** Boot only. Follow `quick.md`.
- **`standard` (default):** Boot + orient. Follow `standard.md`.
- **`--full`:** Boot + orient + tasks + Condition Green. Follow `full.md`.

Read the appropriate level file and follow it exactly.
