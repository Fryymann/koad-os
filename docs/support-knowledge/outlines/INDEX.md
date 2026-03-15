# KoadOS Support Knowledge Base: Topic Index

This document serves as the master index for all technical and conceptual outlines related to the KoadOS ecosystem. It is the primary entry point for Phase 2 (Article Generation) and Phase 3 (RAG-based Support).

## I. Topic List by Category

### A. ARCHITECTURE & CONCEPTS
| Topic | Complexity | Summary |
| :--- | :--- | :--- |
| **The Tri-Tier Model** | basic | Explains the foundational Body/Brain/Link architecture (Citadel, CASS, koad-agent). |
| **The Workspace Hierarchy** | intermediate | Details the System → Citadel → Station → Outpost mapping of the filesystem. |
| **The Body/Ghost Model** | basic | Defines the separation between an agent's ephemeral shell session and its persistent identity. |

### B. CORE SYSTEMS & SUBSYSTEMS
| Topic | Complexity | Summary |
| :--- | :--- | :--- |
| **`koad-citadel`** | advanced | The core OS kernel responsible for sessions, security, and orchestration. |
| **`koad-cass`** | advanced | The agent support system providing memory, intelligence, and context hydration. |
| **`koad-agent boot`** | intermediate | The shell hydration command that fuses a "Ghost" to a "Body". |
| **Agent Session Lifecycle** | intermediate | Describes the `active` → `dark` → `purged` states of an agent session. |

### C. PROTOCOLS & GOVERNANCE
| Topic | Complexity | Summary |
| :--- | :--- | :--- |
| **`RUST_CANON`** | intermediate | The mandatory Rust development standards for the KoadOS project. |

### D. AGENT ROLES & RESPONSIBILITIES
| Topic | Complexity | Summary |
| :--- | :--- | :--- |
| **Tyr: Captain** | basic | Outlines the role and responsibilities of the Lead Architect agent. |

### E. DATA & STORAGE
| Topic | Complexity | Summary |
| :--- | :--- | :--- |
| **SQLite Storage (`cass.db`)**| intermediate | Explains the schema and usage of the persistent "cold path" memory store. |

### F. TOOLING & DEVELOPER WORKFLOW
| Topic | Complexity | Summary |
| :--- | :--- | :--- |
| **Cargo Workspace (`koad-os`)** | intermediate | Details the multi-crate structure and dependency management of the project. |

## II. Topic Relationship Map

- **The Tri-Tier Model**
  - **Relates to:** `koad-citadel`, `koad-cass`, `koad-agent boot` (as its components), `body-ghost-model` (as a core concept).
- **The Workspace Hierarchy**
  - **Relates to:** `koad-cass` (for TCH), `koad-citadel` (for enforcement), `personal-bays` (as a physical implementation).
- **`koad-citadel`**
  - **Relates to:** `agent-session-lifecycle` (which it manages), `koad-sandbox` (which it integrates), `zero-trust-security`.
- **`koad-cass`**
  - **Relates to:** `koad-intelligence` (which it uses), `sqlite-cass-db` (which it owns), `temporal-context-hydration`.
- **`RUST_CANON`**
  - **Relates to:** `cargo-workspace` (which it governs), `contributor-canon` (as a sibling protocol).

## III. Coverage Assessment
This initial pass has covered the highest-level architectural concepts and the primary crates (`citadel`, `cass`, `cli`).

**Key Gaps Remaining:**
- **Detailed Sub-Crate Outlines:** `koad-intelligence`, `koad-sandbox`, `koad-codegraph` need their own dedicated outlines.
- **Protocol Details:** `KSRP`, `PSRP`, and `Contributor Canon` need to be documented.
- **Specific Data Structures:** `FactCard`, `EpisodicMemory`, and `Redis` usage patterns need their own outlines.
- **Agent Roles:** All other agents (Sky, Scribe, Helm, etc.) need outlines.
- **Tooling:** Shell functions and specific CLI commands (e.g., `status`, `doctor`) need outlines.

This index provides a solid foundation for the next stage of knowledge extraction.
