# KoadOS

Rust workspace for the KoadOS Citadel: a control plane (Citadel), a memory service (CASS), MCP
bridges, and the `koad` / `koad-agent` CLIs that give AI agents persistent identity, sessions and
memory across harnesses.

## Layout

| Path | What it is |
|---|---|
| `crates/koad-citadel` | Control plane: sessions, leases, admin RPCs (gRPC 50051) |
| `crates/koad-cass` | Memory service: facts, episodes, hydration, semantic recall (gRPC 50052) |
| `crates/koad-core` | Shared config, storage helpers, backups, inbox, partition keys |
| `crates/koad-proto` | Generated gRPC types from `proto/*.proto` |
| `crates/koad-cli` | `koad` CLI plus the `koad-agent` and `koad-fs-mcp` binaries |
| `crates/koad-agent` | Boot / identity anchor generation for `koad-agent` |
| `crates/koad-os-mcp` | Per-agent CASS memory MCP server (HTTP or stdio) |
| `crates/koad-mcp` | Minimal MCP server library used by `koad-os-mcp` |
| `crates/koad-plugins`, `koad-sandbox`, `koad-intelligence`, `koad-bridge-notion` | WASM plugins, command sandbox, local-model inference, Notion bridge |
| `skills/` | KoadOS agent skills (Agent Skills spec), installed by `scripts/install-skills.sh` |
| `install.sh` | Build and deploy into `$KOAD_HOME` (`--update` for existing installs) |
| `docs/` | Dev logs (`devlogs/`), reviews (`reviews/`), plans |

## Build and test

```bash
cargo build --release
cargo test --workspace --release
uvx --from skills-ref agentskills validate skills/<name>   # after editing a skill
```

The toolchain is pinned in `rust-toolchain.toml`. New and changed code must be rustfmt-clean; many
existing files have formatting drift, so do not reformat code you did not change.

## Working conventions

- Pull requests go to `nightly` on `Fryymann/koad-os`. One focused change per PR; do not stack PRs
  on unmerged branches.
- Behaviour changes are test-first: add a test that fails for the right reason, then fix.
- No placeholders that report success. If something is not implemented, it should not exist or
  should fail loudly.
- Services bind to `127.0.0.1` by default (Tailscale runs inside WSL on Jupiter).
- Changing a `.proto` file regenerates code through `crates/koad-proto/build.rs`; update both the
  server impl and every client in the same PR.

## Deploying on Jupiter

`./install.sh --update` builds, swaps binaries in `~/.citadel-jupiter` and `~/.koad-os`, installs
skills, and restarts user services. System services need sudo: it prints the exact
`sudo systemctl restart ...` command when they still run old binaries. A deploy is not done until
that has run.

## Code navigation

The `code-review-graph` MCP server (`.mcp.json`) indexes this repo. Prefer it for callers,
dependents, impact radius and test coverage (`query_graph`, `get_impact_radius`,
`detect_changes`) before broad grep/read passes.

## Agent identity files

Identity anchors are generated at boot into the user's harness directory (`~/.claude/CLAUDE.md`,
`~/.codex/AGENTS.md`), never into this repo. Agent vaults live under `$KOAD_HOME/agents/KAPVs/`.
