# koad-core

The shared hull of the **KoadOS** spaceship. This crate provides the foundational types, traits, and utilities used across all components of the KoadOS ecosystem.

## 🏗 Overview

`koad-core` serves as the primary library for internal consistency. It defines the common language that the Spine, ASM, and CLI use to communicate and persist state.

## 🔑 Key Modules

- **`config`**: Handles system-wide configuration loading and resolution (e.g., `KoadConfig`).
- **`constants`**: Centralized system constants (ports, sockets, default IDs).
- **`health`**: System diagnostic types and heartbeat monitoring structures.
- **`identity`**: Agent identity definitions, roles (Captain, Officer), and persona metadata.
- **`intelligence`**: Intellectual memory bank types and knowledge retrieval structures.
- **`intent`**: The strongly-typed intent system for routing agent actions.
- **`logging`**: Structured tracing and logging setup (using `tracing`).
- **`session`**: Agent session lifecycle management and hydration types.
- **`storage`**: Database (SQLite) and hot-state (Redis) access abstractions.
- **`types`**: Common domain primitives used throughout the workspace.
- **`utils`**: General-purpose utilities (locks, path resolution, feature gates).

## 🚀 Usage

This crate is a dependency for almost all other crates in the workspace. It is not intended to be used as a standalone binary.

```toml
[dependencies]
koad-core = { path = "../koad-core" }
```
