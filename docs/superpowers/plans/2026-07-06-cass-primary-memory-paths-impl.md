# CASS-Primary Memory Paths — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development or superpowers:executing-plans. Checkbox steps.
>
> **Methodology: SPEC-DRIVEN.** Spec → implementation → verification. Tests verify spec after code.

**Goal:** Route agent memory operations through CASS as the primary surface — `commit_knowledge` writes facts to CASS (gaining metadata + semantic enrichment), `koad intel query` recalls semantically from CASS — with the local `koad.db` demoted to fallback.

**Authority:** Dood ruling 2026-07-06 ("We should route memory paths through CASS"), resolving the `citadel-io` branch divergence. This is a fresh implementation of that branch's intent on current nightly (the branch predates FactCard metadata/partition fields); after landing, `citadel-io` is superseded and can be deleted.

**Architecture:** AdminService gains `cass_grpc_addr` (from `config.network.cass_grpc_addr`); `commit_knowledge` builds a `FactCard` with domain `{partition_key(agent)}:{category}` (partition canon = domain prefix, so committed knowledge is semantically recallable under the agent's partition) and calls `MemoryService::CommitFact` — entering the L1/L2 + enrichment/embedding pipeline. CASS unreachable → fallback to the existing `koad_db.remember` with a degraded-path warning. `koad intel query` calls `SearchSemantic` first (partition = `partition_key(agent_name)`), then prints the local archive.

---

### Task 1: commit_knowledge → CASS

**Files:**
- Modify: `crates/koad-citadel/src/services/admin.rs` (struct, ctor, commit_knowledge :95-130)
- Modify: `crates/koad-citadel/src/kernel.rs:187` (ctor call)
- Check: `crates/koad-citadel/Cargo.toml` (needs `uuid` with v4 + `prost-types`; add if missing, workspace-inherited)

**Spec:**
1. `AdminService` gains `cass_grpc_addr: String`; ctor `new(shutdown_tx, koad_db, cass_grpc_addr)`; kernel passes `config.network.cass_grpc_addr.clone()`.
2. Domain canon helper (pure, unit-testable, in admin.rs):
```rust
/// Domain for committed knowledge: "{partition}:{category}" so facts are
/// recallable under the committing agent's partition (partition canon = domain prefix).
fn knowledge_domain(agent: &str, category: &str) -> String {
    format!("{}:{}", koad_core::utils::partition::partition_key(agent), category)
}
```
3. `commit_knowledge`: keep agent derivation from session_id. Build FactCard:
```rust
        let now = chrono::Utc::now();
        let fact = koad_proto::cass::v1::FactCard {
            id: uuid::Uuid::new_v4().to_string(),
            source_agent: agent_name.clone(),
            session_id: req.session_id.clone(),
            domain: knowledge_domain(&agent_name, &req.category),
            content: req.content.clone(),
            confidence: 1.0,
            tags: req.tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            created_at: Some(prost_types::Timestamp { seconds: now.timestamp(), nanos: 0 }),
            metadata: None, // enrichment worker fills
        };
```
4. Primary: `MemoryServiceClient::connect(self.cass_grpc_addr.clone()).await` → `commit_fact(fact)`. Success message: `"Knowledge committed to CASS (domain: {domain})"`.
5. Fallback: on connect/commit error, `warn!` + existing `self.koad_db.remember(...)` path; success message notes `"(CASS unreachable — committed to local archive)"`. Both fail → `Status::internal`.
6. Unit test for `knowledge_domain`: asserts `starts_with(&format!("{}_", agent))` and `ends_with(":learning")` for agent "clyde", category "learning".

**Verify:** `cargo check --workspace` clean; `cargo test -p koad-citadel 2>&1 | grep 'test result' | grep -v ' 0 failed' | wc -l` → 0.

**Commit:** `feat(citadel): commit_knowledge routes to CASS with local fallback`

---

### Task 2: koad intel query → CASS-first

**Files:**
- Modify: `crates/koad-cli/src/handlers/intel.rs` (Query arm, :22-45)

**Spec:**
1. Before the local query, attempt CASS semantic recall:
```rust
            println!("\n\x1b[1m--- INTEL: Knowledge Query [{}] ---\x1b[0m", term);
            match koad_proto::cass::v1::memory_service_client::MemoryServiceClient::connect(
                config.network.cass_grpc_addr.clone(),
            )
            .await
            {
                Ok(mut cass) => {
                    let query = koad_proto::cass::v1::SemanticQuery {
                        query: term.clone(),
                        partition: koad_core::utils::partition::partition_key(agent_name),
                        limit: limit as u32,
                        min_score: 0.0, // server-side threshold verdict applies
                    };
                    match cass.search_semantic(query).await {
                        Ok(resp) => {
                            let facts = resp.into_inner().facts;
                            if facts.is_empty() {
                                println!("  (CASS: no semantic matches)");
                            }
                            for f in facts {
                                println!("[cass:{}] [{}] {}", f.domain, f.source_agent, f.content);
                            }
                        }
                        Err(e) => println!("  (CASS search failed: {} — local archive only)", e),
                    }
                }
                Err(_) => println!("  (CASS offline — local archive only)"),
            }
```
(Adjust response accessor to the actual `FactResponse` field name; check `proto/cass.proto`.)
2. Existing local loop stays below it, prefixed with a `--- Local Archive ---` subsection line.
3. Existing `tags` filter applies to local results only (CASS results already ranked; don't silently drop them).

**Verify:** `cargo check --workspace` clean; crate tests green.

**Commit:** `feat(cli): intel query recalls semantically from CASS first`

---

### Task 3: Deploy + live verify + cleanup — 🚦 sudo restart gate

1. Push nightly; `./install.sh --update`.
2. **Ian's terminal:** `sudo systemctl restart koad-citadel.service` (cass unchanged — only citadel + CLI binaries).
3. Live loop: `koad intel remember` a sentinel learning → confirm fact lands in cass.db with `{partition}:{category}` domain → wait for enrichment → `koad intel query` paraphrase recalls it via CASS section.
4. Delete superseded branch: `git push origin --delete citadel-io` (intent landed; Dood ruling recorded here).
5. SITREP: record CASS-primary landing; drop the decision item.

**Rollback:** each task one commit; revert restores koad_db paths (no schema changes anywhere).
