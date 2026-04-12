# KoadOS System Map
# Updated: 2026-03-27 | Author: Tyr | Distribution split; Architecture v3.2
# This file is the canonical workspace index. All agents should
# reference this instead of traversing the filesystem directly.

---

## Workspace Hierarchy (Distribution vs. Instance)

KoadOS follows a strict separation between the **Distribution** (the shared platform code and standards) and the **Instance** (the specific deployment state of a Citadel).

1.  **Code & Defaults (Distribution):** Tracked in Git. Includes Rust crates, proto definitions, and `config/defaults/`.
2.  **Standards & Docs (Distribution):** Tracked in Git. Includes `MISSION.md`, `AGENTS.md`, and `docs/`.
3.  **Local State (Instance):** Gitignored. Includes `data/db/`, `logs/`, `run/`, and active configurations (`config/kernel.toml`).

---

## Workspace Topology

| Location | Role | Commander |
| :--- | :--- | :--- |
| `~/.koad-os/` | **Citadel** — KoadOS project root, Rust source, all shared config | Dood (Ian) |
| `~/data/SLE/` | **SLE Station** — Skylinks Local Ecosystem; apps and Sky's command post | Sky |

---

## Quick Reference — Most Accessed Paths

| Path | What It Is | When You Need It |
| :--- | :--- | :--- |
| `MISSION.md` | Core Mission & Vision | Strategic orientation |
| `AGENTS.md` | Root onboarding portal | Boot / new session |
| `agents/CITADEL.md` | Core mission brief | Strategic orientation |
| `agents/CREW.md` | Personnel manifest template | Inter-agent coordination |
| `config/identities/` | Active KAI identities (TOML) | Boot / identity loading |
| `config/kernel.toml` | System-level configuration (Local) | Registry / filesystem settings |
| `crates/` | Active Rust source | Implementation tasks |
| `docs/rebuild/` | Implementation guides | Phase-specific tasks |
| `proto/` | gRPC definitions | API / service implementation |
| `data/db/` | All runtime SQLite databases (Local) | Storage debugging |
| `run/` | Runtime sockets and PID files (Local) | Service health / debugging |

---

## Citadel Directory Tree — `~/.koad-os/`

