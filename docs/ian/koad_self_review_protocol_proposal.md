# Koad Self-Review Protocol

<aside>
🔁

**Protocol:** `review`

**Type:** Iterative feedback loop

**Max Iterations:** 5

**Exit Condition:** Clean pass (no findings above `info` severity) OR iteration limit reached

</aside>

---

## Purpose

A structured, repeatable self-review loop that Koad runs after any coding task. Each iteration performs 7 focused review passes, triages findings, applies fixes, and verifies no regressions — then loops until the output is clean or the ceiling is hit.

---

## Protocol Sequence

> **`lint → verify → inspect → architect → harden → optimize → testaudit → triage → fix → delta`** *(loop ≤ 5)* **→ report**
> 

---

## Review Passes (per iteration)

Each iteration executes these 7 passes **in order**. Every pass produces a structured findings list.

### Pass 1 — `lint`

**Focus:** Static analysis, formatting, unused imports, type errors

- Run all configured linters and formatters for the project language(s).
- **Auto-fix** anything trivial (whitespace, import ordering, trailing commas).
- **Flag** everything else as a finding.
- This pass is the **gate**. If `lint` produces unfixable errors, remaining passes still run but findings are tagged `lint-blocked`.

**Checklist:**

- [ ]  No compiler/type errors
- [ ]  No unused imports or dead variables
- [ ]  Formatting matches project standard
- [ ]  No `TODO` or `FIXME` without a linked issue

---

### Pass 2 — `verify`

**Focus:** Correctness — does the code match the spec or intent?

- Compare implementation against the task spec, ticket, or stated intent.
- Trace every requirement to code that fulfills it.
- Check edge cases: null/empty inputs, boundary values, off-by-one, duplicate events, partial failures.
- Confirm backward compatibility where relevant.

**Checklist:**

- [ ]  Every spec requirement has corresponding implementation
- [ ]  Edge cases identified and handled
- [ ]  No regressions to existing behavior
- [ ]  Error paths return meaningful results (not silent swallows)

---

### Pass 3 — `inspect`

**Focus:** Style, readability, and cognitive complexity

- Are names meaningful and consistent with project conventions?
- Is the code idiomatic for the language?
- Are functions short and single-purpose?
- Is nesting depth reasonable (≤ 3 levels preferred)?
- Is there dead code, commented-out blocks, or misleading comments?

**Checklist:**

- [ ]  All names are descriptive and convention-compliant
- [ ]  No deeply nested logic (flatten or extract)
- [ ]  No commented-out code without justification
- [ ]  Comments explain *why*, not *what*
- [ ]  A new contributor could understand this code without extra context

---

### Pass 4 — `architect`

**Focus:** Design, abstractions, coupling, and boundaries

- Are abstractions justified by actual use, or over-engineered for hypothetical futures?
- Is coupling between modules/systems minimal?
- Could anything that's a network hop be a library call instead?
- Is there duplication that should be consolidated?
- Are boundaries (module, service, API) clean and well-defined?

**Checklist:**

- [ ]  No unnecessary abstraction layers
- [ ]  No duplicated logic across modules
- [ ]  Coupling is minimal and intentional
- [ ]  Boundaries match actual scaling/isolation needs
- [ ]  Data contracts are consistent across integration seams

---

### Pass 5 — `harden`

**Focus:** Security

- No hardcoded secrets, API keys, or credentials.
- All user/external input is validated and sanitized.
- Auth and authz checks are present and correct at every entry point.
- Error messages do not leak internal state, stack traces, or PII.
- Logging does not contain sensitive data.

**Checklist:**

- [ ]  Zero hardcoded secrets or credentials
- [ ]  All inputs validated at trust boundaries
- [ ]  Auth/authz enforced on every endpoint
- [ ]  Error responses are safe for external consumption
- [ ]  Logs are scrubbed of PII and secrets
- [ ]  Dependencies checked against known CVEs

---

### Pass 6 — `optimize`

**Focus:** Performance and runtime efficiency

- Identify N+1 query patterns, redundant DB calls, unnecessary serialization/deserialization cycles.
- Flag over-allocation (threads, connections, buffers sized far beyond actual load).
- Check for synchronous blocking where async is safe, and unnecessary async where sync would suffice.
- Look for work that could be eliminated entirely (redundant transforms, unused computations).

