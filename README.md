# KoadOS — The Agent Operating System

KoadOS is a local-first, model-agnostic agent operating system designed to turn open-source AI models into a self-improving swarm of specialized software development agents. It provides reasoning-only token budgets, externalized cognition, and compounding memory.

## 🛡 Stable Release: v3.2.0 "Citadel Integrity"

The v3.2.0 release focuses on stability, purge of technical debt, and real-time project awareness.

### Key Highlights:
- **AIS Modernization**: "Citadel Pulse & Beacon" system for real-time inter-agent awareness.
- **Project Synchronization**: Mandatory Captain-level enforcement of GitHub Project state.
- **Refined Infrastructure**: Complete purge of Spine-era shims and dead code.
- **Standardized Skills**: Integrated Skill Blueprint/Instance architecture.

For full release details, see the [Changelog](./CHANGELOG.md).

## 🚀 Key Accomplishments (v3.x Series):
*   **Tiered Memory Stack (L1-L4):** A high-performance cognitive backbone featuring Redis (Hot), SQLite (Episodic/Procedural), and Qdrant (Semantic) tiers.
*   **MCP Tool Registry:** Dynamic registration and invocation of WASM-based MCP tools (Skills).
*   **Infrastructure Resilience:** Graceful gRPC degradation for `koad-agent boot` (Dark Mode) and hardened systemd services.
*   **Automated Code Review (ACR):** Multi-stage audit pipeline ensures RUST_CANON and Zero-Panic compliance.
*   **Sovereign Identities:** Established persistent vaults (KAPVs) and working memory for the core agent team (Tyr, Clyde, Cid, Scribe, Helm).

## 🛠 Architecture

KoadOS operates on a **Hybrid Architecture** designed for high performance and isolation:
1.  **Brain (The Citadel):** Running in **Docker**, the Citadel Agent Support System (CASS) provides memory (Redis, Qdrant, SQLite), cognition, and context management.
2.  **Body (The Host):** Running natively on your **Host machine**, the CLI and agents interact with your local filesystem, compilers, and development tools.
3.  **Link (gRPC/Unix Sockets):** The high-speed control plane orchestrating communication between the Brain and Body.

## 📦 Getting Started

### Prerequisites:
*   **Docker Desktop** (Must be running)
*   Rust (Latest Stable)
*   Ollama (For local model inference: Llama 3.2, Qwen 2.5 Coder)

### Installation:
```bash
# Clone the repository
git clone https://github.com/Fryymann/koad-os
cd koad-os

# Run the installation script (handles Docker containers and host dependencies)
./install.sh
```

### Initialization:
```bash
# Initialize the KoadOS environment and verify integrity
./koad-init.sh
```

### Booting an Agent:
```bash
source ~/.koad-os/bin/koad-functions.sh
agent-boot <agent_name>
```

## 📜 The Canon
All development follows the **KoadOS Contributor Canon** and **RUST_CANON**. Methods are verified via KSRP (KoadOS Self-Review Protocol) and PSRP (Post-Session Reflection Protocol).

---
*Build small. Waste nothing. Know more. Trust the Canon.*
