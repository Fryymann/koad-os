## Overview

KoadOS should evolve beyond the static software paradigm — where systems are built once and only improve through manual developer intervention — into a **living, adaptive system** that continuously evaluates, heals, and improves itself using AI and agent infrastructure already native to its architecture.

This feature request formalizes the design intent for a **Self-Improving System (SIS)** layer within KoadOS.

---

## Problem Statement

Traditional software is **inert by default**. It does exactly what it was built to do, degrades silently under changing conditions, and requires a developer to notice, diagnose, and fix issues. This creates:

- **Operational drag** — human attention is required for every degradation event.
- **Slow iteration cycles** — improvements only happen when a developer prioritizes them.
- **Blind spots** — no system-level awareness of its own inefficiencies, drift, or technical debt.
- **Brittleness** — systems that cannot adapt to novel conditions break instead of bending.

KoadOS, built on a multi-agent OS foundation, is uniquely positioned to escape this paradigm.

---

## Proposed Solution

Introduce a **Self-Improving System (SIS)** layer composed of four core capabilities:

### 1. 🩺 Self-Healing

KoadOS agents should detect and recover from failures autonomously.

- **Fault detection** — Agents monitor runtime health metrics, error rates, and behavioral drift across Stations and Outposts.
- **Autonomous recovery** — On detected failure, agents attempt predefined recovery playbooks before escalating to humans.
- **Circuit-breaking** — Agents isolate degraded subsystems to prevent cascading failures across the Citadel.
- **Post-mortem logging** — All self-healing events are recorded to the Koad Stream for review and pattern analysis.

### 2. 📊 Continuous Self-Evaluation

KoadOS should know how well it is performing — not just whether it is running.

- **Performance benchmarking agents** — Dedicated agents run periodic evaluations against defined quality baselines (latency, accuracy, throughput, agent task success rate).
- **Behavioral drift detection** — Agents compare current behavior against canonical specs and flag deviations.
- **Regression surfacing** — Automatically identify when a recent change degraded a previously working subsystem.
- **Scoring & telemetry** — Surface scores per Station, per agent, and per pipeline for human review.

### 3. 🔁 Self-Improvement Loops

KoadOS should generate its own improvement proposals based on evaluation output.

- **Improvement agents (Refiners)** — A class of agent responsible for analyzing evaluation reports and generating ranked improvement proposals.
- **Proposal queue** — Proposals are queued to a structured backlog (Notion database or Koad Stream record), not acted on autonomously.
- **Human-in-the-loop gate** — Ian (or a designated approver) reviews proposals before implementation. Autonomous improvement is opt-in and scoped per subsystem.
- **Auto-patch mode (optional, sandboxed)** — For low-risk, well-bounded changes (e.g. prompt tuning, config adjustments), agents may apply changes in a sandboxed environment and surface results for approval.

### 4. ⚖️ Canon Warden

A dedicated agent responsible for continuously verifying that all KoadOS agents and subsystems remain aligned with the KoadOS Global Canon.

- **Runtime** — Powered by a lightweight local Ollama model. Runs on-device within the Citadel environment; no sovereign model required for routine checks.
- **Management** — Spawned and supervised by Citadel Core. The Citadel controls its lifecycle, scheduling, and resource allocation.
- **Scope** — Reads agent behavior logs, instruction diffs, and Koad Stream records; compares against the canonical ruleset.
- **Escalation protocol** — Violations or ambiguities that require human judgment or sovereign agent review are **elevated and written to `~/.tyr/inbox`** as structured escalation records. Tyr picks these up as part of its normal inbox processing cycle.
- **Escalation record schema** (written to `~/.tyr/inbox`):

```json
{
  "type": "canon_violation" | "canon_ambiguity" | "alignment_drift",
  "severity": "low" | "medium" | "high" | "critical",
  "subsystem": "<station or agent name>",
  "summary": "<one-line description>",
  "evidence": "<log snippet or diff>",
  "recommended_action": "<optional suggestion>",
  "timestamp": "<ISO-8601>"
}
```

