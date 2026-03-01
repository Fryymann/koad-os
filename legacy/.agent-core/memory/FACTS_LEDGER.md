# Facts Ledger

Use this ledger to record immutable truths about the environment, governance, or tooling that every future session should read before acting.

Entries should be brief and factual. When a fact changes (new policy, new requirement), add a new dated entry rather than editing history.

## Sample facts
- `2026-02-22`: Global koadOS kit lives in `~/.koad-os`; support scripts and memory ledgers are shared across workspaces.
- `2026-02-22`: Saveup outputs live under `~/.koad-os/.agent-core/sessions`; lane contexts should not edit the shared ledger unless explicitly approved.
- `2026-02-22`: `/home/ideans/data` is a symlink to `/mnt/c/data`. Accessing files under this path may require shell commands as it resolves outside the default home workspace.

- `2026-02-22`: koadOS CLI now supports automated saveup
- `2026-02-22`: Memory ledgers now have JSON sidecars for programmatic access
- `2026-02-22`: koadOS is now a standalone functional OS with automated boot and saveup protocols.
- `2026-02-22`: Environment is Ubuntu on WSL2 (Windows 11).
- `2026-02-22`: Ian Deans is the operator-engineer for Skylinks at Buchanan Fields, owning both operational workflows and technical systems.
- `2026-02-22`: Key integrations include Stripe ↔ Airtable ↔ WordPress ↔ GCP.
- `2026-02-22`: Google Account `ideans715@gmail.com` is the primary personal account.
- `2026-02-22`: Google Account `ian@skylinksgolf.com` is the primary account for ALL Skylinks development (GitHub, GCP, Airtable, etc.).
- `2026-02-22`: Google Account `iandeans062@gmail.com` is a secondary account, previously noted for GitHub but dev work is centralized on the work email.
- `2026-02-22`: All Skylinks projects are located in `~/data/skylinks/`.
- `2026-02-22`: `~/data/` is the general root directory for all development work.
- `2026-02-22`: GitHub MCP is now connected and integrated into the koadOS Gemini stack, providing advanced repository management and automation capabilities.
- `2026-02-22`: GitHub Authentication Rules:
  - Default: Use `GITHUB_PERSONAL_PAT` (for personal account `iandeans062@gmail.com`).
  - Skylinks: Use `GITHUB_SKYLINKS_PAT` (for work account `ian@skylinksgolf.com`) when working in `~/data/skylinks/` or `/mnt/c/data/skylinks/`.
- `2026-02-22`: User Technical Profile:
  - Background: Long-term programmer/computer user; prefers programmatic interaction.
  - Languages: Rust (Low-level choice), Node.js (Long-term expertise), Python (Initial foundation).
  - Philosophy: Simplicity first; Native tech preferred; Script-driven automation.
