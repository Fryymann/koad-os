# Agent Session Lifecycle

> The state machine governing an agent session from boot to logout — tracking active, dark, and purged states through Redis TTLs, periodic heartbeats, and an automatic reaper task.

**Complexity:** intermediate
**Related Articles:** [koad-citadel](./koad-citadel.md), [koad-agent boot](./koad-agent-boot.md), [The Body/Ghost Model](../architecture/body-ghost-model.md), [koad-cass](./koad-cass.md)

---

## Overview

Every agent session in KoadOS follows a defined lifecycle managed by the `CitadelSessionService`. The lifecycle tracks an agent's session through three states: `active` (alive and heartbeating), `dark` (connection lost, cleanup pending), and `purged` (fully cleaned up, EndOfWatch fired). Understanding this lifecycle is essential for operating, debugging, and configuring KoadOS.

The system uses two complementary Redis structures to represent a session:

1. **`koad:state` hash**: A persistent record of all sessions and their metadata (`SessionRecord`). This is the Citadel's ledger — it knows about every session that has ever been created and not yet purged.

2. **`koad:session:<session_id>` key**: An ephemeral lease key with a Time-To-Live (TTL). This is the "heartbeat anchor". As long as this key exists, the session is considered alive by the reaper.

The elegance of this design is that session liveness is determined by the *presence or absence* of a Redis key, not by timestamps or polling. Redis handles the TTL expiry natively and atomically. The reaper only needs to check whether a key exists — a single `EXISTS` command — rather than computing time deltas across every session.

## How It Works

### State 1: Active (Boot → Heartbeat Loop)

A session enters `active` state when `CitadelSessionService::create_lease()` succeeds:

1. A `SessionRecord` is written to the `koad:state` Redis hash:
   ```
   koad:state[<session_id>] = {
     agent_name: "Tyr",
     session_id: "sid_abcd...",
     body_id: "body_uuid...",
     rank: "Captain",
     status: "active"
   }
   ```

2. A lease key is created with a TTL:
   ```
   SET koad:session:<session_id> "alive" EX <lease_duration_secs>
   ```

The lease key TTL is the heartbeat clock. By default (`lease_duration_secs = 90`), the agent has 90 seconds before its lease key expires.

**Heartbeats keep the session alive**: The agent sends periodic `Heartbeat` RPCs to the Citadel (typically every 30 seconds). Each successful heartbeat executes a Redis `EXPIRE` on the lease key, resetting its TTL back to `lease_duration_secs`. As long as heartbeats arrive regularly, the lease key never expires and the session stays `active`.

### State 2: Dark (Heartbeat Lost)

If the agent's Body (shell process) crashes, disconnects, or is killed, heartbeats stop. The lease key's TTL counts down to zero. When it hits zero, Redis deletes the key automatically.

At this point, the `koad:state` record still exists with `status: "active"` — the Citadel doesn't know the session is dead yet. The session is now "dark": the Ghost is still on the ledger but the Body has gone silent.

**The reaper detects darkness**: A background task in `CitadelSessionService` runs on `reaper_interval_secs` (default: 10 seconds). On each cycle, it:
1. Fetches all session IDs from `koad:state`
2. For each `active` session, checks `EXISTS koad:session:<session_id>` in Redis
3. If the lease key is *missing*, the session is dark. The reaper:
   - Updates `koad:state[<session_id>].status` to `"dark"`
   - Logs a "Ghost in the Machine" event (the Agent's ghost is stuck in a dead body)
   - Records the time the session went dark

### State 3: Purged (EndOfWatch Fired)

**Graceful logout** bypasses the dark state entirely. When the agent calls `CloseSession`:
1. The Citadel immediately removes the `koad:state` record
2. Deletes the lease key (if still present)
3. Fires a `session_closed` event to `koad:stream:system`
4. Returns success to the caller

The session is purged instantly on graceful logout.

**Dark session purge** follows a timeout. After `dark_timeout_secs` (default: 300 = 5 minutes) in the dark state, the reaper:
1. Removes the `koad:state` record (purges the session)
2. Fires a `session_closed` event to `koad:stream:system`

In both cases, the `session_closed` event on the Redis stream triggers CASS's `EndOfWatchPipeline`. The pipeline generates an AI summary of the session's activity and saves it as an `EpisodicMemory` record in `cass.db`. This completes the cognitive loop — even crashed sessions get summarized.

### Full Lifecycle Diagram

```
                    koad-agent boot
                         │
                    CreateLease RPC
                         │
                         ▼
              ┌─────── ACTIVE ──────────────┐
              │  koad:state: status=active  │
              │  koad:session:* TTL=90s     │
              │                             │
              │  [heartbeat every ~30s]     │
              │  → Redis EXPIRE resets TTL  │
              └─────────────────────────────┘
                         │
            ┌────────────┴───────────────┐
            │ Graceful                   │ Crash / network loss
            │ CloseSession RPC           │ (heartbeat stops)
            │                            ▼
            │                 ┌────────  DARK ──────────┐
            │                 │  koad:state: status=dark │
            │                 │  koad:session:* EXPIRED  │
            │                 │                          │
            │                 │  [after dark_timeout_secs]│
            │                 └──────────────────────────┘
            │                            │
            └──────────────┬─────────────┘
                           │
                           ▼
                        PURGED
                  koad:state record removed
                  session_closed event fired
                         │
                         ▼
              CASS EndOfWatchPipeline
              generates EpisodicMemory
```

## Configuration

| Key | Section | Default | Description |
|-----|---------|---------|-------------|
| `lease_duration_secs` | `config/kernel.toml [sessions]` | `90` | TTL for the Redis lease key; how long until a non-heartbeating session goes dark |
| `dark_timeout_secs` | `config/kernel.toml [sessions]` | `300` | How long a dark session persists before being purged |
| `reaper_interval_secs` | `config/kernel.toml [sessions]` | `10` | How often the reaper task checks for dark sessions |

**Tuning guidance**: Set `lease_duration_secs` to at least 3x the expected heartbeat interval to tolerate transient delays. Set `dark_timeout_secs` long enough to allow reconnect after a brief disconnection, but short enough that stale sessions don't block re-boots (remember, a dark session's agent name is still "occupied" until purge).

