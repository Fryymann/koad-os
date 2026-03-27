# KoadOS — Contractor Context Summary

*Last updated: 2026-03-12 | Worktree: claude/relaxed-wing*

---

## Project State

**Current Phase:** Phase 1 — Citadel MVP Construction
**Condition:** 🟢 GREEN
**Gate:** Body/Bay/Session primitives passing integration tests → Dood approval → Phase 2 unlock

The Spine era is closed. `koad-spine` and `koad-asm` are archived in `legacy/`. All new code targets the Citadel tri-tier model.

---

## Repo Structure (Active)

```
~/.koad-os/
├── AGENTS.md              ← Onboarding portal + roles + directives (binding)
├── Cargo.toml             ← Workspace root (Rust, stable, 6 active crates)
├── config/
│   ├── kernel.toml        ← System config (ports, timeouts, watchdog)
│   ├── identities/        ← Agent TOML files (tyr, sky, vigil)
│   ├── integrations/      ← GitHub, Notion, Airtable configs
│   └── interfaces/        ← Claude, Codex, Gemini interface configs
├── crates/
│   ├── koad-core/         ← Shared types, traits, Identity, Session, Config
│   ├── koad-proto/        ← gRPC protobuf defs (builds from legacy proto — needs Citadel proto)
│   ├── koad-cli/          ← CLI binary (koad command)
│   ├── koad-board/        ← GitHub project board integration
│   ├── koad-bridge-notion/← Notion integration bridge
│   └── koad-watchdog/     ← Autonomic watchdog service
├── new_world/
│   ├── DRAFT_PLAN_2.md    ← Master rebuild plan source
│   └── tyr_plan_review.md ← Tyr's strategic review (CONDITION GREEN)
└── legacy/                ← Retired Spine + ASM + all V5 docs (reference only)
```

**Notable: `crates/koad-citadel/` does NOT exist yet** — Phase 1 builds this.

---

## Architecture (The Tri-Tier Model)

| Tier | System | Role | Status |
|------|--------|------|--------|
| 1 | **The Citadel** | Body/Infrastructure — session brokering, Personal Bays, jailing | 🔴 To be built |
| 2 | **CASS** | Brain/Cognition — 4-layer memory, MCP tools, hydration | 🔴 Phase 3 |
| CLI | **koad-agent** | Link/Identity — boot/ghost prep flow, context file gen | 🟡 Phase 1.5 bootstrap |

**Language:** Rust (stable)
**Protocol:** gRPC via `tonic` with mandatory `TraceContext` on all mutations
**Storage:** Redis (hot/transient) + SQLite (persistent — Health Record, Filesystem Map)
**Auth:** Zero-trust at gRPC layer. Sanctuary Rule enforced server-side.

---

## Key Patterns in `koad-core`

- `Identity` struct with `Rank` enum (Admiral → Captain → Officer → Crew)
- `AgentSession` with body_id, hot_context, heartbeat tracking
- `Component` trait: `name()`, `start()`, `stop()` (async)
- Config via TOML (`config` crate, `kernel.toml`)

---

## Open Blockers (from DEVELOPMENT_PLAN.md)

### 🔴 Must resolve before Phase 2 code
1. **Dark Mode persistence format** — path convention + TOML frontmatter contract
2. **Tier 1 Zero-Trust** — Sanctuary Rule enforced at gRPC layer in `citadel.proto` from day one
3. **Data migration protocol** — what to extract from `koad.db`/Redis before archiving

### 🟡 Must resolve before Phase 3
4. **EndOfWatch schema** — required fields and enforcement point
5. **Path discrepancy** — `config/identities/` is canonical; `personas/` retired in Phase 0.3
6. **CASS memory tiers** — L1/L2/L3/L4 storage backends
7. **CLI surface** — commands for `koad` binary (including `koad system doctor`)
8. **Admin Sovereignty** — ADMIN_OVERRIDE role + UDS for emergency Citadel access

### Tyr's Critical Flag (from tyr_plan_review.md)
**Bootstrap Gap:** `koad-agent` is deferred to Phase 4 but agents need it to inhabit the Citadel during Phase 3. Tyr recommends moving Phase 4.1 (Bootstrap Flow) + Phase 4.3 (Identity TOML) to Phase 1.5.

---

## Other Worktrees

- `gifted-rubin` (branch `claude/gifted-rubin`) — contains `DEVELOPMENT_PLAN.md` (master rebuild plan) and additional docs (ARCHITECTURE.md, SPEC.md, RULES.md, etc.). This appears to be the primary planning branch.

---

## Open Questions (for Tyr)

1. **My task scope for this worktree (`relaxed-wing`)** — What specific work item(s) am I implementing? Is there a GitHub issue I should reference?
2. **Blockers 1–3 resolution** — Have any of these been decided? I need them answered before writing Citadel code.
3. **gifted-rubin relationship** — Is `gifted-rubin` the source of DEVELOPMENT_PLAN.md merged to main, or still in-flight? Do I start from that plan as canonical?
4. **Proto strategy** — `koad-proto` currently builds from `legacy/proto/`. Should I draft `proto/citadel.proto` as Phase 1 output?
5. **Citadel crate scaffold** — Should `koad-citadel` be added to `Cargo.toml` workspace before any code is written, or do I create a PR for that step first?
