---
name: koad-system
description: Use when checking KoadOS service health, starting/stopping/restarting the Citadel and CASS, backing up memory databases, keeping a session alive, or recovering from a disconnected session.
license: MIT
compatibility: Requires a KoadOS Citadel install (koad CLI, $KOAD_HOME, running Citadel and CASS services).
metadata:
  author: koados
  version: "2.0.0"
---

# koad system

Health and lifecycle for the Citadel (control plane, gRPC 50051) and CASS (memory service, gRPC 50052).

## Health

```bash
koad system status     # Redis, Citadel, CASS, SQLite — probes the real gRPC ports
koad doctor            # full health board; `koad doctor -f` also cleans stale sockets/PIDs
```

`[FAIL] Not responding at ...` includes the restart command to run. Check health before reporting a service as broken.

## Start / stop / restart

```bash
koad system start
koad system restart
koad system stop --confirm
```

On hosts where systemd supervises the Citadel (Jupiter), these run `sudo -n systemctl <verb> koad-citadel.service koad-cass.service`. When sudo needs a password they stop and print the exact command — **ask the user to run it in a terminal**; you cannot enter a sudo password from a harness. Per-agent memory MCP servers (`clyde-mcp`, `hermes-cass-mcp`) are separate user services and are never touched; restart one with `systemctl --user restart <unit>`.

## Sessions

```bash
koad system heartbeat  # validate the current session and keep it alive
```

Any authenticated `koad` call also counts as activity. A session idle for about 5 minutes is purged; re-mint it with the `agent-boot` skill when a call fails with `Session not found or expired`.

## Backups and checkpoints

```bash
koad system backup     # WAL-safe snapshot of every database -> $KOAD_HOME/backups/<timestamp>/
koad saveup            # identity checkpoint; `koad saveup --full` also backs up every database
```

Back up before risky operations (migrations, bulk deletes, schema changes). Neither command interrupts running services.

## Other

```bash
koad system locks                        # list distributed locks
koad system lock <sector> / unlock <sector>
koad system logs                         # tail or filter KoadOS logs
koad system config                       # print the loaded configuration
koad system auth                         # show which credentials are configured
```

## Destructive

`koad system scrub` removes local state, logs and databases (prep for distribution). Irreversible: back up first and get explicit approval.

## After deploying new binaries

`install.sh --update` restarts user services itself and, for system services still on old code, ends with "still running the old binaries" plus the exact `sudo systemctl restart ...` command. Relay that command to the user; the update is not live until it runs.
