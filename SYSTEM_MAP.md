# KoadOS System Map
# Updated: 2026-04-13 | Author: Tyr | Pivot to Dynamic Knowledge Graph
# This file is the strategic entry point for workspace navigation.
# For deep-dives into specific modules, use the dynamic tools below.

---

## 🧭 Dynamic Navigation (The "Game Map HUD")

KoadOS now uses a **Dynamic System Map (DSM)** powered by `code-review-graph`. Agents and users should use the `koad map` command group for real-time situational awareness.

| Command | Protocol | Role |
| :--- | :--- | :--- |
| `koad map look` | **HUD Scan** | Describe current community, role, and local files. |
| `koad map exits` | **Pathfinding** | List outgoing dependencies and pinned fast-travel paths. |
| `koad map nearby` | **Impact Scan** | List incoming dependencies (impact radius) and local POIs. |
| `koad map goto <target>` | **Fast Travel** | Teleport to a symbol, file, or pin using the graph FTS index. |
| `koad map pins` | **Bookmarks** | List all manually established fast-travel points. |

### 🛠️ Strategic Tools
- **Interactive Graph:** Run `code-review-graph visualize` to open the HTML HUD.
- **Knowledge Wiki:** Explore the community-based documentation at `.code-review-graph/wiki/`.

---

## Workspace Hierarchy (Distribution vs. Instance)

KoadOS follows a strict separation between the **Distribution** (shared platform code) and the **Instance** (deployment state).

1.  **Code & Defaults (Distribution):** Tracked in Git. Includes Rust crates, proto definitions, and `config/defaults/`.
2.  **Standards & Docs (Distribution):** Tracked in Git. Includes `MISSION.md`, `AGENTS.md`, and `docs/`.
3.  **Local State (Instance):** Gitignored. Includes `data/db/`, `logs/`, `run/`, and active configurations.

---

## Workspace Topology

| Location | Role | Commander |
| :--- | :--- | :--- |
| `~/.koad-os/` | **Citadel** — KoadOS project root, Rust source, all shared config | Dood (Ian) |
| `~/data/SLE/` | **SLE Station** — Skylinks Local Ecosystem; apps and Sky's command post | Sky |

---

## Quick Reference — Most Accessed Paths

| Path | What It Is | When You Need It |
| :--- | :--- | :--- |
| `MISSION.md` | Core Mission & Vision | Strategic orientation |
| `AGENTS.md` | Root onboarding portal | Boot / new session |
| `config/identities/` | Active KAI identities (TOML) | Boot / identity loading |
| `crates/` | Active Rust source | Implementation tasks |
| `proto/` | gRPC definitions | API / service implementation |
| `data/db/` | All runtime SQLite databases | Storage debugging |
| `run/` | Runtime sockets and PID files | Service health / debugging |

---

## Knowledge Communities (Functional Map)

The Citadel is currently partitioned into **110 functional communities**. Below are the primary clusters:

| Community | Role | Primary Crates |
| :--- | :--- | :--- |
| **Kernel Core** | System initialization, config, and logging | `koad-core`, `koad-cli` |
| **Signal Corps** | gRPC communication and protobufs | `koad-proto`, `koad-citadel` |
| **Cognition Hub** | Agent memory (CASS) and intelligence routing | `koad-cass`, `koad-intelligence` |
| **Provisioning** | Bay isolation and worktree management | `koad-citadel` |
| **Integration Bridge** | External services (Notion, GitHub, etc.) | `koad-bridge-notion`, `koad-board` |

---

## Agent Vaults Index

| Agent | Vault Path | Runtime | Status |
| :--- | :--- | :--- | :--- |
| **Tyr** | `~/.tyr/` | Gemini | Active — Captain |
| **Clyde** | `~/.koad-os/agents/KAPVs/clyde/` | Claude Code | Active — Officer |
| **Cid** | `~/.koad-os/agents/KAPVs/cid/` | Codex | Active — Engineer |
| **Scribe** | `~/.koad-os/agents/KAPVs/scribe/` | Gemini (flash-lite) | Active — Crew |
| **Helm** | `~/.koad-os/agents/KAPVs/helm/` | Gemini | Active — Officer |
| **Claude** | `~/.koad-os/agents/.claude/` | Claude Code | Active — Contractor |
| **Sky** | `~/data/SLE/.station/agents/sky/` | Gemini | Active — Officer (SLE) |

---

## Stale / Deprecated Items

- `legacy/` — [ARCHIVE] Retired Spine-era artifacts.
- `koad-codegraph` — **DEPRECATED**. Replaced by the `code-review-graph` Dynamic System Map.
- `koad-watchdog` — Removed. Integrated into `koad-citadel`.
