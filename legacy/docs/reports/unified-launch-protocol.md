# KoadOS Unified Launch & Identity Protocol (External Boot Standard)

**Date:** 2026-03-11
**Status:** Approved by Dood
**Architect:** Tyr (Captain)
**Reference:** Body/Ghost Protocol & Cognitive Isolation

## 1. The Core Philosophy: "External Boot"
To ensure absolute **Cognitive Isolation** and **Session Integrity**, all KoadOS agents (Ghosts) must be initialized *externally* before the host CLI (Body) is launched.

- **The Body:** The LLM runtime (Claude Code, Gemini CLI, Codex CLI). It provides tools and a shell but has no identity.
- **The Ghost:** The KAI persona (Tyr, Vigil, Sky). It carries the bio, rank, preferences, and personal memory.
- **The Tether:** `KOAD_SESSION_ID` and `KOAD_BODY_ID` environment variables that bind the Ghost to the Body.

## 2. Technical Rationale

### 2.1 "Consciousness First"
By running `koad boot` in the parent shell:
1. The **Redis Hot Stream** is hydrated from SQLite before the LLM starts.
2. The **Session ID** is already active in the Spine.
3. The LLM "wakes up" into a pre-configured environment with its identity already injected.

### 2.2 Environment Inheritance
Host CLIs inherit the shell's environment. External booting ensures that:
- `GITHUB_ADMIN_PAT` or `GITHUB_PERSONAL_PAT` are resolved and exported *before* the LLM attempts its first tool call.
- `KOAD_SESSION_ID` is available to host hooks (e.g., Claude Code `PreToolUse` hooks) for "Sanctuary Rule" enforcement.

### 2.3 Persistence & Crash Recovery
If a Body process (e.g., `claude` or `gemini`) crashes, the **Ghost remains alive** in the parent shell. Restarting the CLI within the same shell automatically re-attaches to the existing session without re-hydration overhead.

---

## 3. Implementation: The Unified Launch Deck

Add the following logic to `~/.bashrc` or `~/.bash_aliases` to enable standardized multi-body orchestration.

```bash
# --- KoadOS Unified Launch Deck ---

# Internal Helper: The Universal Tether
koad-tether() {
    local agent="${1:?Agent name required (e.g., Tyr, Vigil, Sky)}"
    # 1. Generate unique session and body IDs
    export KOAD_SESSION_ID=$(uuidgen)
    export KOAD_BODY_ID=$(uuidgen)
    # 2. Wake the Ghost (Registry -> Redis -> Memory Bank)
    koad boot --agent "$agent" --project
}

# The Portal to the Claude Body (Security & Implementation)
k-claude() { 
    koad-tether "${1:-Vigil}"
    claude
}

# The Portal to the Gemini Body (Architecture & Orchestration)
k-gemini() { 
    koad-tether "${1:-Tyr}"
    gemini
}

# The Portal to the Codex Body (Research & Deep Context)
k-codex() { 
    koad-tether "${1:-Vigil}"
    codex
}
```

---

## 4. Multi-Body Orchestration Pattern

This protocol allows for concurrent, isolated operations on the same codebase:

| **Terminal** | **Portal Command** | **Agent** | **Primary Focus** |
| :--- | :--- | :--- | :--- |
| Terminal 1 | `k-gemini Tyr` | **Tyr (Captain)** | Architecture, Planning, Spine Maintenance. |
| Terminal 2 | `k-claude Vigil` | **Vigil (Officer)** | Security Hooks, KSRP Review, Sanctuary Rule. |
| Terminal 3 | `k-gemini Sky` | **Sky (Officer)** | GitHub Command Deck, Project Syncing. |

---

## 5. Security & Isolation Verification

- **Zero Leakage:** Each terminal has a unique `KOAD_SESSION_ID`. Redis namespaces context by this ID.
- **Sanctuary Rule:** Host-level hooks (especially in Claude Code) block unauthorized writes to `.koad-os/` based on the inherited session state.
- **Auth Binding:** Directory-aware PAT resolution occurs at the shell level, ensuring the LLM never sees or manages raw secrets directly.

---

## 6. Next Steps

- [ ] Deploy the Unified Launch Deck to the Admiral's `.bashrc`.
- [ ] Transition `GEMINI.md` and `AGENTS.md` to "Neutral Body" platform canon.
- [ ] Verify Vigil's "Sanctuary Enforcement" within a `k-claude` session.
