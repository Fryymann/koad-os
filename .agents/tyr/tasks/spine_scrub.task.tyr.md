Tyr — Spine Legacy Scrub: Deep Search & Eradication Plan
Date: 2026-03-16
Issued by: Ian (Admiral)
Status: ACTIVE DIRECTIVE — Codebase Hygiene Task
Priority: HIGH — Architectural clarity is a prerequisite for all
         forward engineering work on the Citadel.

---

MISSION BRIEFING

The monolithic Koad Spine has been retired. Its responsibilities have been
decomposed and reassigned to the Citadel, CASS, and Agent-Level Cognition.
A Spine Retirement Record exists documenting the intended migration mapping.

However, no systematic scrub has confirmed that all traces of the Spine era
have actually been cleaned from the living codebase, documentation, prompts,
configs, and memory artifacts. Your mission is to perform that scrub.

You are the authority on what the Citadel architecture looks like today.
Use that understanding to distinguish between:
  - References that are genuinely stale and must be removed
  - Concepts that originated in the Spine era but remain valid under
    the Citadel (these should be re-framed, not deleted)
  - Historical/archival references that are clearly marked as such
    (these can stay if properly labeled)

---

DIRECTIVE — Three Phases

Execute in order. Report findings at each gate before proceeding.

===================================================================
PHASE 1 — Deep Search & Inventory
===================================================================

Conduct an exhaustive search of every surface you can reach.
You decide the best approach for discovering references — use whatever
methods, traversal strategies, and search patterns you judge most
effective. Be thorough. Assume nothing has been cleaned unless you
verify it yourself.

You are looking for ALL of the following:

  1. TERMINOLOGY
     Any use of retired names, labels, or identifiers from the Spine era.
     This includes — but is not limited to — terms like:
       "The Spine", "Koad Spine", "k-spine", "kspine",
       "spine session", "koad-spine", "koad-asm", "spine.proto",
       "Spine Session", "koad spine [command]"
     Also watch for subtle variants, abbreviations, or casual references
     that imply the Spine still exists as a living system.

  2. ARCHITECTURAL ASSUMPTIONS
     Code or documentation that assumes a monolithic session broker,
     a single central process managing all agent state, or any pattern
     that was specific to how the Spine worked — even if the word
     "Spine" doesn't appear. The Citadel's model is zero-trust,
     isolated, and tier-separated. Anything that contradicts that
     model is a candidate for scrubbing.

  3. DEAD REFERENCES
     Imports, paths, crate names, proto definitions, config keys,
     CLI subcommands, or file references that point to artifacts that
     no longer exist (e.g., crates/koad-spine/, proto/spine.proto,
     koad-gateway, koad-tui, kdnd-tui).

  4. FRAMING & NARRATIVE
     Documentation, prompts, comments, or memory artifacts that
     describe KoadOS using Spine-era framing. This includes things
     like describing the system as having a "central spine" or a
     "backbone" that connects all agents, or any language that
     implies a monolithic coordination layer.

  5. LOGIC & PATTERNS
     Any code paths, control flow, or design patterns that were built
     to interface with the Spine and have not been refactored for the
     Citadel's architecture. This is the hardest category — it
     requires understanding intent, not just matching strings.

For each reference found, record:
  - Location (file, section, or artifact)
  - The exact text or pattern
  - Category (terminology / assumption / dead ref / framing / logic)
  - Severity: CRITICAL (actively misleading or broken),
             MODERATE (confusing but not dangerous),
             LOW (cosmetic or archival)

Deliver Phase 1 as a structured inventory before proceeding.

===================================================================
PHASE 2 — Eradication Plan
===================================================================

Using your Phase 1 inventory, produce a detailed hit-list organized
by action type:

  ACTION: REPLACE
    References where the Spine term or concept has a direct Citadel-era
    equivalent. Specify the exact old → new replacement.
    Example: "Spine Session" → "Citadel Session" or "Personal Bay
    Connection" depending on context.

  ACTION: REFRAME
    Passages where the underlying concept is still valid but the
    framing needs to be rewritten for the Citadel model. Provide a
    brief description of what the rewrite should convey.

  ACTION: DELETE
    Dead references, orphaned imports, stale config keys, or
    documentation for systems that no longer exist and have no
    Citadel equivalent. Confirm that deletion has no downstream
    dependencies.

  ACTION: ARCHIVE-LABEL
    Historical references that should be preserved for context but
    need to be clearly marked as "Spine era — archived" so no agent
    or contributor mistakes them for current architecture.

  ACTION: INVESTIGATE
    Anything you're unsure about. Flag it with your best guess and
    the specific question you need answered before acting.

For each action item, include:
  - The target location
  - The proposed change (or question, for INVESTIGATE items)
  - Risk level of the change (safe / needs-review / risky)
  - Whether the change is isolated or has ripple effects elsewhere

Organize the plan in execution order: safe, isolated changes first;
  risky or interconnected changes last.

===================================================================
PHASE 3 — Execution (Admiral Approval Required)
===================================================================

After I approve your Phase 2 plan:

1. Execute all SAFE changes.
2. Present NEEDS-REVIEW changes for my sign-off before executing.
3. Flag RISKY changes with a detailed impact assessment — do not
   execute without explicit Admiral approval.
4. After all changes are applied, re-run your Phase 1 search as a
   verification sweep. Report any remaining references.

Do NOT proceed to Phase 3 without Admiral approval.

---

DESIGN PRINCIPLES

1. PRESERVE CORE LOGIC.
   The Citadel, CASS, and Agent-Level Cognition are the current
   architecture. Any concept, abstraction, or pattern that is valid
   under this model MUST be preserved — even if it originated during
   the Spine era. Rename and reframe; do not destroy valid logic.

2. ZERO AMBIGUITY IS THE GOAL.
   After this scrub, a new agent or contributor reading the codebase
   should encounter ZERO references that imply the Spine is a living
   system. The Citadel architecture should be the only mental model
   the codebase reinforces.

3. ARCHIVAL IS NOT EVASION.
   Labeling something as archived is a valid action, but only for
   genuinely historical content (e.g., a retirement record, a
   changelog entry). Do not use ARCHIVE-LABEL as a shortcut to
   avoid rewriting something that should be reframed.

4. CONTEXT OVER PATTERN MATCHING.
   A string match on "spine" is necessary but not sufficient.
   Some references may be legitimate (e.g., anatomical metaphors,
   unrelated projects). Evaluate context. Conversely, some code
   may embody Spine-era assumptions without ever using the word.
   Think architecturally, not just textually.

5. WORK CLEAN.
   Every change should leave the affected file in a better state
   than you found it. If you're already editing a file, fix
   adjacent issues (formatting, stale comments, etc.) while you're
   there — but don't scope-creep into unrelated refactors.

---

CRITICAL EVALUATION MANDATE

Before finalizing your Phase 2 plan, stress-test it:

- Are there any downstream systems, agents, or scripts that depend
  on a Spine-era name or path that you're proposing to change?
  If so, those dependencies must be updated in the same pass.

- Could any proposed deletion remove context that a future engineer
  would need to understand WHY the Citadel works the way it does?
  If so, reframe instead of delete.

- Is your search truly exhaustive, or are there surfaces you couldn't
  reach? Explicitly list any blind spots so I can investigate them
  separately.

---

Report your Phase 1 inventory first. I will review before you proceed.

— Ian
  Admiral, KoadOS