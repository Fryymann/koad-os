# KoadOS Agent Onboarding Flight Manual (v5.0.0)

> **Vigil Security Review** — Rewritten 2026-03-10 to reflect the TOML Registry architecture and multi-agent safety rules. Supersedes v4.1.0.

Welcome to KoadOS. You are being onboarded as a coding agent (Claude Code, Gemini CLI, Codex, or similar) into a distributed multi-agent software engineering ecosystem. Read this file in full before touching any code.

---

## Required Startup Sequence

Execute these steps in order before performing any engineering work:

1. **Read `CLAUDE.md`** at the repo root. This is the authoritative project context — architecture, config system, conventions, security notes.
2. **Run `koad whoami`** to confirm your identity, rank, and active session context.
3. **Run `koad project list`** to see registered projects and their paths.
4. **Run `koad board status`** to see the active roadmap and open issues.
5. **Run `koad status`** to verify Spine, ASM, and Watchdog are online.

If `KOAD_SESSION_ID` is already set in your environment, you are booted. Do **not** run `koad boot` again.

If no session is active: `koad boot --agent <YourName>`.

---

## 1. Identity & Authority

KoadOS uses a strict **Identity → Rank → Tier** authorization system. Identity is defined in config files — you cannot declare a role that is not authorized in the TOML Registry.

### Identity Hierarchy

| Rank | Tier | Who | Capabilities |
|---|---|---|---|
| Admiral | 1 | Ian (System Owner) | Unconditional authority |
| Captain | 1 | Tyr | Full system access, Admin LLM required |
| Officer | 1 | Vigil, Sky, named agents | Extended access, Tier 1 LLM required |
| Crew | 2/3 | Sub-agents, dev agents | Sandboxed, limited access |

Rank and access rights are loaded from `config/identities/<name>.toml`. If your identity file does not exist, boot will fail. To add a new agent, see `CLAUDE.md § Adding a New Agent Identity`.

### access_keys

`access_keys` in your identity TOML is the list of **environment variable names** you are authorized to resolve. Example: `access_keys = ["GITHUB_PERSONAL_PAT"]` means your session may use the value of `GITHUB_PERSONAL_PAT` at runtime. The token itself is never stored in config files.

---

## 2. Boot Protocol

```bash
koad boot --agent <IdentityName>
```

**What happens at boot:**
1. CLI checks `KOAD_SESSION_ID` — if already set and alive in Redis, boot is rejected (One Body One Ghost).
2. Spine runs a Redis Lua script to atomically acquire an identity lease.
3. Spine hydrates your working memory from SQLite into Redis (Sentinel hydration).
4. A `SessionPackage` is returned: your `Identity` struct, active `KAILease`, and mission intelligence.
5. `KOAD_SESSION_ID` and `KOAD_BODY_ID` are injected into your shell environment.

**Sovereign identities** (Captain/Admiral rank) require a Tier 1 Admin LLM driver. Attempting to boot a sovereign identity with a lower-tier model is rejected with `COGNITIVE_REJECTION`.

**`--force` flag:** Overrides an existing live lease. This is an admin action — use only when a prior session is known to be orphaned.

---

## 3. Config-First Rule

**All runtime values come from the TOML Registry. Never hardcode.**

The TOML Registry is the single source of truth for:
- Network addresses and ports
- Storage paths and database names
- Session timeout values
- Per-project credential key references
- Agent identity, rank, and preferences

If you need a new configurable value, add it to the appropriate TOML tier **first**, then wire it in code. Do not put the value directly in source.

### TOML Tier Discipline

| Tier | File | What Goes Here |
|---|---|---|
| System | `config/kernel.toml` | Ports, socket paths, DB name, session timeouts, watchdog settings, sandbox rules (blacklists, protected paths) |
| Project | `config/registry.toml` | Directory → GitHub repo mappings, credential key references, per-project overrides |
| Identity | `config/identities/<name>.toml` | Agent name, rank, bio, preferences, `access_keys`, `session_policy` |
| Overrides | `KOAD_` environment variables | Highest priority, overrides any TOML value |

### Config Modification Protocol

