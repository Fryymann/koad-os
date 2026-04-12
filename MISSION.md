## The Problem

Today's AI coding tools — Claude Code, aider, OpenCode, Cursor, Open WebUI — share a common architecture: **one model, one context, one session.** They give a developer a smart autocomplete partner, but the moment the session ends, the intelligence resets. There is no memory, no growth, no coordination between agents, and no governance over what the AI actually does. They are powerful but disposable.

They're also wasteful. Every session re-discovers what the last session already knew. Every turn dumps the full context window into the model and lets it figure things out from scratch — file discovery, schema lookups, routing decisions, formatting. That's expensive on cloud APIs and brutally slow on local hardware, where every token costs real compute time. The models spend most of their tokens on work that isn't thinking.

Meanwhile, the models themselves are getting smaller, faster, and cheaper to run locally. A 4B parameter model on a laptop can now do work that required a cloud API two years ago. A 14B model on a desktop GPU runs at 50+ tokens per second. The inference layer is solved — what's missing is everything above it.

## The Mission

**KoadOS is a local-first, model-agnostic agent operating system that turns open-source AI models into a self-improving swarm of specialized software development agents — with reasoning-only token budgets, externalized cognition, and compounding memory.**

Where existing tools wrap a single model in a single utility, KoadOS wraps *many* models in an orchestration layer that gives them:

