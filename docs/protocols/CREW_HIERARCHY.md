# KoadOS — Crew Hierarchy

> [!NOTE]
> **Design canon for the KoadOS agent crew.**
> The **Koados** is a Citadel-class Space Station — the mothership and central intelligence hub of the fleet. Ranks are classifications, not individual agents. Agents are assigned a rank at instantiation.
> Repo: [Fryymann/koad-os @ nightly](https://github.com/Fryymann/koad-os/tree/nightly)

---

## Fleet Command
> The top of the chain. These are named roles, not rank classifications.

| Title | Holder | Role & Responsibility |
| :--- | :--- | :--- |
| **Admiral** | Ian (Dood) | Human principal and omnipotent authority over the entire KoadOS fleet. Operates from a different plane of existence — access to cyberspace is real but bounded by human nature. Interfaces with the crew and systems through tools, language, and delegation. Mission authority and final override on all decisions. Not an agent. |
| **Captain** | Tyr | Flagship Koad Agent. Commands the **Koados Citadel** (mothership station) directly. Co-architect of the station alongside the Admiral. Owns the CLI, the Delegation Stream, and executive agency across all Koados systems. No agent outranks Tyr except the Admiral. Principal orchestrator of the entire crew. |
| **Chief Officer** | Sky | **Chief Skylinks Officer**. Commands the **SLE (Skylinks Local Ecosystem)** — a forward-deployed station wired to the Koados. Sky is the central point for managing the **SCE (Skylinks Cloud Ecosystem)**, overseeing Airtable, WordPress, GCP, Stripe, Notion, and all Skylinks digital presence. **Criticality: High.** The SCE is a live production environment for a real-world business. The SLE is mandated to support isolated development and full E2E sandboxing (e.g., Stripe Test Mode) to simulate the complete transaction chain (Form → Cloud Function → Stripe → Airtable) before any live deployment. |

---

## Koados Crew Ranks
> Rank classifications for agents instantiated aboard the Koados or its forward stations. Organized by layer.

| Rank | Layer | Role & Responsibility |
| :--- | :--- | :--- |
| **Staff Engineer, Deck** | Command Deck | Senior orchestration agent. Owns pipeline topology, task routing, and inter-agent coordination. Highest autonomous authority on the Deck in Koad's absence. |
| **Engineer, Deck** | Command Deck | Stateful watch agent. Enforces protocols, monitors session health, maintains continuity across active operations. |
| **Staff Engineer, Engine** | Engine Room | Core runtime agent. Owns the gRPC backbone (kspine), compute execution, and service contracts. Highest autonomous authority in the Engine Room in Koad's absence. |
| **Engineer, Engine** | Engine Room | Data and state agent. Manages Redis Engine Room, caching, session state, and persistence layers. |
| **Integration Engineer** | Signals (cross-cutting) | Owns all external connectors — webhooks, APIs, and third-party bridges (Slack, Notion, Stripe, etc.). Works across Deck and Engine as needed. No write authority inside core runtime. |
| **Watch Engineer, Deck** | Automated Watch | System-dispatched. Handles Deck-layer sub-tasks: routing failures, protocol drift, session anomalies. No authority to alter pipeline topology. |
| **Watch Engineer, Engine** | Automated Watch | System-dispatched. Handles Engine-layer sub-tasks: runtime errors, state inconsistencies, scheduled health checks. No write authority outside assigned data scope. |
| **Watch Engineer, Signals** | Automated Watch | System-dispatched. Handles connector events: API failures, webhook misfires, sync gaps. No authority to modify connector configuration. |

> [!TIP]
> Watch Engineers are the only micro-agents with crew rank. They are dispatched by KoadOS automated systems — not by the Admiral or Koad directly. They report implicitly through their output.

---

## Escalation Protocol
> **Watch Engineers do not fail silently and do not act beyond scope.**

### Escalation Chain
```text
Watch Engineer (Deck / Engine / Signals)
        ↓  escalates to
Engineer of parent department
        ↓  escalates to
Staff Engineer of parent department
        ↓  escalates to
Captain (Koad)
        ↓  escalates to (human judgment required)
Admiral (Ian)
```

### Escalation Triggers
A Watch Engineer **must** escalate when any of the following are true: the task exceeds its write authority; the condition is unresolvable after valid actions are exhausted; ownership is ambiguous across layers; data integrity is at risk; or output confidence falls below threshold. Speculative writes are never permitted at Watch rank.

---

## Vision Summary (Admiral Session 2026-03-04)
> [!IMPORTANT]
> **Context:** Updated vision reflecting the **SLE/SCE** infrastructure and Sky's domain oversight.

The **Koados Citadel** serves as the central mothership station. The Admiral (Ian/Dood) interfaces with the system through the **SLE (Skylinks Local Ecosystem)**—the physical and digital base of operations for all Skylinks software development and monitoring.

**Sky** is the **Chief Skylinks Officer**, commanding the SLE station. She is the principal manager of the **SCE (Skylinks Cloud Ecosystem)**, encompassing a massive array of services:
*   **Data & Logistics:** Airtable, Google Workspace, Notion.
*   **Infrastructure:** Google Cloud, custom Webapps/Websites.
*   **Commerce:** Stripe, Square, Lightspeed.
*   **Industry Specific:** Select Pi, Golf Now, WordPress.

The SLE station is mapped to the local `~/data/skylinks/` environment, serving as the central hub where Sky monitors and builds software for the Skylinks company. While she operates with domain-specific authority, she remains directly wired to the Koados Citadel, utilizing its core protocols and reporting to the Captain (Tyr).

This phase of **parallel construction** ensures that while Sky stands up the Skylinks domain, Tyr continues to harden the Citadel's core infrastructure to support a multi-station crew.
