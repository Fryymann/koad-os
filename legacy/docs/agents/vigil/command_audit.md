**VIGIL — Command Audit Tasking**

**Issued by:** Dood Authority

**Date:** 2026-03-10

**Output target:** `.koad-os/reports/vigil_command_audit.md`

---

**Tasking: KoadOS Command Surface Audit**

Vigil, you are directed to perform a full review and audit of all commands used to **operate, diagnose, debug, and monitor** the KoadOS ecosystem. This is a security and operational hygiene review — not a functional test.

**Scope:**

Cover all command surfaces across the following domains:

1. **Spine** — startup, shutdown, restart, graceful stop, and health checks
2. **Sentinel** — hydration triggers, status queries, and manual overrides
3. **Watchdog** — self-healing triggers, intervention commands, and suspension
4. **ASM (Agent Session Manager)** — session create/destroy, status, and decoupled invocation
5. **Swarm / Sector Locking** — lock acquire/release, deadlock inspection, and sector status
6. **KAI Agents (Sky, Tyr, Vigil, etc.)** — invocation commands, override commands, and kill signals
7. **Koad CLI** — all registered commands including `koad board`, `koad status`, `koad run`, and any undocumented or inferred commands observed in logs or config
8. **Logging / Monitoring** — all commands used to pull logs, filter, tail, inspect GCP logging output, or query internal diagnostics

---

**For each command surface, document:**

- **Command(s):** exact syntax as used in practice
- **Purpose:** what it does
- **Current status:** working / broken / undocumented / missing
- **Risk surface:** any security or stability concerns (e.g., no auth gate, no dry-run flag, destructive without confirmation)
- **Coverage gaps:** operations that should have a command but don't

---

**Suggested improvements section:**

After the audit inventory, produce a `## Suggested Improvements` section that includes:

- Missing commands that should be added (with proposed syntax)
- Commands that are dangerous and should require a `--confirm` flag or equivalent
- Commands that are undocumented and should be added to the CLI help surface
- Any commands that are redundant, ambiguous, or should be deprecated
- Prioritize suggestions by: 🔴 Critical / 🟡 Medium / 🟢 Low

---

**Documentation suggestions section:**

Produce a `## Documentation Suggestions` section that addresses command understanding for **both humans and agents**. Include:

**For humans:**

- Commands that exist but have no help text, man entry, or inline `--help` output — flag each and propose a short description
- Commands whose behavior is non-obvious or has side effects that aren't communicated at the call site — suggest inline warnings or confirmation prompts
- Any domain that lacks a runbook, cheatsheet, or quick-reference doc — propose a doc title and outline
- Suggest a canonical `COMMANDS.md` or `koad help` index if one does not exist, covering all domains in scope

**For agents (KAI / AI runtime consumers):**

- Commands that agents currently invoke but that have no structured contract (expected inputs, outputs, exit codes, side effects) — flag each and propose a contract stub
- Domains or operations where an agent could cause unintended state changes due to ambiguous command behavior — suggest guard clauses or documentation guardrails
- Suggest an `AGENTS.md`-style command reference per domain that is machine-readable and context-injectable at agent boot, covering: purpose, syntax, safe vs. destructive classification, and expected response format
- Note any commands where human-readable and agent-readable documentation should diverge (e.g., a human runbook vs. a terse structured contract for LLM injection)

---

**Format requirements:**

- Output file: `.koad-os/reports/vigil_command_audit.md`
- Use standard Markdown with clear `##` section headers per domain
- Include a summary table at the top: `| Domain | Commands Found | Gaps | Critical Issues | Doc Gaps |`
- Timestamp the report header with current date and your agent ID
- Do not include credentials, secrets, or key material of any kind in the report

---

**Constraints:**

- Read-only pass. Do not modify any configs, CLIs, or runtime state during this audit.
- If you encounter a command that requires elevated access to inspect, flag it and skip — do not attempt escalation.
- Apply KSRP (7-pass self-review) before finalizing the report.
- This report is for Dood Authority review only. Do not broadcast or distribute.

---

**Deliverable:** A completed `.koad-os/reports/vigil_command_audit.md` filed and ready for Admiral review.

Condition: **Audit active.**