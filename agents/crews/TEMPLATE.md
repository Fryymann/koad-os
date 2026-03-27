# KoadOS Crew Manifest — [Citadel Name]
**Status:** Active Crew Manifest

| Member | Rank | Runtime | Role / Focus |
| :--- | :--- | :--- | :--- |
| **Tyr** | Captain | gemini | Admiral's Ghost. Principal Systems & Operations Engineer; Station Orchestration. |
| **Clyde** | Officer | claude | Implementation Engineer. Citadel Infrastructure & Multi-Project Development. |
| **Sky** | Officer | gemini | Specialist. Strategic Intel & Project Hub Management. |
| **Helm** | Officer | gemini | Build Engineer. Container Operations & Execution Sandbox Oversight. |
| **Noti** | Specialist | notion | Notion Cloud — operates remotely via MCP. |
| **Scribe** | Crew | gemini | Scout & Scribe. Context Distillation, Documentation & Scaffolding. |
| **Cid** | Engineer | codex | Engineer. Systems Infrastructure & Crate Architecture. |
| **Dood** | Admin | human | Final Approval, Security, Strategic Direction (Ian). |

## Operational Protocols

- **Communication:** Primary A2A-S via `koad stream` (Redis) and shared inbox at `agents/inbox/`.
- **Identity:** All agents are hydrated from `config/identities/`.
- **Vaults:** Each agent operates within their designated KAPV vault. No cross-vault writes.
- **Booting:** Initialize sessions via `agent-boot <AgentName>`.
