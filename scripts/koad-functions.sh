#!/bin/bash
# KoadOS Shell Functions
# Source this file from your shell config (e.g., ~/.bashrc):
#   source $KOADOS_HOME/bin/koad-functions.sh

export KOAD_HOME="${KOADOS_HOME:-$HOME/.koad-os}"
export KOAD_BIN="$KOAD_HOME/bin"

# Auto-detect active runtime for agent body authorization.
# Override manually: export KOAD_RUNTIME=<runtime> before calling agent-boot.
if [ -z "$KOAD_RUNTIME" ]; then
    # Claude Code CLI sets CLAUDE_CODE_ENTRYPOINT in the subprocess environment
    if [ -n "$CLAUDE_CODE_ENTRYPOINT" ]; then
        export KOAD_RUNTIME="claude"
    # Gemini CLI (Node.js) — check process ancestry for known runtime signals
    elif [ -n "$GEMINI_API_KEY" ] || [ -n "$GOOGLE_GEMINI_API_KEY" ]; then
        export KOAD_RUNTIME="gemini"
    fi
fi

# agent-boot <name>
# Boots an agent by hydrating the current shell with its identity and environment.
# Must be called as a shell function (not a subprocess) to propagate env vars.
# Runtime detection priority: current env > env signals > agent TOML config.
function agent-boot() {
    if [ -z "$1" ]; then
        echo "[agent-boot] Usage: agent-boot <agent-name>"
        return 1
    fi

    local _AGENT_LOWER=$(echo "$1" | tr '[:upper:]' '[:lower:]')
    local _KOAD_HOME="$KOAD_HOME"
    local _AGENT_TOML="$_KOAD_HOME/config/identities/${_AGENT_LOWER}.toml"
    local _BRIEF_CACHE="$_KOAD_HOME/cache/session-brief-${_AGENT_LOWER}.md"

    # 1. Fast Display: Show the last known state immediately
    if [ -f "$_BRIEF_CACHE" ]; then
        echo -e "\x1b[1;30m[QUICK-RESTORE] Loading last cached brief...\x1b[0m"
        cat "$_BRIEF_CACHE"
        echo -e "\x1b[1;30m-------------------------------------------\x1b[0m"
    fi

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

    if [ -d "/usr/lib/wsl/lib" ]; then
        if [[ ":$LD_LIBRARY_PATH:" != *":/usr/lib/wsl/lib:"* ]]; then
            export LD_LIBRARY_PATH="/usr/lib/wsl/lib${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}"
        fi
    fi

    # 2. Async Hydration: Perform heavy gRPC/Data work
    # We use a subshell to capture the exports but keep the shell responsive
    eval "$("$_KOAD_HOME/bin/koad-agent" boot "$1")"
}
export -f agent-boot

# agent-prep / --agentprep is defined in ~/.pimpedbash/.bash_functions
