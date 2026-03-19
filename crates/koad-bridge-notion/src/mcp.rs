use crate::client::NotionClient;
use anyhow::{Context, Result};
use rusqlite::Connection;
use serde_json::Value;
use std::path::PathBuf;

pub struct NotionMcpProxy {
    client: NotionClient,
    db_path: PathBuf,
}

impl NotionMcpProxy {
    pub fn new(api_key: String, db_path: PathBuf) -> Result<Self> {
        let client = NotionClient::new(api_key)?;
        Ok(Self { client, db_path })
    }

    /// Optimized: Try local cache first for page content.
    pub async fn get_page_content(&self, page_id: &str, force_refresh: bool) -> Result<String> {
        if !force_refresh {
            if let Ok(conn) = Connection::open(&self.db_path) {
                let mut stmt = conn.prepare("SELECT content_md FROM pages WHERE page_id = ? AND is_deleted = 0")?;
                let cached: Option<String> = stmt.query_row([page_id], |row| row.get(0)).ok();
                if let Some(content) = cached {
                    return Ok(format!("(Cached) {}", content));
                }
            }
        }

        // Fallback to API + Efficient Parser
        let content = self.client.get_page_content_markdown(page_id).await?;
        
        // Background: Upsert to cache if possible (simplified for now)
        let _ = self.cache_page(page_id, &content);

        Ok(content)
    }

    fn cache_page(&self, page_id: &str, content: &str) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "UPDATE pages SET content_md = ?, synced_at = datetime('now') WHERE page_id = ?",
            [content, page_id],
        )?;
        Ok(())
    }

    fn save_page(&self, page_id: &str, source_id: &str, title: &str, content: &str, properties: &Value) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        
        // Ensure source exists first
        conn.execute(
            "INSERT INTO sync_sources (source_id, source_name, sync_status)
             VALUES (?, ?, 'pending')
             ON CONFLICT(source_id) DO NOTHING",
            [source_id, "Unknown Database"],
        )?;

        let props_json = serde_json::to_string(properties)?;

        conn.execute(
            "INSERT INTO pages (page_id, source_id, title, content_md, properties_json, created_at, updated_at, synced_at) 
             VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'), datetime('now'))
             ON CONFLICT(page_id) DO UPDATE SET 
                title = excluded.title,
                content_md = excluded.content_md,
                properties_json = excluded.properties_json,
                updated_at = excluded.updated_at,
                synced_at = excluded.synced_at",
            [page_id, source_id, title, content, &props_json],
        )?;
        Ok(())
    }

    pub async fn sync_database(&self, database_id: &str) -> Result<usize> {
        let pages = self.client.query_database(database_id).await?;
        let mut count = 0;

        for (id, title, properties) in pages {
            let content = self.client.get_page_content_markdown(&id).await?;
            self.save_page(&id, database_id, &title, &content, &properties)?;
            count += 1;
        }

        // Update sync_sources table
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT INTO sync_sources (source_id, source_name, last_sync_at, sync_status)
             VALUES (?, ?, datetime('now'), 'success')
             ON CONFLICT(source_id) DO UPDATE SET 
                last_sync_at = excluded.last_sync_at,
                sync_status = excluded.sync_status",
            [database_id, "Synced Database"], // Name could be fetched from DB meta if needed
        )?;

        Ok(count)
    }

    pub async fn update_status(&self, page_id: &str, status: &str) -> Result<()> {
        // 1. Discovery: Get property type from local cache
        let conn = Connection::open(&self.db_path)?;
        let props_json: String = conn.query_row(
            "SELECT properties_json FROM pages WHERE page_id = ?",
            [page_id],
            |row| row.get(0),
        )?;
        let props: Value = serde_json::from_str(&props_json)?;

        // Try 'Status' first, then 'Implemented' (as a fallback or specific mapping)
        let (prop_name, value) = if let Some(p) = props.get("Status") {
            ("Status", serde_json::json!({ "status": { "name": status } }))
        } else if let Some(p) = props.get("Implemented") {
            let checked = status.to_lowercase() == "done" || status.to_lowercase() == "completed" || status.to_lowercase() == "true";
            ("Implemented", serde_json::json!({ "checkbox": checked }))
        } else {
            // Fallback to assuming 'Status' is a status type even if not in cache
            ("Status", serde_json::json!({ "status": { "name": status } }))
        };

        // 2. Remote Update
        self.client.update_page_property(page_id, prop_name, value).await?;

        // 3. Local Update (Timestamp only for now, next sync will refresh metadata)
        conn.execute(
            "UPDATE pages SET synced_at = datetime('now') WHERE page_id = ?",
            [page_id],
        )?;
        
        Ok(())
    }

    pub fn export_database(&self, database_id: &str, output_dir: &std::path::Path) -> Result<usize> {
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT title, content_md FROM pages WHERE source_id = ? AND is_deleted = 0")?;
        let mut rows = stmt.query([database_id])?;
        let mut count = 0;

        while let Some(row) = rows.next()? {
            let title: String = row.get(0)?;
            let content: String = row.get(1)?;
            
            // Sanitize title for filename
            let sanitized_title = title
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
                .collect::<String>()
                .replace(" ", "_")
                .to_lowercase();
            
            let file_path = output_dir.join(format!("{}.md", sanitized_title));
            std::fs::write(&file_path, content)?;
            count += 1;
        }

        Ok(count)
    }

    /// Optimized: List databases with token-efficient filtering.
    pub async fn list_databases(&self) -> Result<Value> {
        // This could also be cached in sync_sources table
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT source_id, source_name, last_sync_at FROM sync_sources")?;
        let sources: Vec<Value> = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "last_sync": row.get::<_, Option<String>>(2)?
            }))
        })?.filter_map(Result::ok).collect();

        Ok(serde_json::json!(sources))
    }
}
