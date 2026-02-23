# KoadOS v2.0 (Lean Edition)

The Agnostic AI Coding Agent Framework.

## Architecture
- **Source of Truth**: `~/.koad-os/koad.json`
- **Control Plane**: `~/.koad-os/core/rust/target/release/koad` (Rust CLI)
- **Skills**: `~/.koad-os/skills/` (Dispatcher for agent-built scripts)

## Core Mandate
1. **Boot**: `koad boot --project` (Ingests persona + recent memory + project snapshot)
2. **Remember**: `koad remember <category> "<text>"` (Updates global memory)
3. **Execute**: `koad skill run <path>` (Dispatches automation)

## Directory Structure
- `core/rust/`: Source code for the high-performance CLI.
- `skills/`: Repository of automation scripts.
- `legacy/`: Archived bureaucratic files (MD ledgers, protocols).
