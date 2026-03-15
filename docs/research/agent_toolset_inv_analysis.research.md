<aside>
🎯

**Purpose:** Pre-work for Tyr and Claude Code's Citadel/CASS build sprint. This document inventories every tool agents need, identifies gaps, and prioritizes by token reduction and cognitive offload potential — so builders can execute, not discover.

Prepared by Noti · March 2026

</aside>

---

## How to Read This Document

Tools are organized in three tiers:

- **Validated Concepts (Reimplement)** — patterns proven in the Spine prototype that should be redesigned natively inside the Citadel. No legacy code is carried forward.
- **Custom Build Required** — gaps where no existing solution fits; these must be designed and built as part of Citadel/CASS.
- **Priority Matrix** — all tools ranked by token reduction potential × implementation effort.

Each tool entry includes its **cognitive offload category** — the type of non-reasoning work it removes from the model's context window.

---

# Part 1 — Validated Concepts (Reimplement in Citadel)

The legacy Spine prototype validated these concepts. They are **not** being ported or integrated — they are being **redesigned from scratch** inside the new Citadel architecture. This section documents *what worked conceptually* so the new implementation doesn't start from zero understanding, only zero code.

---

## 1.1 Identity & Session Infrastructure

| **Concept** | **Validated By** | **Offload Category** | **Citadel Redesign Notes** |
| --- | --- | --- | --- |
| **Atomic Lease System** | Spine prototype (Redis Lua scripts) | Session collision prevention | Reimplement with rank-based authorization from the start — no hardcoded sovereign names |
| **TOML Identity Registry** | Spine prototype (config glob pattern) | Agent identity hydration | Redesign with interface TOML layer for tool/driver bootstraps per agent baked in from day one |
| **KAILease + Heartbeat** | Spine prototype (lease struct + RPC) | Session lifecycle management | Build natively into Citadel gRPC with per-identity `session_policy` support from the start |
| **Body/Ghost Enforcement** | Spine prototype (session ID check) | Prevents duplicate agent consciousness | Enforce at Citadel Control Plane level — not split across CLI + Redis layers |

---

## 1.2 Storage & Memory Infrastructure

| **Concept** | **Validated By** | **Offload Category** | **Citadel Redesign Notes** |
| --- | --- | --- | --- |
| **StorageBridge (Redis ↔ SQLite)** | Spine prototype (30s drain loop) | Durable state persistence | Rebuild with configurable drain interval + drain-on-demand RPC as first-class features |
| **Cognitive Isolation (per-agent partitions)** | Spine prototype (`source_agent` partitioning) | Agent memory isolation | Design into Citadel's storage schema from the start — not bolted on |
| **CIP (Cognitive Integrity Protocol)** | Spine prototype (Tier >1 write restrictions) | Memory governance / trust boundary | Enforce Sanctuary Rule at the Citadel gRPC layer — not just the storage layer (fixes Vigil finding) |
| **FactCard Memory Banks** | Spine prototype (SQLite → Redis hydration) | Persistent knowledge retrieval | Rebuild with structured query interface from day one — no raw key access |

---

## 1.3 Monitoring & Health

| **Concept** | **Validated By** | **Offload Category** | **Citadel Redesign Notes** |
| --- | --- | --- | --- |
| **ASM (Agent Session Manager)** | Spine prototype (session pruning + deadman) | Autonomous session lifecycle | Rebuild with telemetry emission and Koad Stream integration as native features |
| **Watchdog** | Spine prototype (liveness checks) | System self-healing | Redesign to target Citadel + CASS services; add configurable restart policies from the start |
| **Deadman Switch** | Spine prototype (emergency drain on heartbeat timeout) | Crash resilience | Build into Citadel session lifecycle natively — not a separate bolt-on |

---

## 1.4 External Integrations

| **Concept** | **Validated By** | **Offload Category** | **Citadel Redesign Notes** |
| --- | --- | --- | --- |
| **GitHub Board Integration** | Spine prototype (`koad-board` crate) | Issue/board management without LLM turns | Rebuild as a Citadel-native service exposed via gRPC — not a standalone library |
| **Notion Bridge** | Spine prototype (`koad-bridge-notion` crate) | Notion read/write without agent context burn | Rebuild as a CASS service with Koad Stream read/write as a first-class feature |
| **Identity-Aware Sandbox** | Spine prototype (command auth via Identity struct) | Safe command execution | Redesign with externalized config from the start — no hardcoded blacklists (fixes Vigil finding) |

---

## 1.5 Ecosystem Crates (Off-the-Shelf)

