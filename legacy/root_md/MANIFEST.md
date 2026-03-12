# KoadOS Source Code Manifest (v3.2)

This manifest serves as the authoritative inventory of the KoadOS source tree, defining the role, ownership, and technical stack of every component.

## 1. System Core (The Construct)
| Component | Path | Language | Agent Owner | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **Koad Spine** | `crates/koad-spine` | Rust | **Koad (Admin)** | Central neural hub, gRPC server, and construct orchestrator. |
| **Koad Gateway** | `crates/koad-gateway` | Rust | **Network Agent** | Unified virtual proxy for Web and WebSocket uplinks. |
| **Koad CLI** | `core/rust` | Rust | **Koad (Admin)** | The primary terminal interface for jacking into the construct. |
| **Koad Proto** | `proto/` | Protobuf | **Architect** | Digital blueprints for inter-component communication. |

## 2. Shared Foundations
| Library | Path | Language | Purpose |
| :--- | :--- | :--- | :--- |
| **Koad Core** | `crates/koad-core` | Rust | Shared logic, strongly-typed Intents, and Buffer traits. |
| **Koad Proto** | `crates/koad-proto` | Rust | Hard-wired Rust bindings for Protobuf blueprints. |
| **Koad Board** | `crates/koad-board` | Rust | Logic for synchronizing with the master Mission Log. |

## 3. Interface Layers
| Interface | Path | Stack | Purpose |
| :--- | :--- | :--- | :--- |
| **Command Deck** | `web/deck` | React/TS | Visual HUD for real-time grid situational awareness. |
| **Terminal HUD** | `crates/koad-tui` | Rust | High-performance terminal dashboard (Ratatui). |
| **Uplink Bridge** | `crates/koad-cli` | Rust | Lightweight gRPC bridge for scripted control. |

## 4. Automation & Skills (Neural Imprints)
| Skill | Path | Language | Purpose |
| :--- | :--- | :--- | :--- |
| **Grid Clean** | `doodskills/repo-clean.py` | Python | Enforces repository hygiene and prevents sector drift. |
| **Log Sync** | `crates/koad-board` | Rust | Synchronizes local data fragments with the Mission Log. |
| **Micro-Agents** | `skills/` | Polyglot | Directory for ephemeral AI neural imprints (Ollama/Gemini). |

## 5. System State (Buffer Locality)
| Resource | Path | Type | Description |
| :--- | :--- | :--- | :--- |
| **Neural Config** | `config/` | TOML | Three-tier registry (Kernel, Registry, Identities). |
| **Master Memory** | `koad.db` | SQLite | Durable storage for history, project nodes, and memory. |
| **Control Signal** | `koad.sock` | UDS | Redis Unix Domain Socket for hot-path data. |
| **Uplink Logs** | `SESSION_LOG.md` | Markdown | Append-only audit trail of grid-wide sequences. |

## 6. Development Artifacts
| Artifact | Path | Purpose |
| :--- | :--- | :--- |
| **Environment** | `venv/` | Python virtual environment for governance scripts. |
| **Test Suite** | `tests/e2e/` | Comprehensive Python-based integration tests. |
| **Binaries** | `bin/` | Compiled production-ready binaries. |

---
*Note: This manifest is a living document. Any structural change to the repository MUST be reflected here.*
