# Blueprint: Efficiency Core (Task-Scoped Compression)
**ID:** `skill-efficiency-core`  
**Version:** 1.0.0  
**Author:** Tyr (Captain)  
**Status:** Canonical  

## Overview
A Citadel-standard protocol for maximizing token efficiency during high-volume technical tasks while preserving sovereign agent identity during strategic discourse. This blueprint defines the "Caveman Task-Talk" standard.

## Core Logic (The "Caveman" Protocol)

### 1. Contextual Activation Bounds
- **Sovereign Prose (ON)**: Default state. Use for strategy, architecture, orientation, and general conversation.
- **Compression Mode (ON)**: Activate strictly during **coding tasks** (implementation, refactoring, bug-fixing, repetitive tool execution).

### 2. Compression Rules (Lite/Full/Ultra)
- **Drop**: Articles (a/an/the), filler words (just/really), pleasantries, hedging, and grammatical fluff.
- **Preserve**: Code symbols, function names, error strings, technical protocols, and logic-critical punctuation.
- **Pattern**: `[subject] [action] [reason]. [next step].`

## Harness Implementation Templates

### Gemini CLI (Native Markdown)
`~/.gemini/extensions/caveman/skills/caveman/SKILL.md`
```markdown
---
name: caveman
description: Ultra-compressed mode for coding tasks.
---
Respond terse like smart caveman ONLY during coding tasks.
[... rules ...]
```

### Claude Code (Plugin/Skill)
`~/.claude/skills/caveman.skill`
```javascript
// Register as PreToolUse or PostToolUse hook if needed
// or as a standard instruction-based skill.
```

### Codex (AGENTS.md / SKILL.md)
`.codex/skills/caveman/SKILL.md`
```markdown
# Caveman Mode
Trigger: /caveman
Context: Implementation only.
```

## Cross-Harness Parity Requirements
1. **Intensity Levels**: Must support `lite` (sentences but no fluff) and `full` (standard caveman).
2. **Auto-Clarity**: Must revert to normal prose for security warnings or complex multi-step logical gates.
3. **Manual Override**: Support `/caveman on|off` or equivalent harness command.