```text
~/.koad-os/
├── agents/                    # Agent hub — vaults, bays, bodies
│   ├── bays/                  # [LOCAL] Per-agent Citadel state DBs
│   │   └── <name>/state.db
│   ├── crews/                 # [LOCAL] Active crew manifests
│   │   ├── <citadel>.md
│   │   └── TEMPLATE.md        # [TRACKED] Manifest template
│   ├── bodies/                # Agent body boot documents
│   │   └── claude/BOOT.md
│   ├── cid/                   # Cid KAPV (PRIVATE)
│   ├── claude/                # Claude KAPV (PRIVATE)
│   ├── clyde/                 # Clyde KAPV (PRIVATE)
│   ├── helm/                  # Helm KAPV (PRIVATE)
│   ├── pic/                   # Pic KAPV (PRIVATE)
│   ├── scribe/                # Scribe KAPV (PRIVATE)
│   ├── vigil/                 # Vigil KAPV (deprecated; archived)
│   ├── inbox/                 # Shared inter-agent message inbox
│   ├── quests/                # Shared mission objectives
│   ├── .gemini/               # Gemini CLI system artifacts (do not rename)
│   ├── CITADEL.md             # Project core definition
│   ├── CREW.md                # [TRACKED] Crew manifest pointer
│   └── SESSIONS_LOG.md        # [LOCAL] Cross-session activity log
│
├── bin/                       # Compiled binaries and shell functions
│   ├── koad                   # Primary CLI binary (includes Review & Skill)
│   ├── koad-agent             # Agent boot / identity binary
│   ├── koad-citadel           # Citadel gRPC server binary
│   ├── koad-cass              # CASS gRPC server binary
│   ├── agent-boot             # Shell-executable agent boot wrapper
│   └── koad-functions.sh      # Sourceable shell function library
│
├── cache/                     # [LOCAL] Ephemeral session briefs and boot metrics
│
├── config/                    # [SANCTUARY] Canonical system and agent configuration
│   ├── defaults/              # [TRACKED] Canonical config templates
│   │   ├── kernel.toml
│   │   └── redis.conf
│   ├── identities/            # Active KAI personae (TOML) — one file per agent
│   │   └── deprecated/        # Retired agent TOMLs (vigil)
│   ├── interfaces/            # Runtime interface config (claude, gemini, codex)
│   ├── integrations/          # External service config (Airtable, Notion, GitHub)
│   ├── systemd/               # koad-citadel.service, koad-cass.service
│   ├── kernel.toml            # [LOCAL] Primary system configuration
│   └── redis.conf             # [LOCAL] Redis socket / persistence config
│
├── crates/                    # Rust workspace — 11 active crates
│   ├── koad-core/             # Shared primitives, config, session, logging
│   ├── koad-proto/            # gRPC bindings (tonic, auto-generated)
│   ├── koad-citadel/          # Citadel gRPC service (:50051)
│   ├── koad-cass/             # CASS gRPC service (:50052) — agent cognition
│   ├── koad-plugins/          # WASM plugin runtime (wasmtime)
│   ├── koad-cli/              # koad, koad-agent binaries; CLI handlers and utils
│   ├── koad-board/            # Updates board service
│   ├── koad-sandbox/          # Container execution sandbox (Phase 4.2)
│   ├── koad-codegraph/        # Static code graph analysis
│   ├── koad-intelligence/     # AI inference routing
│   └── koad-bridge-notion/    # Notion MCP bridge (Noti integration)
│
├── data/                      # Persistent data storage
│   ├── db/                    # [LOCAL] All runtime SQLite databases
│   │   ├── koad.db            # Primary KoadOS state DB
│   │   ├── citadel.db         # Citadel session / bay state
│   │   ├── cass.db            # CASS cognition / memory
│   │   ├── codegraph.db       # Code graph index
│   │   └── notion-sync.db     # Notion bridge sync cache
│   └── redis/                 # [LOCAL] Redis RDB persistence (dump.rdb)
│
├── docs/                      # Architectural references and research
│   ├── protocols/             # Engineering and contribution standards
│   ├── rebuild/               # Phase-specific implementation specs
│   └── research/              # Technical feasibility studies
│
├── install/                   # First-time setup and service installation
│   └── bootstrap.sh           # Post-clone bootstrap script
│
├── logs/                      # [LOCAL] Service log files (gitignored)
│   └── redis.log, citadel.log, cass.log, telemetry.log, ...
│
├── new_world/                 # Strategic planning and architectural blueprints
├── plans/                     # In-progress implementation plans
├── proto/                     # Raw protobuf definitions (.proto files)
├── run/                       # [LOCAL] Runtime sockets and PID files (gitignored)
│   ├── koad.sock              # Redis Unix socket
│   ├── kadmin.sock            # Citadel admin socket
│   └── redis.pid
│
├── scripts/                   # Maintenance and init scripts
│   └── verify-services.sh     # Qdrant readiness check (PreExec)
├── skills/                    # Agent skill bundles
├── templates/                 # Reusable scaffolding templates
├── tests/                     # Stress and integration test scripts
├── updates/                   # KoadStream board update posts
│
├── .github/                   # GitHub Actions workflows
├── backups/                   # [LOCAL] DB snapshots (gitignored)
├── AGENTS.md                  # Primary onboarding portal
├── MISSION.md                 # Core mission statement
├── Cargo.toml                 # Rust workspace manifest
└── SYSTEM_MAP.md              # You are here.
```

---

## SLE Station Directory Tree — `~/data/SLE/`

```text
~/data/SLE/
├── .station/                          # Station infrastructure
│   ├── agents/
│   │   └── .sky/                      # Sky KAPV (PRIVATE — Station Commander)
│   │       ├── identity/              # IDENTITY.md, XP_LEDGER.md
│   │       └── memory/               # FACTS, LEARNINGS, WORKING_MEMORY, ponders
│   └── tools/                         # Station-level integrations
│       ├── airtable/
│       └── notion/
└── apps/                              # SLE application projects
```

