# Citadel SITREP (Situation Report)
**Date:** 2026-07-05
**Current Objective:** v3.2.0 release promotion (nightly → main) and episode partition keying design for CASS semantic recall.

## 🎯 Active Missions
- [ ] **Episode Partition Keying Design:** Close the episode recall gap — `search_semantic` matches episodes by `session_id.contains(partition)`, which is empty under production ids. Design doc in progress; Dood Gate pending.
- [ ] **v3.2.0 Release Promotion:** Merge nightly → main (86+ commits) and cut the v3.2.0 tag.
- [ ] **Fleet Distribution Prep:** Verify fresh installation on external systems and prepare release tag.
- [ ] **AIS Documentation Sync:** Complete operating documents for the unified installer and `--update` flags.
- [ ] **Docker Integration Guide:** Document WSL Resource configuration constraints for new developers.

## 🗂 AIS Backlog
- [ ] Add machine-readable nMap JSON export workflow.
- [ ] Add link-check script for AIS docs and protocol references.
- [ ] Add owner metadata and last-reviewed dates to legacy docs.

## 🛠️ Recent Accomplishments
- **CASS Semantic Enrichment Pipeline (P1, live):** Real embeddings via dedicated embedding client (`InferenceTask::Embedding`), Qdrant L3 with fingerprint fallback removed, enrichment worker consuming the `cass:enrichment` Redis stream with `is_meaningful` guard, `backfill_embeddings` migration binary. Verified 6/6 paraphrase recall. See `docs/cass-semantic-recall-verification.md`.
- **SearchSemantic Threshold Wiring (P2):** `min_score` threshold wired end-to-end; L3 threshold verdict made final in semantic search.
- **CASS Token-Aware Memory Metadata (P2, deployed):** Optional `MemoryMetadata` layer (SQLite L2 + Redis L1), budget-aware hydration, offline backfill binary. See `docs/cass-memory-metadata.md`.
- **Rook Decommission:** Hardcoded agent identity removed from the codebase.
- **Unified Installer & Updater (P1):** Single root entrypoint `install.sh` supporting `--install` and `--update`.

## 🏗️ Architectural Decisions
- **Episode partition keying requires its own design pass** — session ids carry no partition string; fix is a schema/keying decision, not a filter tweak.
- **Private Identity Separation:** User agent settings remain strictly decoupled from core codebase.
- **Rust Toolchain Modernization:** Container builders track modern compiler versions (>=1.90).

## 🔜 Immediate Next Actions
1. Land dead-code cleanup and SITREP refresh on nightly; push.
2. Merge nightly → main, tag v3.2.0, push (Dood approval required).
3. Prune merged local branches; review remote-only branches for deletion (Dood approval required).
4. Complete episode partition keying design doc; obtain Condition Green.