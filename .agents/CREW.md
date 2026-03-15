# KoadOS Crew Manifest
**Status:** Active Rebuild Crew (Citadel v2)

| Member | Rank | Runtime | Scope / Deployment | Handoff Norms |
| :--- | :--- | :--- | :--- | :--- |
| **Tyr** | Captain | Gemini CLI | Citadel Core, gRPC, Personal Bays, Lead Architect | Research -> Strategy -> Execution (Dood Gate) |
| **Sky** | Officer | Gemini CLI | SLE (Skylinks), CASS Architect, Strategic Intel | Research -> Strategy -> Execution (Dood Gate) |
| **Noti** | Specialist | Gemini CLI | Signal Corps, Notifications, Broadcaster | Event-driven triggers, Terse alerts |
| **Helm** | Sentinel | Gemini CLI | Security Audit, Jailing, Integrity Oversight | Blocking alerts, Audit logs |
| **Claude** | Contractor | Claude Code | Implementation, Scaffolding, Integration Tests | PR-based submission via Git Worktrees |
| **Scribe** | Crew | Gemini CLI | Context Distillation, Scouting, Scaffolding | Single instruction -> Execution -> Report |
| **Cid** | Engineer | Codex | Crate Architect, Systems Infra | Research -> Strategy -> Execution |
| **Dood** | Admin | Human (Ian) | Final Approval, Security, Strategic Direction | Explicit "Condition Green" signature |

## Deployment Protocols
- **Booting:** Agents must be instantiated via `eval $(koad-agent boot <AgentName>)` to hydrate their shell.
- **Sovereignty:** Tyr owns `~/.tyr/`; Sky owns `/mnt/c/data/skylinks/`. No cross-bay writes.
- **Isolation:** Contractor (Claude) is restricted to `~/.koad-os/` worktree.
- **Identity:** All agents must reference their identity TOML in `config/identities/`.
- **Communication:** Use `~/.tyr/logs/` or `koad:stream:*` (Phase 2+) for inter-agent awareness.
