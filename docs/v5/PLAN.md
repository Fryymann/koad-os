# KoadOS v5.0 Implementation Plan: Local Micro-Agent Swarm

This document outlines the transition of KoadOS into a tiered **Agentic Swarm** architecture for **v5.0**. This plan prioritizes local execution, deterministic output, and seamless integration with the existing Koad Spine.

---

## Phase 1: Local Intelligence Infrastructure (The "Local Brain")
Establish the core connectivity and resource management for local models.
*   **1.1. `koad-ollama` Driver**: Develop a dedicated Rust crate to interface with the Ollama API.
    *   Implement connection health checks and automatic model pulling (Gemma 3, Qwen 3).
    *   Create a "Model Manager" to handle hot-swapping models to manage VRAM usage.
*   **1.2. Micro-Agent SDK (Python/Rust)**: Provide standardized libraries for creating micro-agents.
    *   **Rust**: Macro-based task definitions for high-speed enforcement.
    *   **Python**: Pydantic-integrated templates for structured JSON extraction.

## Phase 2: Communication & Intent Routing (The "Nervous System")
Expand the Spine to support sub-cognitive task dispatching.
*   **2.1. `MicroTask` Intent Expansion**: Add `Intent::MicroTask(MicroTaskIntent)` to `koad-core`.
    *   Fields: `task_id`, `model_target`, `schema_requirement`, `payload`.
*   **2.2. Dedicated Micro-Bus**: Initialize a new Redis Stream `koad:micro:tasks` for high-frequency, low-latency micro-agent traffic.
*   **2.3. Reflex Routing**: Update the `DirectiveRouter` to recognize tasks that should be intercepted by local models before reaching a cognitive agent.

## Phase 3: Protocols & Schema Enforcement (The "Language")
Ensure that micro-agents remain deterministic and never "hallucinate" conversational text.
*   **3.1. Zero-Text Protocol**: Force all micro-agent responses to be strictly typed JSON.
*   **3.2. Validator Middleware**: Implement a kernel-level validator that checks micro-agent output against the requested schema. If validation fails, the task is automatically re-queued or escalated.
*   **3.3. Task State Ledger**: Store micro-task results in a dedicated `micro_executions` SQLite table for long-term auditing and model performance tracking.

## Phase 4: Initial Micro-Agent Archetypes (The "Reflexes")
Deploy the first generation of specialized local agents.
*   **4.1. The Git Sentinel**: A micro-agent that runs on every `git commit`. It uses a local model to verify commit message compliance and code style.
*   **4.2. The Entity Extractor**: Automatically tags and categorizes all entries sent to `koad remember`.
*   **4.3. The Triage Officer**: Scans incoming GitHub Issues and local `SESSION_LOG.md` entries to prioritize the next task for the cognitive agent.
*   **4.4. The Doc Generator**: A background process that updates `MAP.md` and `README.md` based on code changes detected by the Booster.

## Phase 5: ASM & KCM Integration (The "Orchestration")
Fold micro-agents into the core lifecycle of KoadOS.
*   **5.1. Smart Hydration**: Integrate the Triage micro-agent into the **Agent Session Manager**. When an agent boots, the micro-agent assembles the "Intelligence Package" (Active tasks, recent changes, relevant facts).
*   **5.2. Autonomous KCM**: Transition the **Koad Compliance Manager** to a model-driven system. Instead of regex patterns, KCM uses micro-agents to perform content-aware audits of the repository.
*   **5.3. Admin "Morning Report" Expansion**: Use micro-agents to synthesize all telemetry into a dense, actionable summary for the Admin's Command Deck.

---

### v5.0 Success Criteria
1.  **Latency**: Micro-tasks complete in <500ms locally.
2.  **Cost**: Zero API token usage for all validation, extraction, and formatting tasks.
3.  **Stability**: 100% schema compliance for all micro-agent outputs.
4.  **Autonomy**: The Project Board stays synced based on real-time git activity without manual "Done" updates.
