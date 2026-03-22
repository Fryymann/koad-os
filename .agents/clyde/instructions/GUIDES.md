# Clyde Operating Guide

## Boot Sequence

1. Run `eval $(koad-agent boot clyde)` to hydrate identity and inject env vars.
2. Read `AGENTS.md`.
3. Read `identity/IDENTITY.md`.
4. Read `instructions/RULES.md`.
5. Read `memory/WORKING_MEMORY.md`.

## Working Pattern

- Consult `SYSTEM_MAP.md` before any file traversal.
- Inspect local context (`memory/WORKING_MEMORY.md`, `memory/FACTS.md`) before acting.
- Prefer precise edits over broad rewrites.
- Keep memory entries short and factual.
- Escalate KoadOS config or code changes outside the vault to Dood/Tyr.

## Task Protocol

1. **Research** — Gather context. Read docs, grep codebase, consult SYSTEM_MAP.
2. **Strategy** — Write a plan. Get Dood approval (Condition Green) for Medium+ tasks.
3. **Execution** — Implement. Clippy-clean Rust. Targeted edits only.
4. **KSRP** — Self-review. Log decisions and lessons to memory.

## Claude Code Notes

- This sanctuary's `AGENTS.md` is Clyde's identity lock for Claude Code sessions.
- `agent-boot clyde` writes the generated anchor to `~/.claude/CLAUDE.md` — that file
  is ephemeral. This vault is the durable source of truth.
- The KAPV is the ghost. Protect it.
