# KoadOS Source Code Manifest (v3.2)

This manifest serves as the authoritative inventory of the KoadOS source tree, defining the role, ownership, and technical stack of every component.

## 1. System Core (The Kernel Swarm)
| Component | Path | Language | Agent Owner | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **Koad Kernel** | `crates/koad-spine` | Rust | **Koad (Admin)** | Central event hub, gRPC server, and service orchestrator. |
| **Edge Gateway** | `crates/koad-gateway` | Rust | **Network Agent** | Unified I/O proxy for Web and WebSocket traffic. |
| **Koad CLI** | `core/rust` | Rust | **Koad (Admin)** | The primary control plane binary used by humans and agents. |
| **Proto Spec** | `proto/` | Protobuf | **Architect** | Language-agnostic definitions for gRPC services. |

## 2. Shared Foundations
| Library | Path | Language | Purpose |
| :--- | :--- | :--- | :--- |
| **koad-core** | `crates/koad-core` | Rust | Shared types, strongly-typed Intents, and Storage traits. |
| **koad-proto** | `crates/koad-proto` | Rust | Compiled Rust bindings for Protobuf definitions. |
| **koad-board** | `crates/koad-board` | Rust | Logic for GitHub Project Board synchronization. |

## 3. Interaction Layers
| Interface | Path | Stack | Purpose |
| :--- | :--- | :--- | :--- |
| **Web Deck** | `web/deck` | React/TS | Visual command center for situational awareness. |
| **Terminal UI** | `crates/koad-tui` | Rust | Local high-performance monitoring dashboard (Ratatui). |
| **CLI Bridge** | `crates/koad-cli` | Rust | Lightweight gRPC client for scripted control. |

## 4. Automation & Skills (DoodSkills)
| Skill | Path | Language | Purpose |
| :--- | :--- | :--- | :--- |
| **Repo Clean** | `doodskills/repo-clean.py` | Python | Enforces repository hygiene and zero-drift state. |
| **Board Sync** | `crates/koad-board` | Rust | Synchronizes local tasks with GitHub Project #2. |
| **Micro-Agents** | `skills/` | Polyglot | Directory for ephemeral AI agent logic (Ollama/Gemini). |

## 5. System State (Locality of State)
| Resource | Path | Type | Description |
| :--- | :--- | :--- | :--- |
| **Control Config** | `koad.json` | JSON | Identity, preferences, and driver bootstrap. |
| **Master DB** | `koad.db` | SQLite | Cold path storage for history, projects, and memory. |
| **Control Bus** | `koad.sock` | UDS | Redis Unix Domain Socket for the hot path. |
| **Event Logs** | `SESSION_LOG.md` | Markdown | Append-only audit trail of system-wide actions. |

## 6. Development Artifacts
| Artifact | Path | Purpose |
| :--- | :--- | :--- |
| **Environment** | `venv/` | Python virtual environment for governance scripts. |
| **Test Suite** | `tests/e2e/` | Comprehensive Python-based integration tests. |
| **Binaries** | `bin/` | Compiled production-ready binaries. |

---
*Note: This manifest is a living document. Any structural change to the repository MUST be reflected here.*