**Checklist:**

- [ ]  No N+1 or redundant queries
- [ ]  No unnecessary serialization round-trips
- [ ]  Resource pools sized appropriately for actual load
- [ ]  Async/sync choice is intentional and justified
- [ ]  No computation whose result is discarded

---

### Pass 7 — `testaudit`

**Focus:** Test coverage and test quality

- Are there tests for every public function, endpoint, and event handler?
- Do tests assert **meaningful outcomes**, not just "no crash"?
- Are edge cases from `verify` covered by tests?
- Are there flaky, skipped, or over-mocked tests that no longer validate real behavior?
- Report coverage delta (before → after this change).

**Checklist:**

- [ ]  Every public interface has at least one test
- [ ]  Tests assert correctness, not just execution
- [ ]  Edge cases from `verify` have corresponding test cases
- [ ]  No skipped or flaky tests without a linked issue
- [ ]  Coverage delta is net-positive or justified

---

## Post-Pass Actions (per iteration)

### `triage`

Aggregate all findings from the 7 passes into a single list. Classify each:

| Severity | Meaning | Action |
| --- | --- | --- |
| **critical** | Blocks ship. Correctness, security, or data-loss risk. | Must fix this iteration. |
| **high** | Significant quality or reliability issue. | Must fix this iteration. |
| **medium** | Meaningful improvement. Low risk if deferred. | Fix if possible, otherwise carry forward. |
| **low** | Minor style or optimization nit. | Fix if trivial, otherwise note. |
| **info** | Observation only. No action required. | Log and drop from fix queue. |

### `fix`

Apply fixes for all findings ≥ `low`. For each fix:

- Reference the finding ID and pass that surfaced it.
- Commit with a structured message: `[review:<iteration>/<pass>] <summary>`

### `delta`

Diff the codebase before and after `fix`. Confirm:

- No new regressions introduced by the fixes.
- All previously passing tests still pass.
- If `delta` finds a regression → flag it as a **critical** finding and carry into the next iteration.

---

## Loop Logic

```
iteration = 0
while iteration < 5:
    iteration += 1
    run passes: lint, verify, inspect, architect, harden, optimize, testaudit
    triage findings
    if no findings above 'info':
        break  # clean pass — exit
    fix findings
    delta check
    if delta introduces regression:
        tag regression as critical, continue loop
# on exit:
report
```

---

## On Exit — `report`

Whether exiting clean or at the iteration limit, Koad produces a structured review report:

### Report Sections

1. **Summary** — Iteration count, clean exit (yes/no), total findings surfaced, total fixes applied.
2. **Findings by Pass** — Table of findings per pass, grouped by severity.
3. **Fixes Applied** — List of changes with commit references.
4. **Unresolved Items** — Any findings that remain after max iterations, with severity and justification for deferral.
5. **Rule Evolutions** — Lessons learned that should update Koad's:
    - Lint configuration
    - Review checklists
    - Project-level conventions
    - Test templates

This section is the **Red Queen mechanism** — the review protocol improves itself over time by feeding learnings back into its own rules.

---

## Design Principles

<aside>
📐

- **< 400 LOC per pass.** Defect detection drops sharply with larger scopes. If the changeset is large, chunk it.
- **Automate baselines first.** `lint` runs before human-level passes so reviewers focus on judgment calls, not formatting.
- **Findings are actionable.** Every finding is one of: *required change*, *suggestion*, or *question needing clarification*. No vague commentary.
- **The protocol evolves.** Every `report` includes rule evolutions. If the same finding appears across multiple reviews, it becomes a lint rule or checklist item.
</aside>

---

## Integration with Other Protocols

| Protocol | Relationship |
| --- | --- |
| `saveall` (`reflect → ponder → learn → saveup`) | Run `review` **before** `saveall`. Review catches defects; saveall captures learnings. |
| Spec Review Loop | `review` operates on **code after implementation**. The Spec Review Loop operates on **specs before implementation**. They are complementary gates. |
| CI/CD Pipeline | `lint` pass should mirror CI lint config exactly. Findings from `review` can generate GitHub issues automatically. |