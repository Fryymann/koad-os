# 🛰 Research Report: KoadOS Agent Interop & Context Hydration

**Authors:** Tyr [Captain]
**Date:** March 9, 2026
**Target Milestone:** v4.1.x
**Status:** PROPOSED

## I. Concept: Agent-to-Agent "Signal" Protocol (A2A-S)

**Objective:** Implement a lightweight asynchronous communication layer between KAI Officers (Ghosts) to maintain momentum between sessions without manual "PM status reports."

### 🧠 Deep Dive: The Signal Pattern
To avoid siloed operations, we need a standard way for an agent (like Sky) to flag work for another (like Tyr).

#### 🛠 Proposed Mechanics:
1.  **Ghost Mailbox (Redis-backed):** Use Redis keys (`koad:mailbox:<agent_name>`) as an atomic store for short JSON status signals.
    *   *Payload:* `{ sender: "Sky", timestamp: "2026-03-09T...", priority: "HIGH", message: "Built SCRF, needs Airtable schema sync.", issue_ref: "#12" }`
2.  **Sentinel Integration:** On `koad boot`, the Sentinel process automatically hydrates the Ghost’s "Waiting Signals" into its active memory context.
3.  **The "Pulse" Command:** A new CLI verb `koad signal <agent_name> -m "message"` to push these status hints.

---

## II. Concept: Temporal Context Hydration (TCH)

**Objective:** Provide a mechanism for an agent to "request a memory dump" from the agent they are assisting or replacing.

### 🧠 Deep Dive: Knowledge Hydration
Currently, a "PM review" (like the one I performed for Sky) relies on reading files. TCH adds the **why** and **what was tried** by reading the *internal* state or past turn history of another Ghost.

#### 🛠 Proposed Mechanics:
1.  **Context-Request API:** Allow Agent B to query Agent A's SQLite personal memory partition for tags related to the current `project_id`.
    *   *Tool Command:* `koad hydrate --from <agent_name> --topic <topic_id>`
2.  **Snapshot Replay:** Extract the last 5-10 turns of the assisted agent's interaction into a "Hydration Report" provided as a read-only context block during boot.
3.  **Introspection Summaries:** Each agent session should auto-generate an "EndOfWatch" summary stored in a global `context/hydration/` directory, which future agents can use to skip research phases.

---

## III. Implementation Strategy & Actions
1.  **Phase 1:** Build the `koad signal` command (A2A-S Alpha).
2.  **Phase 2:** Implement the Sentinel boot-hook for automatic signal delivery.
3.  **Phase 3:** Develop the cross-partition SQLite reader for TCH.

*Status: Awaiting Dood Review.*
