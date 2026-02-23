# KoadOS Driver Guide

Drivers allow KoadOS to adapt to different AI agents while maintaining the same core persona.

## Anatomy of a Driver
A driver consists of an entry in `koad.json` and a corresponding markdown file.

### 1. The JSON Entry
Add a new key to the `drivers` object in `~/.koad-os/koad.json`:
```json
"claude": {
  "bootstrap": "~/.koad-os/drivers/claude/BOOT.md",
  "mcp_enabled": false,
  "tools": ["read_file", "write_file"]
}
```

### 2. The Bootstrap File
The `bootstrap` path points to a file containing agent-specific prompts. 
- **Gemini**: Focus on tool calling and long-context usage.
- **Claude**: Focus on XML-tagged structure and reasoning.
- **Codex**: Focus on raw terminal output and script execution.

## Booting an Agent
When an agent starts, it should run:
`koad boot --agent <name>`

The resulting output should be treated as the "System Instructions" for that session.
