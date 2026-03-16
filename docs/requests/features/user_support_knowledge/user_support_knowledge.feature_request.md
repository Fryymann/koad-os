# Support Knowledge Base Build — Agent Prompts

<aside>
📌

**Pipeline:** Tyr (Gemini 2.5 Pro) walks the codebase and produces structured outlines + raw technical notes → Claude (Sonnet 4.6) takes those outlines and writes polished, detailed knowledge articles for human consumption. The finished articles power Scribe (Flash Lite) as a RAG-backed support agent.

</aside>

---

## Pipeline Overview

| **Phase** | **Agent** | **Model** | **Input** | **Output** |
| --- | --- | --- | --- | --- |
| 1 — Extraction | Tyr | Gemini 2.5 Pro | Full KoadOS codebase + canon docs | `.koad-os/docs/support-knowledge/outlines/` — structured outlines + raw notes per topic |
| 2 — Authoring | Claude Code | Sonnet 4.6 | Tyr's outlines + source code references | `.koad-os/docs/support-knowledge/articles/` — polished, detailed KB articles |
| 3 — Serving | Scribe | Flash Lite | Finished KB articles (RAG retrieval) | Fast, accurate answers to human questions about the codebase |

---

## Prompt 1 — Tyr (Gemini 2.5 Pro): Codebase Walk & Outline Generation

<aside>
📋

**Usage:**  Provide the prompt below directly to Tyr in Gemini CLI. Tyr should have the full KoadOS repo in context.

</aside>

<aside>
📋

**Usage:** Paste the contents of the code block below to Claude Code. Claude should have access to the KoadOS repo, specifically the `.koad-os/docs/support-knowledge/outlines/` directory that Tyr has already populated.

</aside>

---

## Post-Pipeline: Scribe Configuration

<aside>
💡

**After Phase 2 is complete**, Scribe needs to be configured to use the articles directory as its retrieval source. The articles in `.koad-os/docs/support-knowledge/articles/` become Scribe's knowledge store. Configuration details will depend on the RAG implementation chosen (embedding-based vector search vs. keyword-based file retrieval).

</aside>