# Agent Session Lifecycle

## Metadata
- Category: CORE SYSTEMS & SUBSYSTEMS
- Complexity: intermediate
- Related Topics: koad-citadel, koad-agent-boot, body-ghost-model, end-of-watch
- Key Source Files: `crates/koad-citadel/src/services/session.rs`, `crates/koad-citadel/src/auth/session_cache.rs`
- Key Canon/Doc References: `AGENTS.md`

## Summary
The KoadOS Agent Session Lifecycle is a state machine that governs an agent's existence from boot to logout. Managed by the `CitadelSessionService`, this lifecycle ensures that agent sessions are properly tracked, maintained, and cleaned up. It uses Redis for fast, ephemeral state management and defines several key states: `active`, `dark`, and `purged`.

## How It Works
The entire lifecycle is orchestrated by the `CitadelSessionService` and its interactions with Redis.

1.  **Boot & Lease Creation (State: `active`)**:
    - A session begins when `koad-agent boot` calls the `CreateLease` RPC.
    - The `CitadelSessionService` creates a `SessionRecord` containing the agent's name, session ID, body ID, rank, and status (`active`).
    - This record is stored as a hash in Redis under the key `koad:state`.
    - A separate key, `koad:session:<session_id>`, is created with a Time-To-Live (TTL) equal to the lease duration (e.g., 90 seconds). This is the **lease key**.

2.  **Heartbeat (Maintaining `active` state)**:
    - The agent's CLI (e.g., Gemini CLI) is expected to send a `Heartbeat` RPC call periodically (e.g., every 30 seconds).
    - When the Citadel receives a valid heartbeat, it **refreshes the TTL** on the `koad:session:<session_id>` lease key, extending the agent's lease.
    - The `SessionRecord` in `koad:state` remains `active`.

3.  **Going Dark (State: `dark`)**:
    - If the agent's "Body" (the shell process) crashes or loses connectivity, it can no longer send heartbeats.
    - The TTL on the `koad:session:<session_id>` lease key will expire. Redis automatically deletes the key.
    - The `SessionRecord` in `koad:state` still exists and its status is still `active`. The Citadel now considers the session "dark" because its lease key is gone.

4.  **Reaping (State Transition: `active` -> `dark`)**:
    - The `CitadelSessionService` runs a background **reaper task**.
    - Periodically (e.g., every 10 seconds), the reaper scans all sessions in `koad:state`.
    - For each `active` session, it checks if the corresponding `koad:session:<session_id>` lease key still exists in Redis.
    - If the lease key is missing, the reaper updates the `SessionRecord`'s status in `koad:state` from `active` to `dark`. It also logs a "Ghost in the Machine" event.

5.  **EndOfWatch (Logout & Purge)**:
    - **Graceful Logout:** If the agent calls `CloseSession`, the Citadel immediately removes the `SessionRecord` from `koad:state` and the lease key from Redis. It then fires the `session_closed` event to the Koad Stream for the EOW pipeline.
    - **Purging Dark Sessions:** The reaper also cleans up sessions that have been `dark` for too long (e.g., > 5 minutes). It purges the `SessionRecord` from `koad:state` and fires the `session_closed` event, ensuring even crashed sessions get summarized by CASS.

This system guarantees that no "Ghost" is left permanently stranded in a dead "Body".

## Key Code References
- **File**: `crates/koad-citadel/src/services/session.rs`
  - **Element**: `CitadelSessionService`, `reap()` method
  - **Purpose**: Contains the primary logic for the entire lifecycle, including the gRPC handlers (`create_lease`, `heartbeat`, `close_session`) and the background reaper task.
- **File**: `crates/koad-citadel/src/auth/session_cache.rs`
  - **Element**: `SessionRecord`, `ActiveSessions`
  - **Purpose**: Defines the data structures for session state and provides the core Redis commands for interacting with the session cache.
- **File**: `crates/koad-core/src/utils/redis.rs`
  - **Element**: `RedisClient`
  - **Purpose**: The shared utility for connecting to and interacting with the Redis server.

## Configuration & Environment
- `config/kernel.toml` -> `[sessions]`:
  - `lease_duration_secs`: The TTL for a session lease key.
  - `dark_timeout_secs`: How long a session can be "dark" before it's a candidate for purging.
  - `reaper_interval_secs`: How often the reaper task runs to check for dark sessions.

## Common Questions a Human Would Ask
- "What happens if my computer goes to sleep during a session?"
- "How does the system clean up crashed agent sessions?"
- "What is a 'dark' session?"
- "Why does my session time out?"
- "How can I change the session lease time?"

## Raw Technical Notes
- The use of Redis TTLs is a highly efficient way to manage liveness. The Citadel doesn't need to poll every agent; it only needs to react when a TTL expires, which is a native Redis feature.
- The separation of the `koad:state` hash (permanent record of all sessions) and the `koad:session:*` keys (ephemeral lease locks) is a key design choice. It allows the reaper to identify dark sessions by checking for the *absence* of a key, which is much faster than checking timestamps on every record.
- This lifecycle is fundamental to the stability of the entire KoadOS ecosystem.
