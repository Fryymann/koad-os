//! Re-embed all memories into Qdrant with the real embedding model, and
//! enqueue enrichment for rows that lack LLM metadata.
//!
//! Usage: backfill_embeddings --db <path> [--qdrant-url <url>] [--apply] [--enqueue]
//! Default is dry-run: reports row counts, writes nothing.
//! --enqueue XADDs unenriched rows to cass:enrichment so the live worker
//! backfills LLM metadata (requires KOADOS_HOME for Redis UDS). It works
//! standalone — `--enqueue` without `--apply` performs no collection drop
//! and no re-embedding; use it for post-outage recovery of dropped
//! enrichment stream entries.
//!
//! DESTRUCTIVE when --apply: drops and recreates both Qdrant collections
//! (fingerprint-vector points are unrecoverable garbage; L2 SQLite is the
//! source of truth and is never modified by this tool).

use anyhow::{Context, Result};
use koad_cass::services::enrichment::is_enriched;
use koad_cass::storage::QdrantTier;
use koad_intelligence::router::InferenceRouter;
use koad_proto::cass::v1::{EpisodicMemory, FactCard};
use rusqlite::Connection;
use std::sync::Arc;

const BATCH: usize = 32;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let db = arg(&args, "--db").ok_or_else(|| anyhow::anyhow!("--db <path> is required"))?;
    let qdrant_url = arg(&args, "--qdrant-url")
        .or_else(|| std::env::var("KOADOS_URL_QDRANT").ok())
        .unwrap_or_else(|| "http://127.0.0.1:6334".to_string());
    let apply = args.iter().any(|a| a == "--apply");
    let enqueue = args.iter().any(|a| a == "--enqueue");

    let conn = Connection::open(&db).with_context(|| format!("opening sqlite db {db}"))?;
    let facts = load_facts(&conn).context("loading fact_cards")?;
    let episodes = load_episodes(&conn).context("loading episodic_memories")?;
    let unenriched = facts.iter().filter(|f| !is_enriched(&f.metadata)).count()
        + episodes
            .iter()
            .filter(|e| !is_enriched(&e.metadata))
            .count();

    println!("facts: {}", facts.len());
    println!("episodes: {}", episodes.len());
    println!("rows missing LLM enrichment: {}", unenriched);

    if !apply && !enqueue {
        println!("dry-run: no writes. Re-run with --apply to drop collections and re-embed, and/or --enqueue to queue unenriched rows.");
        return Ok(());
    }

    if apply {
        let intelligence = Arc::new(InferenceRouter::new_default()?);
        let qdrant = QdrantTier::new(&qdrant_url, Some(intelligence))
            .await
            .context("Qdrant unreachable")?;

        println!("recreating collections (drops fingerprint vectors)...");
        qdrant.recreate_collections().await.context(
            "recreate failed — is Ollama running? (dimension probe requires the embedding model)",
        )?;

        let mut done = 0usize;
        for chunk in facts.chunks(BATCH) {
            qdrant
                .commit_facts(chunk.to_vec())
                .await
                .with_context(|| format!("embedding facts batch ending at {done}"))?;
            done += chunk.len();
            println!("facts embedded: {}/{}", done, facts.len());
        }

        use koad_cass::storage::MemoryTier;
        for (i, ep) in episodes.iter().enumerate() {
            qdrant
                .record_episode(ep.clone())
                .await
                .with_context(|| format!("embedding episode {}", ep.session_id))?;
            if (i + 1) % BATCH == 0 || i + 1 == episodes.len() {
                println!("episodes embedded: {}/{}", i + 1, episodes.len());
            }
        }
    }

    if enqueue && unenriched > 0 {
        println!(
            "enqueuing {} unenriched rows for the live worker...",
            unenriched
        );
        let home = std::env::var("KOADOS_HOME")
            .or_else(|_| std::env::var("KOAD_HOME"))
            .context("--enqueue requires KOADOS_HOME (or KOAD_HOME) for the Redis socket")?;
        let redis = koad_core::utils::redis::RedisClient::new(&home, false).await?;
        let tier = koad_cass::storage::RedisTier::new(redis.pool.clone());
        for f in facts.iter().filter(|f| !is_enriched(&f.metadata)) {
            tier.enqueue_enrichment("fact", &f.id, &f.source_agent)
                .await?;
        }
        for e in episodes.iter().filter(|e| !is_enriched(&e.metadata)) {
            tier.enqueue_enrichment("episode", &e.session_id, &e.session_id)
                .await?;
        }
        println!("enqueued. The live worker will enrich metadata in the background.");
    } else if unenriched > 0 {
        // apply-without-enqueue: embeddings written, LLM metadata still missing.
        println!(
            "note: {} rows lack LLM metadata. Re-run with --enqueue (services running) \
to have the live worker backfill it; embeddings are present either way.",
            unenriched
        );
    }
    println!("done.");
    Ok(())
}

fn load_facts(conn: &Connection) -> Result<Vec<FactCard>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_agent, session_id, domain, content, confidence, tags, metadata_json
         FROM fact_cards",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(FactCard {
            id: row.get(0)?,
            source_agent: row.get(1)?,
            session_id: row.get(2)?,
            domain: row.get(3)?,
            content: row.get(4)?,
            confidence: row.get::<_, f64>(5)? as f32,
            tags: row
                .get::<_, String>(6)?
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            created_at: None,
            metadata: row
                .get::<_, Option<String>>(7)?
                .and_then(|s| serde_json::from_str(&s).ok()),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn load_episodes(conn: &Connection) -> Result<Vec<EpisodicMemory>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, project_path, summary, turn_count, timestamp, task_ids, metadata_json
         FROM episodic_memories",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(EpisodicMemory {
            session_id: row.get(0)?,
            project_path: row.get(1)?,
            summary: row.get(2)?,
            turn_count: row.get::<_, i64>(3)? as u32,
            timestamp: None,
            task_ids: row
                .get::<_, String>(5)?
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            metadata: row
                .get::<_, Option<String>>(6)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            partition: String::new(),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
