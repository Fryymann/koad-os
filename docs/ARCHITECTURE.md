# KoadOS Architecture

This document describes the strategic architecture and topology of the KoadOS ecosystem.

## Workspace Hierarchy (Distribution vs. Instance)

KoadOS follows a strict separation between the **Distribution** (shared platform code) and the **Instance** (deployment state).

1.  **Code & Defaults (Distribution):** Tracked in Git. Includes Rust crates, proto definitions, and `config/defaults/`.
2.  **Standards & Docs (Distribution):** Tracked in Git. Includes `MISSION.md`, `AGENTS.md`, and `docs/`.
3.  **Local State (Instance):** Gitignored. Includes `data/db/`, `logs/`, `run/`, and active configurations.

## Workspace Topology

| Location | Role | Commander |
| :--- | :--- | :--- |
| `~/.koad-os/` | **Citadel** — KoadOS project root, Rust source, all shared config | Dood (Ian) |
| `~/data/SLE/` | **SLE Station** — Skylinks Local Ecosystem; apps and Sky's command post | Sky |

## Knowledge Communities (Functional Map)

The Citadel is currently partitioned into functional communities. Below are the primary clusters:

| Community | Role | Primary Crates |
| :--- | :--- | :--- |
| **Kernel Core** | System initialization, config, and logging | `koad-core`, `koad-cli` |
| **Signal Corps** | gRPC communication and protobufs | `koad-proto`, `koad-citadel` |
| **Cognition Hub** | Agent memory (CASS) and intelligence routing | `koad-cass`, `koad-intelligence` |
| **Provisioning** | Bay isolation and worktree management | `koad-citadel` |
| **Integration Bridge** | External services (Notion, GitHub, etc.) | `koad-bridge-notion`, `koad-board` |

## Graph-Centric Navigation

KoadOS has transitioned from static documentation maps to a **Graph-Centric Navigation** model. This shift ensures that the system topology is always accurate and reflects the actual state of the codebase.

### The Dynamic System Map (DSM)
The topology is now powered by `code-review-graph`, which builds a real-time index of symbols, files, and dependencies.

- **CLI Navigation:** Use `koad map` (e.g., `koad map look`, `koad map goto`) for terminal-based exploration.
- **Visual HUD:** Use `code-review-graph visualize` to generate and open a visual representation of the workspace graph.

This model allows for dynamic "Fast Travel" and impact analysis that static files cannot provide.
