# Skill Request — Scribe: Support Knowledge Base Q&A
---
## Purpose
Implement a support-kb skill for Scribe that enables him to answer developer questions about how the KoadOS codebase functions. When a human asks "How does boot hydration work?" or "What is the Body/Ghost model?" — Scribe retrieves the relevant knowledge article(s), synthesizes an accurate answer, and cites the source material.
This is Scribe's first production skill and serves as the reference implementation for the KoadOS Skill System pattern.
---
## Problem Statement
Scribe (Flash Lite) is fast and cheap but has no innate knowledge of the KoadOS codebase. Without a structured retrieval mechanism, Scribe would need the entire codebase in context to answer questions — which defeats the purpose of using a lite model.
The Support Knowledge Base (authored by Claude in Phase 2 of the KB pipeline) provides thorough, pre-written articles covering every major KoadOS subsystem. Scribe needs a skill that:
1. Indexes the knowledge articles for fast retrieval
1. Matches human questions to the most relevant article(s)
1. Retrieves the matched content into Scribe's context
1. Synthesizes a thorough answer using the article content
1. Cites the source article(s) so the human can go deeper
---
## Skill Anatomy (Per Skill System Architecture)
Implement the following directory structure:
```plain text
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
## SKILL.md Specification
The SKILL.md file should contain the following frontmatter and instruction body:
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
The SKILL.md body should instruct Scribe to follow this workflow:
Phase 1 — Intent Classification
- Parse the user's question to extract the core topic and intent
- Map the topic to one or more KB categories: architecture, core-systems, protocols, agent-roles, data-storage, tooling
- If the question spans multiple topics, identify the primary and secondary articles
Phase 2 — Article Retrieval
- Consult the article index (references/article-index.md) to identify the best-match article(s)
- Load the primary article's content into context
- If the primary article cross-links to related articles that are directly relevant, load the FAQ section of those secondary articles as well
- Context budget: Load at most 3 articles per query to stay within Flash Lite's effective context
Phase 3 — Answer Synthesis
- Answer the question using only the content from retrieved articles
- Do not hallucinate or infer beyond what the articles state
- If the articles distinguish between implemented and planned features, preserve that distinction in the answer
- Use KoadOS canonical terminology (Citadel, Station, Outpost, Body/Ghost, CASS, etc.)
- If the question references something not covered by any article, say so explicitly and suggest the user check with Tyr or Noti
Phase 4 — Citation
- Cite the source article(s) at the end of the answer
- Format: Source: <article-title> (.koad-os/docs/support-knowledge/articles/<category>/<slug>.md)
- If the answer used FAQ entries, cite the specific Q&A
---
## Pattern Implementation Map
This skill implements 6 of the 8 Skill System Architecture patterns:
Patterns deferred to v2:
- P5: Subagent Delegation — v1 is single-agent. v2 could delegate complex cross-cutting questions to a grader sub-agent that evaluates answer quality.
- P8: Bundled Scripts — v1 scripts are simple index/search/retrieve. v2 could bundle more sophisticated search (embedding-based) once Qdrant is available.
---
## Scripts Specification
### scripts/index-articles.sh
Purpose: Scans .koad-os/docs/support-knowledge/articles/ and generates references/article-index.md — a structured index Scribe consults for article discovery.
Behavior:
- Walk the articles directory tree
- For each article, extract: title (H1), one-line description (blockquote), complexity, category, FAQ questions
- Output a structured index organized by category with topic → file path mapping
- Include all FAQ questions as a flat searchable list with article back-references
- Idempotent — safe to re-run. Overwrites the existing index.
When to run: After KB articles are created or updated. Can be triggered by koad-refresh or run manually.
Estimated complexity: ~50-80 lines bash. Uses grep, sed, awk — no external dependencies.
### scripts/search-articles.sh
Purpose: Given a search query (keywords or natural language), returns ranked article matches from the index.
Behavior:
- Accept a query string as argument
- Search against article titles, descriptions, FAQ questions, and category names
- Return top 3 matches with file paths and match context
- Simple keyword matching for v1 (Qdrant semantic search in v2)
Estimated complexity: ~30-50 lines bash. Grep-based search against the index file.
### scripts/retrieve-article.sh
Purpose: Given an article slug or path, returns the article content (or a specific section).
Behavior:
- Accept an article path or <category>/<slug> identifier
- Return the full article content by default
- Optional --section flag to return only a specific section (e.g., --section FAQ)
- Optional --brief flag to return only Overview + FAQ (reduced context cost)
Estimated complexity: ~20-40 lines bash. File read + optional section extraction with sed.
---
## Eval Specification
### _eval/test-prompts.md
Contain at least 15 test questions spanning all 6 KB categories:
Architecture (3 questions):
- "How does the Citadel → Station → Outpost hierarchy work?"
- "What is the Body/Ghost model and why does KoadOS use it?"
- "How does the agent map system help agents navigate the codebase?"
Core Systems (3 questions):
- "How does boot hydration work for agents?"
- "What is CASS and what are its three memory tiers?"
- "How does Koad Stream handle inter-agent communication?"
Protocols (3 questions):
- "Walk me through the KoadOS Development Canon steps."
- "What are the KSRP passes and when do I run them?"
- "What happens at an Approval Gate if the agent doesn't get explicit approval?"
Agent Roles (2 questions):
- "What is Tyr's role in KoadOS?"
- "What's the difference between a crew agent and an officer agent?"
Data & Storage (2 questions):