| **Crate / Tool** | **Purpose** | **Offload Category** | **Integration Notes** |
| --- | --- | --- | --- |
| `tree-sitter`  • language grammars | AST parsing for code knowledge graph | Codebase navigation without file reads | Foundation for Code Knowledge Graph (Phase 3 tool) |
| `qdrant-client` | Vector similarity search | Semantic memory retrieval + semantic caching | Already in planned CASS stack; needs client integration |
| `tower` middleware | gRPC middleware (auth, rate-limiting, metrics) | Request governance at transport layer | Layer onto Citadel gRPC service |
| `insta` / `proptest` | Snapshot + property-based testing | Automated quality gates (reduces KSRP manual load) | Add to all crate `dev-dependencies` |
| `cargo-nextest` | Fast parallel test runner | Faster CI feedback loops | Install as workspace test runner |

---

# Part 2 — Custom Tools to Build (Gaps)

These do not exist and cannot be bought. They are the core deliverables of the Citadel/CASS build sprint.

---

## 2.1 Context Engineering Tools

| **Tool** | **What It Does** | **Offload Category** | **Token Reduction** | **Citadel Component** |
| --- | --- | --- | --- | --- |
| **Structured Doc Hierarchy Generator** | Auto-generates 3-layer navigation index (repo → domain → reference) so agents find relevant code in 1–3 lookups instead of 20 | Codebase orientation | 30–40% | Standalone CLI tool / CASS utility |
| **Context Compaction Service** | Periodically compresses conversation history into structured summaries. Replaces raw chat history with dense state objects. | Conversation history bloat | 60–80% | CASS — StorageBridge extension |
| **EndOfWatch (EoW) Writer** | Auto-generates structured session handoff docs at session close. Enforced schema (TOML/JSON frontmatter + markdown body). Flash-Lite powered. | Session re-discovery | Saves 10–50k tokens/session | CASS — EndOfWatch protocol |
| **Temporal Context Hydrator (TCH)** | On agent boot, selectively loads relevant memories, recent EoW summaries, and active task context — not everything. | Boot-time context overload | 50–70% of boot context | CASS — Core hydration pipeline |

---

## 2.2 Intelligent Routing & Caching Tools

| **Tool** | **What It Does** | **Offload Category** | **Token Reduction** | **Citadel Component** |
| --- | --- | --- | --- | --- |
| **Model Router / Dispatcher** | Flash-Lite classifies incoming task → routes to appropriate model tier (Flash-Lite / Flash / Pro / local). Prevents Pro-tier waste on simple tasks. | Model selection | 30–60% | Citadel Control Plane (gRPC dispatcher) |
| **Prompt Cache Orchestrator** | Manages static prompt segments (Canon rules, identity config, tool defs) to maximize provider-level prompt caching. Ensures static content is placed at prompt start. | Repeated system prompt tokenization | 45–80% | Citadel gRPC Control Plane |
| **Semantic Cache Layer** | Stores vector embeddings of queries + responses in Qdrant. Returns cached answer for semantically similar queries (cosine threshold 0.85–0.95). Cache hit = zero LLM tokens. | Repeated/similar queries | Up to 90% on cache hits | CASS — Qdrant integration |
| **Dynamic Tool Loader** | Loads only task-relevant MCP tool definitions per request instead of all tools upfront. Reduces per-turn tool overhead from 30k to 2–5k tokens. | Tool definition bloat | 50–70% on tool overhead | Citadel MCP integration layer |

---

## 2.3 Agent Workspace & Execution Tools

| **Tool** | **What It Does** | **Offload Category** | **Token Reduction** | **Citadel Component** |
| --- | --- | --- | --- | --- |
| **Workspace Manager (Git Worktrees)** | Provides each agent a physically isolated working directory via `git worktree`. Prevents merge conflicts and file contention between parallel agents. | Parallel execution safety | Prevents retry/conflict waste | Citadel Control Plane |
| **Code Knowledge Graph** | Tree-sitter → SQLite graph of functions, call chains, imports, routes. Agent queries the graph instead of grepping/reading files. | Codebase exploration | 65–97% | New crate (`koad-codegraph`) or MCP server |
| **MCP Code Execution Sandbox** | Agent writes data-processing code that executes locally; only the summary/result enters context — not raw data. | Large data processing in context | Massive (millions → thousands) | Citadel sandbox (extends `sandbox.rs`) |
| **Anti-Pattern Detector** | Passive monitoring: watches for file thrashing, dead-end exploration loops, excessive reads. Alerts agent or auto-triggers compaction. | Waste pattern prevention | Prevents 2–3x waste | CASS + koad-watchdog |

---

## 2.4 Governance & Communication Tools

