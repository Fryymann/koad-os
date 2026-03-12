use anyhow::{Context, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::path::Path;

pub struct IdentityRecord {
    pub id: String,
    pub name: String,
    pub bio: String,
    pub tier: i32,
}

pub struct KoadDB {
    pub pool: Pool<SqliteConnectionManager>,
}

impl KoadDB {
    pub fn new(path: &Path) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path);
        let pool = r2d2::Pool::new(manager).context("Failed to create DB pool.")?;
        Ok(Self { pool })
    }

    pub fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().context("Failed to get DB connection.")
    }

    pub fn get_identity(&self, id: &str) -> Result<Option<IdentityRecord>> {
        let conn = self.get_conn()?;
        let mut stmt =
            conn.prepare("SELECT id, name, bio, tier FROM identities WHERE id = ?1 OR name = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(IdentityRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                bio: row.get(2)?,
                tier: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn verify_role(&self, identity_id: &str, role: &str) -> Result<bool> {
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT count(*) FROM identity_roles WHERE identity_id = ?1 AND role = ?2")?;
        let count: i32 = stmt.query_row(params![identity_id, role], |r| r.get(0))?;
        Ok(count > 0)
    }

    pub fn get_primary_role(&self, identity_id: &str) -> Result<Option<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT role FROM identity_roles WHERE identity_id = ?1")?;
        let rows = stmt.query_map(params![identity_id], |row| row.get::<_, String>(0))?;

        let mut roles = Vec::new();
        for r in rows {
            roles.push(r?);
        }

        if roles.is_empty() {
            return Ok(None);
        }

        let standard_tiers = vec!["admin", "pm", "officer", "crew"];
        for tier in standard_tiers {
            if roles.contains(&tier.to_string()) {
                return Ok(Some(tier.to_string()));
            }
        }

        Ok(Some(roles[0].clone()))
    }

    pub fn remember(
        &self,
        category: &str,
        text: &str,
        tags: Option<String>,
        _tier: i32,
        agent: &str,
    ) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO knowledge (category, content, tags, timestamp, origin_agent) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![category, text, tags, chrono::Utc::now().to_rfc3339(), agent],
        )?;
        Ok(())
    }

    pub fn query_knowledge(
        &self,
        term: &str,
        limit: usize,
        agent_filter: Option<&str>,
    ) -> Result<Vec<(String, String, String, String)>> {
        let conn = self.get_conn()?;
        let search_pattern = format!("%{}%", term);
        let limit_i64 = limit as i64;

        if let Some(agent) = agent_filter {
            let mut stmt = conn.prepare("SELECT category, content, tags, origin_agent FROM knowledge WHERE (content LIKE ?1 OR tags LIKE ?1 OR origin_agent = ?2) AND (origin_agent = ?2 OR ?1 = '%%') ORDER BY timestamp DESC LIMIT ?3")?;
            let rows = stmt.query_map(params![search_pattern, agent, limit_i64], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2).unwrap_or_else(|_| "".to_string()),
                    row.get(3)?,
                ))
            })?;
            let mut results = Vec::new();
            for r in rows {
                results.push(r?);
            }
            Ok(results)
        } else {
            let mut stmt = conn.prepare("SELECT category, content, tags, origin_agent FROM knowledge WHERE content LIKE ?1 OR tags LIKE ?1 ORDER BY timestamp DESC LIMIT ?2")?;
            let rows = stmt.query_map(params![search_pattern, limit_i64], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2).unwrap_or_else(|_| "".to_string()),
                    row.get(3)?,
                ))
            })?;
            let mut results = Vec::new();
            for r in rows {
                results.push(r?);
            }
            Ok(results)
        }
    }
}
