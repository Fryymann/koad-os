Tyr — Support Knowledge Base: Codebase Walk & Outline Generation
Date: 2026-03-15
Issued by: Ian (Admiral)
Status: ACTIVE DIRECTIVE — Knowledge Engineering Task
Priority: HIGH — Foundation for Scribe's human-facing support capability.

---

SITUATION BRIEFING

We are building a Support Knowledge Base for KoadOS. The goal: when a human
(Ian or a future contributor) asks "How does X work?" — Scribe (gemini-2.5-
flash-lite) can retrieve a thorough, detailed article and answer the question
accurately.

The pipeline has three phases:
  Phase 1 (YOU — Tyr): Walk the codebase. Produce structured outlines and raw
    technical notes for every major topic. Output to:
    .koad-os/docs/support-knowledge/outlines/
  Phase 2 (Claude Code — Sonnet 4.6): Takes your outlines + source references
    and writes polished, detailed knowledge articles.
  Phase 3 (Scribe — Flash Lite): Serves the finished articles via RAG to
    answer human questions.

YOU are Phase 1. Your job is extraction and structuring — NOT polished writing.
Produce raw, thorough, technically accurate outlines that give Claude everything
it needs to write excellent articles without re-reading the entire codebase.

---

MISSION

Perform a systematic walk of the KoadOS codebase and canon documentation.
For each identified topic, produce a structured outline file in:
  .koad-os/docs/support-knowledge/outlines/<topic-slug>.md

---

PHASE 1 — TOPIC DISCOVERY

Walk the codebase and identify every topic that a human might ask about.
Organize topics into these categories:

  A. ARCHITECTURE & CONCEPTS
     The big-picture systems and mental models.
     Examples:
       - Citadel → Station → Outpost workspace hierarchy
       - Body/Ghost model (shell environment vs. AI agent)
       - Agent rank system and scope boundaries
       - Map system (Atlas, Chart, Local, Passport)
       - Information flow patterns (downward, upward, lateral)
       - The agents/ folder convention

  B. CORE SYSTEMS & SUBSYSTEMS
     The actual running code — what it does and how it works.
     Examples:
       - koad boot — shell environment hydration
       - koad CLI — command structure and dispatch
       - CASS (Citadel Agent Services Stack) — Redis/SQLite/Qdrant layers
       - Koad Stream — message bus protocol
       - Agent lifecycle (boot → orient → work → EndOfWatch)
       - Memory system (current Memory Bank + future CASS)

  C. PROTOCOLS & GOVERNANCE
     The rules agents follow.
     Examples:
       - KoadOS Development Canon (the 9-step sequence)
       - KSRP (Koad Self-Review Protocol) — passes, severity, loop logic
       - PSRP (Post-Sprint Reflection Protocol) — Saveup entries
       - Sovereign GitHub Protocol
       - Approval Gates and escalation rules
       - Failure & Recovery Protocol

  D. AGENT ROLES & RESPONSIBILITIES
     Who does what.
     Examples:
       - Tyr — Chief Engineer role and capabilities
       - Scribe — Documentation and review role
       - Vigil — (if applicable) Monitoring role
       - Agent classification (crew, officer, commander, etc.)

  E. DATA & STORAGE
     How data is structured and persisted.
     Examples:
       - SQLite schema and usage patterns (koad.db)
       - Redis hot memory and pub/sub patterns
       - Qdrant vector storage and semantic retrieval
       - EndOfWatch summaries and session logs
       - Saveup entries and memory bank structure

  F. TOOLING & DEVELOPER WORKFLOW
     How to work with and within the system.
     Examples:
       - Shell functions (koad-context, koad-refresh, koad-map, etc.)
       - Git flow and branch conventions
       - Testing approach and coverage tooling
       - Cargo workspace structure
       - Environment variables and configuration

Do NOT limit yourself to these examples. Discover additional topics from the
actual codebase. If a module, subsystem, or concept exists in the code or
docs, it needs an outline.

---

PHASE 2 — OUTLINE GENERATION

For EACH discovered topic, create a file:
  .koad-os/docs/support-knowledge/outlines/<category>/<topic-slug>.md

Use this directory structure:
  outlines/
    architecture/
    core-systems/
    protocols/
    agent-roles/
    data-storage/
    tooling/

Each outline file MUST follow this template:

---BEGIN TEMPLATE---
# <Topic Title>

