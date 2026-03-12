# koad-spine

The central orchestration engine of the **KoadOS** ecosystem. This crate provides the backend gRPC services, state management, and coordinate control logic for the Neural Grid.

## 🏗 Overview

`koad-spine` (also known as the **Backbone**) is the heartbeat of the Citadel. It maintains the hot state in Redis, facilitates durable persistence in SQLite, and provides the gRPC interface for agents to interact with the system.

## 🔑 Key Components

### **Engine Subsystem (`src/engine/`)**
The brain of the Spine, responsible for:
- **`asm`**: Orchestrating the Agent Session Manager lifecycle.
- **`context_cache`**: Managing the sub-second context hydration cache.
- **`identity`**: The authoritative registry for active KAI Officers.
- **`kernel`**: Core control-plane logic and system state transitions.
- **`sandbox`**: Enforcing execution policies and safety guardrails.
- **`storage_bridge`**: The unified bridge between hot state (Redis) and cold memory (SQLite).

### **RPC Subsystem (`src/rpc/`)**
The primary interface of the Spine:
- Implements the `SpineService` gRPC server.
- Handles heartbeat processing, session registration, and context hydration requests.

## ⚙️ Operation

The Spine typically runs as a background daemon, bound to a Unix Domain Socket or a local network port.

```bash
# Start the Spine
kspine
```

## 🚀 Usage

This crate is the primary server and is not typically used as a library dependency by other crates, except for internal testing.

```toml
[dependencies]
koad-spine = { path = "../koad-spine" }
```
