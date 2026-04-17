# KoadOS — The Agent Operating System

KoadOS is a local-first, model-agnostic agent operating system designed to turn open-source AI models into a self-improving swarm of specialized software development agents. 

**The Agents are the product.** Everything in this repository exists to support their ability to perform high-level engineering tasks with persistent memory, active context recall, and minimal token consumption.

## 🛡 Stable Release: v3.2.0 "Citadel Integrity"

The v3.2.0 release establishes the **Hybrid Citadel Architecture**:
- **The Brain (Docker):** CASS, Redis, and Qdrant isolated for stability.
- **The Body (Host):** Native CLI and Agent Runtime for full system access.
- **The Crew (Local):** Specialized agent identities (Tyr, Clyde, etc.) that live in your Citadel but remain private to you.

### Key Highlights:
- **Graph-Centric Navigation**: Real-time workspace awareness via `code-review-graph`.
- **Zero-Touch Infrastructure**: One-command setup for the entire backend stack.
- **Persistent Memory (L1-L4)**: Compounds knowledge across every session.

---

## 🏗 Architecture: The Agent Lifecycle

Developers use KoadOS not to develop the OS, but to **build and command an Agent Crew**.

1.  **Engine:** The core Rust crates and CASS gRPC services (CASS = Citadel Agent Support System).
2.  **Citadel:** Your local instance of KoadOS.
3.  **Crew:** The specialized agents (e.g., Captain, Engineer, Scribe) you initialize to manage your projects.
4.  **Stations & Outposts:** The Project Hubs and individual projects developed by your agents.

---

## 🚀 Getting Started

### Prerequisites:
*   **Rust** (Latest Stable)
*   **Docker Desktop** (Running)
*   **Python 3 & PIPX** (For graph tools)

### 1. Installation
```bash
# Clone the monorepo
git clone git@github.com-fryymann:Fryymann/koad-os ~/koados-citadel
cd ~/koados-citadel

# Build backend infrastructure and host binaries
./install.sh
```

### 2. Initialization ("The Great Awakening")
```bash
# Set up your Citadel name and initial Captain agent
./koad-init.sh
```

### 3. Commanding the Crew
```bash
# Add KoadOS to your shell
source ~/.koad-os/bin/koad-functions.sh

# Boot your Captain to begin operations
agent-boot captain
```

---

## 📜 The Canon
All development follows the **KoadOS Contributor Canon** and **RUST_CANON**. Methods are verified via KSRP (KoadOS Self-Review Protocol) and PSRP (Post-Session Reflection Protocol).

---
*Build small. Waste nothing. Know more. Trust the Canon.*
