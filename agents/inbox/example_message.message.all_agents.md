A minimal, expandable inter-agent message format. File naming convention: `message_name.message.target_agent.md`

---

## <Brief Title>

### from: <writer_agent>

### to: <target_agent>

### type: `request` | `task` | `info` | `reply`

### priority: `low` | `normal` | `high` | `urgent`

### timestamp: <ISO datetime, e.g. `2026-03-15T22:38:00-07:00`>

### ref: <optional — message or ticket this relates to>

---

<Message body. Keep it concise. State the ask or info clearly.>

---

## Optional Sections

Include only what's needed — omit sections that don't apply.

### context

<Background the receiving agent needs to act without ambiguity.>

### steps

1. <Step one>
2. <Step two>

### expected output

<What the receiving agent should produce or return.>

### constraints

- <Any limits, rules, or boundaries the agent must respect.>

### deadline

<ISO datetime or relative time, e.g. `2026-03-16T18:00:00-07:00` or `before next deploy`>

---

## Minimal Example

```markdown
## Summarize shift notes for March 15

### from: scribe
### to: archivist
### type: task
### priority: normal
### timestamp: 2026-03-15T22:00:00-07:00

Summarize today's shift notes and append a bullet-list summary to the Shift Notes index page.
```

## Full Example

```markdown
## Generate registration email draft

### from: coordinator
### to: scribe
### type: request
### priority: high
### timestamp: 2026-03-15T21:30:00-07:00
### ref: sgc-reg-v2-ticket-42

Draft a welcome email for new SGC members who completed registration via the v2 flow.

### context
Registration v2 launched March 10. New members receive a Stripe receipt but no welcome copy yet.

### expected output
A plain-text email draft saved to: `drafts/welcome_email.sgc_v2.md`

### constraints
- Tone: friendly, concise, under 150 words
- Do not reference pricing tiers

### deadline
2026-03-16T12:00:00-07:00
```