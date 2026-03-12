**Tyr — ASM & Spine Reliability Review (KoadConfig Integration)**

```
# ASM & Spine Reliability Review — KoadConfig Integration
# Authority: Dood Override — Deep Review Protocol
# Output: ~/koad-os/reports/asm-spine-koadconfig-review.md

## Objective

Perform a comprehensive review of the Agent Session Manager (ASM), the boot
sequence, Body/Ghost tethering, and how these systems connect through the Spine.
Then analyze how our new KoadConfig + TOML configuration system can be leveraged
to address known reliability issues — specifically: errant cognitive switching
(wrong Ghost loaded or leaked into wrong session), session integrity failures,
and ASM/Spine fragility under concurrent multi-agent operation.

Produce a structured report at: ~/koad-os/reports/asm-spine-koadconfig-review.md

## Scope of Review

### Phase 1 — Architecture Audit (Read & Map)

Trace the full lifecycle of an agent session from boot to teardown. Document
the current implementation of each step:

1. **Boot Sequence** (`koad boot --agent <Name>`)
   - How is the agent name resolved from config/?
   - How is KOAD_SESSION_ID generated and bound?
   - What happens if config/ is malformed or the agent entry is missing?
   - What is the exact order of operations between identity load, Redis cache,
     Sentinel hydration, and session registration?
   - Where are the race conditions? What happens if two boots fire within the
     30-second grace window?

2. **Body/Ghost Tethering**
   - How is the Ghost (identity + memory) bound to the Body (CLI session)?
   - What mechanism enforces One Body, One Ghost? Where does enforcement live
     (ASM? Spine core? Boot script?) and what are the failure modes?
   - How is KOAD_SESSION_ID propagated to child processes, hooks, and subagents?
   - What happens to the tether if the terminal crashes vs. clean exit?

3. **Agent Session Manager (ASM)**
   - How does the ASM track active sessions? (Redis keys? Heartbeat intervals?)
   - How does heartbeat monitoring work? What is the timeout threshold?
   - How does the ASM detect and purge stale/orphaned sessions?
   - What is the interaction between the ASM and the Autonomic Pruner's
     30-second grace period? Are there edge cases where a valid session gets
     pruned or an invalid one persists?
   - How does the ASM interact with Sentinel on session recovery/restart?

4. **Spine Connectivity**
   - How do the ASM, Sentinel, Watchdog, and Pruner coordinate?
   - What is the dependency graph? (e.g., does Sentinel depend on ASM having
     a registered session before hydrating?)
   - What happens to in-flight sessions if the Spine process itself restarts?
   - How does `koad:state` in Redis stay consistent when multiple agents are
     active across terminals?

### Phase 2 — Failure Analysis (Known Problems)

Document every known or plausible failure mode related to:

1. **Cognitive Mismatch / Errant Switching**
   - Scenarios where Agent A's identity, memory, or context leaks into
     Agent B's session.
   - Scenarios where Sentinel hydrates the wrong agent's memory into a session.
   - Race conditions during concurrent boot of multiple agents.
   - What happens if Redis keys collide or are overwritten by a second boot?

2. **Session Integrity Failures**
   - Orphaned sessions that persist beyond agent termination.
   - Sessions that lose their identity binding mid-operation.
   - Heartbeat failures that trigger premature purge of active sessions.
   - State corruption when multiple agents write to `koad:state` concurrently.

3. **Recovery Gaps**
   - Does the Watchdog correctly restore sessions after a Spine crash?
   - What state is lost vs. recoverable after an ungraceful Spine restart?
   - Can Sentinel re-hydrate a session that was active but lost its Redis state?

### Phase 3 — KoadConfig Integration Analysis

With the new KoadConfig + TOML configuration system now in place, analyze how
it can be used to harden each of the above areas:

1. **Configuration-Driven Session Policy**
   - Can session timeout thresholds, heartbeat intervals, and grace periods
     be defined in TOML rather than hardcoded?
   - Can per-agent session policies be declared? (e.g., Tyr gets longer
     heartbeat tolerance for deep architecture work; Vigil gets strict
     short-timeout for security sweeps)
   - Can One Body/One Ghost enforcement mode be configurable (strict vs.
     graceful handoff)?

2. **Identity Binding via Config**
   - Can KoadConfig serve as the canonical identity resolution layer,
     replacing or augmenting raw config/ lookups?
   - Can TOML agent profiles include a cryptographic fingerprint or version
     hash that the ASM validates at boot, preventing stale/mismatched identity
     injection?
   - Can config validation at boot catch malformed agent entries before they
     reach the Spine?

3. **Cognitive Isolation Hardening**
   - Can KoadConfig define explicit memory partition boundaries per agent,
     making Sentinel's hydration target verifiable against config?
   - Can Redis key namespacing be derived from config rather than convention,
     reducing the risk of key collisions?
   - Can config declare which agents are allowed to run concurrently and on
     which Bodies, enabling the ASM to reject conflicting boot requests?

4. **Spine Resilience**
   - Can KoadConfig define Spine recovery behavior (e.g., on crash: purge all
     sessions vs. attempt recovery from last known state)?
   - Can Watchdog thresholds and self-healing triggers be configurable?
   - Can the config system itself detect drift between the running Spine state
     and the declared config, triggering reconciliation?

5. **Upgrade Path & Migration**
   - What existing hardcoded values in the ASM, Sentinel, Watchdog, and Pruner
     should be migrated to TOML config?
   - What is the priority order for migration (most-fragile-first)?
   - Are there values that MUST remain hardcoded for safety?

## Report Format

Structure the report as:

```

# ASM & Spine Reliability Review — KoadConfig Integration

**Date:** YYYY-MM-DD

**Reviewer:** Tyr

**Authority:** Dood Override

## Executive Summary

[2-3 paragraph synthesis of findings and top recommendations]

## 1. Architecture Audit

### 1.1 Boot Sequence

### 1.2 Body/Ghost Tethering

### 1.3 Agent Session Manager

### 1.4 Spine Connectivity

## 2. Failure Analysis

### 2.1 Cognitive Mismatch Scenarios

### 2.2 Session Integrity Failures

### 2.3 Recovery Gaps

## 3. KoadConfig Integration Recommendations

### 3.1 Session Policy

### 3.2 Identity Binding

### 3.3 Cognitive Isolation

### 3.4 Spine Resilience

### 3.5 Migration Priority

## 4. Implementation Roadmap

[Ordered list of recommended changes with effort estimates and risk ratings]

## 5. Open Questions for Dood

[Anything requiring architectural decisions or approval]

```

## Constraints

- Do NOT make changes to the codebase during this review. Read-only.
- Cite specific files, functions, and line numbers for every finding.
- If you cannot trace a code path, flag it explicitly as "UNVERIFIED" —
  do not speculate.
- Apply KSRP Pass 4 (Architect) and Pass 5 (Harden) lens throughout.
- This is a deep review, not a surface scan. Take the time to trace actual
  code paths. I want ground truth, not assumptions.

## Deliverable

~/koad-os/reports/asm-spine-koadconfig-review.md

When complete, present the Executive Summary and top 3 recommendations to Dood
and await approval before any implementation begins.
```