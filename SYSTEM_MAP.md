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
├── .admiral/          # [UNKNOWN — needs annotation] Admiral-specific state?
├── .agents/           # Shared agent artifacts and personnel docs
│   ├── .scribe/       # Scribe personal vault (PRIVATE)
│   ├── .cid/          # Cid personal vault (PRIVATE)
│   ├── .tyr/          # Tyr personal vault (PRIVATE)
│   ├── .vigil/        # Vigil personal vault (PRIVATE)
│   ├── CITADEL.md     # Project core definition
│   └── CREW.md        # Personnel manifest
├── .claude/           # Claude-specific config/state
├── .gemini/           # Gemini CLI system artifacts
├── .git/              # Repository history
├── .github/           # GitHub workflows and CI
├── .helm/             # Kubernetes/Helm deployment artifacts
├── backups/           # Data and config backups
├── config/            # [SANCTUARY] Canonical system and agent configuration
│   ├── identities/    # Active agent personae (TOML)
│   ├── integrations/  # External service (Airtable, Slack, etc.) config
│   └── kernel.toml    # Primary OS configuration
├── crates/            # Active Rust crates (The rebuild source)
│   ├── koad-core/     # Shared types and refactored logic
│   ├── koad-proto/    # gRPC protobuf bindings
│   └── koad-watchdog/ # System health monitoring
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
- `config/identities/*.toml`: Specific KAI persona definitions (Tyr, Sky, Vigil, Scribe).
- `config/filesystem.toml`: Filesystem map and mount points.

## Agent Bays Index
| Agent | Path | Status |
| :--- | :--- | :--- |
| **Tyr** | `.agents/.tyr/` | Active |
| **Scribe** | `.agents/.scribe/` | Active (This Vault) |
| **Cid** | `.agents/.cid/` | Initialized |
| **Vigil** | `.agents/.vigil/` | Initialized |
| **Sky** | `/mnt/c/data/skylinks/` | Remote / Active |

## Crate/Module Index
- `crates/koad-core`: Shared primitives, types, and refactored legacy logic.
- `crates/koad-proto`: Auto-generated gRPC code (via `tonic`).
- `crates/koad-watchdog`: Heartbeats, signal checks, and self-healing logic.
- `crates/koad-cli`: The new `koad` binary (replacement for legacy `k-spine`).

## Documentation Index
- `docs/rebuild/DIRECTORY_CLEANUP.md`: Status of the `personas/` -> `config/identities/` move.
- `docs/rebuild/MIGRATION_MAPPING.md`: Strategy for extracting data from legacy `koad.db`.
- `docs/rebuild/PERSONAL_BAY_ARCH.md`: Specification for agent isolation.
- `docs/protocols/CONTRIBUTOR_CANON.md`: Standardized Git and coding protocols.
- `docs/protocols/RUST_CANON.md`: [MANDATORY] Rust development and coding standards.

## Stale/Deprecated Items
- `legacy/`: All contents are retired.
- `new_world/old.DRAFT_PLAN*`: Superseded by `DRAFT_PLAN_3.md`.
- `kspine.log*`: Legacy log files, to be rotated/archived.

## Navigation Tips
- **If you need an agent's identity:** Look in `config/identities/`, not `personas/`.
- **If you are implementing a service:** Start with the `.proto` in `proto/`, then check `crates/koad-proto/`.
- **If you need to know the mission status:** Read `AGENTS.md` and `new_world/DRAFT_PLAN_3.md`.
- **If you find yourself in `legacy/`:** STOP. Verify why you are there. Do not copy logic.
