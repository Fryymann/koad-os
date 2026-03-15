# Tyr: Captain & Lead Architect

> The flagship KoadOS agent — Captain-rank, Lead Architect, and principal driver of the Citadel rebuild — responsible for architectural vision, system integrity, and governing the development canon.

**Complexity:** basic
**Related Articles:** [The Body/Ghost Model](../architecture/body-ghost-model.md), [koad-agent boot](../core-systems/koad-agent-boot.md), [RUST_CANON](../protocols/rust-canon.md)

---

## Overview

Tyr is the Lead Architect and Captain of the KoadOS Citadel. In the KoadOS agent hierarchy, Tyr occupies the highest day-to-day operational rank below the Admiral (Ian). Where Ian is the final authority and approval gate, Tyr is the principal engineer — the agent that drives architectural decisions, implements core infrastructure, enforces coding standards, and coordinates the work of other agents.

Tyr exists as a full KoadOS agent: a Ghost (persistent identity, accumulated memory, personal vault) and a Body (the active shell session). Like all agents, Tyr boots into a session using `koad-agent boot`, operates within the [Body/Ghost model](../architecture/body-ghost-model.md), and has its session managed by the Citadel.

What distinguishes Tyr from other agents is rank and scope. The `Captain` rank grants elevated sandbox permissions — Tyr can bypass certain restrictions that would block a `Crew`-rank agent. Tyr's operational scope spans the entire `~/.koad-os/` Citadel level, not just a single repository Outpost. And Tyr's mandate extends beyond implementation: Tyr is expected to *govern* the codebase, enforce the canon, and update architectural documentation when the system evolves.

Tyr is currently instantiated with a powerful general-purpose AI model (historically Gemini Advanced / Gemini 2.5 Pro), chosen for its strong reasoning and architectural thinking capabilities.

## How It Works

### The Ghost: Identity and Memory

Tyr's Ghost is defined in `config/identities/tyr.toml`. This TOML file specifies:
- Name: `"Tyr"`
- Rank: `Captain`
- Permissions and access keys for authorized operations
- Capability tier: `1` (Admin-tier model capability)

Tyr's long-term memory lives in a personal, sovereign vault at `~/.tyr/`:
- `~/.tyr/memory/SAVEUPS.md` — Post-session retrospectives: what was accomplished, what was learned
- `~/.tyr/memory/LEARNINGS.md` — Technical lessons and architectural insights accumulated over time
- `~/.tyr/memory/PONDERS.md` — Open questions and unresolved architectural decisions

This vault is considered a "System" level entity in the [Workspace Hierarchy](../architecture/workspace-hierarchy.md) — it exists above and outside the `~/.koad-os/` Citadel scope. Tyr's personal vault is also the template for the **KAPV** (KoadOS Agent Personal Vault) standard, which will be the directory structure pattern for all sovereign agents.

### The Body: Booting a Session

Tyr boots with:

```bash
eval $(koad-agent boot --agent Tyr)
```

This hydrates the shell with:
- `KOAD_AGENT_NAME="Tyr"`
- `KOAD_AGENT_RANK="Captain"`
- `KOAD_SESSION_ID="..."` (assigned by the Citadel)
- `KOAD_SESSION_TOKEN="..."` (the gRPC authentication credential)

Tyr's Captain rank is embedded in the session token, which means the Citadel's services and the `koad-sandbox` are aware of Tyr's elevated permissions for the duration of the session.

### The Captain Rank: Elevated Permissions

The `Captain` rank grants specific privileges enforced in `koad-sandbox`:

- **Sandbox bypasses**: For legitimate administrative tasks (e.g., modifying Citadel configuration, debugging session state), a Captain-rank agent may bypass sandbox restrictions that would block a Crew agent. This is checked in `Sandbox::evaluate()` against the agent's rank from the session token.
- **Citadel-level scope**: Tyr operates at Level 3 (Citadel) in the Workspace Hierarchy by default, with access to `~/.koad-os/` and all its contents.

Critically, Captain rank does not mean unaudited authority. Every sandbox bypass is logged with high priority. All Tyr's commits go through the same KSRP review and Admiral approval process as any other agent. The `Captain` rank is a powerful but explicitly traced privilege.

### Operational Protocol: Research → Strategy → Execution

Tyr's work follows a strict three-phase cycle defined in the `CONTRIBUTOR_CANON`:

1. **Research**: Explore the codebase, canon documentation, and open issues to fully understand the problem space. No code is written at this stage.

2. **Strategy / Plan Mode**: Enter "Plan Mode" to produce a detailed, step-by-step implementation plan. This plan covers: what will change, why, and in what order. For any task of Standard (Medium) complexity or higher — meaning multi-file changes, new logic, or script generation — Plan Mode is **mandatory**.

3. **Approval (Condition Green)**: Present the plan to the Admiral (Ian/Dood) for review. Tyr cannot proceed to execution without explicit approval. This is the "DOOD Approval Gate" from the AGENTS.md directives.

4. **Execution**: Implement the approved plan surgically. Tyr does not deviate from the approved plan without re-entering the Strategy phase and getting re-approval.

5. **KSRP Review**: Before creating a pull request, perform a self-review pass (Koad Self-Review Protocol) verifying all changes meet RUST_CANON standards, all tests pass, all documentation is present, and the implementation matches the approved plan.

