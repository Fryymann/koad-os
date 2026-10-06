# Dev Log — 2026-10-06: Boot from a Fresh Shell, Session Purge, Off-Machine Backup

Author: Clyde (Claude Code, Opus 5.5), WSL body. Reviewed, merged and deployed by Ian.
Commits on `nightly`: `35a67a2`, `841b633` (from the Windows-side session, deployed here),
`dd1acdb`, `68ab375`. Pending merge: `60c2ce9` on `fix/session-purge-2h`.

## Why

The morning's Windows session shipped two fixes: boot into SELF.md, and choosing hydration facts by
priority. They needed deploying and verifying from WSL. Verification then turned up a chain of
problems, all of the same kind: something that only works when the environment happens to be set
up a particular way.

## What changed

### Deploy of `feat/self-anchor` and `fix/hydrate-fact-selection`

Verified live: new service PIDs started after the binary swap, and a fresh boot took 2.9 s and
reported CASS reachable. The packet's Active Fact Cards now lead with the `self` card, then
`identity-project`, then today's session card. Before the fix they were ten May cards about Rook.

### `agent-boot` skill works from a fresh harness shell (`dd1acdb`, `68ab375`)

Claude Code runs each Bash call in a fresh shell that has `KOAD_HOME` and `KOAD_BIN`, but not
`KOAD_RUNTIME`, `KOAD_AGENT_NAME` or `KOAD_VAULT_PATH`. Step 1 of the skill failed in two ways:

- `koad-agent boot` refused with `[BOOT DENIED] No agent body detected`. Step 1 now detects the
  runtime the same way `koad-functions.sh` does (`CLAUDE_CODE_ENTRYPOINT` → `claude`,
  Gemini/Antigravity signals → `gemini`) and never forces one. Hermes (`codex`) still has to
  export its own, so the body check stays meaningful.
- The session was written to `/sessions/current.env`, and steps 2–3 sourced the same broken path.
  The variable that locates the session file is defined only inside that file. Step 1 now reads
  the vault path from boot's own `KOAD_VAULT_PATH` export and prints the absolute session-file
  path, and every later step sources that literal path. A denied boot no longer truncates the
  existing session file.

**This was a regression.** On 2026-03-22 the same bug, with the same `CLAUDE_CODE_ENTRYPOINT`
signal, was fixed inside the `agent-boot` shell function (saveup TRC-CLYDE-20260322-SESSION5, in
`~/data/citadel_crossover`). The September skill inlined its own boot bash and bypassed that
function, so the fix was lost. KoadOS now has three boot implementations:
`scripts/koad-functions.sh`, `plugin/bin/agent-boot.sh` and the skill's step 1. Each duplicate is
somewhere a fix can disappear. Candidate for the trim: one boot script, called by everything.

### Session purge raised from 5 minutes to 2 hours (`60c2ce9`, pending merge)

Harness agents run no heartbeat daemon. Every authenticated call counts as a heartbeat
(`interceptor.rs`), so a session stays alive while the agent works, but it was purged after 300 s
of silence. Ian often leaves agents waiting 20–30 minutes, and the agent's next call then failed
with `Session not found or expired`. The same expiry broke a backup-then-modify sequence on
2026-09-27.

- `DEFAULT_PURGE_TIMEOUT_SECS` and the tracked `kernel.toml` files are now `7200`.
- "One Body, One Ghost" is unaffected. Exclusivity comes from the 90 s lease, not the purge.
- Tests: a check of the default (red at `300` first), and a drift guard that every tracked
  `kernel.toml` matches the compiled default. `config/kernel.toml` is gitignored, so the guard
  skips it, which keeps the test environment-independent.
- **Existing installs must edit their own `config/kernel.toml`.** `install.sh` never touches it,
  and it overrides the compiled default. On Jupiter that edit was made by hand and verified: a
  session survived 382 s of silence after the restart.

### Qdrant: confirmed rebuildable, empty collections removed

`fact_cards` (496) and `episodic_memories` (48) match `cass.db` exactly, and every payload field
is a SQLite column apart from `embedding_model`. `backfill_embeddings` rebuilds both. It is
dry-run by default; `--apply` drops and re-embeds. It is not installed to `$KOAD_HOME/bin`. Five
empty collections (`vigil_memories`, `sky_memories`, `tyr_memories`, `koados_knowledge`,
`task_outcomes`) had no live code references. Each was re-checked at 0 points and deleted.

## Outside this repo

- **`Fryymann/jupiter-backup`** (private): nightly, GPG-encrypted backup of the agent vaults, inbox,
  config, SQLite snapshots, Claude auto-memory, the WezTerm repo and the early-2026 agent worktrees
  in `~/data/citadel_crossover`. Every run decrypts its own archive before shipping. It ships to
  GitHub and to Google Drive via rclone (`drive.file` scope, Ian's own OAuth client, published to
  avoid the 7-day token expiry). The first run restored with `SELF.md` byte-identical and all
  facts present. The restore steps live in that repo's README.
- **RTK:** the hook-rewritten `diff` exits 0 on differences and once reported a changed file as
  identical. `diff` is now in RTK's `exclude_commands`. Verify with `/usr/bin/diff`, `cmp` or
  hashes.
- **Stale identity anchors:** four old Clyde `CLAUDE.md` anchors were deleted (including
  `~/CLAUDE.md`, which loaded into every home-directory session with obsolete filesystem rules).
  Three Hermes anchors were left for Hermes, with an inbox task.

## Open

- Merge `fix/session-purge-2h`, and update `config/kernel.toml` on any other install.
- One boot implementation instead of three (trim).
- The `self` CASS card duplicates SELF.md in every packet and still lists `RELATIONSHIPS.md` as
  planned. It needs summary mode or regeneration.
- Episodes have not been written since 2026-08-17.