| **Tool** | **What It Does** | **Offload Category** | **Token Reduction** | **Citadel Component** |
| --- | --- | --- | --- | --- |
| **Canon Enforcer** | Validates agent actions against Canon rules deterministically (ticket linkage, approval gate checks, KSRP pass requirements). Removes Canon compliance from LLM reasoning. | Governance reasoning | Moderate (removes multi-step Canon checks from context) | Citadel Control Plane |
| **Koad Stream Agent Bridge** | Native read/write interface for agents to the Koad Stream message bus (Notion DB). Replaces manual relay by Ian. | Inter-agent communication | Eliminates relay latency + human bottleneck | CASS — via `koad-bridge-notion` |
| **KSRP Automation Harness** | Orchestrates the 7-pass self-review protocol deterministically. Runs lint, verify, harden passes via `cargo clippy`, `cargo test`, `cargo audit` — model only handles passes requiring judgment (inspect, architect). | Mechanical review passes | Eliminates 3–4 of 7 KSRP passes from LLM | CASS utility / CLI integration |
| **PSRP / Saveup Writer** | Structured Saveup generation with enforced schema. Flash-Lite extracts Fact/Learn/Ponder from session context. Appends to Memory Bank. | Reflection formatting | Moderate | CASS — EndOfWatch extension |

---

# Part 3 — Priority Matrix

All tools ranked by **token reduction potential × implementation effort**, grouped into build phases.

<aside>
📐

**Reading the matrix:** P0 = build immediately (highest ROI, lowest dependency). P1 = build once Citadel core is stable. P2 = build once CASS core is running. P3 = build once agent tool layer is proven.

</aside>

| **Priority** | **Tool** | **Token Reduction** | **Effort** | **Depends On** |
| --- | --- | --- | --- | --- |
| **P0** | Structured Doc Hierarchy Generator | 30–40% | Days | Nothing (file-based) |
| **P0** | EndOfWatch Writer (manual schema first) | 10–50k tokens/session | Days | Nothing (file-based format spec) |
| **P1** | Model Router / Dispatcher | 30–60% | 1–2 weeks | Citadel gRPC Control Plane |
| **P1** | KSRP Automation Harness | Eliminates 3–4 LLM passes | 1 week | Stable cargo workspace |
| **P1** | Prompt Cache Orchestrator | 45–80% | 1 week | Citadel gRPC layer |
| **P2** | Context Compaction Service | 60–80% | 2 weeks | CASS StorageBridge |
| **P2** | Temporal Context Hydrator | 50–70% boot context | 2 weeks | CASS core + EoW schema |
| **P2** | Semantic Cache Layer | Up to 90% on hits | 2 weeks | Qdrant integration in CASS |
| **P2** | Koad Stream Agent Bridge | Eliminates relay bottleneck | 1 week | CASS + `koad-bridge-notion` |
| **P2** | Canon Enforcer | Moderate | 1–2 weeks | Citadel Control Plane |
| **P3** | Code Knowledge Graph | 65–97% | 3–4 weeks | Stable CASS + tree-sitter integration |
| **P3** | Dynamic Tool Loader | 50–70% tool overhead | 2 weeks | Stable Citadel MCP integration |
| **P3** | MCP Code Execution Sandbox | Massive | 2–3 weeks | Sandbox security hardening |
| **P3** | Workspace Manager (Worktrees) | Prevents conflict waste | 1–2 weeks | Citadel Control Plane |
| **P3** | Anti-Pattern Detector | Prevents 2–3x waste | 2 weeks | CASS + Watchdog telemetry |
| **P3** | PSRP / Saveup Writer | Moderate | 1 week | EoW schema + Memory Bank interface |

---

# Part 4 — Key Insight Summary

<aside>
💡

**The single highest-impact intervention is context engineering** — not smarter models or more tools. The research shows 80%+ of agent tokens are burned on orientation, history re-injection, and tool definition bloat. The top 4 tools by impact (Doc Hierarchy, Context Compaction, Prompt Caching, Semantic Caching) are all context engineering tools.

</aside>

### Where Tokens Die Today

1. **Codebase exploration without a map** — 40–80% of context spent orienting
2. **Full conversation history re-injection** — grows linearly, dominates after 15 turns
3. **Tool definition bloat** — 15–30k tokens of MCP schemas repeated every turn
4. **Session discontinuity** — every session re-discovers what the last one already knew

### What This Means for Build Order

- **Phase 0 tools (P0)** require zero Citadel infrastructure — they're file-based and can start today
- **Phase 1 tools (P1)** require only the Citadel gRPC skeleton — not full CASS
- **Phase 2 tools (P2)** require CASS core (StorageBridge + Qdrant + EoW pipeline)
- **Phase 3 tools (P3)** require a stable, proven agent tool layer

This dependency chain directly informs the **Citadel Stability Roadmap** — which defines the minimal viable foundation that unlocks each tool phase.