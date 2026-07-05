//! Backfill partition keys for episodic_memories rows lacking one, then
//! optionally re-upsert episode points to Qdrant with the new payload field.
//! Usage: backfill_episode_partitions --db <path> [--qdrant <url>] [--apply]
//! Default is dry-run: reports what would change, writes nothing.

use anyhow::{Context, Result};
use rusqlite::Connection;

fn arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn derive_agent(session_id: &str) -> Option<String> {
    // SID-{agent}-{uuid}
    if let Some(rest) = session_id.strip_prefix("SID-") {
        if let Some((agent, _)) = rest.split_once('-') {
            if !agent.is_empty() {
                return Some(agent.to_string());
            }
        }
    }
    // legacy: {agent}-{date...}
    if let Some((agent, _)) = session_id.split_once('-') {
        if !agent.is_empty() && agent.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Some(agent.to_string());
        }
    }
    None
}

/// Load all partitioned episodes with timestamp and metadata preserved, so the
/// Qdrant re-upsert payload loses nothing (SQLite row is source of truth).
fn load_partitioned_episodes(
    conn: &Connection,
) -> Result<Vec<koad_proto::cass::v1::EpisodicMemory>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, project_path, summary, turn_count, timestamp, task_ids, metadata_json, partition
         FROM episodic_memories WHERE partition != ''",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(koad_proto::cass::v1::EpisodicMemory {
            session_id: row.get(0)?,
            project_path: row.get(1)?,
            summary: row.get(2)?,
            turn_count: row.get::<_, i64>(3)? as u32,
            // Column is RFC3339 TEXT; make_episode_payload reads .seconds.
            timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .ok()
                .map(|dt| prost_types::Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }),
            task_ids: row
                .get::<_, String>(5)?
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            metadata: row
                .get::<_, Option<String>>(6)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            partition: row.get(7)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let db = arg(&args, "--db").ok_or_else(|| anyhow::anyhow!("--db <path> is required"))?;
    let qdrant_url = arg(&args, "--qdrant");
    let apply = args.iter().any(|a| a == "--apply");

    let conn = Connection::open(&db)?;
    // Idempotent: ensure the column exists even on an older DB.
    let _ = conn.execute(
        "ALTER TABLE episodic_memories ADD COLUMN partition TEXT NOT NULL DEFAULT ''",
        [],
    );

    let rows: Vec<String> = {
        let mut stmt =
            conn.prepare("SELECT session_id FROM episodic_memories WHERE partition = ''")?;
        let mapped = stmt.query_map([], |r| r.get::<_, String>(0))?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()?
    };

    let mut updated = 0usize;
    let mut skipped: Vec<String> = vec![];
    for sid in &rows {
        match derive_agent(sid) {
            Some(agent) => {
                let partition = koad_core::utils::partition::partition_key(&agent);
                if apply {
                    conn.execute(
                        "UPDATE episodic_memories SET partition = ?1 WHERE session_id = ?2",
                        rusqlite::params![partition, sid],
                    )?;
                }
                println!("{} {} -> {}", if apply { "SET " } else { "PLAN" }, sid, partition);
                updated += 1;
            }
            None => skipped.push(sid.clone()),
        }
    }
    println!(
        "{}: {} rows partitioned, {} underivable{}",
        if apply { "APPLIED" } else { "DRY-RUN" },
        updated,
        skipped.len(),
        if skipped.is_empty() { String::new() } else { format!(": {:?}", skipped) }
    );

    // Qdrant re-upsert phase: push partitioned rows back so payloads gain the field.
    if let (Some(url), true) = (&qdrant_url, apply) {
        use koad_cass::storage::{MemoryTier, QdrantTier};
        use koad_intelligence::router::InferenceRouter;
        use std::sync::Arc;

        let intelligence = Arc::new(InferenceRouter::new_default()?);
        let qdrant = QdrantTier::new(url, Some(intelligence))
            .await
            .context("Qdrant unreachable")?;

        // Loaded AFTER the SQLite update so partitions reflect this run.
        let episodes = load_partitioned_episodes(&conn).context("loading episodic_memories")?;
        let mut n = 0usize;
        for ep in &episodes {
            qdrant
                .record_episode(ep.clone())
                .await
                .with_context(|| format!("re-upserting episode {}", ep.session_id))?;
            n += 1;
        }
        println!("QDRANT: re-upserted {} episode points to {}", n, url);
    } else if qdrant_url.is_some() {
        println!("dry-run: skipping Qdrant re-upsert (requires --apply).");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::derive_agent;

    #[test]
    fn derives_sid_and_legacy_formats() {
        assert_eq!(derive_agent("SID-clyde-01d0aefa").as_deref(), Some("clyde"));
        assert_eq!(derive_agent("SID-hermes-025daf6d").as_deref(), Some("hermes"));
        assert_eq!(derive_agent("clyde-2026-05-09").as_deref(), Some("clyde"));
        assert_eq!(derive_agent("20260606_140451_01834a"), None);
    }
}
