# KoadOS Master Navigation Map (v3.2)

This index provides the definitive mapping of KoadOS features to their source code and compiled binaries.

## 1. Core Services (The Spine)
| Service | Purpose | Source Root | Binary / Entry |
| :--- | :--- | :--- | :--- |
| **Kernel (Spine)** | Central RPC & Event Hub | `crates/koad-spine/src` | `~/.koad-os/bin/kspine` |
| **Edge Gateway** | Web Deck & WebSocket Proxy | `crates/koad-gateway/src` | `~/.koad-os/bin/kgateway` |
| **Command Processor** | Intent Routing (Async) | `crates/koad-spine/src/engine/router.rs` | Part of Kernel |
| **Storage Bridge** | Redis/SQLite Unification | `crates/koad-spine/src/engine/storage_bridge.rs` | Part of Kernel |
| **Compliance Manager** | Repo Hygiene (KCM) | `crates/koad-spine/src/engine/kcm.rs` | Part of Kernel |

## 2. Interface Points
| Interface | Protocol | Definition | Implementation |
| :--- | :--- | :--- | :--- |
| **Kernel Service** | gRPC (UDS/TCP) | `proto/kernel.proto` | `crates/koad-spine/src/rpc/mod.rs` |
| **Spine Service** | gRPC (UDS/TCP) | `proto/spine.proto` | `crates/koad-spine/src/rpc/mod.rs` |
| **Command Bus** | Redis PubSub | `koad:commands` | `crates/koad-spine/src/engine/router.rs` |
| **Event Stream** | Redis Stream | `koad:events:stream` | System-wide |

## 3. User Interfaces
| UI | Technology | Source Root | URL / Entry |
| :--- | :--- | :--- | :--- |
| **Command Deck** | React / Vite | `web/deck/src` | `http://localhost:3000` |
| **Koad CLI** | Rust / Clap | `core/rust/src` | `/usr/local/bin/koad` |
| **Koad Dash (TUI)** | Rust / Ratatui | `crates/koad-tui/src` | `koad dash` |

## 4. Shared Libraries (Crates)
- **koad-core**: Shared types, Intent enums, and Identity logic.
- **koad-proto**: Generated gRPC code.
- **koad-board**: GitHub Project integration logic.

## 5. Automation Skills
- **Repo Clean**: `doodskills/repo-clean.py`
- **Version Bump**: `doodskills/version-bump.py`
- **Detective**: `skills/gemini/detective.py`
