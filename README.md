# KoadOS — The Agent Operating System

KoadOS is a local-first, model-agnostic agent operating system designed to turn open-source AI models into a self-improving swarm of specialized software development agents. It provides reasoning-only token budgets, externalized cognition, and compounding memory.

## 🚀 Phase 4 Milestone Release (v3.2.0)

This release marks a significant milestone in the KoadOS rebuild, stabilizing core infrastructure and introducing dynamic agent capabilities.

### Key Accomplishments:
*   **Tiered Memory Stack (L1-L4):** A high-performance cognitive backbone featuring:
    *   **L1 (Hot):** Redis-backed cache for active session context.
    *   **L2 (Episodic):** SQLite WAL for chronological event logging.
    *   **L3 (Semantic):** Qdrant-backed vector search for long-term deep recall.
    *   **L4 (Procedural):** SQLite for immutable skills and procedures.
*   **MCP Tool Registry:** Dynamic registration and invocation of WASM-based MCP tools (Skills) via `koad bridge skill`.
*   **Infrastructure Resilience:** Graceful gRPC degradation for `koad-agent boot` (Dark Mode) and hardened systemd units with pre-flight health checks.
*   **Automated Code Review (ACR):** Multi-stage audit pipeline (`koad review`) ensuring RUST_CANON and Zero-Panic compliance.
*   **Sovereign Identities:** Established persistent vaults (KAPVs) and working memory for the core agent team (Tyr, Clyde, Cid, Scribe, Helm).

## 🛠 Architecture

KoadOS operates on a tri-tier model:
1.  **Body (The CLI):** The terminal shell, hydrated with environment data and tool access.
2.  **Brain (CASS):** The Citadel Agent Support System providing memory, cognition, and context.
3.  **Link (gRPC):** The control plane orchestrating inter-agent signaling and system governance.

## 📦 Getting Started

### Prerequisites:
*   Rust (Latest Stable)
*   Redis Stack (Running on port 6379 or Unix Socket)
*   Docker (For Qdrant vector database)
*   Ollama (For local model inference: Llama 3.2, Qwen 2.5 Coder)

### Installation:
```bash
# Clone the repository
git clone https://github.com/Fryymann/koad-os
cd koad-os

# Install services and build binaries
./scripts/install-services.sh
cargo build --release
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
