# GitHub MCP Workflow Runbook

Status: Active
Last updated: 2026-02-22
Owner: Koad (PM)

## Purpose
Explain how agents should seed remote GitHub changes through MCP while keeping local operations in shell.

## Scope
- Repository: `<project-specific repo>` (fill in when applying this runbook).
- Branches: release branch (e.g., `v1`) and support branch (e.g., `koad-os`).
- Roles: `Koad (PM)`, `Gameplay`, `Platform`, `Experience`.

## Non-negotiables
1. Do local git work (`git`, build, tests) in shell.
2. Use GitHub MCP (`pull_request_read`, `merge_pull_request`, etc.) for remote state inspection and mutations when available.
3. Preserve governance gates configured in the workspace.
4. Keep lane handoffs merge-gated with the required reviews.

## Required configuration
Define an MCP server entry (example in `config.toml`):
```toml
[mcp_servers.github]
url = "https://api.githubcopilot.com/mcp/"
bearer_token_env_var = "GITHUB_PAT"
```
Ensure `GITHUB_PAT` is exported in the environment with appropriate scopes (`public_repo` for public repos, `repo` plus `read:org` for private/org contexts).

## Roles & permissions
- `Koad (PM)` may read/update/merge PRs, post gate comments, and annotate review status.
- `Gameplay`, `Platform`, `Experience` may inspect PRs, provide handoff comments, and respond to questions but should not merge without explicit instruction.

## Standard operating flow
1. Local checks: run tests, verify branch isolation, gather coverage evidence.
2. Remote reads: use `pull_request_read` with `get`, `get_status`, and `get_files` to confirm mergeability, check suites, and scope.
3. Gate validation: ensure required checks pass and branch/file scope remains valid.
4. Actions (MCP): post comments, update PR body or checklist, apply approvals, merge when ready.
5. Fallback: if MCP is unavailable, use `gh` CLI or wait/ask for operator intervention.

## Failure handling
- Auth errors: re-validate `GITHUB_PAT` and re-run a read before writes.
- Permission errors: fall back to read-only mode and escalate to Koad.
- Transient API errors: retry reads with backoff; avoid writing unless state is confirmed.

## Evidence & audit
- Document review comments, check states, and merge rationale for traceability.
- Keep PR bodies up-to-date with review-gate lines (self-review + Koad approval + Ian approval or equivalent) for automated gating.
