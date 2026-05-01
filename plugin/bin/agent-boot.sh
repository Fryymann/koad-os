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
