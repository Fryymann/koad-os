---
name: agent-boot
description: Use when starting a KoadOS agent session, re-hydrating mid-session, or booting a named agent for the first time. Accepts an agent name and optional level flag (--quick, --full). Default level is standard.
license: MIT
compatibility: Requires a KoadOS Citadel install (koad CLI, $KOAD_HOME, running Citadel and CASS services).
metadata:
  author: koados
  version: "2.0.0"
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

The session environment variables (`KOAD_AGENT_NAME`, `KOAD_AGENT_ROLE`, `KOAD_AGENT_RANK`, `KOAD_AGENT_BIO`) are the source of truth. Establish persona from them.

- **Never** run `agent-prep` (or `--agentprep`), and never edit these variables to change identity.
- Boot without a name argument; rely on `$KOAD_AGENT_NAME`.
- Do not hand-edit the generated identity anchor (`~/.claude/CLAUDE.md`, `~/.codex/AGENTS.md` or `~/.gemini/GEMINI.md`, depending on runtime). Boot regenerates it. Boot never writes to a project's own instruction files.

## Env does not survive between tool calls

Under a harness that runs each shell command in a fresh shell (Claude Code, Hermes Agent), the exports from boot die when the call returns, and later `koad` calls fail with `Unauthenticated: Missing x-session-id header`. Boot must persist the session env to a file, and every later call must source it.

## How to Execute

**Step 1 — mint the session and persist the env:**

```bash
# Fresh harness shells don't inherit KOAD_RUNTIME. Detect it like koad-functions.sh; never force it.
if [ -z "$KOAD_RUNTIME" ]; then
  if [ -n "$CLAUDE_CODE_ENTRYPOINT" ]; then export KOAD_RUNTIME=claude
  elif [ -n "$GEMINI_API_KEY$GOOGLE_GEMINI_API_KEY$ANTIGRAVITY_AGENT" ]; then export KOAD_RUNTIME=gemini
  fi
fi
SESSFILE="$KOAD_VAULT_PATH/sessions/current.env"
"$KOAD_BIN/koad-agent" boot "$KOAD_AGENT_NAME" 2>/dev/null | grep -E '^export ' | sed 's/;$//' > "$SESSFILE"
chmod 600 "$SESSFILE"
grep -c KOAD_SESSION_TOKEN "$SESSFILE"
```

The count must be `1`. `0` means no session was minted: run the same boot command without `2>/dev/null` and read stderr. `[OFFLINE] KoadOS Citadel is not reachable` means the Citadel is down (see the `koad-system` skill). The `[QUICK-RESTORE]` banner is only a cached brief and proves nothing.

**Step 2 — verify the tether:**

```bash
source "$KOAD_VAULT_PATH/sessions/current.env"; koad system heartbeat
```

`[OK] Heartbeat transmitted` means the Citadel accepted the session. Anything else: re-run step 1.

**Step 3 — prefix every later koad call:**

```bash
source "$KOAD_VAULT_PATH/sessions/current.env"; koad <command>
```

Any authenticated call keeps the session alive. After about 5 minutes without one, the session is purged; when a call fails with `Session not found or expired`, repeat step 1.

In an interactive terminal the classic form works directly:

```bash
source "$KOAD_HOME/bin/koad-functions.sh" && agent-boot
```

**Clean/scheduled environments:** cron and other non-interactive shells may have empty `KOAD_HOME` and `KOAD_AGENT_NAME`. Never expand an empty `KOAD_HOME` into `/bin/koad-functions.sh`. If a trusted identity anchor provides the Citadel home and identity, source that absolute `koad-functions.sh` and boot that same identity explicitly (for example Jupiter Hermes: `source /home/ideans/.citadel-jupiter/bin/koad-functions.sh && agent-boot hermes`). Never infer or switch identity from an unrelated repository anchor. If no trusted identity is available, stop rather than guess.

## Known Traps (verified 2026-09-26)

| Trap | Reality |
|---|---|
| `koad boot -a <name>` | Legacy path. Rejects capitalized names and refuses to run while any session is active in the body. Use `koad-agent boot` (step 1). |
| Level flag rejected (`--quick`, `--full`) | Some deployed wrappers pass straight through to `koad-agent boot`, which rejects them. Drop the flag and follow the level file manually; never swap the agent name to work around it. |
| `[BOOT DENIED] No agent body detected` | `KOAD_RUNTIME` doesn't match the identity's `runtime`. Step 1 detects Claude Code and Gemini. Any other harness has to export its own runtime. Never set a runtime your harness isn't, because that defeats the body check. |
| Session file has no token | Boot could not reach the Citadel. Do not retry blindly or claim success; check `koad system status`. Use the partition-bound CASS MCP for memory work meanwhile. |

## Boot Levels

- **`--quick`:** boot only. Follow `quick.md`.
- **`standard` (default):** boot and orient. Follow `standard.md`.
- **`--full`:** boot, orient, tasks, Condition Green. Follow `full.md`.

Read the level file and follow it.