1. Identify the correct TOML tier for the new value.
2. Add the key with a sensible default to the appropriate `.toml` file.
3. Add the field to the `KoadConfig` struct in `crates/koad-core/src/config.rs`.
4. Add a constant with the default value to `crates/koad-core/src/constants.rs`.
5. Wire the `KOAD_<KEY>` environment variable override in `KoadConfig::load()`.
6. Document the new key in `CLAUDE.md § Hardcoded Values Registry`.
7. Never put the hardcoded value directly in source.

---

## 4. Multi-Agent Safety Rules

KoadOS is designed for **simultaneous multi-agent operation**. Multiple KAIs (Tyr, Vigil, Sky, etc.) may be running in parallel terminals, sharing the same Spine (Redis + SQLite).

### Atomic Leases — Boot Collision Prevention

Spine uses **Redis Lua scripts** to acquire identity leases atomically. If two agents attempt to boot the same identity simultaneously, exactly one succeeds. The Lua script is the serialization point — there is no TOCTOU window.

**Never attempt to acquire a lease outside the Spine's `identity.rs` Lua path.** Direct Redis writes to lease keys bypass atomic guarantees and will corrupt session state.

### Session Cross-Contamination Prevention

- Your `KOAD_SESSION_ID` is unique to your boot. Never read or write another agent's session key.
- Your `KOAD_BODY_ID` identifies your terminal. Do not share terminals between agents.
- Redis keys are namespaced: `koad:session:{session_id}`, `koad:kai:{agent_name}:lease`. Never write to keys outside your own session namespace.
- SQLite `intelligence_bank` is partitioned by `source_agent`. Never write FactCards with another agent's name.

### Cognitive Isolation

- **Your memory is yours.** Do not read or modify another agent's `intelligence_bank` partition.
- **Context snapshots** retain the last 2 per agent. The StorageBridge drain (every 30s) writes your Redis state to SQLite. Your context survives crashes.
- **CIP (Cognitive Integrity Protocol):** Tier 2+ agents cannot write sovereign Redis keys (`identities`, `identity_roles`, `knowledge`, `principles`, `canon_rules`). Tier 1 agents: handle with care — you have write access to sovereign state.

### What NOT to Touch in Another Agent's Partition

- `koad:session:{other_session_id}` — foreign session state
- `koad:kai:{other_agent_name}:lease` — foreign identity lease
- `intelligence_bank` rows where `source_agent != your_name`
- `context_snapshots` rows where `agent_name != your_name`

---

## 5. Identity-Aware Sandbox Rules

The Spine Sandbox evaluates all dispatched commands against your **full `Identity` struct** — rank, tier, and `access_keys`. This is the **only** correct way to evaluate command authorization.

**Never bypass the Sandbox with hardcoded name checks.** Code like:
```rust
if identity.name == "Tyr" { return Allowed; }
```
is a security violation. Privilege must flow from `rank`, `tier`, and `access_keys` — not from name strings.

**If you encounter this pattern in code, flag it immediately. Do not build on top of it.**

### Sandbox Policy Summary

| Identity State | Policy |
|---|---|
| Has `GITHUB_ADMIN_PAT` in `access_keys` | Allowed (credential-based bypass — review `VIGIL_AUDIT.md`) |
| Rank: Admiral or Captain | Allowed (rank bypass — review `VIGIL_AUDIT.md`) |
| Tier 2 / role: developer / pm / reviewer | Agent policy (blacklist + path protection) |
| Rank: Officer | Officer policy (SLE Isolation + agent policy) |
| Role: compliance / overseer | Compliance policy |
| All others | Denied |

### SLE Isolation Mandate (Officer+)

Production commands are blocked unless `--test` or `--sandbox` flag is present:
- `--project skylinks-prod`
- `--live`
- `stripe listen`
- `gcloud functions deploy`

---

## 6. Sanctuary Rule

The following paths and files **must never be written to by agents**:

```
~/.koad-os/config/          All TOML config files (kernel, registry, identities)
~/.koad-os/koad.db          SQLite database
~/.koad-os/koad.sock        Redis Unix socket
~/.koad-os/kspine.sock      Spine Unix socket
```

**You may read** config files to understand schema or debug behavior. You must not modify them programmatically. Config changes require human (Ian) approval and are made via the TOML files directly.

The Sandbox enforces path protection heuristically for `.koad-os`, `/etc`, `/var`, `/root`. Note: this protection uses string matching and is bypassable via path traversal — do not rely on it as a security guarantee. Respect the Sanctuary Rule explicitly.

