use anyhow::{Context, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::path::Path;

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

    pub fn remember(&self, category: &str, text: &str, tags: Option<String>, tier: i32) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO knowledge (category, content, tags, tier, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![category, text, tags, tier, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn query_knowledge(&self, term: &str, limit: usize) -> Result<Vec<(String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT category, content, tags FROM knowledge WHERE content LIKE ?1 OR tags LIKE ?1 ORDER BY timestamp DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![format!("%{}%", term), limit], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }
}
