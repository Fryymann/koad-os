# Agent Boot Skill & koad-agents Plugin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the `agent-boot` bash function into a versioned `koad-agents` Claude plugin inside the `koad-os` repo, backed by a canonical `agent-boot.sh` script and a three-level skill (quick / standard / full).

**Architecture:** The boot logic moves from `koad-functions.sh` into `plugin/bin/agent-boot.sh` as the canonical source. `koad-functions.sh` becomes a thin shell-function wrapper that sources the plugin script (required so `eval`'d env var exports propagate to the calling shell). `koad-init.sh` deploys `agent-boot.sh` to `$KOAD_HOME/bin/` alongside the other binaries. The skill files in `plugin/skills/agent-boot/` instruct agents how to invoke the boot sequence at three verbosity levels.

**Tech Stack:** Bash, Claude Code plugin format (SKILL.md + package.json), existing `koad-agent` Rust binary (unchanged)

---

## File Map

| Action | Path | Responsibility |
|--------|------|----------------|
| Create | `plugin/package.json` | Claude plugin manifest |
| Create | `plugin/bin/agent-boot.sh` | Canonical boot logic (moved from koad-functions.sh) |
| Modify | `scripts/koad-functions.sh` | Thin wrapper — sources `$KOAD_HOME/bin/agent-boot.sh` |
| Modify | `koad-init.sh` | Deploy `plugin/bin/agent-boot.sh` → `$KOAD_HOME/bin/` |
| Create | `plugin/skills/agent-boot/SKILL.md` | Core skill: frontmatter, argument parsing, level routing |
| Create | `plugin/skills/agent-boot/quick.md` | Level A: boot only |
| Create | `plugin/skills/agent-boot/standard.md` | Level B: boot + orient (default) |
| Create | `plugin/skills/agent-boot/full.md` | Level C: boot + orient + tasks + Condition Green |

---

## Task 1: Create Plugin Scaffold

**Files:**
- Create: `plugin/package.json`
- Create: `plugin/bin/.gitkeep`
- Create: `plugin/skills/agent-boot/.gitkeep`

- [ ] **Step 1: Create the directory structure**

```bash
mkdir -p /home/ideans/koados-citadel/plugin/bin
mkdir -p /home/ideans/koados-citadel/plugin/skills/agent-boot
```

- [ ] **Step 2: Write `plugin/package.json`**

```json
{
  "name": "koad-agents",
  "version": "1.0.0",
  "type": "module"
}
```

Save to: `plugin/package.json`

The version should be bumped in lockstep with the OS release cycle. No `main` entry — this is a pure skill plugin (no JS hook).

- [ ] **Step 3: Verify structure**

```bash
find /home/ideans/koados-citadel/plugin -type f | sort
```

Expected output:
```
/home/ideans/koados-citadel/plugin/package.json
```

- [ ] **Step 4: Commit**

```bash
cd /home/ideans/koados-citadel
git add plugin/package.json
git commit -m "feat(plugin): scaffold koad-agents plugin manifest"
```

---

## Task 2: Extract Boot Logic into `plugin/bin/agent-boot.sh`

**Files:**
- Create: `plugin/bin/agent-boot.sh`

The current `agent-boot` function body lives at lines 25–64 of `scripts/koad-functions.sh`. This task moves that logic into the plugin as the canonical source. The script must be **sourced, not executed** — it uses `local` variables and `eval` that require a live shell function context.

- [ ] **Step 1: Write `plugin/bin/agent-boot.sh`**

```bash
#!/usr/bin/env bash
# agent-boot.sh — Canonical KoadOS agent boot logic.
# MUST be sourced from within a shell function, never executed directly.
# Called by the agent-boot() wrapper in koad-functions.sh.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "[agent-boot] ERROR: This script must be sourced, not executed directly." >&2
    echo "[agent-boot] Usage: source agent-boot.sh <agent-name>" >&2
    exit 1
fi

if [ -z "$1" ]; then
    echo "[agent-boot] Usage: agent-boot <agent-name>"
    return 1
fi

local _AGENT_LOWER
_AGENT_LOWER=$(echo "$1" | tr '[:upper:]' '[:lower:]')
local _KOAD_HOME="$KOAD_HOME"
local _AGENT_TOML="$_KOAD_HOME/config/identities/${_AGENT_LOWER}.toml"
local _BRIEF_CACHE="$_KOAD_HOME/cache/session-brief-${_AGENT_LOWER}.md"

# 1. Fast Display: Show the last known state immediately
if [ -f "$_BRIEF_CACHE" ]; then
    echo -e "\x1b[1;30m[QUICK-RESTORE] Loading last cached brief...\x1b[0m"
    cat "$_BRIEF_CACHE"
    echo -e "\x1b[1;30m-------------------------------------------\x1b[0m"
fi

# 2. Runtime Detection: env signals take priority over TOML config
if [ -z "$KOAD_RUNTIME" ]; then
    if [ -n "$CLAUDE_CODE_ENTRYPOINT" ]; then
        export KOAD_RUNTIME="claude"
    elif [ -n "$GEMINI_API_KEY" ] || [ -n "$GOOGLE_GEMINI_API_KEY" ]; then
        export KOAD_RUNTIME="gemini"
    elif [ -f "$_AGENT_TOML" ]; then
        local _rt
        _rt=$(grep -E "^runtime[[:space:]]*=" "$_AGENT_TOML" | head -n1 | cut -d'"' -f2)
        [ -n "$_rt" ] && export KOAD_RUNTIME="$_rt"
    fi
fi

# 3. WSL GPU/CUDA path fix
if [ -d "/usr/lib/wsl/lib" ]; then
    if [[ ":$LD_LIBRARY_PATH:" != *":/usr/lib/wsl/lib:"* ]]; then
        export LD_LIBRARY_PATH="/usr/lib/wsl/lib${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}"
    fi
fi

# 4. Async Hydration: eval koad-agent boot output to propagate env vars
eval "$("$_KOAD_HOME/bin/koad-agent" boot "$1")"
```

Save to: `plugin/bin/agent-boot.sh`

- [ ] **Step 2: Make it non-executable (sourced scripts should not be chmod +x)**

```bash
chmod -x /home/ideans/koados-citadel/plugin/bin/agent-boot.sh
```

- [ ] **Step 3: Commit**

```bash
cd /home/ideans/koados-citadel
git add plugin/bin/agent-boot.sh
git commit -m "feat(plugin): add canonical agent-boot.sh boot logic"
```

---

## Task 3: Update `koad-functions.sh` to Thin Wrapper

**Files:**
- Modify: `scripts/koad-functions.sh`

Replace the body of the `agent-boot()` function with a single `source` call. The function wrapper is kept because `eval` in `agent-boot.sh` must run in a live function scope to export vars into the calling shell.

- [ ] **Step 1: Verify current function body in `scripts/koad-functions.sh`**

```bash
grep -n "agent-boot\|function agent" /home/ideans/koados-citadel/scripts/koad-functions.sh
```

Confirm the function exists and note the line range.

- [ ] **Step 2: Replace the function body**

Find the current `agent-boot()` function block (everything between `function agent-boot() {` and its closing `}`) and replace it with:

```bash
# agent-boot <name> [args]
# Boots an agent by hydrating the current shell with its identity and environment.
# Must be called as a shell function (not a subprocess) to propagate env vars.
# Boot logic is canonical in plugin/bin/agent-boot.sh — do not add logic here.
function agent-boot() {
    source "$KOAD_HOME/bin/agent-boot.sh" "$@"
}
export -f agent-boot
```

- [ ] **Step 3: Verify the file parses cleanly**

```bash
bash -n /home/ideans/koados-citadel/scripts/koad-functions.sh && echo "SYNTAX OK"
```

Expected: `SYNTAX OK`

- [ ] **Step 4: Commit**

```bash
cd /home/ideans/koados-citadel
git add scripts/koad-functions.sh
git commit -m "refactor(functions): agent-boot delegates to plugin/bin/agent-boot.sh"
```

---

## Task 4: Update `koad-init.sh` to Deploy `agent-boot.sh`

**Files:**
- Modify: `koad-init.sh`

`koad-init.sh` already deploys `scripts/koad-functions.sh` to `$KOAD_HOME/bin/` using `cp`. Add the same pattern for `agent-boot.sh`.

- [ ] **Step 1: Find the existing koad-functions.sh deploy block**

```bash
grep -n "koad-functions" /home/ideans/koados-citadel/koad-init.sh
```

Note the line numbers of the `if [[ -f "scripts/koad-functions.sh" ]]` block (currently around line 137–141).

- [ ] **Step 2: Add the agent-boot.sh deploy block immediately after it**

Insert after the koad-functions.sh deploy block:

```bash
if [[ -f "plugin/bin/agent-boot.sh" ]]; then
    cp plugin/bin/agent-boot.sh "$BIN_DIR/agent-boot.sh"
    ok "agent-boot.sh installed to $BIN_DIR"
else
    warn "plugin/bin/agent-boot.sh not found."
fi
```

- [ ] **Step 3: Verify the file parses cleanly**

```bash
bash -n /home/ideans/koados-citadel/koad-init.sh && echo "SYNTAX OK"
```

Expected: `SYNTAX OK`

- [ ] **Step 4: Commit**

```bash
cd /home/ideans/koados-citadel
git add koad-init.sh
git commit -m "feat(init): deploy agent-boot.sh to KOAD_HOME/bin on init"
```

---

## Task 5: Deploy and Smoke Test

Verify the full chain end-to-end before writing skill files.

- [ ] **Step 1: Run the init deploy from the repo directory**

```bash
cd /home/ideans/koados-citadel && bash koad-init.sh 2>&1 | grep -E "agent-boot|OK|WARN|ERR"
```

Expected: `agent-boot.sh installed to /home/ideans/.citadel-jupiter/bin`

- [ ] **Step 2: Verify the deployed file exists**

```bash
ls -la /home/ideans/.citadel-jupiter/bin/agent-boot.sh
```

Expected: file present, NOT executable (`-rw-r--r--`)

- [ ] **Step 3: Verify the thin wrapper sources it correctly**

```bash
bash -c '
  source /home/ideans/.citadel-jupiter/bin/koad-functions.sh
  type agent-boot
' 2>&1 | head -5
```

Expected: `agent-boot is a function` followed by the wrapper body.

- [ ] **Step 4: Run a live boot and verify env hydration**

```bash
bash -c '
  source /home/ideans/.citadel-jupiter/bin/koad-functions.sh
  agent-boot clyde
  echo "KOAD_AGENT_NAME=$KOAD_AGENT_NAME"
  echo "KOAD_RUNTIME=$KOAD_RUNTIME"
' 2>&1 | tail -10
```

Expected: `KOAD_AGENT_NAME=clyde` and `KOAD_RUNTIME=claude` in output.

- [ ] **Step 5: Commit smoke test confirmation (no code change — just note in working memory if needed)**

---

## Task 6: Write `plugin/skills/agent-boot/SKILL.md`

**Files:**
- Create: `plugin/skills/agent-boot/SKILL.md`

- [ ] **Step 1: Write the core skill file**

```markdown
---
name: agent-boot
description: Use when starting a KoadOS agent session, re-hydrating mid-session, or booting a named agent for the first time. Accepts an agent name and optional level flag (--quick, --full). Default level is standard.
---

# Agent Boot

Boots a KoadOS agent: hydrates shell identity, exports env vars, and orients the session.

## Usage

```
agent-boot <name>           # standard (default)
agent-boot <name> --quick   # boot only, no orientation
agent-boot <name> --full    # boot + full situational report
```

## How to Execute

Run the following Bash command (must run in the terminal — not a subprocess):

```bash
source /home/ideans/.citadel-jupiter/bin/koad-functions.sh && agent-boot <name>
```

Replace `<name>` with the target agent (e.g., `clyde`, `tyr`).

## Boot Levels

- **`--quick`:** Boot only. Follow `quick.md`.
- **`standard` (default):** Boot + orient. Follow `standard.md`.
- **`--full`:** Boot + orient + tasks + Condition Green. Follow `full.md`.

Read the appropriate level file and follow it exactly.
```

Save to: `plugin/skills/agent-boot/SKILL.md`

- [ ] **Step 2: Verify frontmatter is valid (name uses only letters/numbers/hyphens, description starts with "Use when")**

```bash
head -5 /home/ideans/koados-citadel/plugin/skills/agent-boot/SKILL.md
```

Expected: frontmatter block with `name: agent-boot` and description starting `Use when`.

- [ ] **Step 3: Commit**

```bash
cd /home/ideans/koados-citadel
git add plugin/skills/agent-boot/SKILL.md
git commit -m "feat(skill): add agent-boot core skill with level routing"
```

---

## Task 7: Write `plugin/skills/agent-boot/quick.md`

**Files:**
- Create: `plugin/skills/agent-boot/quick.md`

- [ ] **Step 1: Write the quick level file**

```markdown
# Agent Boot — Quick Level

Use for: mid-session re-hydration, scoped subagent spawn, CI context.

## Steps

1. Run the boot command:

```bash
source /home/ideans/.citadel-jupiter/bin/koad-functions.sh && agent-boot <name>
```

2. Confirm the session header printed (look for `--- KoadOS Session: <name> ---`).

3. Stop. Await user direction — do not orient or summarize.
```

Save to: `plugin/skills/agent-boot/quick.md`

- [ ] **Step 2: Commit**

```bash
cd /home/ideans/koados-citadel
git add plugin/skills/agent-boot/quick.md
git commit -m "feat(skill): add agent-boot quick level"
```

---

## Task 8: Write `plugin/skills/agent-boot/standard.md`

**Files:**
- Create: `plugin/skills/agent-boot/standard.md`

- [ ] **Step 1: Write the standard level file**

```markdown
# Agent Boot — Standard Level (Default)

Use for: normal session open.

## Steps

1. Run the boot command:

```bash
source /home/ideans/.citadel-jupiter/bin/koad-functions.sh && agent-boot <name>
```

2. Run situational awareness:

```bash
koad map look
```

3. Check service health:

```bash
koad system status
```

4. Read working memory open items from the session brief output (printed during boot).

5. Report to user:
   - Identity confirmed (agent name + rank)
   - Service state (which of Redis / Citadel / CASS are ACTIVE or OFFLINE)
   - Any open items surfaced from working memory

Do not begin work until the user gives direction.
```

Save to: `plugin/skills/agent-boot/standard.md`

- [ ] **Step 2: Commit**

```bash
cd /home/ideans/koados-citadel
git add plugin/skills/agent-boot/standard.md
git commit -m "feat(skill): add agent-boot standard level"
```

---

## Task 9: Write `plugin/skills/agent-boot/full.md`

**Files:**
- Create: `plugin/skills/agent-boot/full.md`

- [ ] **Step 1: Write the full level file**

```markdown
# Agent Boot — Full Level

Use for: start of a new major session, post-incident recovery, inter-agent handoff.

## Steps

1. Run the boot command:

```bash
source /home/ideans/.citadel-jupiter/bin/koad-functions.sh && agent-boot <name>
```

2. Run situational awareness:

```bash
koad map look
```

3. Check service health:

```bash
koad system status
```

4. Read open tasks from the agent vault:

```bash
ls $KOAD_VAULT_PATH/tasks/
cat $KOAD_VAULT_PATH/tasks/*.md 2>/dev/null || echo "No open task files."
```

5. Assert Condition Green:
   - All required services (Redis, Citadel gRPC, CASS gRPC) must be ACTIVE
   - If any are OFFLINE, flag them explicitly before proceeding
   - Do not begin implementation work with degraded services unless Dood explicitly approves

6. Deliver full situational report to Dood:
   - Identity confirmed
   - Service state (GREEN / DEGRADED — list any OFFLINE services)
   - Open tasks (list titles)
   - Blockers (anything preventing Condition Green)
   - Ready for orders
```

Save to: `plugin/skills/agent-boot/full.md`

- [ ] **Step 2: Commit**

```bash
cd /home/ideans/koados-citadel
git add plugin/skills/agent-boot/full.md
git commit -m "feat(skill): add agent-boot full level"
```

---

## Task 10: Final Verification

- [ ] **Step 1: Verify complete plugin file structure**

```bash
find /home/ideans/koados-citadel/plugin -type f | sort
```

Expected:
```
/home/ideans/koados-citadel/plugin/bin/agent-boot.sh
/home/ideans/koados-citadel/plugin/package.json
/home/ideans/koados-citadel/plugin/skills/agent-boot/SKILL.md
/home/ideans/koados-citadel/plugin/skills/agent-boot/full.md
/home/ideans/koados-citadel/plugin/skills/agent-boot/quick.md
/home/ideans/koados-citadel/plugin/skills/agent-boot/standard.md
```

- [ ] **Step 2: Verify no `TBD` or placeholder text in any skill file**

```bash
grep -rn "TBD\|TODO\|placeholder\|fill in" /home/ideans/koados-citadel/plugin/skills/
```

Expected: no output.

- [ ] **Step 3: Verify the deployed `agent-boot.sh` is current (re-run init)**

```bash
cd /home/ideans/koados-citadel && bash koad-init.sh 2>&1 | grep agent-boot
```

Expected: `agent-boot.sh installed to /home/ideans/.citadel-jupiter/bin`

- [ ] **Step 4: Run live end-to-end boot one final time**

```bash
bash -c '
  source /home/ideans/.citadel-jupiter/bin/koad-functions.sh
  agent-boot clyde
' 2>&1 | grep -E "KoadOS Session|BOOT|ERROR"
```

Expected: `--- KoadOS Session: clyde ---` and `[BOOT] Neural link hydrated for agent 'clyde'.` — no ERROR lines.

- [ ] **Step 5: Final commit**

```bash
cd /home/ideans/koados-citadel
git log --oneline -8
```

Confirm all tasks are represented in recent commits.
