# Design Deep Dive — Sweep 08: The Idea Pipeline & Event Routing

> [!IMPORTANT]
> **Status:** PLAN MODE (Intelligent Orchestration)
> **Goal:** Design the end-to-end flow for transforming raw human input into structured, labeled, and assigned GitHub issues. Leverage local micro-agents (Ollama) for low-latency, low-cost processing.

---

## 1. The Entry Point: The "Input Buffer"
The Admiral needs a frictionless way to dump ideas.
- **Web Deck:** A persistent "Think Tank" text area.
- **CLI:** `koad dispatch "idea text..."`
- **Mechanism:** Both interfaces emit a `dispatch:idea` event to the **Redis Event Bus**.

## 2. The Processor: Micro-Agent Intake (Ollama)
We will use a dedicated **Autonomous Intake Listener** (powered by a local 7B model like Qwen or Mistral via Ollama).

### **Intake Sequence:**
1. **Listen:** The Intake Listener detects a `dispatch:idea` event.
2. **Analyze:** The micro-agent queries the **Station Knowledge Archive** (SQLite) for current project maps and standards.
3. **Refine:** The agent transforms the raw text into a structured GitHub Issue payload:
    - **Title:** Action-oriented and concise.
    - **Body:** Markdown-formatted with "Problem" and "Intended Solution."
    - **Context:** Automatically identifies the relevant repository (e.g., `koad-os`, `skylinks-api`).
    - **Metadata:** Suggests labels (`v5.0`, `core`, `bug`) and priority.
4. **Emit:** The listener emits a `governance:issue-draft` event.

## 3. The Execution: GitHub Integration
A second listener (the **GitHub Connector**) handles the actual API interaction.
- **Role:** It takes the `governance:issue-draft`, performs a final validation against the repo's contribution rules, and executes the `gh issue create` command.
- **Handoff:** Once the issue is created, it emits `github:issue-created` with the new issue number.

## 4. The Awareness Loop: Signal Propagation
This is where the station "comes alive."
- **Dashboard:** The Web Deck hears `github:issue-created` and instantly adds the card to the visual board.
- **Chief Officer Awareness:** **Sky** (if active) receives a signal that a new issue has been added to her SLE domain. She can then decide to "auto-assign" if the priority is high.
- **Notification:** The **Signal Corps** formats a high-signal message for Slack or the TUI Status Bar: *"New Issue #118 [Core] created from Idea."*

## 5. Tracing the Idea
The `trace_id` established in Sweep 05 is critical here.
- The `trace_id` generated at the moment you type the idea is attached to the GitHub issue metadata. 
- You can run `koad dood inspect TRC-IDEA-123` to see the entire lifecycle: *"Idea Received -> Ollama Refined -> GitHub Created -> Sky Notified."*

---

## **Refined Implementation Strategy (v5.0)**
1.  **Event Schema:** Define the `DispatchIdea` Protobuf message.
2.  **Ollama Bridge:** Implement a standard `LocalModelClient` in `koad-core` to talk to the Ollama API.
3.  **Intake Daemon:** Create a lightweight background process (or Spine task) that maintains the "Intake" listener loop.
4.  **Prompt Engineering:** Develop the "Intake Prompt" that gives the micro-agent awareness of our labeling and repository structure.

---
*Next Sweep: The Agent Chassis State Machine (The Docking Protocol).*
