# Support Knowledge Base Build — Agent Prompts
---
## Pipeline Overview
---
## Prompt 1 — Tyr (Gemini 2.5 Pro): Codebase Walk & Outline Generation
```plain text
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
```
---
## Prompt 2 — Claude Code (Sonnet 4.6): Knowledge Article Authoring
```plain text
Claude — Support Knowledge Base: Article Authoring
Date: 2026-03-15
Issued by: Ian (Admiral, KoadOS)
Status: ACTIVE DIRECTIVE — Knowledge Engineering Task
Priority: HIGH — Final articles power Scribe's human-facing support capability.

---

SITUATION BRIEFING

We are building a Support Knowledge Base for KoadOS — a Rust-based multi-agent
operating system. The knowledge base will be served by Scribe (gemini-2.5-flash-
lite) as a RAG-backed support agent that answers human questions about how the
codebase functions.

The pipeline:
  Phase 1 (COMPLETE — Tyr / Gemini 2.5 Pro): Walked the codebase and produced
    structured outlines with raw technical notes for every major topic.
    Located at: .koad-os/docs/support-knowledge/outlines/

  Phase 2 (YOU — Claude / Sonnet 4.6): Take Tyr's outlines and write polished,
    detailed, human-readable knowledge articles.
    Output to: .koad-os/docs/support-knowledge/articles/

  Phase 3 (Scribe — Flash Lite): Serves your finished articles via RAG to
    answer human questions about the codebase.

YOU are Phase 2. Your job is authoring — taking raw technical extraction and
turning it into clear, thorough, well-structured articles that a human (or
a lightweight LLM doing retrieval) can use to fully understand each topic.

---

SOURCE MATERIAL

Your primary input is:
  .koad-os/docs/support-knowledge/outlines/INDEX.md  ← start here
  .koad-os/docs/support-knowledge/outlines/<category>/<topic>.md

Each outline contains:
  - Metadata (category, complexity, related topics, source files)
  - Summary
  - How It Works (technical breakdown)
  - Key Code References (file paths, functions, structs)
  - Configuration & Environment
  - Common Questions a Human Would Ask
  - Raw Technical Notes

You also have access to the full KoadOS codebase. Use it to:
  - Verify Tyr's claims against actual source code
  - Fill in gaps where Tyr's outlines are thin
  - Pull code snippets for illustration when they clarify a concept
  - Cross-reference between topics for accuracy

---

MISSION

For each outline in outlines/, produce a corresponding article in:
  .koad-os/docs/support-knowledge/articles/<category>/<topic-slug>.md

Mirror the directory structure:
  articles/
    architecture/
    core-systems/
    protocols/
    agent-roles/
    data-storage/
    tooling/

---

ARTICLE FORMAT

Each article MUST follow this structure:

---BEGIN TEMPLATE---
# <Topic Title>

> <One-line description: what this is and why it matters.>

**Complexity:** basic | intermediate | advanced
**Related Articles:** [Topic A](../category/topic-a.md), [Topic B](...)

---

## Overview

<2-4 paragraphs explaining the concept at a high level. Assume the reader is
a developer who is new to KoadOS but experienced with Rust and systems
programming. Explain the WHY before the HOW. What problem does this solve?
Where does it sit in the overall architecture?>

## How It Works

<Detailed walkthrough of the system/concept. Use subsections (### headers)
to break up complex topics. Include:>

  - Step-by-step flows where applicable
  - Key data structures and their purposes
  - Decision points and branching logic
  - Interactions with other KoadOS systems (link to their articles)

<Use code snippets sparingly but effectively — show the essential code that
illuminates the concept, not full function dumps. Annotate snippets with
comments explaining what matters.>

## Configuration

<Environment variables, config files, CLI flags that affect this system.
Format as a table or definition list. Include default values and examples.>

## Failure Modes & Edge Cases

<What can go wrong? How does the system degrade? What should a human know
about error states, recovery, and debugging?>

## FAQ

<Take the "Common Questions" from Tyr's outline and answer each one
thoroughly. These are the primary retrieval targets for Scribe's RAG —
the questions should be natural language as a human would phrase them,
and the answers should be complete enough to stand alone.>

Format each as:
### Q: <Question as a human would ask it?>
<Thorough answer. 2-5 sentences minimum. Reference specific files, commands,
or config when relevant.>

## Source Reference

<List the key source files for this topic. A developer who wants to read the
actual implementation can start here.>
  - `<file path>` — <one-line description of what's relevant in this file>
---END TEMPLATE---

---

WRITING STANDARDS

1. WRITE FOR HUMANS, NOT AGENTS.
   These articles will be read by developers (primarily Ian) and retrieved
   by Scribe to answer human questions. Write in clear, direct technical
   prose. No filler. No hedging. No "it should be noted that" — just say it.

2. BE THOROUGH.
   A human asking "How does boot hydration work?" should get a complete
   answer from the article — not a summary that sends them to read source
   code. The article IS the explanation. Source references are for those
   who want to go deeper, not a substitute for explanation.

3. PRESERVE KOAD-OS VOICE AND TERMINOLOGY.
   KoadOS has its own terminology: Citadel, Station, Outpost, Body/Ghost,
   CASS, Koad Stream, Saveup, EndOfWatch, Canon, KSRP, PSRP, Sanctuary,
   Admiral, etc. Use these terms consistently. On first use in each article,
   briefly define or contextualize the term, then use it naturally.

4. CROSS-LINK BETWEEN ARTICLES.
   When an article references a concept covered by another article, link to
   it: [Body/Ghost Model](../architecture/body-ghost-model.md). This builds
   the knowledge graph that Scribe can traverse.

5. VERIFY AGAINST SOURCE CODE.
   Tyr's outlines are your starting point, not gospel. If an outline claims
   a function does X, open the file and confirm. If the code diverges from
   the outline, trust the code and note the discrepancy.

6. DISTINGUISH IMPLEMENTED VS. PLANNED.
   KoadOS has ambitious design docs that outpace the current implementation.
   Clearly mark anything that is designed but not yet built:
     > **Note:** CASS integration is currently in design phase. The current
     > implementation uses the Memory Bank (Notion pages) as described below.
     > See [CASS](../core-systems/cass.md) for the planned architecture.

7. MAKE FAQs RETRIEVAL-OPTIMIZED.
   Scribe will use these FAQs as primary retrieval targets. Write questions
   the way a human would actually phrase them — conversational, specific.
   Write answers that are self-contained — a human should understand the
   answer even if they only read that FAQ entry.

---

EXECUTION PLAN

1. Read .koad-os/docs/support-knowledge/outlines/INDEX.md to understand the
   full topic landscape and relationships.

2. Process topics in this order (dependencies first):
   a. Architecture articles (these are referenced by everything else)
   b. Core Systems articles
   c. Protocols articles
   d. Agent Roles articles
   e. Data & Storage articles
   f. Tooling articles

3. For each topic:
   a. Read the outline file
   b. Read the key source files referenced in the outline
   c. Write the article following the template above
   d. Verify all code references are accurate
   e. Add cross-links to related articles (even if not yet written —
      use the expected path based on the topic slug)

4. After all articles are written, create:
   .koad-os/docs/support-knowledge/articles/INDEX.md
   Containing:
     - Full article list organized by category
     - For each article: title, complexity, one-line summary, link
     - A "Start Here" reading order for someone new to KoadOS
     - Any coverage gaps or topics that need additional articles

5. Also create:
   .koad-os/docs/support-knowledge/articles/GLOSSARY.md
   A comprehensive glossary of all KoadOS-specific terms with definitions.
   This becomes Scribe's terminology reference.

---

QUALITY GATE

Before declaring Phase 2 complete, self-check:
  [ ] Every outline has a corresponding article
  [ ] Every article follows the template structure
  [ ] All code references point to real files that exist in the codebase
  [ ] Cross-links use correct relative paths
  [ ] INDEX.md is complete and accurate
  [ ] GLOSSARY.md covers all KoadOS-specific terms
  [ ] FAQs are written as natural human questions with self-contained answers
  [ ] No article describes unimplemented features as if they're live
      (or clearly marks them as planned)

---

BEGIN

Start by reading the INDEX.md to understand the full scope. Then begin with
the architecture category — these foundational articles will be referenced
by everything else.

Present a brief status update after completing each category.

— Ian
  Admiral, KoadOS
```
---
## Post-Pipeline: Scribe Configuration
