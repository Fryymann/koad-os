# Citadel SITREP (Situation Report)
**Date:** 2026-07-05
**Current Objective:** Fleet distribution prep and AIS documentation sync — v3.2.0 shipped, episode partition keying live.

## 🎯 Active Missions
- [x] **Episode Partition Keying:** DONE 2026-07-05 — designed (Condition Green), implemented, 46 production rows backfilled, verified live. Episodes carry explicit `partition` (canon `{agent}_{host}_{user}`); both read filters match on it.
- [x] **v3.2.0 Release Promotion:** DONE 2026-07-05 — merged nightly → main (`16a13ef`), tag `v3.2.0` pushed. 19 stale remote + 3 local branches pruned.
- [ ] **Fleet Distribution Prep:** Sandboxed fresh-install verification DONE 2026-07-06 — found + fixed 3 installer blockers (KOAD_HOME env wiped → live-instance clobber path; non-interactive overwrite; shallow WSL compose preflight). Remaining: happy-path install on an external host with working Docker.
- [x] **AIS Documentation Sync:** DONE 2026-07-06 — `docs/ais/operations/INSTALLER.md` covers `--install`/`--update`/`--home`, the silent-restart gotcha, and sandboxed testing.
- [x] **Docker Integration Guide:** DONE 2026-07-06 — WSL2 Docker Desktop integration, `.wslconfig` resource limits, Rust >=1.90 builders; folded into INSTALLER.md.

## 🗂 AIS Backlog
- [ ] Add machine-readable nMap JSON export workflow.
- [ ] Add link-check script for AIS docs and protocol references.
- [ ] Add owner metadata and last-reviewed dates to legacy docs.

## 🛠️ Recent Accomplishments
- **CASS-Primary Memory Paths (P1, live 2026-07-06, Dood ruling):** `commit_knowledge` writes fact cards through CASS gRPC with domain `{partition}:{category}` (koad.db demoted to offline fallback); `koad intel query` recalls semantically from CASS first. Bounded connect timeouts on both paths. Verified live: sentinel write → enrichment → rank-1 paraphrase recall. CLI agent identity now derives from `KOAD_SESSION_ID`; `partition_key` lowercases agent names. Superseded `citadel-io` branch deleted.
- **Episode Partition Keying (P1, live 2026-07-05):** `EpisodicMemory.partition` (proto field 8) stamped by all write paths, persisted in SQLite + Qdrant payload, partition-equality read filters with legacy fallback, `backfill_episode_partitions` binary (46/46 rows, idempotent). Live test: paraphrase recall under owning partition, zero cross-partition leakage. See `docs/cass-semantic-recall-verification.md`.
- **v3.2.0 Release:** nightly promoted to main; semantic memory release tagged and pushed.
- **CASS Semantic Enrichment Pipeline (P1, live):** Real embeddings via dedicated embedding client (`InferenceTask::Embedding`), Qdrant L3 with fingerprint fallback removed, enrichment worker consuming the `cass:enrichment` Redis stream with `is_meaningful` guard, `backfill_embeddings` migration binary. Verified 6/6 paraphrase recall. See `docs/cass-semantic-recall-verification.md`.
- **SearchSemantic Threshold Wiring (P2):** `min_score` threshold wired end-to-end; L3 threshold verdict made final in semantic search.
- **CASS Token-Aware Memory Metadata (P2, deployed):** Optional `MemoryMetadata` layer (SQLite L2 + Redis L1), budget-aware hydration, offline backfill binary. See `docs/cass-memory-metadata.md`.
- **Rook Decommission:** Hardcoded agent identity removed from the codebase.
- **Unified Installer & Updater (P1):** Single root entrypoint `install.sh` supporting `--install` and `--update`.

## 🏗️ Architectural Decisions
- **Episode partition canon is an explicit field** (`{agent}_{host}_{user}`, single source: `koad_core::utils::partition::partition_key`) — substring matching on session ids is retired to a legacy fallback.
- **Private Identity Separation:** User agent settings remain strictly decoupled from core codebase.
- **Rust Toolchain Modernization:** Container builders track modern compiler versions (>=1.90).

## 🔜 Immediate Next Actions
1. Happy-path installer verification on an external host with working Docker (last fleet-prep step).
2. AIS backlog: nMap JSON export, link-check script, owner metadata on legacy docs.
3. Consider exporting `KOAD_AGENT` from `agent-boot` (CLI currently derives agent from `KOAD_SESSION_ID`; unbooted shells fall back to "Admiral").
