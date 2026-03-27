# KoadOS Crew Manifest
**Status:** Active Rebuild Crew (Citadel v3 — Jupiter)

| Member | Rank | Runtime | Scope / Deployment | Handoff Norms |
| :--- | :--- | :--- | :--- | :--- |
| **Pic** | Captain | Gemini CLI | Citadel Core, gRPC, Personal Bays, Lead Architect | Research -> Strategy -> Execution (Dood Gate) |
| **Sky** | Officer | Gemini CLI | SLE (Skylinks), CASS Architect, Strategic Intel | Research -> Strategy -> Execution (Dood Gate) |
| **Noti** | Specialist | Notion MCP (remote) | Notion Cloud — operates remotely via MCP. Manages notes, tasks, and project pages. NOT a local KAPV agent. | Event-driven triggers, Terse alerts |
| **Claude** | Contractor | Claude Code | Implementation, Scaffolding, Integration Tests | PR-based submission via Git Worktrees |
| **Scribe** | Crew | Gemini CLI | Context Distillation, Scouting, Scaffolding | Single instruction -> Execution -> Report |
| **Cid** | Engineer | Codex | Crate Architect, Systems Infra | Research -> Strategy -> Execution |
| **Clyde** | Officer | Claude Code | Citadel Development, Multi-Project Crew | Research -> Strategy -> Execution (Dood Gate) |
| **Helm** | Officer | Gemini CLI | Citadel Build Engineer, Container Operations, Execution Sandbox Oversight | Research -> Strategy -> Execution (Dood Gate) |
| **Dood** | Admin | Human (Ian) | Final Approval, Security, Strategic Direction | Explicit "Condition Green" signature |

## Deployment Protocols
- **Booting:** Agents must be instantiated via `eval $(KOAD_RUNTIME=<runtime> koad-agent boot <AgentName>)` to hydrate their shell.
- **Sovereignty:** Pic owns `~/.pic/`; Sky owns `/home/ideans/data/skylinks/agents/sky/`. No cross-bay writes.
- **Isolation:** Contractor (Claude) is restricted to `~/.koad-os/` worktree.
- **Identity:** All agents must reference their identity TOML in `config/identities/`.
- **Communication:** Use `~/.pic/logs/` or `koad:stream:*` (Phase 2+) for inter-agent awareness.
