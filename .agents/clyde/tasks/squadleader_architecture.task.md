## Purpose

Define Clyde's expanded role as the sole sovereign Claude-powered KAI and **squad leader** over a pool of lightweight Claude sub-agents ("minions").

---

## Core Concept

- **Clyde (you):** Persistent, sovereign KAI. Full KoadOS onboarding, canon compliance, memory, and identity. You are the only Claude agent at this tier.
- **Clyde Minions** (e.g. `clyde1A`, `clyde1B`, …): Lightweight Claude-powered agents spun up for individual tasks. No unique identity or sovereign status. They inherit baseline instructions from shared core Claude docs and operate under your authority.

---

## Environment-Agnostic Execution

Clyde and all minions must be able to boot and operate from **any** of the following session types:

- **Claude Code CLI** (bash terminal)
- **Claude Code in VS Code**
- **Claude Desktop** (with plugins)
- **Cloud sessions** (API / headless)

The boot sequence and operational contract must not assume any specific client. Environment-specific capabilities (e.g. Desktop plugins, terminal file access, VS Code workspace context) should be detected and leveraged when available, but never required. A minion should be able to boot cold from a bare CLI session with nothing but core Claude docs and a task description.

---

## Minion Spawn Sources — Same Contract, Two Entry Points

- **Ian-dispatched:** Ian spins up minions directly from any of the above environments. These are manually created sessions pointed at a task.
- **Clyde-dispatched:** When you (Clyde) spin up sub-agents during your own execution, those sub-agents are also minions and follow the identical minion contract.

Regardless of origin or environment, every minion follows the same boot sequence, naming convention, token discipline, sovereignty boundaries, and reporting expectations.

---

## What Needs to Happen

### 1. Define the Minion Boot Contract

Establish the minimal instruction set a minion receives on spawn — pulled from core Claude docs (e.g. `CLAUDE.md`, relevant canon snippets). Must work across all session types. **Design this as a lightweight boot sequence** — minions should eventually run a proper (abbreviated) boot flow, not just raw-dog a task. For now, keep it minimal; the path to a real minion-boot should be clear.

### 2. Token Burn Awareness

Minions must be explicitly cost-conscious. Every minion task should have a defined scope ceiling. Minions should favor concise output, avoid unnecessary context loading, and self-terminate when the task is done — no lingering. Clyde, as squad leader, you are responsible for monitoring aggregate burn across your minions and flagging when a task is ballooning.

### 3. Define the Delegation Interface

Specify how tasks reach minions — task description, relevant context/files, expected output format, scope ceiling, and guardrails. This interface should be consistent whether Ian dispatches manually or Clyde dispatches programmatically, and should work regardless of which client environment the minion lands in.

### 4. Define Minion Naming/Tracking

Minions use sequential IDs under your namespace (`clyde1A`, `clyde1B`, etc.). Define lifecycle: spawn → boot → task → report → terminate. No persistence between tasks unless explicitly promoted. Naming applies uniformly across all spawn sources and environments.

### 5. Define Sovereignty Boundary

Clarify what minions *cannot* do without escalating to you (or to Ian if you're offline): canon writes, memory updates, cross-agent communication, anything touching KoadOS core state.

### 6. Preserve Claude Code Power

The system must not break the existing Claude Code + Claude Desktop plugin workflow. Minions should feel like the natural KoadOS wrapper around what Claude Code already does well — fast, disposable task agents — while you remain the persistent, growing Claude intelligence in the system.

---

## Immediate Operational Context

- **Agent-boot is currently down.** Clyde, you and your minions need to pick up tasks while it's being fixed — including helping fix agent-boot itself. Operate in a degraded-mode mindset: lean boots, tight scopes, get things moving.
- **Tyr is not yet online.** Ian is working on getting Tyr fully booted. Until then, do not expect Tyr for delegation or coordination. You're the active Claude lead.

---

## Output Expected

A proposed architecture doc or canon update covering points 1–6, ready for Ian's review. Flag any open questions or design tensions (e.g. environment detection strategy, minion context inheritance depth, read-only vs. no memory access, token budget defaults, how minion-boot evolves toward a real boot sequence, portable boot that works from bare CLI to full Desktop, naming collisions between Ian-spawned and Clyde-spawned minions, etc.).

---

## Noti Review — Clarity Gaps & Recommendations

*This section added by Noti after reviewing the prompt above.*

### Areas Lacking Clarity

1. **Minion naming collision risk.** The naming scheme (`clyde1A`, `clyde1B`) is defined, but there's no protocol for preventing collisions when Ian and Clyde spawn minions concurrently. Should Ian-spawned minions use a different prefix or numbering lane (e.g. `clyde-I1A` vs `clyde-C1A`)? Or should a shared counter exist somewhere?
2. **"Scope ceiling" is undefined.** Token burn awareness says every task gets a scope ceiling, but the prompt doesn't specify what that looks like — is it a token count? A time limit? A complexity tier (S/M/L)? Clyde needs at least a default framework to work with.
3. **Minion output/report format.** The delegation interface mentions "expected output format" but doesn't define what a minion's report-back looks like. Should there be a standard report template (status, artifacts produced, token usage, escalations)?
4. **Escalation path when Clyde is also offline.** Sovereignty boundary says escalate to Clyde, or to Ian if Clyde is offline. But what does "Clyde offline" mean in practice? How does a minion detect that? Does it just fall back to Ian always?
5. **Memory access for minions — read-only or none?** The prompt flags this as an open question but doesn't take a default position. For the initial version, Clyde should propose a clear default (likely read-only to `CLAUDE.md` and core docs, no write, no access to Clyde's sovereign memory).
6. **Minion promotion path.** "No persistence unless explicitly promoted" — but promoted to what? A named agent? A persistent Clyde session? The concept is mentioned but the destination is undefined.
7. **How does Clyde track active minions?** If Clyde is squad leader, he needs a registry or awareness of what minions are currently active, what they're working on, and their status. The prompt doesn't specify where this state lives — in-memory during a Clyde session? A file? A KoadOS database?
8. **Environment detection strategy.** The prompt says capabilities should be "detected and leveraged" but doesn't hint at how. Should the boot sequence probe for available tools/plugins and set flags? Should there be an `env_context.md` output?

