# koad-asm

The Agent Session Manager (ASM) micro-daemon for **KoadOS**.

## 🏗 Overview

`koad-asm` is a standalone daemon that monitors agent heartbeats, enforces session isolation, and manages the hydration state of active KAIs. It was decoupled from the Spine to ensure high-availability session monitoring.

## 🔑 Core Responsibilities

- **Heartbeat Monitoring**: Watch for active neural pulses from agents (e.g., Tyr, Sky).
- **Session Pruning**: Autonomously purge stale sessions and volatile context if heartbeats fail.
- **Body Enforcement**: Enforce the "One Body, One Ghost" protocol to prevent session leakage.
- **State Hydration**: Coordinate with the Spine to hydrate transient context for newly waking agents.

## ⚙️ Operation

`koad-asm` typically runs as a background process, automatically managed by the **Autonomic Watchdog** or the **Spine**.

```bash
# Start the ASM
koad-asm
```

## 🚀 Usage

```toml
[dependencies]
koad-asm = { path = "../koad-asm" }
```
