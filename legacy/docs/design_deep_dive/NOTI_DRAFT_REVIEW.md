# Design Deep Dive — Noti Draft Review (Research & Information)

> [!IMPORTANT]
> **Context:** This is a review of `noti_draft_1.md`. These are not instructions, but observations and potential design patterns identified by Noti for consideration in the v5.0 architecture.

---

## 1. High-Signal Concepts to Adopt

### **A. The "Docking Station" Metaphor**
Noti's definition of KoadOS as the **"Dock"** rather than the **"Brain"** is superior to our current "Ship" or "Citadel" definitions.
- **Why it works:** It clarifies that the AI Model (Gemini/Claude) is raw intelligence, and KoadOS provides the **Identity, Memory, Tools, and Awareness**. 
- **Design Intent:** We should design the v5.0 `AgentChassis` to explicitly "dock" a model, injecting these four pillars as a standard context package.

### **B. Token Austerity (The "Anti-MCP" Tool Layer)**
Noti identifies a critical flaw: generic tools are "token sinks."
- **Proposed Pattern:** **"Sharp Tools"** that are project-tuned. Instead of `read_file`, a tool like `get_module_context("auth")` returns a pre-formatted, compressed bundle of code, tests, and issues.
- **Design Intent:** This aligns with our "Surgical Parser" goal. We should design tools that do more per invocation to minimize round-trips and token waste.

### **C. The Event Bus (The Nervous System)**
Noti proposes an architecture where everything is an event (`emitter/listener`).
- **Why it works:** It solves our "Active Diagnostics" problem. If a Stripe webhook fails (SCE), it emits an event. The Spine (Signal Corps) listens and propagates that awareness to the Admiral's TUI and Sky's orientation.
- **Design Intent:** v5.0 should replace direct polling with a **Redis Streams-based Event Bus**.

---

## 2. Structural Insights: The "Five Systems"

Noti identifies five distinct systems that mirror our "Four Angles" but add a key missing piece: **Git Orchestration.**

1.  **Dock Runtime:** (Our Angle 1/2) — The always-on Rust process.
2.  **Agent Chassis:** (Our Identity Model) — The boot/hydration sequence.
3.  **Workspace Manager:** **(The New Piece)** — Proposes using **Git Worktrees** for agent isolation. This is a brilliant solution for supporting parallel agents (Koad and Sky) without path conflicts.
4.  **Crew System:** (Our Hierarchy) — Ranks and workflows (Lead, Developer, Specialist).
5.  **Tool Layer:** (Our Angle 3) — Pre-formatted, token-efficient responses.

---

## 3. The "Tyr" vs "Koad" Identity
Noti uses the name **"Tyr"** as the flagship persona. 
- **Observation:** While we use "Tyr," the concept of a **"Flagship Persona"** with the deepest memory and cross-project scope is a powerful way to define my (Tyr's) role relative to Sky. 
- **Design Intent:** Formalize "Captain" rank as the "Flagship Persona" with total-station awareness.

---

## 4. Conclusion & Integration
Noti's draft confirms our move toward a **Stateless, Event-Driven Architecture.** Her addition of the **Git Worktree Workspace Manager** is a critical design element we must think about for v5.0.

### **Integrated Plan Addition:**
- **Workspace Manager:** We need to design a system that spawns isolated worktrees for agents to prevent "Split-Brain" file edits.

---
*Next Action: Cross-reference Noti's "Event Bus" with our "Signal Corps" design in Sweep 07.*