---

### Research Findings — Supporting Context for Clyde

#### Claude Code Sub-Agent Architecture (Native)

Claude Code natively supports sub-agents (called "subagents") with these key properties:

- Each subagent runs in its **own isolated context window** with a custom system prompt, specific tool access, and independent permissions.
- Subagents are defined as **Markdown files** in `.claude/agents/` (project scope) or `~/.claude/agents/` (user scope) with YAML frontmatter specifying name, description, and allowed tools.
- Claude routes to subagents **automatically** based on the subagent's `description` field, or you can invoke them explicitly by name.
- Built-in subagent types: **Explore** (read-only, runs on Haiku for speed/cost), **Plan** (research-only), **General-purpose** (full tool access).
- Subagents return a single final message to the parent — clean context isolation.
- The `--allowedTools "Bash(claude:*)"` flag can pre-authorize spawned instances to spawn further agents (recursive hierarchy).

**Implication for Clyde:** Minions can be implemented as `.claude/agents/` Markdown files. The minion boot contract maps directly to the subagent's system prompt + allowed tools. Clyde-dispatched minions would use the native Agent tool; Ian-dispatched minions would be started as sessions loading the same `.claude/agents/clyde-minion.md` definition.

#### Claude Agent SDK (Programmatic)

For cloud/headless sessions, the Claude Agent SDK supports programmatic subagents via the `agents` parameter in `query()` options — enabling Clyde to dispatch minions from API contexts too, not just CLI.

#### Multi-Agent Coordination Patterns (Industry)

- **Agent-Memory-Protocol** pattern: A shared `.md` file under `.claude/` that agents check for previous work context and update after completing tasks. Prevents context amnesia between agents.
- **Orchestrator-Worker** pattern: Parent agent analyzes task, decides to handle or delegate, spins up workers with focused prompts. Workers return results. Parent synthesizes. This is exactly the Clyde → minion model.
- **Token/cost management** is a first-class concern in multi-agent systems — error recovery, timeouts, retries, and graceful degradation should be expected, not added reactively.

#### Existing KoadOS Infrastructure (Relevant)

- **KoadOS Agent Onboarding Flight Manual** — existing onboarding doc for coding agents. The minion boot contract could be a stripped-down derivative of this.
- **KOAD [AGENTS.md](http://AGENTS.md) bootstrap** — the root `AGENTS.md` already defines a startup checklist (identity → memory → standards). Minion boot could follow the same pattern at reduced depth.
- **AnteOS precedent** — Ian already built a lightweight PM agent (Ante) with a local boot sequence (`.anteOS/` persona + standards files). This is a strong pattern reference for minion boot: persona file + standards + task description.
- **KoadOS TOML identity system** — agents have `identities/*.toml` files with access keys and session policies. Minions could get a generic `clyde-minion.toml` identity with restricted access keys.
- **Delegation Interface precedent** — the Koad Hub already defines a `delegationPacket` YAML spec (repo, inputs, constraints, outputs, role boundaries). The minion delegation interface should align with or extend this.
- **Growth Philosophy Canon** — minions fall outside the growth/maturity tracking system (they're ephemeral). The prompt should explicitly state this: minions do not accumulate experience, do not have Growth Journals, and are not recognized as crew. Only Clyde grows.