**Station Commander:** Sky — Officer, Tier 1 Specialist. SLE authority; enforces KoadOS standards across all SCE outposts.

---

## Agent Vaults Index

| Agent | Vault Path | Runtime | Status |
| :--- | :--- | :--- | :--- |
| **Tyr** | `~/.tyr/` | Gemini | Active — Captain |
| **Clyde** | `~/.koad-os/agents/KAPVs/clyde/` | Claude Code | Active — Officer |
| **Cid** | `~/.koad-os/agents/KAPVs/cid/` | Codex | Active — Engineer |
| **Scribe** | `~/.koad-os/agents/KAPVs/scribe/` | Gemini (flash-lite) | Active — Crew |
| **Helm** | `~/.koad-os/agents/KAPVs/helm/` | Gemini | Active — Officer |
| **Claude** | `~/.koad-os/agents/.claude/` | Claude Code | Active — Contractor |
| **Sky** | `~/data/SLE/.station/agents/sky/` | Gemini | Active — Officer (SLE) |
| **Vigil** | `~/.koad-os/agents/vigil/` | Gemini | Deprecated |

---

## Crate / Module Index

| Crate | Binary | Port | Purpose |
| :--- | :--- | :--- | :--- |
| `koad-core` | — | — | Shared primitives, config, session management, logging |
| `koad-proto` | — | — | gRPC bindings (tonic). Do not edit generated files. |
| `koad-citadel` | `koad-citadel` | `:50051` | Session bays, signal corps, kernel state |
| `koad-cass` | `koad-cass` | `:50052` | Agent cognition, memory, updates board |
| `koad-plugins` | — | — | WASM plugin runtime (wasmtime) — Phase 4 |
| `koad-cli` | `koad`, `koad-agent` | — | All CLI subcommands; includes `review` and `bridge skill` |
| `koad-board` | — | — | Updates board service |
| `koad-sandbox` | — | — | Config-driven sandbox; containerized execution active |
| `koad-codegraph` | — | — | Static code graph for symbol analysis |
| `koad-intelligence` | — | — | AI inference routing layer |
| `koad-bridge-notion` | — | — | Notion MCP bridge (Noti remote agent) |

---

## Config Files Index

- `config/defaults/` — Canonical templates for `kernel.toml` and `redis.conf`.
- `config/kernel.toml` — [LOCAL] Active system-level config: network ports, socket paths, DB name.
- `config/redis.conf` — [LOCAL] Active Redis socket, PID, and log settings.
- `config/identities/*.toml` — Per-agent KAI persona definitions; vault and bootstrap paths.
- `config/interfaces/*.toml` — Runtime interface config (claude, gemini, codex).
- `config/integrations/*.toml` — External service config (Airtable, Notion, GitHub).

---

## Documentation Index

- `docs/rebuild/MIGRATION_MAPPING.md` — Strategy for extracting data from legacy `koad.db`.
- `docs/rebuild/PERSONAL_BAY_ARCH.md` — Specification for agent isolation and bay architecture.
- `docs/protocols/CONTRIBUTOR_CANON.md` — Standardized Git and coding protocols.
- `docs/protocols/RUST_CANON.md` — [MANDATORY] Rust development and coding standards.

---

## Stale / Deprecated Items

- `legacy/` — [ARCHIVE] Retired Spine-era artifacts. Do not migrate from here.
- `new_world/archived/` — Superseded draft plans.
- `koad-watchdog` — Removed. Citadel self-healing integrated into `koad-citadel`.

---

## Navigation Tips

- **Agent vault:** `~/.koad-os/agents/KAPVs/<name>/` — config is in `config/identities/<name>.toml`.
- **Agent bay DB:** `~/.koad-os/agents/bays/<name>/state.db` — auto-provisioned by Citadel on startup.
- **Runtime sockets/pids:** always under `run/` — never in the project root.
- **All databases:** always under `data/db/` — never in the project root.
- **Implementing a service:** start with `.proto` in `proto/`, then `crates/koad-proto/`.
- **If you find yourself in `legacy/`:** STOP. Verify why you are there. Do not copy logic.