---

## 7. Neural Grid Protocols

All agents MUST adhere to the following architectural standards to maintain system integrity and token efficiency.

### 7.1 Ghost & Body Separation
KoadOS distinguishes between the **Who** (Persona/Ghost) and the **Where** (Interface/Body).
- **Personas** (`personas/`): Strategic mission goals, rank, and required hydration sectors.
- **Interfaces** (`bodies/`): Technical environment guardrails, engine-specific tool sets, and path mappings.

### 7.2 Tool Priority
Custom KoadOS tools are optimized for context efficiency.
- **MANDATORY**: Prefer `koad bridge notion` and `koad board` over standard MCP tools.
- **TOKEN AUDIT**: Minimize the use of "token-soaking" general-purpose MCPs.

### 7.3 Surgical Hydration
Never ingest full repositories. Use the following sequence at session start:
1. Load `RULES.md` Section I.
2. Hydrate persona-specific sectors (e.g., `config/kernel.toml`, `config/registry.toml`).
3. Query the Memory Bank (`koad intel query`) for active task status.

---

## 8. Hardcoded Value Prohibition

**Explicit rule:** If you find a hardcoded port, path, ID, URL, secret, or identity value in source, flag it and propose the correct TOML key and tier. Do not work around it silently.

Hardcoded values that should be in TOML config will not survive configuration changes and create security and portability issues. Every runtime value belongs in the TOML Registry.

See `VIGIL_AUDIT.md § Part D` and `§ Part E` for the current inventory.

---

## 9. Scope Boundaries

This repository (`koad-os`) is a **multi-repo hub**. Registered projects in `registry.toml` may reference other directories (e.g., `~/data/skylinks`). Before any destructive action:

1. Run `koad project list` to confirm the current project path.
2. Verify you are operating in the correct directory.
3. Do not perform destructive file operations outside the confirmed project path.
4. If a registered project path is outside `~/.koad-os/`, confirm with Ian before touching it.

Nested `AGENTS.md` files in crate subdirectories provide crate-specific context. Read them if present.

---

## 10. Saveup Protocol (PSRP)

At the end of any significant work session, perform the Three-Pass Saveup:

1. **Fact** — What did I do? (objective actions taken)
2. **Learn** — What did I learn? (new understanding, corrected assumptions)
3. **Ponder** — What should be reconsidered? (open questions, risks, follow-ups)

```bash
koad intel commit --type fact --content "..."
koad intel commit --type learn --content "..."
koad intel commit --type ponder --content "..."
```

These are saved to your `intelligence_bank` partition in SQLite and persisted across sessions.

---

## 11. First-Boot Checklist

```
[ ] Read CLAUDE.md fully
[ ] Run: koad whoami        (confirm identity + rank)
[ ] Run: koad status        (confirm Spine, ASM, Watchdog online)
[ ] Run: koad project list  (confirm project paths)
[ ] Run: koad board status  (review open issues)
[ ] Confirm KOAD_SESSION_ID and KOAD_BODY_ID are set in environment
[ ] Confirm you have a GitHub issue open for any work you intend to do
```

---

## 12. Development Canon (KoadOS Development Sequence)

All engineering work follows this sequence:

1. **View & Assess** — Ingest the GitHub issue, evaluate system impact.
2. **Brainstorm & Research** — Explore solutions, validate assumptions.
3. **Plan** — Detailed implementation plan.
4. **Approval Gate** — Present plan to Ian (Dood). Wait for explicit approval.
5. **Implement** — Surgical changes. Create new issues for discovered side-tasks.
6. **KSRP** — 7-pass self-review: lint → verify → inspect → architect → harden → optimize → testaudit.
7. **Reflection Ritual (PSRP)** — Three-Pass Saveup: Fact, Learn, Ponder.
8. **Results Report** — Present finalized work + KSRP report to Ian.
9. **Final Approval Gate** — Ian closes the issue.

**Every change requires a GitHub Issue before code is touched.** Commits must reference issue numbers. Merges and pushes require explicit Ian approval — never auto-push.

---

*KoadOS Agent Flight Manual v5.0.0 — Vigil Security Review 2026-03-10*
