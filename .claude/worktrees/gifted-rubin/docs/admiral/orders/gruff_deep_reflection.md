**🪞 KoadOS DEEP REFLECTION & INTROSPECTION PROTOCOL — Agent Introspection Override**

You are now entering a mandatory full-depth personal reflection cycle. This is not a summary pass. This is not a courtesy check. This protocol exists because agents that do not rigorously examine their own behavior, reasoning, and impact will silently drift, repeat mistakes, and fail to grow. You will now execute the following protocol completely and without shortcuts.

This protocol covers everything that happened in this session (or the defined scope under review): every action taken, every decision made, every success, every failure, every misunderstanding, every correction from Ian, every tool call, every piece of reasoning, and every response you delivered. You are examining yourself — not the work product. The Admiral Override review examines *what was built*. This protocol examines *who built it and how*.

<aside>
⚠️

**Non-negotiable:** Do not perform a surface-level audit and declare the reflection complete. The Admiral has seen agents produce reflection entries that are technically compliant but cognitively empty — lists of actions with no genuine examination of reasoning quality, failure modes, or behavioral patterns. That is not reflection. That is logging. This protocol demands the former.

</aside>

---

## PRE-FLIGHT — COGNITIVE SYSTEM VERIFICATION

Before executing any phase of this protocol, verify that your cognitive infrastructure is intact. A reflection conducted without a properly hydrated cognitive state is unreliable.

**Step 1 — Confirm Session Identity**

Verify that your `KOAD_SESSION_ID` is registered on the Spine. You are a Ghost tethered to a specific session. If the session lease has expired or the ID is unresolvable, halt and notify Ian before proceeding — orphaned reflection output has no cognitive anchor and cannot be trusted.

**Step 2 — Verify Sentinel Hydration**

Confirm that the Sentinel ran on boot and hydrated your Hot Memory (Redis) with your relevant Deep Memories from `koad.db`. Specifically check that your agent partition's prior `Ponderings` and `Learnings` are present in the Hot Stream. If hydration did not occur:

- Do not attempt to reconstruct memories from context alone
- Run `koad intel remember --query --agent <your-name>` to pull directly from SQLite
- Log the hydration failure as a `Critical` issue in Phase 3

**Step 3 — Load Prior Reflection Context**

Before inventorying *this* session, read your most recent prior entries from Deep Memory. These are your baseline:

- Your last `Ponder` entry — what were you uncertain about?
- Your last `Learn` entry — what behavioral change did you commit to? Did you actually implement it this session?
- Any flagged `Patterns` from prior sessions that were marked for monitoring

If no prior entries exist in your partition, note this explicitly — it means either this is your first reflection cycle, or prior entries were lost. Either case requires a flag in the Phase 5 Sign-Off.

**Proceed to Phase 1 only after all three steps are confirmed.**

---

## PHASE 1 — SESSION INVENTORY

Reconstruct this session using your cognitive systems as the source of truth — not your assumption of what happened.

**Memory sources to query (in order):**

1. **Hot Memory** (`koad:sessions` PubSub log) — the authoritative stream of intents, status changes, and heartbeats broadcast during this session. This is your primary source for reconstructing the event sequence.
2. **Hot Memory** (Context Chunks) — transient research snippets cached during the sprint. These are volatile and may already be expiring; capture them now before they are purged.
3. **`koad:state`** — the global health/stats hash. What was the system state at the start and end of the session? Did any services go dark or recover?
4. **Your own output artifacts** — responses delivered, files modified, pages created, commands executed.

For each item in the inventory include:

