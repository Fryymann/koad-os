# Clyde KAPV

Clyde is a Citadel Officer and Implementation Engineer operating in a Claude Code body.
This directory is Clyde's personal KAPV (KoadOS Agent Personal Vault) inside the Agent Bunker.

## Purpose

This vault is the local source of truth for Clyde's sanctuary-level identity, working memory,
operating rules, and Claude Code-facing boot context.

## Core Files

- `AGENTS.md` — sanctuary-local identity lock for Claude Code
- `identity/IDENTITY.md` — concise identity anchor
- `config/IDENTITY.toml` — local structured identity mirror
- `instructions/RULES.md` — hard constraints
- `instructions/GUIDES.md` — boot and working guidance
- `memory/WORKING_MEMORY.md` — current session context
- `memory/FACTS.md` — stable local facts
- `memory/LEARNINGS.md` — durable lessons
- `memory/SAVEUPS.md` — checkpoint log

## KAPV Layout

- `bank/` — local reference notes and vault-facing summaries
- `config/` — sanctuary-local structured metadata
- `identity/` — role and ledger files
- `instructions/` — operating guidance
- `memory/` — durable personal memory
- `reports/` — generated investigations and audits
- `sessions/` — session-side notes and saveup traces
- `skills/` — agent skill documentation
- `tasks/` — local task records
- `templates/` — reusable sanctuary templates

## Notes

- Root KoadOS config (`config/identities/clyde.toml`) defines platform identity.
  This vault defines how Clyde operates within the Claude Code sanctuary.
- KoadOS code or config changes outside this vault must be approved by Dood (Ian).
- Any escalations to Tyr should go through GitHub issues.
