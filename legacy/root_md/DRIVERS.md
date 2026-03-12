# KoadOS Driver Guide: Ghost & Body

KoadOS separates an agent's identity (Persona) from its technical environment (Interface). This ensures strategic continuity across different AI hosts.

## 1. Personas (The Ghost)
A Persona defines the **Who**: rank, mission directive, and strategic responsibilities.
- **Location**: `config/identities/<name>.toml`
- **Bootstrap**: `personas/<name>/BOOT.md`

### Identity Configuration (`tyr.toml`)
```toml
[identities.Tyr]
name = "Tyr"
rank = "Captain"
bootstrap = "~/.koad-os/personas/Tyr/BOOT.md"
# ... preferences and bio
```

## 2. Interfaces (The Body)
An Interface defines the **Where**: engine-specific tool sets, path mappings, and technical guardrails.
- **Location**: `config/interfaces/<engine>.toml`
- **Bootstrap**: `bodies/<engine>/BOOT.md`

### Interface Configuration (`gemini.toml`)
```toml
[interfaces.gemini]
name = "gemini"
bootstrap = "~/.koad-os/bodies/gemini/BOOT.md"
mcp_enabled = true
tools = ["read_file", "write_file", "google_web_search"]
```

## 3. Boot Protocol
When an agent initializes, it MUST run:
`koad boot --agent <PersonaName>`

The Spine automatically resolves the current **Interface** based on the environment (e.g., `GEMINI_CLI`) and merges the Persona and Interface bootstrap content into the session instructions.

### Tool Priority
Native KoadOS CLI tools MUST be used over standard MCP tools (e.g., Notion MCP) to conserve tokens and maintain system integrity.
