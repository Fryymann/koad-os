# Design Deep Dive — Sweep 09: The Agent Chassis (Docking Protocol)

> [!IMPORTANT]
> **Status:** PLAN MODE (Lifecycle & State Machines)
> **Goal:** Design a robust, atomic lifecycle for Koad Agents. Ensure that "Docking" a model into the Citadel follows a strict sequence to prevent identity collisions, state leakage, and stale leases.

---

## 1. The "Docking Station" Philosophy
As established in the Noti Draft, KoadOS is the **Dock**. The AI model (Gemini, etc.) is raw intelligence that "clicks" into the Station. 
- The **Agent Chassis** is the structural interface that provides the model with its identity, memory, and tools.

## 2. The Agent State Machine
We are moving away from binary "Wake/Dark" states to a formal state machine managed in Redis.

### **States:**
1.  **DORMANT:** Identity exists in SQLite but has no active Redis lease.
2.  **DOCKING:** `koad boot` initiated. Redis lease acquisition in progress.
3.  **HYDRATING:** Lease acquired. Spine is assembling the "Context Package" (Identity + Memory + Live Awareness).
4.  **ACTIVE (WAKE):** Model is hydrated and ready for commands. Heartbeat pulse is stable.
5.  **WORKING:** Agent is currently executing a tool or generating code.
6.  **DARK:** Heartbeat missed (>30s). Session is preserved but marked as unresponsive.
7.  **TEARDOWN:** Manual exit or timeout. Lock is released, state is drained to SQLite, and worktrees are cleaned.

## 3. The "Cold Boot" vs. "Re-Attach" Sequence
To solve the `IDENTITY_LOCKED` bug, the Chassis must handle existing leases intelligently.

### **The "Docking" Logic:**
- **Request:** Agent `Tyr` requests to boot.
- **Check:** Spine checks `koad:identities:leases` in Redis.
- **Path A (New):** No lease found. Proceed to `DOCKING`.
- **Path B (Existing):** Lease found.
    - If `status == DARK`: The Spine assumes a crash. It performs an **Atomic Reset** (nukes the old session entry) and allows the new boot.
    - If `status == ACTIVE`: The Spine rejects the boot with `IDENTITY_IN_USE` unless a `--force` flag is used (which triggers a remote "Kill Signal" to the old session).

## 4. The "Brain Drain" (Persistence Mandate)
Every `TEARDOWN` must be atomic. 
- **The Protocol:** An agent cannot release its identity lock until its "Session Memory" (the learnings from the current chat) has been successfully written to the `knowledge` table in SQLite.
- **Safety:** This ensures we never "forget" a lesson just because a terminal window was closed.

## 5. The Adapter Layer (Universal Ports)
The Chassis defines a standard **Dock Port Contract**. 
- **Input:** A standardized Protobuf `HydratedContext` package.
- **Output:** Standardized `ToolCall` and `Event` emissions.
- **Adapters:** We will build specific adapters for `GeminiCLI`, `ClaudeCode`, and `Ollama` that translate this chassis data into the provider's specific format.

---

## **Refined Implementation Strategy (v5.0)**
1.  **Chassis Module:** Implement the `AgentChassis` struct in the Spine Engine to manage these state transitions.
2.  **Redis State Guards:** Use Lua scripts in Redis to ensure lease acquisition and status updates are atomic (preventing race conditions between Tyr and Sky).
3.  **Heartbeat Refactor:** Move heartbeat logic into a dedicated background thread in the CLI binary to ensure it persists even during long-running tool calls.
4.  **The "Kill Signal":** Implement a gRPC broadcast that can tell an active agent to "Eject" if its identity is re-leased elsewhere.

---
## **Conclusion of PLAN MODE**
With this sweep, the v5.0 architecture is fully mapped across all five systems:
1.  **Dock Runtime** (Stateless Spine, Redis Authority).
2.  **Agent Chassis** (The Docking Protocol).
3.  **Workspace Manager** (Git Worktrees).
4.  **Crew System** (Hierarchy & Trace IDs).
5.  **Tool Layer** (Surgical Parsers & Sharp Tools).

---
*Next Step: Final Review by the Admiral before the v5.0 Sprint initialization.*