## Metadata
- Category: <A-F from above>
- Complexity: basic | intermediate | advanced
- Related Topics: <comma-separated list of other topic slugs>
- Key Source Files: <list of file paths in the codebase>
- Key Canon/Doc References: <list of Notion page names or doc files>

## Summary
<2-3 sentences: what this is, why it exists, and what problem it solves.>

## How It Works
<Detailed technical breakdown. Be thorough. Include:>
  - The sequence of operations / data flow
  - Key functions, structs, or modules involved (with file paths)
  - Important design decisions and why they were made
  - Edge cases and failure modes
  - How this interacts with other systems

## Key Code References
<For each critical code element:>
  - File: <path>
  - Element: <function/struct/module name>
  - Purpose: <what it does in context of this topic>
  - Notable: <any non-obvious behavior or important details>

## Configuration & Environment
<Any env vars, config files, or settings that affect this system.>
<Format: VAR_NAME — what it controls — where it's set>

## Common Questions a Human Would Ask
<List 5-10 questions a human might ask about this topic.>
<These become the retrieval targets for Scribe's RAG.>
  - "How does X work?"
  - "What happens when Y fails?"
  - "Where is Z configured?"
  - "Why does the system do A instead of B?"

## Raw Technical Notes
<Dump any additional technical detail here that doesn't fit above.>
<Code snippets, implementation notes, gotchas, historical context.>
<Better to over-include than under-include — Claude will curate.>
---END TEMPLATE---

---

PHASE 3 — INDEX GENERATION

After all outlines are written, create:
  .koad-os/docs/support-knowledge/outlines/INDEX.md

This file should contain:
  1. A full topic list organized by category
  2. For each topic: title, complexity, one-line summary
  3. A cross-reference map showing topic relationships
  4. A coverage assessment: what areas of the codebase are NOT yet covered

---

EXECUTION RULES

1. THOROUGHNESS OVER BREVITY.
   You are the extraction layer. Claude can trim — Claude cannot invent
   technical detail it never received. When in doubt, include more.

2. CITE EXACT FILE PATHS.
   Every claim about how the code works must reference the specific file(s).
   Claude and future maintainers need to verify against source.

3. FOLLOW THE CODE, NOT YOUR ASSUMPTIONS.
   If a system described in canon docs hasn't been implemented yet, note it
   as "Planned / Not Yet Implemented" with the doc reference. Do not
   describe unimplemented systems as if they exist in the codebase.

4. DISTINGUISH CURRENT STATE VS. DESIGN INTENT.
   The canon docs describe the intended architecture. The codebase reflects
   current reality. When they diverge, document BOTH:
     - Current implementation: <what the code actually does>
     - Design intent: <what the canon/docs say it should do>
     - Gap: <what's missing or different>

5. PRESERVE KOAD-OS TERMINOLOGY.
   Use the canonical terms: Citadel, Station, Outpost, Body/Ghost, CASS,
   Koad Stream, Saveup, EndOfWatch, Canon, KSRP, PSRP, Sanctuary, etc.
   Do not simplify or rename concepts.

6. FLAG AMBIGUITIES.
   If something in the codebase is unclear, contradictory, or poorly
   documented — flag it explicitly in the outline. Don't paper over gaps.

---

DELIVERABLES

When complete, the .koad-os/docs/support-knowledge/ directory should contain:

  outlines/
    INDEX.md                           ← master topic index
    architecture/
      citadel-station-outpost.md
      body-ghost-model.md
      agent-rank-system.md
      map-system.md
      ...
    core-systems/
      koad-boot.md
      koad-cli.md
      cass.md
      koad-stream.md
      agent-lifecycle.md
      ...
    protocols/
      development-canon.md
      ksrp.md
      psrp.md
      github-protocol.md
      ...
    agent-roles/
      tyr.md
      scribe.md
      ...
    data-storage/
      sqlite-koad-db.md
      redis-hot-memory.md
      qdrant-semantic.md
      ...
    tooling/
      shell-functions.md
      git-flow.md
      cargo-workspace.md
      ...

The exact topics will be determined by your codebase walk. The above is
illustrative — discover the real topology and document what actually exists.

---

BEGIN

Start with Phase 1 (Topic Discovery). Walk the full codebase directory
structure, read key files, and produce a preliminary topic list organized
by category. Present the topic list to me for review before proceeding
to Phase 2 (Outline Generation).

— Ian
  Admiral, KoadOS