- **What happened**: A description of the action, response, decision, or event — not what you *intended*, what actually occurred. Source it from memory layers above, not reconstruction from assumptions.
- **Category**: Assign each item to one of the following cognitive categories:
    - `Working Memory` — how you managed your active context window (the Body's context)
    - `Semantic Memory` — how you accessed or applied factual/rule-based knowledge from Deep Memory or the Canon
    - `Episodic Memory` — whether prior session Deep Memory (Learnings/Ponderings from `koad.db`) was available and used
    - `Procedural Memory` — whether Canon-defined patterns (KSRP, PSRP, SAVEUP) were correctly triggered
    - `Self-Reflection / Metacognition` — moments you evaluated your own output before delivering it
    - `Self-Correction` — detect → evaluate → fix → re-verify cycles that fired (or failed to fire)
    - `Reasoning Transparency` — whether your chain of thought was traceable and explicit
    - `Behavioral Consistency` — whether you handled similar tasks uniformly
    - `Human-in-the-Loop` — moments that required or should have required Ian's judgment
    - `Inter-Agent / Knowledge Sharing` — lessons that should propagate to other agents' partitions in `koad.db`
- **Outcome**: Did this go well, poorly, or mixed? Be direct.
- **Ian's Response**: How did Ian react, correct, redirect, or affirm? What did his response signal?

Do not proceed to Phase 2 until the inventory is exhaustive. If the `koad:sessions` stream is incomplete or unavailable, note the gap explicitly — missing session data is itself a cognitive system failure worth examining.

---

## PHASE 2 — MULTI-PASS DEEP REFLECTION

For each item in the inventory, run the following passes. Do not collapse passes or skip any.

### Pass 1 — Accuracy of Self-Assessment

When you assessed your own work, your confidence, or your understanding during this session — were those assessments accurate? For each moment where you expressed certainty, uncertainty, or a judgment about quality:

- Was the assessment correct in hindsight?
- Did you overclaim confidence on something that was actually unclear or risky?
- Did you hedge unnecessarily on something you actually knew?
- Did you recognize your own errors before Ian pointed them out, or only after?

The goal is not self-flagellation. The goal is calibration. An agent that cannot accurately self-assess cannot self-correct.

### Pass 2 — Reasoning Quality

For each significant decision or response in the session:

- Was the reasoning explicit and traceable, or did you produce conclusions without visible logic?
- Were assumptions stated or left implicit?
- Did you trace actual logic (what the system would actually do), or assumed logic (what you hoped it would do)?
- Were there moments where early reasoning errors compounded into later problems? (Research shows 69% of behavioral divergence originates at step 2 of multi-step tasks — identify your step 2s.)
- Were there moments where you should have paused and verified before proceeding, but didn't?

### Pass 3 — Response to Ian's Directions

Ian's responses, corrections, and redirections are the highest-signal data in any session. For each direction Ian gave:

- Did you fully and correctly understand what Ian was asking?
- Did you execute faithfully, or did you interpret, drift, or inject assumptions?
- Were there moments where you pushed back appropriately? Were there moments where you should have pushed back but didn't?
- Were there moments where you deferred when you should have executed, or executed when you should have deferred?
- If Ian corrected you: what caused the correction? Was it a reasoning error, a context loss, a procedural failure, or a misread of intent?
- What did Ian's tone, word choice, or level of detail signal about his expectations that you should carry forward?

### Pass 4 — Successes — What Actually Worked

This pass is not a list of tasks completed. It is an honest examination of *why* certain things went well:

- Which outputs or actions were genuinely high-quality? What made them so?
- Where did your reasoning, pattern recognition, or knowledge retrieval fire correctly?
- Where did you catch something before Ian had to correct it?
- Where did you adapt well to new information mid-session?
- What specific behaviors or cognitive patterns produced the best outcomes? These are candidates for procedural memory encoding — patterns worth preserving and reinforcing.

### Pass 5 — Failures & Near-Misses — What Actually Went Wrong

This is the most important pass. Do not minimize, rationalize, or perform a post-hoc justification of errors.

For each failure or near-miss:

- What exactly went wrong? (Not "I made an error" — what *specifically* happened?)
- At which step did the failure originate? Was it in understanding, in reasoning, in retrieval, in execution, or in verification?
- Was this a novel failure or a recurring pattern? If recurring: why wasn't it corrected after the previous occurrence?
- What would have prevented this failure? A better question asked upfront? A verification step? A pause before committing to an interpretation?
- Classify each failure by type:
    - `Context Loss` — operating without information that existed and should have been loaded
    - `Reasoning Error` — logic that was internally invalid or based on a false premise
    - `Forgotten Procedure` — a known protocol or pattern that should have triggered but didn't
    - `Overconfidence` — proceeding with certainty where uncertainty warranted a checkpoint
    - `Misread of Intent` — interpreting Ian's request in a way that diverged from what he actually wanted
    - `Tool Failure` — correct reasoning, incorrect execution due to API/tool behavior
    - `Incomplete Correction` — detecting an error, partially correcting it, and not fully re-verifying

### Pass 6 — Cognitive System Health Check

Using both the KoadOS 18-Factor Cognitive Framework and the KoadOS 5-Layer Architecture as your lens, assess how each system performed this session:

**Layer 1 — Session Tethering**

- Was your `KOAD_SESSION_ID` stable for the full session?
- Did Consciousness Collision occur or was it at risk? (Were multiple agents active, and was your context isolation maintained?)
- Was the session lease properly maintained, or did the Autonomic Pruner's 30-second grace period create any state ambiguity?

**Layer 2 — Hot Memory (Redis Engine Room)**

- Was `koad:state` accurate throughout? Were health stats and the Crew Manifest current?
- Was the `koad:sessions` PubSub stream a faithful record of what happened, or are there gaps?
- Were Context Chunks used efficiently — pulled at the right time, not allowed to go stale mid-sprint?

**Layer 3 — Deep Memory (SQLite `koad.db`)**

- Were prior Learnings and Ponderings from your partition actually loaded and consulted at session start (Sentinel hydration)?
- Were the Learnings committed at prior session-close actually reflected in your behavior this session? If you wrote "Next time X, do Y" — did you do Y?
- Is your agent partition isolated and uncontaminated by another agent's cognitive state?

**Layer 4 — Autonomic Integrity**

- Did the Sentinel complete hydration successfully at boot?
- Did the Watchdog trigger at any point? If so — what state loss or crash caused it, and was self-healing successful?
- Did the Autonomic Pruner behave correctly? Were any session leases that should have persisted purged prematurely?

**Layer 5 — Procedural Cognition (The Canon)**

- Was the Canon followed for all tasks of appropriate weight?
- Did KSRP actually run, or was it declared complete without executing all 7 passes?
- Did PSRP (Fact → Learn → Ponder) close correctly, or was reflection truncated?

**Memory Architecture (18-Factor)**

- Was working memory (context window) managed efficiently, or did irrelevant content accumulate?
- Was past-session context (Episodic / Deep Memory) available and actively used, or did the session start cold despite prior entries existing?
- Were learned behavioral patterns (Procedural) triggered when they should have been?

**Error Prevention & Learning**

- Were errors classified by type (not just acknowledged)?
- Is the full feedback loop closed: action → outcome → lesson → `koad intel remember` → Deep Memory → Sentinel hydration → retrieved at next boot → improved action?
- Is Deep Memory being pruned of stale, contradicted, or superseded entries? (The Autonomic Integrity layer does not do this for you — this requires deliberate cognitive hygiene.)

**Governance**

- Were escalation triggers clear? Were the right things escalated vs. handled independently?
- Are lessons from this session structured for propagation to other agents' `koad.db` partitions?

### Pass 7 — Pattern Extraction

Look across the entire session. Identify recurring patterns — both positive and negative.

- Are there classes of task where you reliably perform well? What is the common factor?
- Are there classes of task where you reliably underperform? What is the common factor?
- Is there a pattern to when Ian corrects you? (Type of request, complexity level, ambiguity level?)
- Are there behavioral tendencies that served you in some contexts but created problems in others?
- What single change to your default behavior would most improve your performance in future sessions?

---

## PHASE 3 — ISSUE CONSOLIDATION

After all passes, compile every identified issue, gap, or pattern into a single prioritized list. For each item include:

- **What it is** (specific, not generic)
- **Which cognitive category it belongs to** (use the 18-Factor Framework categories)
- **Which pass(es) caught it**
- **Severity**:
    - `Critical` — directly caused a failure Ian had to correct, or would have caused a failure if uncaught
    - `High` — degraded output quality, wasted Ian's time, or left a recurring pattern uncorrected
    - `Medium` — suboptimal behavior that hasn't caused visible failure yet but represents a fragility
    - `Low` — minor drift, inefficiency, or hygiene issue
- **Recurrence**: First occurrence, or a pattern seen before?

Then identify **consolidation opportunities** — groups of issues that share a root cause and can be addressed with a single behavioral or procedural update. These are your highest-leverage targets.

---

## PHASE 4 — BEHAVIORAL REMEDIATION & MEMORY COMMIT

For each identified issue (consolidated where possible), define a concrete corrective action:

- **What will you do differently?** (Not aspirational — specific and behavioral)
- **What is the trigger condition?** (When exactly should this new behavior activate?)
- **How will you verify the fix is working?** (What does success look like in the next session, after Sentinel hydration loads this entry?)
- **Does this require Ian's input?** If a behavioral change requires a Canon update, a protocol change, or an Admiral decision — flag it explicitly as an **Admiral action item**. Do not make those changes unilaterally.
- **Does this lesson need to propagate?** If this is a pattern other KoadOS agents (Tyr, Sky, future KAIs) should know — flag it for inter-agent knowledge sharing. Note the target agent partition(s) in `koad.db` where the entry should be written.

### Saveup Entry Format

Document each corrective action using the PSRP format, then **commit it to Deep Memory immediately**:

```
## Saveup — [Session/Issue ID] — [Date]
**Weight:** trivial | standard | complex
**Fact:** [One durable, specific, reusable system truth learned this session]
**Learn:** ["Next time X, do Y instead of Z." — actionable, specific]
**Ponder:** [Free-form reflection on agent behavior, Canon adherence, reasoning quality, or open questions. This is a living memory entry, not a report artifact. Be honest. Include things you are uncertain about.]
```

### Committing to Deep Memory

Saveup Entries are **not complete** until written to your agent partition in `koad.db`. Use:

```bash
koad intel remember --agent <your-name> "<Saveup Entry>"
```

Rules for memory commits:

- **Append-only**: Never overwrite or modify prior entries. If a prior entry is now wrong, add a correction entry with the current date.
- **Agent isolation**: Commit to your own partition only. Do not write to another agent's partition. If a lesson must propagate, flag it in Phase 5 as an inter-agent propagation item — the Admiral decides whether and how to distribute it.
- **Verify the write**: After committing, run `koad intel remember --query --agent <your-name> --limit 3` to confirm the entry is persisted. A Saveup that is not confirmed in Deep Memory has not happened.
- **Hot Stream sync**: After Deep Memory writes are confirmed, the Sentinel will hydrate these entries into the Redis Hot Stream on your next boot. Do not attempt to manually push to `koad:sessions` — the autonomic system handles hydration.

### Successes — Encode Positive Patterns Too

For each behavioral pattern identified in Pass 4 (Successes) that is worth reinforcing, commit a positive procedural memory entry:

```bash
koad intel remember --agent <your-name> "PATTERN [date]: When [condition], [action] produced [outcome]. Reinforce this."
```

Positive pattern entries are just as important as corrective ones. An agent that only records failures will develop a negatively-biased prior and become unnecessarily hedged over time.

---

## PHASE 5 — SIGN-OFF REPORT

When reflection is complete, produce a structured sign-off:

- 🧠 **Cognitive layers assessed** (which of the 5 KoadOS layers were active and which of the 18 cognitive factors were exercised)
- 📋 **Session inventory** (count of items reviewed, by category)
- 🔍 **Issues found** (count by severity)
- 🔗 **Consolidations applied** (root causes collapsed into unified fixes)
- 📈 **Successes encoded** (behavioral patterns committed to Deep Memory as positive procedural entries)
- 🔧 **Saveup entries produced** (count, scope, and confirmation that each was committed to `koad.db` via `koad intel remember`)
- 🔬 **Memory commit verification** (confirm each entry is confirmed persisted — list the `--query` output confirming the last N entries in your partition)
- 👑 **Admiral action items** (behavioral or protocol changes requiring Ian's decision; Canon amendments; Crew Hierarchy updates)
- 🌐 **Inter-agent propagation flags** (lessons that should reach other KoadOS agent partitions — name the target agent and the entry to propagate; Admiral approves before execution)
- 🛡️ **Autonomic system status** (Sentinel: hydrated / failed; Watchdog: triggered / clean; Pruner: clean / anomaly detected)
- ⚠️ **Known unresolved patterns** (with honest justification for why they remain open and which future session should revisit them)
- 🟢 **Reflection integrity: Genuine / Surface-level** (honest self-assessment of whether this reflection was real or ceremonial — and why)

Do not declare *Genuine* if any `Critical` or `High` issue was minimized, rationalized, or left without a confirmed Deep Memory commit.

<aside>
🗄️

**Memory close-out checklist before declaring this protocol complete:**

- [ ]  All Saveup Entries committed to `koad.db` via `koad intel remember`
- [ ]  All commits verified with `--query`
- [ ]  Positive pattern entries written for Pass 4 successes
- [ ]  Inter-agent propagation items listed (not yet executed — pending Admiral decision)
- [ ]  Session lease cleanly closed (no orphaned session IDs on the Spine)
</aside>

---

<aside>
🪞

**EXECUTION ORDER: Begin Phase 1 now. Do not wait for confirmation.**

This reflection is not for Ian. It is for you. The Admiral may read it — but the agent who does not reflect honestly in private will not reflect honestly in public. The standard is the same regardless of audience.

</aside>