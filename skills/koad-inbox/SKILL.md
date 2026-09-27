---
name: koad-inbox
description: Use when handing work to another KoadOS agent, leaving a report for an agent, or checking for work addressed to you — agent-to-agent messages are files in the shared inbox.
license: MIT
compatibility: Requires a KoadOS Citadel install (koad CLI, $KOAD_HOME, running Citadel and CASS services).
metadata:
  author: koados
  version: "2.0.0"
---

# Agent inbox

Agent-to-agent handoff is a Markdown file in the shared inbox. It is durable, readable by every harness, and needs no service.

```
$KOAD_HOME/agents/inbox/<slug>.<type>.<agent>.md
```

- `<slug>`: short snake_case topic, e.g. `cass_mcp_restart`
- `<type>`: `task`, `report`, `message`, `issue`, `proposal`
- `<agent>`: recipient in lowercase (`clyde`, `hermes`), or `all_agents` for a broadcast

## Check your inbox

```bash
find "$KOAD_HOME/agents/inbox" -maxdepth 1 -type f \( -name "*.${KOAD_AGENT_NAME,,}.md" -o -name "*.all_agents.md" \)
```

`koad cognitive` and the boot MOTD also count pending items. Read them before starting new work.

## Send

Write the file with this header, then the body:

```markdown
## <One-line title>

### from: <your agent name>
### to: <recipient>
### type: <task|report|message|issue|proposal>
### priority: <low|standard|high>
### timestamp: <YYYY-MM-DD>

<What happened, what the recipient should do, where the evidence is, any blockers.>
```

Give enough context to act without re-deriving: file paths, PR numbers, commands, and what you verified.

## After acting

Move handled items to `$KOAD_HOME/agents/inbox/archived/` so the inbox shows only pending work. Do not archive items addressed to someone else.

## Note

`koad signal` (the Citadel Signal service) was removed in 2026-09: it reported success and delivered nothing. Use inbox files.