- **One Body, One Ghost** — every KoadOS agent is the fusion of a *body* (the terminal shell, hydrated with environment data and tool access) and a *ghost* (the agent's persistent identity, memories, and cognition stored through the Citadel). A session begins when `koad-agent boot <agent>` tethers ghost to body — the model gains not just tools but *who it is* and *what it knows*. Memory isn't bolted on; it's half the agent
- **Specialized roles** — each agent is tuned for a specific domain (dev, ops, docs, review)
- **Coordination protocols** — agents delegate to each other, share context, and escalate blockers
- **Governance and quality gates** — every action follows the Canon; no agent runs unsupervised
- **Cognition offloading** — everything that *isn't* reasoning is handled outside the model's context. File discovery, schema lookups, template formatting, routing decisions — the harness and Citadel handle these deterministically so the model only spends tokens on *actual thinking*
- **Self-improvement loops** — agents compound their effectiveness through structured reflection, accumulated knowledge, and feedback from other agents and developers. The base model doesn't change — the system around it gets smarter

## The Architecture

### Layer 1 — Inference (The Models)

Ollama serves local open-source models (Qwen3, Gemma 3, Qwen 2.5 Coder, Llama, and whatever comes next). Cloud APIs (Gemini, Codex, Claude) are available as overflow for tasks that exceed local capability. KoadOS is model-agnostic — models are swappable resources, not identities.

### Layer 2 — The Agent Harness (koad CLI)

The `koad` CLI is the bridge between models and the real world. It gives agents:

- Filesystem access (read, write, watch)
- Shell execution (build, test, deploy)
- Memory (SQLite, Redis, Qdrant)
- The Koad Stream message bus for inter-agent communication
- Tool integrations (GitHub, Notion, Airtable, Stripe, and more)

This is the layer where tools like Claude Code and aider stop — but KoadOS extends it to support *multiple concurrent agents* with distinct roles and shared infrastructure.

Critically, the harness **pre-processes and compresses context** before it ever hits the model. Instead of pasting a 500-line file into context, the harness extracts the relevant 30 lines. Instead of asking the model "what tool should I use?", the Citadel routes deterministically. The model receives *minimal, surgical context* — not a firehose. Every token the model processes is a token that required intelligence.

### Layer 3 — The Citadel (Always-On Orchestrator)

The Citadel is the beating heart of KoadOS. It is an always-on application that:

- **Monitors** — watches project state, repo health, CI pipelines, and agent activity
- **Coordinates** — routes tasks to the right agent based on role, capability, and current load
- **Supports** — provides shared context, memory retrieval, and escalation paths so agents aren't working blind
- **Governs** — enforces the Canon, approval gates, and quality protocols across all agent actions
- **Budgets** — tracks token usage per agent, enforces context window limits, and manages context lifecycle (summarize, retrieve, or re-prompt)

The Citadel doesn't do the work — it ensures the work gets done correctly, efficiently, and that no token is wasted getting there.

Supporting the Citadel is **CASS (Citadel Agent Support System)** — the module that provides agents with direct access to Citadel services: memory retrieval, context hydration, task routing, and escalation pathways. CASS is the interface through which each agent's ghost connects to the shared infrastructure, ensuring every agent operates with the right context at the right time without burning tokens to find it.

### Layer 4 — The Canon (Governance & Growth)

The Development Canon, KSRP, and PSRP aren't bureaucracy — they're the *learning infrastructure*. Every task an agent completes produces:

- A **KSRP Report** — structured self-review that catches errors before they ship
- A **Saveup Entry** — facts learned, lessons captured, reflections logged
- **Memory Bank updates** — durable knowledge that persists across sessions and informs future decisions

The Memory Bank is a **cognition cache**. Every Saveup entry is pre-computed knowledge that replaces tokens the model would otherwise spend re-deriving the same conclusions. A Saveup that says *"Next time X, do Y instead of Z"* saves an entire reasoning chain on the next encounter. Over hundreds of tasks, this creates a **compounding intelligence loop** — agents get faster, cheaper to run, and more effective. Not because the base model improves, but because the system accumulates operational knowledge and feeds it back as compressed, high-signal context. The agents grow more competent — and they help other agents and developers do the same.

## The Vision

<aside>
🎯

**KoadOS exists to prove that a well-governed swarm of small, local AI agents — with externalized cognition, reasoning-only token budgets, and compounding memory — can match or exceed the output of any single frontier model burning orders of magnitude more compute.**

</aside>

The bet is simple: **intelligence without governance is unreliable. Governance without intelligence is slow. Raw compute without efficiency is wasteful. KoadOS delivers all three — intelligent, governed, and lean.**

Small models, running locally, spending tokens only on real thinking, accumulating knowledge, improving continuously, coordinated by an always-on orchestrator, building software under a disciplined protocol — that is the mission.

## How KoadOS Differs

| **Capability** | **Claude Code / aider / Cursor** | **KoadOS** |
| --- | --- | --- |
| Model support | Single agent, provider-flexible | Model-agnostic, local-first, cloud overflow |
| Agent count | One agent per session | Swarm of specialized micro agents |
| Memory | Session-scoped (resets) | Persistent memory bank (SQLite/Redis/Qdrant) |
| Coordination | None | Koad Stream message bus + Citadel orchestrator |
| Quality control | Model-dependent (no structured review protocol) | Canon gates, KSRP self-review, approval protocol |
| Token efficiency | Full context every turn, no budget | Cognition offloading, context compression, deterministic routing |
| Self-improvement | None | PSRP reflection, Saveup cognition cache, compounding knowledge |
| Always-on | No (interactive only) | Citadel monitors and coordinates 24/7 |

---

## Current State vs. End Goal

<aside>
🚧

**This mission describes the full vision.** KoadOS is under active development and not all capabilities are realized yet. Key current-state realities:

- **Officer-rank agents and above** (Tyr, Scribe, Sky) currently require frontier models (Gemini, Claude, or Codex) due to the complexity of their coordination and reasoning tasks.
- **Local Ollama models** (Qwen, Gemma, Llama) are viable for Enlisted-rank micro-agents and targeted subtasks today — full local-first operation for all ranks is the end goal.
- **The Citadel** is in early build — orchestration, monitoring, and governance features are being stood up incrementally.
- **Phase 4 (COMPLETE):** Dynamic Tools & Containerized Sandboxes — externalizing tool execution and isolating environments.
- **Phase 5 (COMPLETE):** koad-agent MVP — Context Generation Engine and standalone boot/task management are live.
- **Phase 6 (COMPLETE):** Canon Lock — Documentation distillation and architectural stabilization.
- **Phase 7 (ACTIVE):** v3.2.0 Stable Release — "Citadel Integrity" push.
</aside>

---

*This is KoadOS. Build small. Waste nothing. Know more. Trust the Canon.*