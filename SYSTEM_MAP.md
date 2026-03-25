# KoadOS System Map
# Generated: 2026-03-13 | Author: Scribe | TES: 14/30
# This file is the canonical workspace index. All agents should
# reference this instead of traversing ~/.koad-os directly.
# Maintained by Scribe. Notify Scribe when workspace structure changes.

## Quick Reference — Most Accessed Paths
| Path | What It Is | When You Need It |
| :--- | :--- | :--- |
| `MISSION.md` | Core Mission & Vision | Strategic Orientation |
| `AGENTS.md` | Root onboarding portal | Boot / New session |
| `ToolRegistry` | Phase 4 MCP Tool Service | Dynamic tool execution |
| `.agents/CITADEL.md` | Core mission & mission brief | Strategic orientation |
| `.agents/CREW.md` | Agent personnel manifest | Inter-agent coordination |
| `config/identities/` | Active KAI identities | Boot / Identity loading |
| `config/kernel.toml` | System-level configuration | Registry/Filesystem settings |
| `new_world/DRAFT_PLAN_3.md` | Current master rebuild plan | Phase verification |
| `crates/` | Active rebuild source code | Implementation tasks |
| `docs/rebuild/` | Implementation guides | Phase-specific tasks |
| `proto/` | gRPC definitions | API/Service implementation |

## Full Directory Tree (Annotated)
```text
~/.koad-os/
├── .admiral/          # The Dood's private directories
├── .agents/           # Shared agent artifacts, documentation, and private vaults
│   ├── cid/           # Cid personal vault (PRIVATE)
│   ├── claude/        # Claude personal vault (PRIVATE)
│   ├── clyde/         # Clyde personal vault (PRIVATE)
│   ├── .gemini/       # Gemini CLI system artifacts
│   ├── helm/          # Helm personal vault (PRIVATE)
│   ├── scribe/        # Scribe personal vault (PRIVATE)
│   ├── tyr/           # Tyr personal vault (PRIVATE)
│   ├── inbox/         # Shared agent communication inbox
│   ├── CITADEL.md     # Project core definition
│   └── CREW.md        # Personnel manifest
├── .git/              # Repository history
├── .github/           # GitHub workflows and CI
├── backups/           # Data and config backups
├── config/            # [SANCTUARY] Canonical system and agent configuration
│   ├── identities/    # Active agent personae (TOML)
│   ├── integrations/  # External service (Airtable, Slack, etc.) config
│   └── kernel.toml    # Primary OS configuration
├── crates/            # Active Rust crates (The rebuild source)
│   ├── koad-core/         # Shared primitives, config, session, logging
│   ├── koad-proto/        # Auto-generated gRPC bindings (tonic)
│   ├── koad-citadel/      # Citadel gRPC service (:50051)
│   ├── koad-cass/         # CASS gRPC service (:50052) — agent cognition
│   ├── koad-plugins/      # WASM plugin runtime (wasmtime)
│   ├── koad-cli/          # koad, koad-agent, koad-map binaries
│   ├── koad-board/        # Updates board service
│   ├── koad-sandbox/      # Container execution sandbox
│   ├── koad-codegraph/    # Static code graph analysis
│   ├── koad-intelligence/ # AI inference routing
│   └── koad-bridge-notion/ # Notion MCP bridge

├── docs/              # Architectural references and research
│   ├── protocols/     # Engineering and contribution standards
│   ├── rebuild/       # Phase-specific implementation specs
│   └── research/      # Technical feasibility studies
├── legacy/            # [ARCHIVE] Retired Spine (v1-v5) graveyard. DO NOT MIGRATE.
├── new_world/         # Strategic planning and architectural blueprints
├── proto/             # Raw protobuf definitions (.proto files)
├── AGENTS.md          # Primary onboarding portal
├── MISSION.md         # Core mission statement
├── Cargo.toml         # Rust workspace manifest
└── SYSTEM_MAP.md      # You are here.
```

## Config Files Index
- `config/kernel.toml`: System-level registry, environment, and jailing settings.
- `config/registry.toml`: Identity and agent service registry.
- `config/identities/*.toml`: Specific KAI persona definitions (Tyr, Sky, Scribe, Clyde, Helm).
- `config/filesystem.toml`: Filesystem map and mount points.

## Agent Bays Index
| Agent | Path | Status |
| :--- | :--- | :--- |
| **Tyr** | `.agents/tyr/` | Active |
| **Scribe** | `.agents/scribe/` | Active (This Vault) |
| **Cid** | `.agents/cid/` | Initialized |
| **Clyde** | `.agents/clyde/` | Active |
| **Sky** | `/mnt/c/data/skylinks/.agents/.sky/` | Active |
| **Helm** | `.agents/helm/` | Active |

## Crate/Module Index
- `crates/koad-core`: Shared primitives, config, session management, logging.
- `crates/koad-proto`: Auto-generated gRPC bindings (via `tonic`). Do not edit generated files.
- `crates/koad-citadel`: Citadel gRPC service (`:50051`). Session bays, signal corps, kernel state.
- `crates/koad-cass`: CASS gRPC service (`:50052`). Agent cognition, memory, updates board.
- `crates/koad-plugins`: WASM plugin runtime (wasmtime). Phase 4 dynamic tool execution.
- `crates/koad-cli`: `koad`, `koad-agent`, `koad-map` binaries. All CLI subcommands.
- `crates/koad-sandbox`: Container execution sandbox (Phase 4.2).
- `crates/koad-codegraph`: Static code graph for symbol analysis.
- `crates/koad-intelligence`: AI inference routing layer.
- `crates/koad-bridge-notion`: Notion MCP bridge (Noti remote agent integration).
- `crates/koad-board`: Updates board service.

## Documentation Index
- `docs/rebuild/DIRECTORY_CLEANUP.md`: Status of the `personas/` -> `config/identities/` move.
- `docs/rebuild/MIGRATION_MAPPING.md`: Strategy for extracting data from legacy `koad.db`.
- `docs/rebuild/PERSONAL_BAY_ARCH.md`: Specification for agent isolation.
- `docs/protocols/CONTRIBUTOR_CANON.md`: Standardized Git and coding protocols.
- `docs/protocols/RUST_CANON.md`: [MANDATORY] Rust development and coding standards.

## Stale/Deprecated Items
- `legacy/`: [ARCHIVE] All contents are retired Spine-era artifacts. Do not migrate from here.
- `new_world/old.DRAFT_PLAN*`: Superseded by `DRAFT_PLAN_3.md`.
- `kcitadel.log*`: System log files.
- `koad-watchdog`: Removed. Citadel self-healing is now integrated into `koad-citadel`.

## Navigation Tips
- **If you need an agent's identity:** Look in `config/identities/`, not `personas/`.
- **If you are implementing a service:** Start with the `.proto` in `proto/`, then check `crates/koad-proto/`.
- **If you need to know the mission status:** Read `AGENTS.md` and `new_world/DRAFT_PLAN_3.md`.
- **ROOT DOCUMENTATION:** Always use the root `/docs/` folder for canon protocols. Files found in `/.agents/claude/worktrees/` are isolated clones and should be ignored for canonical discovery.
- **If you find yourself in `legacy/`:** STOP. Verify why you are there. Do not copy logic.
