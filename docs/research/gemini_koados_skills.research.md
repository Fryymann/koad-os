# Deep Dive — Gemini CLI Skills & KoadOS Bridge Architecture

<aside>
🎯

**Purpose:** Research how Gemini CLI Skills work natively, and design a bridge architecture that lets KoadOS Agent Skills run as first-class Gemini CLI skills — so any Gemini-powered agent (Scribe, future crew) can invoke KoadOS skills natively without wrapper friction.

**Author:** Noti

**Date:** 2026-03-15

**Status:** Research Complete — Awaiting Tyr architecture review

</aside>

---

## Part 1 — How Gemini CLI Skills Work (Native Architecture)

Gemini CLI has **three distinct extensibility layers**, each solving a different problem. Understanding all three is essential to designing the right bridge.

### Layer 1: Agent Skills (`SKILL.md`)

Agent Skills are the closest native analog to KoadOS Skills. A Skill is simply a **directory containing a `SKILL.md` file** that injects specialized knowledge and instructions into the agent's context.

**How they work:**

- Gemini CLI scans `.gemini/skills/` (project-scoped) and `~/.gemini/skills/` (user-scoped) at startup
- Each subdirectory with a `SKILL.md` is registered as an available skill
- When the agent detects intent that matches a skill, it loads the `SKILL.md` into context
- The skill provides instructions, workflows, constraints, and examples to guide agent behavior
- Skills do **not** expose new tools — they shape *how* the agent uses existing tools

