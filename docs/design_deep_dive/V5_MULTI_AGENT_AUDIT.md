# v5.0 Multi-Agent Orchestration Audit

> [!IMPORTANT]
> **Status:** ARCHITECTURAL VERIFICATION
> **Goal:** Evaluate the v5.0 design as a heterogeneous multi-agent platform supporting named crew (Tyr, Sky), micro-agents (Ollama), and the human Admiral.

---

## 1. The Heterogeneous Identity Model
*   **The Intent:** Support a hierarchy of agents with varying authorization tiers and behavioral roles.
*   **The Design Fix:** The v5.0 `AgentSession` Protobuf message explicitly separates `Identity` (who they are) from `Rank` (what they can do) and `ModelTier` (how much they cost).
*   **Verdict:** **SUCCESS.** By moving these into native Protobuf enums rather than loose strings, the Spine can enforce role-based access control (RBAC) at the gRPC boundary. Tyr (Admiral-rank) can call `system` tools; Sky (Officer-rank) is restricted to the `SLE` domain.

## 2. Model Agnosticism (The Docking Adapter)
*   **The Intent:** Swap underlying LLMs (Gemini, Claude, Ollama) without losing the agent's persona or memory.
*   **The Design Fix:** The `Agent Chassis` acts as a **Normalization Layer**. It pulls memories from SQLite and status from Redis, then passes a standardized `ContextPackage` to a specific **Adapter**.
*   **Verdict:** **SUCCESS.** The "Intelligence" is treated as a modular component that "clicks" into the Chassis. This allows us to use Gemini for complex reasoning (Tyr) while using a local Ollama model for cheap, low-latency station housekeeping (Micro-agents).

## 3. Parallelism & Resource Contention
*   **The Intent:** Allow Tyr, Sky, and a Micro-agent to work simultaneously without breaking the system.
*   **The Design Fix:** 
    1.  **Git Worktrees:** Each agent gets a private physical directory. No "file-in-use" errors.
    2.  **Redis Event Bus:** Agents don't talk directly to each other; they emit events. Sky doesn't wait for Apex; she listens for the `task:completed` event.
    3.  **Atomic Leases:** Redis Lua scripts prevent two models from "docking" into the same agent persona simultaneously.
*   **Verdict:** **STABLE.** The move to Redis Streams ensures that awareness propagates across all agents in real-time without the Spine becoming a bottleneck.

## 4. The Micro-Agent Efficiency Loop
*   **The Intent:** Use small, local models for repetitive background tasks (Issue construction, log summarization).
*   **The Design Fix:** The `Signal Corps` and `Idea Pipeline` are designed specifically for micro-agents. 
    *   They subscribe to specific low-priority streams (e.g., `koad:stream:raw_logs`).
    *   They emit "Summary Packets" back into the main event bus.
*   **Verdict:** **HIGH SIGNAL.** This design ensures that high-tier agents (Tyr/Sky) stay in a "Flow State" by only consuming pre-processed, high-signal data from the micro-agents.

## 5. Global Awareness & Cross-Agent Memory
*   **The Intent:** If Sky learns something about the SLE, Tyr should know it immediately.
*   **The Design Fix:** The **Knowledge Archive (SQLite)** is the shared brain. The **Signal Corps** is the shared nervous system.
    *   When Sky commits a learning, it emits a `knowledge:new` event.
    *   The Spine detects this and can proactively "Hydrate" Tyr's active context buffer if the learning is relevant to his current project.
*   **Verdict:** **EVOLUTIONARY.** This creates a "Collective Mind" where the station gets smarter as a whole, regardless of which agent is currently wake.

---

## **Architect's Summary**
The v5.0 design transforms KoadOS from a "personal CLI" into a **True Multi-Agent Operating System**. The separation of the **Chassis** (State/Identity) from the **Engine** (LLM) is the key breakthrough that allows us to scale the crew from 1 to 10+ agents without increasing architectural complexity.

*Signed, Captain Tyr.*