## Failure Modes & Edge Cases

**What happens if my computer goes to sleep during a session?**
The shell process is suspended, heartbeats stop, and the lease key's TTL runs out. When you wake up, the session will be in the `dark` state. After `dark_timeout_secs` (5 minutes by default), it will be purged. You'll need to `eval $(koad-agent boot --agent <name>)` again in your shell. If you wake before the purge deadline, it's possible to have a "dark but not yet purged" session blocking your re-boot — wait for the reaper cycle or ask for a manual purge.

**Heartbeat interval vs. lease duration mismatch.**
If the heartbeat interval in the agent CLI is longer than `lease_duration_secs`, sessions will frequently go dark even when the agent is active. Ensure the heartbeat interval is set to roughly `lease_duration_secs / 3`. The heartbeat interval is configured on the client side (in the AI CLI's settings), not in `kernel.toml`.

**Redis outage during an active session.**
Heartbeats will fail (Redis is unavailable). The Citadel logs the failure but doesn't immediately kill the session — it depends on whether the heartbeat failure causes the CLI to halt. The lease key will expire during the outage. On Redis recovery, the session will appear dark to the reaper on its next cycle.

**`dark_timeout_secs` is set too low.**
Sessions are purged before the agent has a chance to reconnect after a brief interruption. The "One Body, One Ghost" rule blocks a re-boot until the old session is purged. Set `dark_timeout_secs` to at least 5-10 minutes in practice.

## FAQ

### Q: What happens if my computer goes to sleep during a session?
When your machine sleeps, the agent process is suspended and stops sending heartbeats. The Redis lease key TTL expires during the sleep. When you wake up, the Citadel's reaper will have (or soon will) mark your session as dark. After `dark_timeout_secs` (default: 5 minutes) in the dark state, the session is purged and an `EndOfWatch` summary is generated by CASS. You'll need to open a new shell and run `eval $(koad-agent boot --agent <name>)` to start a fresh session.

### Q: How does the system clean up crashed agent sessions?
The reaper task in `CitadelSessionService` runs every `reaper_interval_secs` (default: 10 seconds). It checks every `active` session's lease key in Redis. If the key is missing (TTL expired because heartbeats stopped), it marks the session `dark`. After `dark_timeout_secs`, the session is purged and a `session_closed` event fires, triggering CASS's `EndOfWatchPipeline` to generate a retrospective for the crashed session.

### Q: What is a "dark" session?
A dark session is one where the `SessionRecord` still exists in Redis (`koad:state`) but the lease key (`koad:session:<session_id>`) has expired — meaning the agent's Body has gone silent (crashed, network loss, machine sleep). The session is in limbo: the Ghost is still "in" a Body, but that Body isn't alive. Dark sessions block re-boots because "One Body, One Ghost" sees the agent name as occupied. The reaper resolves dark sessions by purging them after `dark_timeout_secs`.

### Q: Why does my session time out?
Sessions time out when the agent stops sending heartbeats to the Citadel. The lease key TTL (`lease_duration_secs`, default: 90 seconds) expires, and the reaper marks the session dark. This typically means the shell process was killed, the AI CLI stopped, or network connectivity was lost. The session doesn't "time out" while heartbeats are flowing — a healthy, connected session can remain active indefinitely.

### Q: How can I change the session lease time?
Edit `config/kernel.toml` under the `[sessions]` section. Increase `lease_duration_secs` for a longer grace period before sessions go dark, `dark_timeout_secs` for a longer window before dark sessions are purged, or `reaper_interval_secs` to change how frequently the reaper checks. Restart the Citadel for changes to take effect.

## Source Reference

- `crates/koad-citadel/src/services/session.rs` — `CitadelSessionService`; `create_lease()`, `heartbeat()`, `close_session()`, and the `reap()` background task
- `crates/koad-citadel/src/auth/session_cache.rs` — `SessionRecord` struct and `ActiveSessions`; Redis hash operations for session state
- `crates/koad-core/src/utils/redis.rs` — `RedisClient`; shared Redis connection and utility methods
- `config/kernel.toml` — `[sessions]` section; all lifecycle timing parameters