- **Autonomy boundary** — The Canon Warden **never modifies** agent instructions or system state directly. It observes, evaluates, and escalates. All remediation is performed by Tyr or Ian.

### 5. 🧠 Institutional Memory & Pattern Learning

KoadOS should learn from its own history.

- **Failure pattern indexing** — All incidents, recoveries, and regressions are indexed and searchable.
- **Success pattern reinforcement** — Patterns that correlate with strong performance are identified and encoded into agent instructions or system defaults.
- **Cross-session memory** — Relevant context from prior sessions informs current agent behavior (already partially implemented via Effective Memories).
- **Canon alignment checks** — Agents periodically verify their behavior against KoadOS Global Canon and flag divergence.

---

## Architecture Fit

| SIS Capability | KoadOS Component |
| --- | --- |
| Fault detection & recovery | Station-level health agents + Citadel Core supervisor |
| Performance evaluation | Dedicated Refiner agent class |
| Improvement proposals | Koad Stream proposal records + Notion review queue |
| Pattern learning | CASS (Citadel Agent State Store) + Effective Memories |
| Canon alignment | Canon Warden (Ollama-powered, Citadel-managed; escalates to Tyr) |

---

## Design Principles

- **Human-in-the-loop by default.** Autonomous action is bounded and opt-in. The system surfaces proposals; humans approve them.
- **Canon-first.** Self-improvement proposals must comply with KoadOS Global Canon & Rules of Engagement. The system cannot propose changes that violate its own governance layer.
- **Observable.** Every SIS action — healing, evaluation, proposal — is logged to Koad Stream. Nothing is silent.
- **Incremental.** Ship self-healing first. Add evaluation. Add improvement loops. Do not try to build the full SIS in one sprint.
- **Sandboxed experimentation.** Autonomous patch application only in isolated environments, never directly to production Stations.

---

## Milestones

1. **M1 — Self-Healing Foundation**
    - Define health contract per Station
    - Build fault detection hooks into Citadel Core
    - Implement recovery playbook executor
    - Log all events to Koad Stream
2. **M2 — Evaluation Layer**
    - Define performance baselines per agent class
    - Build benchmarking agent (Refiner v0)
    - Surface behavioral drift reports
3. **M3 — Improvement Loop (Human-Gated)**
    - Build Refiner proposal queue in Notion
    - Define proposal schema (subsystem, issue, proposed fix, risk tier)
    - Wire Ian review → approval → implementation pathway
4. **M4 — Auto-Patch Sandbox (Optional)**
    - Define auto-patchable change types (prompt tuning, config, thresholds)
    - Build sandboxed apply + diff + approval flow

---

## Open Questions

- What is the canonical definition of "healthy" for each Station and agent class?
- How do Refiners get access to Koad Stream history without creating a feedback loop?
- What is the risk tier classification system for auto-patch eligibility?
- How does SIS interact with the Git Flow Specification — do improvement patches go through standard PR flow?

---

## References

- [**KoadOS Global Canon & Rules of Engagement**](https://www.notion.so/KoadOS-Global-Canon-Rules-of-Engagement-319fe8ecae8f8064a430c2a677a62188?pvs=21)
- [KoadOS — Git Flow Specification](https://www.notion.so/KoadOS-Git-Flow-Specification-1dd3d333382e46d69192281802f60ad3?pvs=21)
- [Koad Stream — Protocol](https://www.notion.so/Koad-Stream-Protocol-4ba0dafed6bb4526bda44680993b450f?pvs=21)
- [KoadOS Agent Reference Book](https://www.notion.so/KoadOS-Agent-Reference-Book-4be9a42f60684c15b93d7850d1588395?pvs=21)
- [Citadel Stability Roadmap — Minimal Viable Foundation](https://www.notion.so/Citadel-Stability-Roadmap-Minimal-Viable-Foundation-0276f93ce15141e4a6d8bf26b243777f?pvs=21)
- [Agent Toolset Inventory & Gap Analysis — Citadel Prep](https://www.notion.so/Agent-Toolset-Inventory-Gap-Analysis-Citadel-Prep-b08ab44ce2a6423aae8e2b2443eb6264?pvs=21)