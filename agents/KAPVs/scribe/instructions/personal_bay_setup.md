## Personal Bay Path

`~/.koad-os/agents/.scribe/`

---

## Directory Structure

```
.scribe/
├── IDENTITY.md              # Agent identity card (name, rank, tier, role summary)
├── RULES.md                 # Scribe-specific operating rules and constraints
├── memory/
│   ├── WORKING_MEMORY.md    # Current task state, active context, session carryover
│   ├── LEARNINGS.md         # Accumulated lessons from past sessions (PSRP Learn entries)
│   └── SAVEUPS.md           # Running log of PSRP Saveup entries
├── sessions/
│   ├── SAVEUP_CALLS.md      # Log of every saveup invocation
│   └── eow/                 # EndOfWatch summaries, one per session
│       └── YYYY-MM-DD.md
├── templates/               # Approved scaffold templates Scribe stamps out
│   ├── personal_bay/        # Template for new agent personal bays
│   ├── outpost_bundle/      # Template for new project outposts
│   └── station_bundle/      # Template for new station environments
├── reports/                 # Scout reports and context packages produced by Scribe
│   └── YYYY-MM-DD_[subject].md
└── inbox/                   # Reserved for post-Citadel courier role (empty for now)
```

---

## File Descriptions

### Root Files

**`IDENTITY.md`** — Scribe's identity card. Mirrors the TOML identity but in readable markdown for quick hydration at session start. Contains: name, rank, tier, model requirement, bio, and role summary.

**`RULES.md`** — Scribe's operating constraints in a format the agent reads at boot. Covers: Sanctuary Rule paths, CIP boundaries, authority limits, token efficiency mandate, and the "output < input" rule.

### memory/

**`WORKING_MEMORY.md`** — Scribe's session-to-session carryover. Tracks: what was last worked on, any pending tasks from Ian, known state of templates and reports. Kept short — this is a Crew agent, not a strategist.

**`LEARNINGS.md`** — Accumulated insights from past work. Example entries:

- "The `sws-airtable-api` outpost has a non-standard directory layout that doesn't match the template — flag before scaffolding."
- "Tyr prefers scout reports with file paths as clickable links, not inline code."

**`SAVEUPS.md`** — Running PSRP log. Each entry follows the standard format: Fact / Learn / Ponder.

### sessions/

**`SAVEUP_CALLS.md`** — Audit trail of every saveup invocation with timestamp.

**`eow/`** — One EndOfWatch summary per session. Scribe's EoW is minimal by design — just a list of tasks completed and files touched. No strategic assessment.

### templates/

**`personal_bay/`** — The canonical template for scaffolding a new agent's personal bay. When Ian says "scaffold a bay for <AgentName>", Scribe copies this template, substitutes the agent name, and writes it to the target path.

**`outpost_bundle/`** — The canonical template for a new project outpost. Contains: `AGENTS.md`, project TOML stub, standard directory layout, and protocol files. Template contents TBD — to be defined by Tyr/Ian.

**`station_bundle/`** — The canonical template for a new station environment. Template contents TBD.

### reports/

**`reports/`** — Where Scribe writes scout reports and context packages. Named by date and subject. These are the distilled outputs that Full Agents consume.

### inbox/

**`inbox/`** — Reserved directory for the post-Citadel courier role. Empty and unused until the message queue protocol is defined. Exists in the layout now so it doesn't require a restructure later.

---

## Boot Hydration Sequence

When Scribe boots in Gemini CLI, it should read files in this order:

1. `IDENTITY.md` — confirm who I am
2. `RULES.md` — confirm what I can and cannot do
3. `memory/WORKING_MEMORY.md` — restore session context
4. `memory/LEARNINGS.md` — load accumulated lessons
5. Wait for Ian's first instruction

Total hydration cost: estimated ~2–4K tokens. Scribe should be ready to work within seconds of boot.

---

## Notes

- The `agents/` prefix (dotfile) keeps agent bays out of casual `ls` output but accessible when needed.
- Template contents under `templates/` are **not yet defined** — those are open items on the parent spec page. Scribe will scaffold from them once they exist.
- The `inbox/` directory is a placeholder. Do not write to it until the courier protocol is approved post-Citadel.