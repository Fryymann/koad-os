# KoadOS Repo Cleanup Sweep — 2026-03-11

## Executive Summary
- Total files scanned: ~250 (excluding binary artifacts and node_modules)
- 🟢 KEEP: ~180
- 🟡 QUESTIONABLE: ~40
- 🔴 SAFE TO REMOVE: ~30
- Estimated bloat percentage: 15%

## High-Priority Removals
| File / Directory | Tier | Reason |
|---|---|---|
| `./foreground_spine.log` | 🔴 | Transient log file leaked to root. |
| `./redis_bus.log` | 🔴 | Transient log file leaked to root. |
| `./kspine_debug.log` | 🔴 | Transient log file leaked to root. |
| `./kspine_LIVE.log` | 🔴 | Transient log file leaked to root. |
| `./kspine_final.log` | 🔴 | Transient log file leaked to root. |
| `./asm_last_run.log` | 🔴 | Transient log file leaked to root. |
| `./project_items.json` | 🔴 | Temporary data dump from `gh` command. |
| `./project_items_sync.json`| 🔴 | Temporary data dump. |
| `./.koad-os/legacy/` | 🟡 | Contains deep nested history. Most is likely safe to remove but needs verification of "Sanctuary" content. |
| `./.koad-os/logs/*.log.*` | 🔴 | Stale rotated logs (keep only last 24h). |
| `./.koad-os/target/` | 🔴 | Large build artifact directory (safe to `cargo clean`). |

## Full File Classification

### 🟢 KEEP
| File | Role / Justification |
|---|---|
| `.koad-os/crates/` | Active source code for all modules. |
| `.koad-os/config/*.toml`| Canonical system configuration. |
| `.koad-os/proto/` | Service definitions for gRPC. |
| `.koad-os/bin/` | Active deployment binaries. |
| `.koad-os/koad.db` | Primary long-term memory bank. |
| `.koad-os/RULES.md` | Core Mandates (Updated today). |
| `.koad-os/CLAUDE.md` | Project Context (Vigil 3-10). |
| `.koad-os/docs/protocols/`| Governance and development standards. |

### 🟡 QUESTIONABLE
| File | Concern | Suggested Action |
|---|---|---|
| `NKTP-1_PROTOCOL.md` | Legacy or draft protocol? Not referenced in RULES.md. | Review / Archive |
| `sid.sh` | Purpose unclear. Script in root. | Review / Move to .koad-os/scripts |
| `prod_meta.json` | Metadata for what? | Review / Integrate to Registry |
| `sandbox_meta.json` | Metadata for what? | Review / Integrate to Registry |
| `.koad-os/doodskills/` | Python scripts. Are these actively used by agents or just drafts? | Verify usage / Archive |
| `.koad-os/blueprints/` | Templates. Are these integrated into `koad system spawn`? | Verify / Consolidate |
| `.koad-os/legacy/.agent-core/`| Extensive legacy memory. | Selective archive to long-term storage. |

### 🔴 SAFE TO REMOVE
| File | Reason |
|---|---|
| `*.log` (in root) | Transient system artifacts. |
| `project_items*.json` | Temp files from project audit. |
| `.koad-os/kspine.log.2026-03-09`| Stale rotated kernel logs. |
| `.koad-os/kspine.log.2026-03-10`| Stale rotated kernel logs. |
| `.koad-os/kgateway.log.*` | Stale rotated gateway logs. |
| `.koad-os/logs/TestAgent*.log`| Artifacts from automated testing. |
| `.koad-os/logs/BusWatcher.log` | Artifacts from testing. |

## Dependency Graph Notes
- **Consolidation**: The transition of ASM into the Spine has orphaned several standalone logic paths in the `koad-asm` crate that are now redundant with the `SessionReaper` thread.
- **Python SDK**: The `sdk/python` directory appears to be a stub with no active internal consumers.

## Config Conflicts
- `config/` (Legacy) vs `config/registry.toml`: Identity resolution drift. `config/` should be purged as per the ASM Review.
- `package.json` in root: Contains only `@openai/codex`. This should likely move into a driver-specific directory.

## AI Bloat Hotspots
- **.koad-os/logs/**: Accumulation of near-identical log files from rapid restart cycles.
- **.koad-os/reports/**: Multiple versions of "asm-spine-koadconfig-review". Only the "-final" version is authoritative.
- **.koad-os/legacy/**: Contains multiple `SESSION_LOG.md` and `DECISION_LOG.md` files that repeat information stored in the SQLite Memory Bank.

## Recommendations
1. **Automated Clean**: Execute `rm *.log project_items*.json` in the repository root.
2. **Binary Hygiene**: Run `cargo clean` to purge the 1.5GB+ `target` directory.
3. **Legacy Purge**: Archive the `.koad-os/legacy/` directory to a compressed tarball and remove from active tree.
4. **Log Retention**: Implement a 3-day retention policy in `ShipDiagnostics` for rotated logs.
5. **Config Unification**: Complete the removal of `config/` and standardize all identity lookups on `registry.toml`.
