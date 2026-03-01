# Canonical Required Sources

List of infrastructure files that must exist for the global kit to operate.

| Scope | Required source | Notes |
| --- | --- | --- |
| Global standards | `.agent-ops/STANDARDS_REGISTRY.md` | Base standard definitions for the kit. |
| Startup context | `.agent-core/ops/STARTUP_CHECKLIST.md` | Defines the required boot order. |
| Role routing | `.agent-core/ops/ROLE_BOOT_PROTOCOL.md` | Provides the role question/guidance. |
| Saveup protocol | `.agent-core/ops/SAVEUP_PROTOCOL.md` | Outlines how to capture continuity. |
| Memory ledgers | `.agent-core/memory/WORKING_MEMORY.md`, `.agent-core/memory/LEARNINGS.md`, `.agent-core/memory/FACTS_LEDGER.md`, `.agent-core/memory/PATTERNS.md`, `.agent-core/memory/USER_PREFERENCES.md` | Captures current context, lessons, and preferences. |
| Decision log | `.agent-ops/decisions/DECISION_LOG.md` | Durable policy and governance decisions. |
| Session log | `.agent-ops/sessions/SESSION_LOG.md` | Operational summaries for support work. |
| Standards manifest | `.standards/standards_registry.md` | Mirror of coded standard entries. |
| Project profile | `.standards/project_profile.md` | Describes the current governance assumptions. |
| Sync manifest | `.standards/sync_manifest.json` | Indicates when standards were last refreshed. |

## Blocker contract
If any required source is missing, stop standards-dependent work and log the missing path along with the timestamp.
