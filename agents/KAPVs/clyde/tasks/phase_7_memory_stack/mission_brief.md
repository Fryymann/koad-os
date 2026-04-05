# Mission Brief: Phase 7 — CASS Tiered Memory Stack (L1-L4)
**Status:** ✅ COMPLETE
**Lead:** Clyde (Officer)
**Objective:** Refactor and expand CASS memory services to support hot (Redis), episodic (SQLite), and semantic (Qdrant) storage tiers.

## Ⅰ. Strategic Roadmap (SDD)
- [x] **Phase 1: Foundation & L1** (clyde-dev) — Add `qdrant-client` and implement `RedisTier`.
- [x] **Phase 2: Semantic Stack** (clyde-dev) — Implement `QdrantTier` for vector search.
- [x] **Phase 3: Fact Extraction** (clyde-dev) — Integrate auto-extraction in `record_episode`.
- [x] **Phase 4: Validation & Review** (clyde-qa) — Semantic retrieval verification and benchmarking.


## Ⅱ. Team Task Packets

### **clyde-dev (Memory Stack Implementation)**
- **Task ID:** `clyde-20260403-dev-03`
- **Objective:** Refactor storage and implement L1-L3.
- **Requirements:** 
    - Create a `MemoryTier` trait with `commit_fact` and `query_facts`.
    - Refactor `CassStorage` into `SqliteTier`.
    - Implement `RedisTier` (using `fred` in `koad-core`).
    - Implement `QdrantTier` (requires adding `qdrant-client`).
    - Orchestrate these in `MemoryService`.

### **clyde-qa (Validation & Audit)**
- **Task ID:** `clyde-20260403-qa-03`
- **Objective:** Verify semantic retrieval and performance.
- **Requirements:** 
    - Perform semantic search tests using `QueryFacts`.
    - Verify data persistence in `data/db/cass.db` (L2/L4).
    - Ensure `koad review` passes on all modified crates.
    - Measure and report gRPC response times for multi-tiered queries.

## Ⅲ. Governance & Reporting
- Maintain `TEAM-LOG.md`.
- Escalate any library or infra blocks to `ESCALATIONS.md`.
- **Note:** Ensure Qdrant is running (`scripts/verify-services.sh` should be used during testing).

---
*Authorized by Tyr (Captain) | 2026-04-03*
