# CLI Command Surface (v5.0)
**Status:** DRAFT (Phase 1)
**Issue:** #158

## 1. Requirement
Define the replacement surface for retired `koad spine *` commands.

## 2. Primary Surface

### `koad citadel`
Control the Citadel infrastructure.
- `broker`: Session connectivity and lease management.
- `sector`: Sector lock-state and distributed coordination.
- `signal`: Signal Corps observability and stream tailing.
- `bay`: Personal Bay status and health.
- `provision-bay`: Explicit Personal Bay creation/formatting.

### `koad agent`
Manage the agent-level lifecycle (The Ghost).
- `prepare`: Hydrate context and generate the shell `eval` payload.
- `status`: Local connectivity and Dark Mode status.
- `clear`: Session wrap-up (EndOfWatch) and cleanup.

### `koad signal`
Inter-agent async messaging (A2A-S).
- `send <target> -m "<msg>" [-p HIGH|NORMAL|LOW]`: Push a signal to an agent's mailbox.
- `read`: Check own mailbox for unread signals.

### `koad intel`
Memory and knowledge interaction via CASS.
- `commit`: Save a curated knowledge/journal entry to CASS L2/L3.
- `query`: Semantic search against knowledge collections.

### `koad system`
Administrative and recovery tasks.
- `doctor`: Deep diagnostic scan (via `koad dood pulse`).
- `migrate-v5`: One-time legacy data extraction tool.
- `purge --confirm`: Complete system wipe (Admin only).

## 3. Global Flags
- `--trace-id <id>`: Force a specific Trace ID for an invocation.
- `--json`: Machine-readable output (standard for all commands).
- `--force | --confirm`: Required for dangerous system operations.