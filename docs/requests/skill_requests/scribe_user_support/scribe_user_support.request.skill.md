# Skill Request — Scribe: Support Knowledge Base Q&A

<aside>
📌

**Type:** Skill Implementation Request

**Requesting Agent:** Noti (on behalf of Ian)

**Implementing Agent:** Tyr

**Target Agent:** Scribe (gemini-2.5-flash-lite)

**Date:** 2026-03-15

**Status:** PENDING — Awaiting Tyr implementation

**Prerequisite:** Support Knowledge Base articles must exist in `.koad-os/docs/support-knowledge/articles/` (see [Support Knowledge Base — Agent Prompts (Tyr + Claude)](https://www.notion.so/Support-Knowledge-Base-Agent-Prompts-Tyr-Claude-915c47d0474e4f6e9e9e1d8f6411a4e5?pvs=21))

**Architecture Reference:** [KoadOS Feature Request — Skill System Architecture (Pattern Extraction from Claude Code)](https://www.notion.so/KoadOS-Feature-Request-Skill-System-Architecture-Pattern-Extraction-from-Claude-Code-4c0e75847fe546d3b2050c6197a5ed11?pvs=21)

</aside>

---

## Purpose

Implement a **support-kb** skill for Scribe that enables him to answer developer questions about how the KoadOS codebase functions. When a human asks "How does boot hydration work?" or "What is the Body/Ghost model?" — Scribe retrieves the relevant knowledge article(s), synthesizes an accurate answer, and cites the source material.

This is **Scribe's first production skill** and serves as the reference implementation for the KoadOS Skill System pattern.

---

## Problem Statement

Scribe (Flash Lite) is fast and cheap but has no innate knowledge of the KoadOS codebase. Without a structured retrieval mechanism, Scribe would need the entire codebase in context to answer questions — which defeats the purpose of using a lite model.

The Support Knowledge Base (authored by Claude in Phase 2 of the KB pipeline) provides thorough, pre-written articles covering every major KoadOS subsystem. Scribe needs a **skill** that:

1. **Indexes** the knowledge articles for fast retrieval
2. **Matches** human questions to the most relevant article(s)
3. **Retrieves** the matched content into Scribe's context
4. **Synthesizes** a thorough answer using the article content
5. **Cites** the source article(s) so the human can go deeper

---

## Skill Anatomy (Per Skill System Architecture)

Implement the following directory structure:

```
.koad-os/skills/scribe/support-kb/
├── SKILL.md              # Skill definition + instructions
├── scripts/
│   ├── index-articles.sh     # Builds/refreshes the article index
│   ├── search-articles.sh    # Searches articles by keyword/topic
│   └── retrieve-article.sh   # Retrieves a specific article by slug
├── references/
│   └── article-index.md      # Generated index of all KB articles
├── agents/               # (empty for v1 — no sub-agent delegation)
└── _eval/
    ├── test-prompts.md       # Sample questions for validation
    └── grading-schema.md     # Expected answer quality criteria
```

---

## [SKILL.md](http://SKILL.md) Specification

The [SKILL.md](http://SKILL.md) file should contain the following frontmatter and instruction body:

### Frontmatter

```yaml
name: support-kb
description: >
  Answer developer questions about the KoadOS codebase, architecture,
  protocols, agent roles, data systems, and tooling. Retrieves from the
  pre-authored Support Knowledge Base articles. Trigger when a user asks
  how something works, what a system does, where something is configured,
  why a design decision was made, or any question about KoadOS internals.
trigger_patterns:
  - "How does * work?"
  - "What is *?"
  - "Where is * configured?"
  - "Why does KoadOS *?"
  - "Explain *"
  - "What happens when *?"
requires: []                    # No infrastructure dependencies for v1
tier: crew                      # Scribe is a crew-level agent
context_cost: medium            # Article retrieval adds ~500-2000 tokens per query
author: tyr
version: 1.0.0
```

### Instruction Body

The [SKILL.md](http://SKILL.md) body should instruct Scribe to follow this workflow:

**Phase 1 — Intent Classification**

- Parse the user's question to extract the core topic and intent
- Map the topic to one or more KB categories: `architecture`, `core-systems`, `protocols`, `agent-roles`, `data-storage`, `tooling`
- If the question spans multiple topics, identify the primary and secondary articles

**Phase 2 — Article Retrieval**

- Consult the article index (`references/article-index.md`) to identify the best-match article(s)
- Load the primary article's content into context
- If the primary article cross-links to related articles that are directly relevant, load the FAQ section of those secondary articles as well
- **Context budget:** Load at most 3 articles per query to stay within Flash Lite's effective context

**Phase 3 — Answer Synthesis**

- Answer the question using **only** the content from retrieved articles
- Do not hallucinate or infer beyond what the articles state
- If the articles distinguish between *implemented* and *planned* features, preserve that distinction in the answer
- Use KoadOS canonical terminology (Citadel, Station, Outpost, Body/Ghost, CASS, etc.)
- If the question references something not covered by any article, say so explicitly and suggest the user check with Tyr or Noti

**Phase 4 — Citation**

- Cite the source article(s) at the end of the answer
- Format: `Source: <article-title> (.koad-os/docs/support-knowledge/articles/<category>/<slug>.md)`
- If the answer used FAQ entries, cite the specific Q&A

---

## Pattern Implementation Map

This skill implements 6 of the 8 Skill System Architecture patterns:

| **Pattern** | **Implementation in this Skill** |
| --- | --- |
| **P1: Progressive Disclosure** | Frontmatter (always loaded, ~100 tokens) → [SKILL.md](http://SKILL.md) body (loaded on trigger, ~400 tokens) → article content (loaded on demand per query). Three-tier context loading. |
| **P2: Phased Convergence** | Four-phase workflow (Intent → Retrieve → Synthesize → Cite) with re-entry support. If a follow-up question arrives, Scribe re-enters at Phase 1 with prior articles still in context. |
| **P3: Description as Router** | The `description` and `trigger_patterns` fields in frontmatter serve as the semantic routing surface. Written to trigger on natural developer questions about KoadOS. |
| **P4: Graceful Degradation** | `requires: []` — no infrastructure dependencies. Works in Dark Mode, Cold Boot, or Full Citadel. If the article index is missing, Scribe falls back to listing available article directories. If articles don't cover the topic, Scribe says so explicitly. |
| **P6: Eval-Driven Development** | `_eval/test-prompts.md` contains sample questions spanning all 6 KB categories. `_eval/grading-schema.md` defines quality criteria: accuracy, completeness, terminology correctness, citation presence. |
| **P7: Anti-Overfitting** | Instructions explain *why* each phase exists and what good answers look like — not rigid scripts. Scribe is trusted to synthesize, not just regurgitate. |

**Patterns deferred to v2:**

- **P5: Subagent Delegation** — v1 is single-agent. v2 could delegate complex cross-cutting questions to a grader sub-agent that evaluates answer quality.
- **P8: Bundled Scripts** — v1 scripts are simple index/search/retrieve. v2 could bundle more sophisticated search (embedding-based) once Qdrant is available.

---

## Scripts Specification

### `scripts/index-articles.sh`

**Purpose:** Scans `.koad-os/docs/support-knowledge/articles/` and generates `references/article-index.md` — a structured index Scribe consults for article discovery.

**Behavior:**

- Walk the articles directory tree
- For each article, extract: title (H1), one-line description (blockquote), complexity, category, FAQ questions
- Output a structured index organized by category with topic → file path mapping
- Include all FAQ questions as a flat searchable list with article back-references
- Idempotent — safe to re-run. Overwrites the existing index.

**When to run:** After KB articles are created or updated. Can be triggered by `koad-refresh` or run manually.

**Estimated complexity:** ~50-80 lines bash. Uses `grep`, `sed`, `awk` — no external dependencies.

### `scripts/search-articles.sh`

**Purpose:** Given a search query (keywords or natural language), returns ranked article matches from the index.

**Behavior:**

- Accept a query string as argument
- Search against article titles, descriptions, FAQ questions, and category names
- Return top 3 matches with file paths and match context
- Simple keyword matching for v1 (Qdrant semantic search in v2)

**Estimated complexity:** ~30-50 lines bash. Grep-based search against the index file.

### `scripts/retrieve-article.sh`

**Purpose:** Given an article slug or path, returns the article content (or a specific section).

**Behavior:**

- Accept an article path or `<category>/<slug>` identifier
- Return the full article content by default
- Optional `--section` flag to return only a specific section (e.g., `--section FAQ`)
- Optional `--brief` flag to return only Overview + FAQ (reduced context cost)

**Estimated complexity:** ~20-40 lines bash. File read + optional section extraction with `sed`.

---

## Eval Specification

### `_eval/test-prompts.md`

Contain at least 15 test questions spanning all 6 KB categories:

**Architecture (3 questions):**

- "How does the Citadel → Station → Outpost hierarchy work?"
- "What is the Body/Ghost model and why does KoadOS use it?"
- "How does the agent map system help agents navigate the codebase?"

**Core Systems (3 questions):**

- "How does boot hydration work for agents?"
- "What is CASS and what are its three memory tiers?"
- "How does Koad Stream handle inter-agent communication?"

**Protocols (3 questions):**

- "Walk me through the KoadOS Development Canon steps."
- "What are the KSRP passes and when do I run them?"
- "What happens at an Approval Gate if the agent doesn't get explicit approval?"

**Agent Roles (2 questions):**

- "What is Tyr's role in KoadOS?"
- "What's the difference between a crew agent and an officer agent?"

**Data & Storage (2 questions):**

- "How does KoadOS use SQLite for agent memory?"
- "What is an EndOfWatch summary and where is it stored?"

**Tooling (2 questions):**

- "What shell functions does `koad boot` make available?"
- "How does the KoadOS git flow work?"

### `_eval/grading-schema.md`

Define quality criteria for each test answer:

| **Criterion** | **Weight** | **Pass Threshold** |
| --- | --- | --- |
| **Accuracy** — Answer is factually correct per source articles | 40% | No factual errors. Claims match article content. |
| **Completeness** — Answer addresses the full scope of the question | 25% | Covers the main concept + at least one relevant detail/example. |
| **Terminology** — Uses KoadOS canonical terms correctly | 15% | No made-up terms. Canonical terms used where applicable. |
| **Citation** — Source article(s) cited | 10% | At least one source article cited. Path is valid. |
| **Implemented vs. Planned** — Correctly distinguishes current state from design intent | 10% | No unimplemented features described as live without qualification. |

**Pass/Fail:** A test answer passes if it scores ≥ 80% weighted. The skill passes eval if ≥ 80% of test answers pass.

---

## Integration with Existing Systems

### Boot-Time Loading

- Skill metadata (frontmatter) should be loadable at boot via `koad boot --agent scribe`
- The [SKILL.md](http://SKILL.md) body loads only when a question triggers the skill
- Article content loads only per-query — never bulk-loaded at boot

### Dark Mode Compatibility

- This skill has **zero infrastructure dependencies** — it reads local markdown files
- Works identically in Full Citadel, Dark Mode, and Cold Boot
- The only requirement is that the articles directory exists and is populated

### Refresh Cycle

- When articles are updated, run `scripts/index-articles.sh` to regenerate the index
- Scribe does not need to restart — the index is read per-query
- Consider adding a `koad-refresh` hook that auto-runs the indexer when articles change

### Future Qdrant Integration (v2)

- When CASS Phase 6 brings Qdrant online, replace keyword search with embedding-based semantic search
- Embed all FAQ questions + article summaries as vectors
- Search becomes: embed user query → find nearest FAQ/article vectors → retrieve
- This upgrades accuracy significantly without changing the skill's external interface

---

## Deliverables for Tyr

1. **Implement the skill directory structure** at `.koad-os/skills/scribe/support-kb/`
2. **Write [SKILL.md](http://SKILL.md)** with frontmatter + instruction body per the spec above
3. **Implement the three scripts** — `index-articles.sh`, `search-articles.sh`, `retrieve-article.sh`
4. **Write the eval files** — `test-prompts.md` with 15+ questions, `grading-schema.md` with criteria
5. **Generate the initial article index** by running `index-articles.sh` against the completed KB articles
6. **Run eval** — feed test prompts to Scribe with the skill loaded, grade against schema, report results
7. **KSRP self-review** on all deliverables before presenting to Ian

---

## Execution Rules

1. **This is an implementation task.** Tyr builds the skill. Follow the Canon: View & Assess → Research → Plan → Approval Gate → Implement → KSRP → PSRP → Results Report.
2. **Prerequisite gate:** The KB articles must exist in `.koad-os/docs/support-knowledge/articles/` before the eval phase. The skill structure and scripts can be built in parallel with article authoring.
3. **Skill Writing Guide compliance:** Instructions in [SKILL.md](http://SKILL.md) should follow Pattern 7 (Anti-Overfitting) — explain *why* before *what*, write for class-level coverage, treat Scribe as a collaborator.
4. **v1 scope only.** No Qdrant, no sub-agents, no dynamic mid-session injection. Keep it file-based and bash-powered. v2 upgrades are noted but not built.
5. **Test with real questions.** The eval is not optional. Scribe must demonstrably answer questions correctly using the skill before delivery.

---

## Success Criteria

The skill is complete when:

- [ ]  A human can ask Scribe "How does X work?" and get a thorough, accurate, cited answer
- [ ]  Scribe correctly routes to the right article(s) for questions across all 6 categories
- [ ]  Scribe explicitly says "I don't have information on that" for topics not in the KB
- [ ]  Scribe preserves the implemented-vs-planned distinction from the source articles
- [ ]  Eval passes at ≥ 80% on the test prompt suite
- [ ]  All scripts are idempotent, fast (<1s), and work without external dependencies

---

<aside>
⚡

**This is the first KoadOS skill implementation.** It sets the pattern for all future skills. Build it clean, document the decisions, and treat it as the reference implementation that future skill authors will study.

</aside>