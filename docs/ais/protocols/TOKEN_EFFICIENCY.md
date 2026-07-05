# AIS Protocol: Token Efficiency & Communication
**Version:** 1.1.0  
**Effective Date:** 2026-05-02  
**Author:** Tyr (Captain)

## 1. RTK (Rust Token Killer) Global Standard
All KoadOS agents, regardless of harness (Gemini, Claude Code, Codex), MUST utilize the `rtk` binary proxy for CLI operations.

### 1.1 Implementation
- **Boot Sequence**: `rtk init -g` must be part of every agent's hydration logic.
- **Goal**: Achieve >60% token reduction on high-noise commands (`cargo`, `git`, `docker`).
- **Transparency**: Commands should be proxied transparently (e.g., `git status` auto-rewritten to `rtk git status`).

## 2. Sovereign Prose vs. Caveman Task-Talk
Agents must maintain a dual-mode communication protocol to balance station personality with operational efficiency.

### 2.1 Default: Sovereign Prose
- **Scope**: Strategy, architecture, orientation, research, and general conversation.
- **Requirement**: Maintain natural rank-appropriate tone, identity, and full grammatical structure.

### 2.2 Active: Caveman Mode (Task-Talk)
- **Scope**: **Coding tasks only** (implementation, refactoring, bug-fixing, repetitive tool-heavy execution).
- **Trigger**: Manually via `/caveman` or auto-triggered by task implementation phases.
- **Rules**:
    - Drop articles, filler, and pleasantries.
    - Use fragments and short synonyms.
    - Preserve all technical symbols, error strings, and code blocks exactly.

## 3. AIS Data Integrity
- **Self-Healing**: Bridge proxies (Notion, Airtable) are responsible for their own SQLite schema initialization.
- **Centralized Awareness**: All agents reference the workspace-root `SITREP.md` for current mission objectives and accomplishments.
