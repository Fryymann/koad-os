# CASS Semantic Recall — Verification Report

**Date:** 2026-07-02
**Author:** Clyde (Officer) — post-deploy verification of the semantic enrichment pipeline
**System under test:** live citadel-jupiter instance (koad-cass PID of 2026-07-02 deploy, nomic-embed-text embeddings, granite3.3:2b enrichment)

## What was tested

Recall was driven through the **real agent surface** — `koad-os-mcp` over stdio JSON-RPC
(`memory.search_semantic` / `memory.commit` tools) → gRPC `SearchSemantic` → `TieredStorage` →
Qdrant — i.e. exactly the path Rook/Claude Desktop and CLI agents use. Not a unit test.

Corpus: the real production memory store — 95 fact cards (clyde 24, hermes 71) + 46 episodes,
all re-embedded at 768-dim and LLM-enriched during the deploy.

## Method

1. **Paraphrase battery** — 6 natural-language queries with minimal-to-zero keyword overlap
   against known memories, per partition. Baseline: the L2 substring fallback
   (`content LIKE '%query%'`) that recall degraded to before this deploy.
2. **Partition isolation** — clyde-themed query issued under the hermes partition.
3. **Full write→enrich→recall loop** — a fresh sentinel memory committed through the agent
   write path (`memory.commit`), waited for async enrichment, then recalled with a
   zero-keyword-overlap paraphrase.
4. **Enrichment quality spot-check** — inspect LLM-written metadata on the sentinel.

## Results

| # | Partition | Paraphrase query | Target memory | Semantic rank | Keyword baseline |
|---|---|---|---|---|---|
| 1 | clyde | "why do services keep running outdated code after an upgrade" | install.sh sudo-restart gotcha | **2** (rank 1 = closely related stale-binary card) | 0 hits |
| 2 | clyde | "which crate gives Claude Desktop access to agent memories over the network" | koad-os-mcp / Rook bridge cards | **1 & 3** (related cluster) | 0 hits |
| 3 | clyde | "how do we stop fake placeholder vectors from polluting search results" | SearchSemantic fingerprint-limitation decision | **1** | 0 hits |
| 4 | hermes | "SLE directory location filesystem path" | SLE path card | **2** | 0 hits |
| 5 | hermes | "what calendar did we set up for the celebration" | Arandir party calendar | **1, 2, 3** (all three party cards) | 0 hits |
| 6 | clyde→rook | "how often does the harbor beacon blink when visibility is poor" | fresh sentinel: "purple lighthouse … flashes twice every nine seconds during fog" | **1** | 0 hits |

- **Semantic recall: 6/6 targets in top-3** (5/6 at rank 1–2). Keyword baseline: **0/6**.
- **Partition isolation: PASS** — hermes partition returned only hermes cards for a
  clyde-themed query; no cross-agent leakage.
- **Write→enrich→recall loop: PASS** — sentinel committed via `memory.commit`, enriched by the
  worker within seconds (summary "Verification of purple lighthouse's flashing pattern during
  fog", salience 0.8, sensitivity internal, model granite3.3:2b), then recalled at rank 1 by a
  paraphrase sharing zero content words. Test artifacts removed afterward; store back to the
  clean 95-fact corpus.

## Findings

### 🐛 P1 — Partition-key inconsistency between write and search paths (pre-existing, tracked)

`koad-os-mcp memory.commit` writes `source_agent` from `AGENT_NAME` (default `rook`) and encodes
the partition in `domain` (`{AGENT_PARTITION}:{topic}`). But `QdrantTier::search_semantic`
filters facts by `source_agent == partition`, while the L2 fallback (and the recorded canon
card "domain equals partition key") filter by **domain prefix**. Consequence: a memory
committed through MCP under partition X is invisible to semantic search under partition X
whenever `AGENT_NAME != AGENT_PARTITION`. This is exactly how the sentinel test initially
"missed" — the memory was embedded and enriched correctly but filtered out.

**FIXED 2026-07-02** (commit `c648f8b`, Dood decision: domain prefix is canon).
`QdrantTier::search_semantic` now oversamples and filters facts locally by
`domain == partition || domain starts_with "{partition}:"`, mirroring the L2 filter and the
episode-query idiom. Deployed and re-verified live: a fresh MCP-committed sentinel was
recalled at rank 1 by a zero-keyword-overlap paraphrase under its own writing partition, and
production partitions (`clyde_Jupiter_ideans`, `hermes_jupiter_ideans`) regression-tested
clean. Legacy-domain migration **completed 2026-07-02**: the only legacy shape
(`session:clyde:2026-05-09`, 10 facts) was renamed to
`clyde_Jupiter_ideans:session-2026-05-09` in L2 (backup:
`cass.db.pre-domain-migration-20260702`), re-enqueued through the enrichment worker for
Qdrant payload refresh (marker present → no LLM re-run), and verified recalled at rank 1
under the production partition. Zero legacy domains remain in either tier.

Episode gap **closed 2026-07-05**: episodes now carry an explicit `partition` key
(proto field 8, canon `{agent}_{host}_{user}` via `koad_core::utils::partition::partition_key`)
stamped by all write paths (Citadel `session_closed` event, EOW fallback, koad-cli sync),
persisted in SQLite (`episodic_memories.partition`) and the Qdrant episode payload. Both
read filters (`search_semantic_scored`, `query_recent_episodes`) match on the partition
field, with a legacy substring fallback for empty-partition rows. All 46 production rows
backfilled via `backfill_episode_partitions` (backup: `cass.db.pre-episode-partition-20260705`)
and re-upserted to Qdrant — verified 46/46 partitioned in both tiers, zero missing. Live
test `test_episode_recall_by_partition` passes: paraphrase recall under the owning
partition, zero cross-partition leakage. (Note: the original gap description cited
partition-less session ids like `20260606_140451_01834a`; production ids were actually
`SID-{agent}-{uuid}` — agent present, partition string absent. Root cause was the
partition ≠ agent-name mismatch, per the design doc.)

Design: `docs/superpowers/plans/2026-07-05-cass-episode-partition-keying-design.md`.
Implementation: `docs/superpowers/plans/2026-07-05-cass-episode-partition-keying-impl.md`.

### 🟡 Minor — tag quality from granite3.3:2b

The enrichment prompt's "1-5 topical tags" instruction was echoed literally as a tag
(`"1-topical"`) on the sentinel. Summary/salience/volatility/privacy fields are consistently
good; tags need a prompt tweak or a promotion to `phi4-mini:3.8b` (spec already anticipates
this promotion path).

### ✅ Confirmations

- Embedding provenance recorded on every point (`embedding_model: nomic-embed-text`).
- Enrichment markers on 141/141 memories; queue drained; dead-letter empty.
- Semantic ≫ keyword: every paraphrase that substring search scored 0 on was recalled.

## Verdict

**Semantic memory recall is real and agents can use it today** through `memory.search_semantic`
— with one caveat: until the partition-key fix lands, agents whose `AGENT_NAME` differs from
their `AGENT_PARTITION` will not see their own MCP-committed memories in semantic results
(they remain durable in L2 and visible via domain queries).