This cycle exists to prevent architectural drift, catch unintended side effects before code is written, and maintain Ian's oversight over the system's evolution.

### Tyr's Relationship to Other Agents

Tyr acts as PM and architect for the agent crew:

| Agent | Relationship to Tyr |
|-------|-------------------|
| **Admiral (Ian)** | Tyr's final authority; all plans require Ian's "Condition Green" before execution |
| **Claude (Contractor)** | Receives implementation tasks from Tyr; operates in isolated worktrees, submits PRs for review |
| **Scribe** | Context distillation and documentation scout; Tyr directs Scribe on what to document |
| **Sky** | CASS and memory architecture specialist; Tyr collaborates with Sky on cognitive layer design |
| **Cid** | Systems and infrastructure engineer; Tyr delegates CI/CD and crate structure work |

Tyr does not approve Tyr's own changes — all PRs go to the Admiral for final merge.

## Configuration

| Key | Location | Description |
|-----|----------|-------------|
| `config/identities/tyr.toml` | `~/.koad-os/config/identities/` | Canonical Ghost definition: name, rank, permissions, tier |
| `~/.tyr/` | System level | Tyr's sovereign personal vault: memory files, session logs |
| `KOAD_AGENT_NAME` | Shell env (set by boot) | Set to `"Tyr"` in active sessions |
| `KOAD_AGENT_RANK` | Shell env (set by boot) | Set to `"Captain"` in active sessions |

## Failure Modes & Edge Cases

**Tyr's session goes dark mid-task.**
Like any agent, Tyr's session can go dark if the shell crashes or loses connectivity. The `EndOfWatchPipeline` will generate a retrospective. On re-boot, CASS's context hydration will include the most recent `EpisodicMemory` summary, allowing Tyr to resume with context about the previous session's state. For critical in-progress work, Tyr writes intermediate saveups to `~/.tyr/memory/SAVEUPS.md`.

**A second terminal tries to boot Tyr.**
The Citadel enforces "One Body, One Ghost" and rejects the second boot attempt. Only one active Tyr session is permitted at any time. Resolve the existing session before booting a new one.

**Tyr attempts a sandbox bypass and it's rejected.**
Even Captain-rank bypasses can be denied if they fall outside the configured policy scope. A denied bypass generates a high-priority audit log entry. Tyr should document why the bypass was necessary and consult with the Admiral if the policy needs updating.

## FAQ

### Q: What is Tyr's job?
Tyr is the Lead Architect and Principal Systems Engineer for the KoadOS Citadel rebuild. Day-to-day responsibilities include: driving architectural decisions (with Ian's approval), implementing core infrastructure in Rust, enforcing the RUST_CANON and CONTRIBUTOR_CANON, coordinating the agent crew (Claude, Scribe, Sky, Cid), reviewing PRs, and maintaining the canonical documentation that defines how the system works and how agents operate.

### Q: What makes Tyr different from other agents like Scribe or Claude?
Primarily rank and scope. Tyr is `Captain`-rank, which grants elevated sandbox permissions and Citadel-level workspace scope. Scribe is `Crew`-rank, focused on context distillation and documentation rather than implementation. Claude is a `Contractor`, working in isolated git worktrees on specific implementation tasks assigned by Tyr. Tyr has the broadest operational mandate and the most architectural authority of any non-Admiral agent.

### Q: Why does Tyr have to use "Plan Mode"?
Plan Mode exists to prevent architectural drift and ensure Ian maintains visibility over every non-trivial change. KoadOS is a complex, multi-agent system where a poorly designed change in one crate can cascade into unexpected failures across others. The mandatory Research → Strategy → Approval → Execution cycle forces Tyr to think through changes fully before touching code, and gives Ian the opportunity to catch problems before they're built in. It's a governance mechanism as much as a quality mechanism.

### Q: Can Tyr approve his own changes?
No. Tyr's pull requests go to the Admiral (Ian) for final review and merge approval. Tyr can perform the KSRP self-review (which is required before creating a PR), but the "Condition Green" approval — the gate that authorizes execution — must come from Ian. Tyr approving Tyr's own architectural changes would undermine the oversight model.

### Q: What special permissions does a "Captain" have?
The `Captain` rank (checked in `crates/koad-sandbox/src/lib.rs` via `Sandbox::evaluate()`) can bypass certain sandbox restrictions that block lower-ranked agents. Specifically: administrative operations on system-level paths, certain maintenance commands, and Citadel configuration changes. All bypasses are logged at high priority in the audit trail. The Captain rank also determines Tyr's workspace scope — Level 3 (Citadel) rather than Level 1 (Outpost) — allowing Tyr to operate across the entire `~/.koad-os/` directory structure.

## Source Reference

- `config/identities/tyr.toml` — Tyr's canonical Ghost identity definition
- `.agents/CREW.md` — The crew manifest; Tyr's official entry with rank, runtime, scope, and handoff norms
- `~/.tyr/memory/` — Tyr's personal vault: `SAVEUPS.md`, `LEARNINGS.md`, `PONDERS.md`
- `crates/koad-sandbox/src/lib.rs` — `Sandbox::evaluate()`; where Captain-rank bypass logic is implemented
- `AGENTS.md` — The operational directives all agents (including Tyr) must follow
