# KoadOS — The Frontier Agent Exoskeleton

KoadOS is a local-first capability layer designed to enhance frontier agent harnesses (Claude Code, Gemini CLI, Codex) with persistent memory, active workspace awareness, and compounding cognitive context.

**The Agents are the product.** KoadOS provides the infrastructure that allows frontier models to transcend the single-session limit, reducing token burn while increasing system-wide engineering accuracy.

## 🛡 Stable Release: v3.2.0 "Citadel Integrity"

The v3.2.0 release establishes the **Frontier Uplink Architecture**:
- **Capability Layer (Docker):** CASS (Citadel Agent Support System), Redis, and Qdrant provide the "Brain" enhancements.
- **Navigation Engine (Host):** `code-review-graph` provides real-time workspace topology to the agent.
- **Unified Harnessing:** Standardized environment and tool access across all supported agent CLIs.

---

## 🏗 Architecture: The Capability Stack

KoadOS works *with* your existing frontier agent tools:

1.  **Harness Layer:** You run `claude`, `gemini`, or `codex` as your primary interface.
2.  **KoadOS Layer (The Enhancer):**
    *   **Context:** `koad map` provides the agent with a high-fidelity "look" at the codebase via the knowledge graph.
    *   **Memory:** CASS records episodic and procedural facts, allowing the agent to "remember" previous sessions.
    *   **Governance:** The Citadel Kernel manages identities, permissions, and skill registration.

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
