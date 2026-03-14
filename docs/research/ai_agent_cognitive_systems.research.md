<aside>
🧠

Research compiled by Noti on March 7, 2026. Covers the full spectrum of factors a robust AI agent cognitive architecture must address, drawn from current literature and established patterns in agentic AI systems.

</aside>

---

## A. Memory Architecture

### 1. Working / Short-term Memory

The active session context. What the agent is currently holding in its context window. Must be managed to avoid context bloat — research shows noise accumulation degrades reasoning quality and that most behavioral divergence traces back to early-step errors compounding through a cluttered context.

**Key concern:** Is the context window being used efficiently, or does irrelevant information persist and reduce signal quality?

### 2. Semantic Memory

Persistent factual knowledge: rules, configs, definitions, domain expertise. Not tied to specific events — this is the agent's general knowledge base. Must be structured and searchable, and kept current when facts change.

**Key concern:** Is it queryable, or just raw text that floods context?

### 3. Episodic Memory

Records of specific past experiences and their outcomes, enabling case-based reasoning. "Last time I did X, Y happened." Foundation for learning from mistakes across sessions.

**Key concern:** Are past experiences actually retrievable in a structured way, or lost at session end?

### 4. Procedural Memory

Encoded behavioral patterns — *how* to do things correctly. Separate from facts (semantic) and events (episodic). When Tyr learns the right way to handle a task class, is that pattern stored somewhere it will be retrieved the next time the same class of task appears?

**Key concern:** This is the layer most directly responsible for eliminating repeated mistakes. If it's missing or not triggered, mistakes will recur indefinitely.

---

## B. Cognitive Processes

### 5. Self-Reflection / Metacognition

After generating an output or plan, the agent evaluates its own work before delivery. A dedicated reflection pass — not just "does this look right" but a structured critique against specific criteria. Research shows reflection passes produce significantly higher quality and more consistent output.

**Key concern:** Is reflection actually modifying behavior, or is it ceremonial?

### 6. Self-Correction Loops

The full Detect → Evaluate → Correct → Re-verify cycle. Not just logging an error and continuing — actually fixing and confirming the fix before proceeding. The critic/reviewer pattern: separate the generator from the evaluator.

**Key concern:** Does the correction loop actually change output, or does it dead-end after logging?

### 7. Chain of Thought Transparency

Making reasoning steps explicit and traceable. Enables post-hoc identification of where reasoning went wrong — useful for both Ian and Noti when diagnosing failures.

**Key concern:** Without transparent reasoning, errors are invisible and root cause analysis is guesswork.

### 8. Behavioral Consistency

For identical or near-identical tasks, does the agent produce consistent action sequences? A 2025 study across 3,000 agent runs found agents produce 2–4.2 distinct action sequences per 10 runs on *identical* inputs. Tasks with consistent behavior achieve 80–92% accuracy; inconsistent tasks drop to 25–60%.

**Key concern:** Consistency is the strongest single predictor of agent reliability.

### 9. Early Divergence Detection

In multi-step tasks, a checkpoint mechanism at early steps — especially step 2 (the first substantive action) — to catch the agent going off-track before errors compound. Research shows 69% of behavioral divergence occurs at step 2.

**Key concern:** Catching mistakes early is dramatically more efficient than correcting downstream damage.

---

## C. Error Prevention & Learning

### 10. Error Classification & Logging

Categorizing mistakes by type (tool failure, reasoning error, forgotten step, context loss, etc.) rather than a binary error flag. Typed errors enable pattern detection over time.

**Key concern:** Without typed errors, you can't identify which *class* of mistake keeps recurring.

### 11. Corrective Example Store

When a mistake is corrected, the correction is stored as a retrievable example injected into future prompts for similar tasks. Based on Google research ("Teaching AI Agents by Error Correction on Plans") — one of the most effective patterns for eliminating repeated mistakes.

**Key concern:** Does the retrieval actually trigger when a similar task class appears? A store that's never queried is useless.

### 12. Feedback Loop Closure

The full cycle: **Action → Outcome → Evaluation → Lesson → Storage → Retrieval → Improved Action.** Many systems implement the first few steps but never close the loop. Lessons are logged but never retrieved when the same class of task recurs.

**Key concern:** Where does the loop break? This is the root cause of "we implemented reflection but Tyr still makes the same mistake."

### 13. Memory Pruning & Relevance Filtering

A strategy for removing stale, contradicted, or superseded memories. Without pruning, the memory store only grows, accumulating noise that degrades retrieval quality.

**Key concern:** Memory that is never pruned becomes a liability rather than an asset.

---

## D. Context & Session Management

### 14. Context Compression / Summarization

As sessions grow, raw context becomes noise. The agent must compress and distill, retaining key information while discarding redundancy. Research on MemAgent shows near-lossless performance is achievable with aggressive compression — humans don't try to remember every word, they abstract key ideas.

**Key concern:** Is context being actively managed, or does it just accumulate until the token limit is hit?

### 15. Cross-Session Continuity

Explicit handoff between sessions: what was I working on, what's the current state, what's next. Without this, every session starts cold and the agent must reconstruct context from scratch — wasting tokens and risking inconsistency.

**Key concern:** Is there an explicit session-start state restoration, or does Tyr effectively wake up with amnesia?

### 16. Selective Retrieval

When the agent needs a memory or fact, it queries for only what's relevant to the current task — not loading everything. The difference between a useful cognitive system and a context-flooding one.

**Key concern:** Retrieval that pulls too broadly floods the context window; retrieval that's too narrow misses relevant context.

---

## E. Governance & Multi-Agent Coordination

### 17. Human-in-the-Loop Checkpoints

For high-risk or ambiguous decisions, the system escalates to Ian rather than guessing. Clear escalation triggers defined in advance. Prevents the agent from confidently committing to wrong paths on consequential decisions.

**Key concern:** Are escalation triggers clearly defined, or does the agent guess when it should be asking?

### 18. Inter-Agent Knowledge Sharing

When Tyr learns something (a mistake, a pattern, a better approach), is that knowledge accessible to other KAs (Sky, future agents)? In a multi-agent system, lessons learned should propagate — not be siloed in one agent's context.

**Key concern:** A mistake Tyr makes shouldn't need to be independently discovered by Sky.

---

## Sources & References

- IBM Think: *What Is AI Agent Memory?* — episodic/semantic/procedural breakdown
- MachineLearningMastery: *Beyond Short-term Memory* — long-term memory types
- Galileo AI: *Self-Evaluation in AI Agents* — reflection and chain of thought
- arXiv 2602.11619: *When Agents Disagree With Themselves* — behavioral consistency study (3,000 runs)
- Google ADK Docs: *Choose a design pattern for agentic AI* — loop/self-correction patterns
- JetBrains Research: *Cutting Through the Noise* — context management efficiency
- arXiv WebCoach: *Self-Evolving Web Agents with Cross-Session Memory* — cross-session persistence
- Google/TDCommons: *Teaching AI Agents by Error Correction on Plans* — corrective example stores
- Concentrix: *12 Failure Patterns of Agentic AI Systems* — governance and failure taxonomy
- DEV Community: *AI Agent Memory Management — Stateless to Intelligent* — Rust-based memory architecture