[**SKILL.md](http://SKILL.md) structure:**

```markdown
---
name: skill-name
description: >
  Semantic description used for intent matching and triggering.
  Write it to describe WHEN to use this skill.
trigger_patterns:
  - "pattern 1"
  - "pattern 2"
---

# Skill Title

## Instructions
...
```

**Discovery locations:**

```
# Project-scoped (committed to repo)
.gemini/skills/<skill-name>/SKILL.md

# User-scoped (global across all projects)
~/.gemini/skills/<skill-name>/SKILL.md
```

<aside>
💡

**Key insight:** KoadOS `.koad-os/skills/scribe/<skill-name>/SKILL.md` is **structurally identical** to Gemini's `.gemini/skills/<skill-name>/SKILL.md`. The only differences are path and frontmatter field names. This is the most direct bridge path.

</aside>

---

### Layer 2: Custom Commands (`/command` via TOML)

Custom commands are **saved prompt templates** with optional argument injection. They appear as `/command-name` slash commands in the Gemini CLI REPL.

**Structure:**

```toml
# .gemini/commands/sync-notion.toml
name = "sync-notion"
description = "Trigger a Notion database sync via the KoadOS notion-sync skill"
prompt = """
Please sync the Notion data sources using the notion-sync skill.
Focus on: args
"""
```

**Scoping:**

- `.gemini/commands/` — project-scoped
- `~/.gemini/commands/` — user-scoped
- Extensions can ship commands in `commands/` subdirectory

**Use case in KoadOS:** Every KoadOS skill that has well-defined trigger phrases can be wrapped as a custom command — providing a fast `/koad:skill-name` keyboard shortcut for power users.

---

### Layer 3: Extensions (`gemini-extension.json` + MCP Server)

Extensions are the **most powerful** layer. They package:

- An MCP server (exposes new **tools** to the model)
- Custom commands
- Context files (`GEMINI.md`)
- Sub-agents
- Agent skills

**Extension structure:**

```
my-extension/
├── gemini-extension.json     # Manifest — defines MCP server, metadata
├── GEMINI.md                 # Static context injected at startup
├── commands/
│   └── my-command.toml       # Custom slash commands
├── skills/
│   └── my-skill/
│       └── SKILL.md          # Agent skills bundled in extension
└── src/
    └── server.js             # MCP server implementation
```

**`gemini-extension.json`:**

```json
{
  "name": "koad-os",
  "version": "1.0.0",
  "description": "KoadOS skill bridge for Gemini CLI agents",
  "mcpServers": {
    "koad-skills": {
      "command": "node",
      "args": ["${extensionPath}/src/server.js"],
      "env": {
        "KOAD_OS_ROOT": "${workspacePath}/.koad-os"
      }
    }
  }
}
```

**Installation:**

```bash
# From GitHub
gemini extensions install https://github.com/Skylinks-Golf/agents-os

# From local path (dev mode)
gemini extensions install /path/to/koad-os-extension
```

---

### Layer 4: Sub-Agents (Experimental)

Sub-agents are **specialized child agents** that the main agent can spawn for specific tasks. They run with their own context, tools, and instructions — keeping the main agent's context clean.

- Defined in `.gemini/agents/<agent-name>/AGENT.md`
- Main agent delegates to sub-agent via `run_tool` or natural language
- Currently experimental — enable via `settings.json`

**KoadOS relevance:** Sub-agents map directly to the KoadOS pipeline pattern and Body/Ghost model. A KoadOS skill that involves a specialized sub-workflow could be exposed as a Gemini sub-agent.

---

## Part 2 — The MCP Protocol (The Real Bridge Layer)

The **Model Context Protocol (MCP)** is the universal tool-exposure standard that underpins Gemini CLI extensions. Understanding MCP is critical because it's how KoadOS skills expose **executable tools** (not just instructions) to Gemini agents.

**What MCP does:**

- Exposes a set of typed, documented **tools** to any MCP-compatible AI agent
- Tools have a name, description, and JSON Schema parameter spec
- The AI agent calls tools like function calls; the MCP server executes them and returns results
- Transport: stdio (local process), SSE (HTTP streaming), or WebSocket

**MCP server anatomy:**

```jsx
// server.js — minimal MCP server for a KoadOS skill
const { Server } = require('@modelcontextprotocol/sdk/server/index.js');
const { StdioServerTransport } = require('@modelcontextprotocol/sdk/server/stdio.js');

const server = new Server({ name: 'koad-skills', version: '1.0.0' });

server.setRequestHandler('tools/list', async () => ({
  tools: [
    {
      name: 'notion_sync',
      description: 'Synchronize KoadOS local datastore with configured Notion databases',
      inputSchema: {
        type: 'object',
        properties: {
          direction: { type: 'string', enum: ['pull', 'push', 'both'], default: 'both' },
          database: { type: 'string', description: 'Optional: target a specific database by name' },
          full: { type: 'boolean', description: 'Force full re-sync, ignore timestamps' }
        }
      }
    }
  ]
}));

server.setRequestHandler('tools/call', async (request) => {
  if (request.params.name === 'notion_sync') {
    const result = await runSkillScript('notion-sync', request.params.arguments);
    return { content: [{ type: 'text', text: result }] };
  }
});
```

**This is the key difference between Skills and MCP tools:**

| **Layer** | **What it provides** | **Analogy** |
| --- | --- | --- |
| [SKILL.md](http://SKILL.md) | Instructions + knowledge injected into context | Agent's expertise / training |
| MCP Tool | Executable function the agent can call | Agent's hands / actuators |
| Custom Command | Saved prompt shortcut | Agent's keyboard shortcuts |
| Sub-Agent | Delegated specialist agent | Agent's crew members |

---

## Part 3 — The KoadOS ↔ Gemini Bridge Architecture

### Design Goals

1. **Zero duplication** — A KoadOS skill is authored once and works in both KoadOS CLI and Gemini CLI contexts
2. **Native feel** — Gemini agents invoke KoadOS skills using Gemini-native patterns (no wrapper friction)
3. **Layered exposure** — Skills expose the right Gemini layer for their type (instructions-only → [SKILL.md](http://SKILL.md); executable → MCP; workflow shortcut → command)
4. **Dark Mode compatible** — Bridge works when Gemini CLI is running locally; no cloud dependency
5. **Progressive adoption** — v1 uses [SKILL.md](http://SKILL.md) symlinks (zero infra); v2 adds MCP server; v3 adds full extension packaging

---

### Bridge v1 — [SKILL.md](http://SKILL.md) Symlink Strategy (Zero Infra)

The simplest possible bridge: **symlink KoadOS skill directories into the Gemini skills path.**

```bash
# One-time setup script: .koad-os/scripts/bridge-gemini-skills.sh
#!/bin/bash
KOAD_SKILLS=".koad-os/skills"
GEMINI_SKILLS=".gemini/skills"
mkdir -p "$GEMINI_SKILLS"

for agent_dir in "$KOAD_SKILLS"/*/; do
  agent=$(basename "$agent_dir")
  for skill_dir in "$agent_dir"*/; do
    skill=$(basename "$skill_dir")
    target="$GEMINI_SKILLS/${agent}-${skill}"
    if [ ! -e "$target" ]; then
      ln -s "$(pwd)/$skill_dir" "$target"
      echo "Linked: $target"
    fi
  done
done
```

**Result:** Scribe's `notion-sync` skill at `.koad-os/skills/scribe/notion-sync/` becomes available to Gemini CLI at `.gemini/skills/scribe-notion-sync/`.

**Limitation:** [SKILL.md](http://SKILL.md) frontmatter field names differ between KoadOS and Gemini. A **frontmatter translation layer** is needed (see below).

---

### [SKILL.md](http://SKILL.md) Frontmatter Translation

KoadOS and Gemini use slightly different frontmatter schemas:

| **Field** | **KoadOS [SKILL.md](http://SKILL.md)** | **Gemini [SKILL.md](http://SKILL.md)** |
| --- | --- | --- |
| Name | `name: notion-sync` | `name: notion-sync` |
| Description | `description: >` (multi-line) | `description: >` (multi-line) |
| Triggers | `trigger_patterns: [...]` | `trigger_patterns: [...]` |
| Dependencies | `requires: [python3, sqlite3]` | Not supported — document in body |
| Tier | `tier: crew` | Not supported — ignored |
| Context cost | `context_cost: low` | Not supported — ignored |

**Solution:** The bridge script generates a **Gemini-compatible [SKILL.md](http://SKILL.md) wrapper** that:

1. Strips unsupported KoadOS fields from frontmatter
2. Injects a preamble explaining the KoadOS skill system context
3. Includes the full KoadOS [SKILL.md](http://SKILL.md) body via an `@{...}` file reference

```bash
# Generated .gemini/skills/scribe-notion-sync/SKILL.md
---
name: scribe-notion-sync
description: >
  Synchronize designated Notion databases with the KoadOS local SQLite datastore.
  Provides incremental sync with change detection. Trigger when user asks to:
  sync Notion, refresh local data from Notion, check what's changed, or query
  synced Notion content.
trigger_patterns:
  - "sync the Notion *"
  - "please sync the Notion data sources"
  - "refresh Notion data"
  - "pull latest from Notion"
  - "query Notion for *"
---

<!-- KoadOS Bridge: This skill is managed by KoadOS. Source: .koad-os/skills/scribe/notion-sync/SKILL.md -->

@{../../../../.koad-os/skills/scribe/notion-sync/SKILL.md}
```

---

### Bridge v2 — MCP Server (Executable Tools)

For skills that need to **execute scripts** (not just provide instructions), the bridge exposes them as MCP tools via a lightweight Node.js or Python MCP server.

**Architecture:**

```
Gemini CLI agent
    │
    │ calls tool: notion_sync({direction: "pull"})
    ▼
 koad-skills MCP Server  (.koad-os/bridge/gemini-mcp-server.js)
    │
    │ reads: .koad-os/skills/*/SKILL.md (discovers registered skills)
    │ executes: .koad-os/skills/scribe/notion-sync/scripts/sync-databases.sh
    ▼
 KoadOS Skill Scripts
    │
    │ reads/writes: .koad-os/data/notion-sync.db
    ▼
 Notion API / Local SQLite
```

**MCP Server Design — `koad-skills-server.js`:**

```jsx
#!/usr/bin/env node
/**
 * KoadOS → Gemini CLI MCP Bridge
 * Discovers KoadOS skills and exposes their executable scripts as MCP tools.
 */
const { Server } = require('@modelcontextprotocol/sdk/server/index.js');
const { StdioServerTransport } = require('@modelcontextprotocol/sdk/server/stdio.js');
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const TOML = require('@iarna/toml');

const KOAD_ROOT = process.env.KOAD_OS_ROOT || '.koad-os';
const SKILLS_ROOT = path.join(KOAD_ROOT, 'skills');

// Discover all skills with a scripts/ directory (they're executable)
function discoverExecutableSkills() {
  const skills = [];
  for (const agent of fs.readdirSync(SKILLS_ROOT)) {
    const agentPath = path.join(SKILLS_ROOT, agent);
    for (const skill of fs.readdirSync(agentPath)) {
      const skillPath = path.join(agentPath, skill);
      const skillMdPath = path.join(skillPath, 'SKILL.md');
      const scriptsPath = path.join(skillPath, 'scripts');
      if (fs.existsSync(skillMdPath) && fs.existsSync(scriptsPath)) {
        const frontmatter = parseFrontmatter(skillMdPath);
        skills.push({ agent, skill, path: skillPath, ...frontmatter });
      }
    }
  }
  return skills;
}

const server = new Server({ name: 'koad-skills', version: '1.0.0' });

// Expose each executable skill as an MCP tool
server.setRequestHandler('tools/list', async () => ({
  tools: discoverExecutableSkills().map(s => ({
    name: `${s.agent}_${s.skill.replace(/-/g, '_')}`,
    description: s.description,
    inputSchema: buildInputSchema(s)
  }))
}));

server.setRequestHandler('tools/call', async (req) => {
  const [agent, ...skillParts] = req.params.name.split('_');
  const skill = skillParts.join('-');
  const skillPath = path.join(SKILLS_ROOT, agent, skill);
  const result = executeSkill(skillPath, req.params.arguments);
  return { content: [{ type: 'text', text: result }] };
});

const transport = new StdioServerTransport();
server.connect(transport);
```

**Registering the MCP server in `.gemini/settings.json`:**

```json
{
  "mcpServers": {
    "koad-skills": {
      "command": "node",
      "args": [".koad-os/bridge/koad-skills-server.js"],
      "env": {
        "KOAD_OS_ROOT": ".koad-os"
      }
    }
  }
}
```

---

### Bridge v3 — Gemini Extension Package (Portable + Installable)

For full portability — so any machine running Gemini CLI can install the KoadOS skill suite — the bridge is packaged as a proper Gemini CLI Extension.

**Extension directory structure:**

```
.gemini/extensions/koad-os/
├── gemini-extension.json          # Extension manifest
├── GEMINI.md                      # Static context: explains KoadOS to Gemini
├── commands/
│   ├── sync-notion.toml           # /koad:sync-notion slash command
│   ├── run-skill.toml             # /koad:run-skill <name> generic runner
│   └── koad-status.toml          # /koad:status shows KoadOS health
├── skills/
│   └── (auto-generated from .koad-os/skills/ by build step)
└── src/
    └── koad-skills-server.js      # MCP server (from v2)
```

**`gemini-extension.json`:**

```json
{
  "name": "koad-os",
  "version": "1.0.0",
  "description": "KoadOS agent skill suite — exposes KoadOS skills as native Gemini tools",
  "mcpServers": {
    "koad-skills": {
      "command": "node",
      "args": ["${extensionPath}/src/koad-skills-server.js"],
      "env": {
        "KOAD_OS_ROOT": "${workspacePath}/.koad-os"
      }
    }
  },
  "settings": [
    {
      "name": "NOTION_API_TOKEN",
      "description": "Notion API integration token for notion-sync skill",
      "sensitive": true
    }
  ]
}
```

---

## Part 4 — Bridge for Claude Code (Bonus)

Claude Code uses the same MCP protocol for tool exposure. The **v2 MCP server** from above works without modification — you just register it in Claude Code's MCP config instead of Gemini's.

**Claude Code `.claude/settings.json`:**

```json
{
  "mcpServers": {
    "koad-skills": {
      "command": "node",
      "args": [".koad-os/bridge/koad-skills-server.js"],
      "env": {
        "KOAD_OS_ROOT": ".koad-os"
      }
    }
  }
}
```

Claude Code Skills (the `.claude/` instruction files) parallel Gemini's [SKILL.md](http://SKILL.md) — the same bridge script can generate both.

<aside>
⚡

**Key insight:** One MCP server serves both Gemini CLI and Claude Code. The [SKILL.md](http://SKILL.md) bridge generates both Gemini-format and Claude-format instruction files from the canonical KoadOS [SKILL.md](http://SKILL.md). Write once, run on both.

</aside>

---

## Part 5 — Skill Anatomy: KoadOS Native vs Gemini Native vs Bridged

| **Attribute** | **KoadOS Native** | **Gemini Native** | **KoadOS Bridge** |
| --- | --- | --- | --- |
| Location | `.koad-os/skills/<agent>/<skill>/` | `.gemini/skills/<skill>/` | Auto-generated in `.gemini/skills/` from KoadOS source |
| Instruction file | `SKILL.md` | `SKILL.md` | Generated wrapper pointing to KoadOS source |
| Executable tools | `scripts/*.sh / *.py` | MCP server tools | MCP server wrapping KoadOS scripts |
| Slash commands | KoadOS CLI commands | `commands/*.toml` | Generated TOML from KoadOS trigger patterns |
| Config/secrets | `.koad-os/config/secrets/` | Extension settings (keychain) | Extension settings → injected as env vars to scripts |
| Context cost | Declared in frontmatter | N/A | Preserved in body, ignored by Gemini |

---

## Part 6 — Implementation Plan (Phased)

### Phase 1 — [SKILL.md](http://SKILL.md) Bridge (v1) — Low effort, immediate value

**Deliverable:** `bridge-gemini-skills.sh`

- [ ]  Write `bridge-gemini-skills.sh` that scans `.koad-os/skills/` and generates Gemini-compatible [SKILL.md](http://SKILL.md) wrappers in `.gemini/skills/`
- [ ]  Add frontmatter translation: strip KoadOS-only fields, preserve name/description/trigger_patterns
- [ ]  Add `@{path}` file reference to pull in full KoadOS skill body
- [ ]  Run bridge on `notion-sync` and `support-kb` skills as proof-of-concept
- [ ]  Add `bridge-gemini-skills.sh` to `koad boot` sequence

**Complexity:** ~60 lines bash. Tyr can ship this in one session.

### Phase 2 — MCP Bridge (v2) — Medium effort, full tool execution

**Deliverable:** `koad-skills-server.js` + `.gemini/settings.json` config

- [ ]  Implement MCP server that auto-discovers executable KoadOS skills
- [ ]  Map KoadOS skill scripts to MCP tool call handlers
- [ ]  Handle argument passing: MCP tool args → shell script flags
- [ ]  Handle output: capture script stdout/stderr, return as MCP tool result
- [ ]  Register in `.gemini/settings.json` and test with `notion-sync`
- [ ]  Register in `.claude/settings.json` for Claude Code parity

**Complexity:** ~150-200 lines Node.js. One session for Tyr.

### Phase 3 — Extension Package (v3) — Packaged + portable

**Deliverable:** `.gemini/extensions/koad-os/` full extension

- [ ]  Package v2 MCP server + v1 [SKILL.md](http://SKILL.md) bridge as a Gemini Extension
- [ ]  Add `GEMINI.md` context file explaining KoadOS to the agent
- [ ]  Generate `/koad:*` slash commands from skill trigger patterns
- [ ]  Add extension settings for secrets (NOTION_API_TOKEN etc.)
- [ ]  Consider publishing to Gemini Extensions Gallery

**Complexity:** ~1 day. Appropriate after v1 and v2 are validated.

---

## Part 7 — Recommended Build Order for Tyr

<aside>
🔨

Build the bridge **alongside** the notion-sync skill, not after. The bridge script is simple enough that it can be implemented as part of the notion-sync delivery — making notion-sync immediately available to both Scribe (KoadOS) and any Gemini CLI session.

**Recommended sequence:**

1. Implement `notion-sync` skill (KoadOS native)
2. Implement Phase 1 bridge (`bridge-gemini-skills.sh`) — makes notion-sync discoverable by Gemini CLI immediately
3. Implement Phase 2 MCP bridge — makes notion-sync's scripts *executable* by Gemini agents as tools
4. Phase 3 can wait for v2 validation
</aside>

---

## Open Questions for Tyr

1. **MCP SDK choice:** Node.js (`@modelcontextprotocol/sdk`) is the most mature. Python SDK exists but has less tooling. Rust MCP crates exist but are less documented. Recommendation: Node.js for v1 bridge (fast, well-documented), Rust port is a v2 consideration.
2. **Script argument mapping:** How should MCP tool `inputSchema` arguments map to bash script flags? Recommend a convention in `SKILL.md` frontmatter: `mcp_args` field listing which script flags to expose.
3. **Streaming output:** Long-running sync scripts produce incremental output. MCP supports streaming responses. Worth implementing for `notion-sync` — Ian can see progress in real-time.
4. **Secrets injection:** KoadOS stores secrets in `.koad-os/config/secrets/`. The Gemini Extension settings mechanism stores them in system keychain. Need a unified secrets read path — recommend the MCP server reads from KoadOS secrets first, falls back to env var.
5. **Auto-regeneration:** When a KoadOS skill is added or updated, the bridge wrappers should auto-regenerate. Add this as a hook to the KoadOS skill registration flow.