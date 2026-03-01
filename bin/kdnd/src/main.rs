use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use futures::future::join_all;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};

#[derive(Parser)]
#[command(name = "kdnd")]
#[command(about = "KoadOS D&D Sync Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync character data from D&D Beyond to Notion and local DB
    Sync {
        #[arg(short, long)]
        character_id: Option<String>,
        #[arg(short, long)]
        json: bool,
        #[arg(short, long)]
        force: bool,
    },
    /// Dispatch a sync task to the Koad Daemon
    Dispatch {
        #[arg(short, long)]
        character_id: Option<String>,
    }
}

#[derive(Deserialize, Debug, Clone)]
struct NotionPage {
    id: String,
    properties: serde_json::Value,
}

struct NotionClient {
    client: reqwest::Client,
    token: String,
}

struct LocalDb {
    path: String,
}

struct CharacterContext {
    campaign_name: Option<String>,
    campaign_id: Option<String>,
    party_name: Option<String>,
    party_id: Option<String>,
}

impl LocalDb {
    fn new(path: String) -> Self {
        Self { path }
    }

    fn get_last_json(&self, id: &str) -> Result<Option<String>> {
        let conn = Connection::open(&self.path)?;
        let mut stmt = conn.prepare("SELECT full_json FROM character_syncs WHERE dnd_beyond_id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let json: String = row.get(0)?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    fn save_sync(&self, id: &str, name: &str, data: &serde_json::Value, ctx: &CharacterContext) -> Result<()> {
        let conn = Connection::open(&self.path)?;
        let json_str = serde_json::to_string(data)?;
        conn.execute(
            "INSERT OR REPLACE INTO character_syncs (dnd_beyond_id, name, full_json, campaign_id, campaign_name, party_id, party_name, last_sync) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
            params![id, name, json_str, ctx.campaign_id, ctx.campaign_name, ctx.party_id, ctx.party_name],
        )?;
        Ok(())
    }
}

impl NotionClient {
    fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }

    async fn get_page(&self, page_id: &str) -> Result<serde_json::Value> {
        let url = format!("https://api.notion.com/v1/pages/{}", page_id);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.token))?);
        headers.insert("Notion-Version", HeaderValue::from_static("2022-06-28"));

        let res = self.client.get(&url).headers(headers).send().await?.json::<serde_json::Value>().await?;
        Ok(res)
    }

    async fn query_database(&self, db_id: &str) -> Result<Vec<NotionPage>> {
        let url = format!("https://api.notion.com/v1/databases/{}/query", db_id);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.token))?);
        headers.insert("Notion-Version", HeaderValue::from_static("2022-06-28"));

        let res = self.client.post(&url)
            .headers(headers)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(results) = res["results"].as_array() {
            let pages = results.iter()
                .filter_map(|v| serde_json::from_value::<NotionPage>(v.clone()).ok())
                .collect();
            Ok(pages)
        } else {
            Ok(vec![])
        }
    }

    async fn search_pages(&self, query: &str) -> Result<Vec<serde_json::Value>> {
        let url = "https://api.notion.com/v1/search";
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.token))?);
        headers.insert("Notion-Version", HeaderValue::from_static("2022-06-28"));

        let res = self.client.post(url)
            .headers(headers)
            .json(&json!({
                "query": query,
                "filter": { "property": "object", "value": "page" }
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(res["results"].as_array().cloned().unwrap_or_default())
    }

    async fn get_block_children(&self, block_id: &str) -> Result<Vec<serde_json::Value>> {
        let url = format!("https://api.notion.com/v1/blocks/{}/children", block_id);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.token))?);
        headers.insert("Notion-Version", HeaderValue::from_static("2022-06-28"));

        let res = self.client.get(&url)
            .headers(headers)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(res["results"].as_array().cloned().unwrap_or_default())
    }

    async fn delete_block(&self, block_id: &str) -> Result<()> {
        let url = format!("https://api.notion.com/v1/blocks/{}", block_id);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.token))?);
        headers.insert("Notion-Version", HeaderValue::from_static("2022-06-28"));

        self.client.delete(&url).headers(headers).send().await?;
        Ok(())
    }

    async fn append_block_children(&self, block_id: &str, children: Vec<serde_json::Value>) -> Result<()> {
        let url = format!("https://api.notion.com/v1/blocks/{}/children", block_id);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.token))?);
        headers.insert("Notion-Version", HeaderValue::from_static("2022-06-28"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        self.client.patch(&url)
            .headers(headers)
            .json(&json!({ "children": children }))
            .send()
            .await?;
        Ok(())
    }

    async fn update_page(&self, page_id: &str, properties: serde_json::Value) -> Result<()> {
        let url = format!("https://api.notion.com/v1/pages/{}", page_id);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.token))?);
        headers.insert("Notion-Version", HeaderValue::from_static("2022-06-28"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let res = self.client.patch(&url)
            .headers(headers)
            .json(&json!({ "properties": properties }))
            .send()
            .await?;
        
        if !res.status().is_success() {
            let err_text = res.text().await?;
            anyhow::bail!("Notion Update Error: {}", err_text);
        }
        Ok(())
    }
}

async fn fetch_dnd_beyond_char(char_id: &str) -> Result<serde_json::Value> {
    let url = format!("https://character-service.dndbeyond.com/character/v5/character/{}", char_id);
    let res = reqwest::get(&url).await?.json::<serde_json::Value>().await?;
    Ok(res["data"].clone())
}

#[derive(Serialize)]
struct SyncResult {
    character: String,
    status: String,
    details: Option<String>,
}

fn chunk_text(text: &str, size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = text;
    while !current.is_empty() {
        let (chunk, rest) = if current.len() > size {
            current.split_at(size)
        } else {
            (current, "")
        };
        chunks.push(chunk.to_string());
        current = rest;
    }
    chunks
}

fn extract_id_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("").to_string()
}

async fn get_name_from_relation(notion: &NotionClient, cache: Arc<Mutex<HashMap<String, String>>>, id: &str) -> Option<String> {
    {
        let cache_lock = cache.lock().unwrap();
        if let Some(name) = cache_lock.get(id) {
            return Some(name.clone());
        }
    }

    if let Ok(page) = notion.get_page(id).await {
        let name = page["properties"]["Name"]["title"][0]["plain_text"].as_str()
            .or(page["properties"]["Name"]["rich_text"][0]["plain_text"].as_str())
            .or(page["properties"]["title"]["title"][0]["plain_text"].as_str())
            .map(|s| s.to_string());
        
        if let Some(n) = &name {
            let mut cache_lock = cache.lock().unwrap();
            cache_lock.insert(id.to_string(), n.clone());
        }
        return name;
    }
    None
}

async fn sync_character(
    notion: Arc<NotionClient>, 
    db: Arc<LocalDb>, 
    page: NotionPage, 
    force: bool,
    name_cache: Arc<Mutex<HashMap<String, String>>>
) -> SyncResult {
    let props = &page.properties;
    let char_name = props["Name"]["title"][0]["plain_text"].as_str().unwrap_or("Unknown").to_string();
    let dnd_url = props["DnD Beyond"]["url"].as_str().unwrap_or("");

    if dnd_url.is_empty() {
        return SyncResult { character: char_name, status: "Skipped".into(), details: Some("No URL".into()) };
    }

    let id = extract_id_from_url(dnd_url);
    if id.is_empty() {
        return SyncResult { character: char_name, status: "Error".into(), details: Some("Invalid URL".into()) };
    }

    // RESOLVE CONTEXT (Campaign/Party)
    let campaign_id = props["🛡️ Campaigns"]["relation"][0]["id"].as_str();
    let party_id = props["Player Party"]["relation"][0]["id"].as_str();

    let mut ctx = CharacterContext {
        campaign_name: None,
        campaign_id: campaign_id.map(|s| s.to_string()),
        party_name: None,
        party_id: party_id.map(|s| s.to_string()),
    };

    if let Some(cid) = campaign_id {
        ctx.campaign_name = get_name_from_relation(&notion, Arc::clone(&name_cache), cid).await;
    }
    if let Some(pid) = party_id {
        ctx.party_name = get_name_from_relation(&notion, Arc::clone(&name_cache), pid).await;
    }

    let data = match fetch_dnd_beyond_char(&id).await {
        Ok(d) => d,
        Err(e) => return SyncResult { character: char_name, status: "Error".into(), details: Some(format!("Fetch failed: {}", e)) },
    };
    
    if data.is_null() {
        return SyncResult { character: char_name, status: "Skipped".into(), details: Some("Private character".into()) };
    }

    // DELTA CHECK
    let current_json_str = serde_json::to_string(&data).unwrap_or_default();
    let last_json_str = db.get_last_json(&id).unwrap_or(None);
    
    let mut updated_notion = false;
    if force || last_json_str.as_ref() != Some(&current_json_str) {
        if let Err(e) = db.save_sync(&id, &char_name, &data, &ctx) {
            eprintln!("  Warning: Failed to save to local DB for {}: {}", char_name, e);
        }

        let levels: i64 = data["classes"].as_array().unwrap_or(&vec![]).iter()
            .map(|c| c["level"].as_i64().unwrap_or(0))
            .sum();

        let mut stats = [0i64; 7];
        if let Some(base_stats) = data["stats"].as_array() {
            for s in base_stats {
                if let (Some(stat_id), Some(val)) = (s["id"].as_u64(), s["value"].as_i64()) {
                    if stat_id >= 1 && stat_id <= 6 {
                        stats[stat_id as usize] = val;
                    }
                }
            }
        }

        let current_hp = data["baseHitPoints"].as_i64().unwrap_or(0) 
            - data["removedHitPoints"].as_i64().unwrap_or(0)
            + data["temporaryHitPoints"].as_i64().unwrap_or(0);

        let mut class_list = Vec::new();
        if let Some(classes) = data["classes"].as_array() {
            for c in classes {
                if let Some(name) = c["definition"]["name"].as_str() {
                    class_list.push(json!({ "name": name }));
                }
            }
        }

        let mut updates = json!({
            "Level": { "number": levels },
            "HP": { "number": current_hp },
            "STR": { "number": stats[1] },
            "DEX": { "number": stats[2] },
            "CON": { "number": stats[3] },
            "INT": { "number": stats[4] },
            "WIS": { "number": stats[5] },
            "CHA": { "number": stats[6] },
            "Class": { "multi_select": class_list }
        });

        if let Some(species_name) = data["race"]["fullName"].as_str() {
            updates["Species"] = json!({ "select": { "name": species_name } });
        }

        if let Err(e) = notion.update_page(&page.id, updates).await {
            return SyncResult { character: char_name, status: "Error".into(), details: Some(format!("Notion update failed: {}", e)) };
        }
        updated_notion = true;

        let sync_page_name = format!("{} DnD Beyond Sync", char_name);
        if let Ok(search_results) = notion.search_pages(&sync_page_name).await {
            for sp in search_results {
                let sp_title = sp["properties"]["title"]["title"][0]["plain_text"].as_str().unwrap_or("");
                if sp_title == sync_page_name {
                    let sp_id = sp["id"].as_str().unwrap_or("");
                    if !sp_id.is_empty() {
                        if let Ok(children) = notion.get_block_children(sp_id).await {
                            for child in children {
                                let _ = notion.delete_block(child["id"].as_str().unwrap_or("")).await;
                            }
                        }
                        let formatted_json = serde_json::to_string_pretty(&data).unwrap_or_default();
                        let chunks = chunk_text(&formatted_json, 2000);
                        let rich_text: Vec<serde_json::Value> = chunks.iter().take(100).map(|c| json!({
                            "type": "text",
                            "text": { "content": c }
                        })).collect();

                        let _ = notion.append_block_children(sp_id, vec![json!({
                            "object": "block",
                            "type": "code",
                            "code": {
                                "rich_text": rich_text,
                                "language": "json"
                            }
                        })]).await;
                    }
                }
            }
        }
    } else {
        let _ = db.save_sync(&id, &char_name, &data, &ctx);
    }

    SyncResult { 
        character: char_name, 
        status: if updated_notion { "Success".into() } else { "Cached".into() }, 
        details: Some(format!("Campaign: {}, Party: {}", ctx.campaign_name.unwrap_or("None".into()), ctx.party_name.unwrap_or("None".into()))) 
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let notion_token = env::var("NOTION_PAT").context("NOTION_PAT env var not set")?;
    let db_id = "2e4fe8ec-ae8f-8004-a0ca-c477fa219bc4"; 
    let db_path = format!("{}/.koad-os/data/dnd_syncs.db", env::var("HOME")?);

    let notion = Arc::new(NotionClient::new(notion_token));
    let db = Arc::new(LocalDb::new(db_path));
    let name_cache = Arc::new(Mutex::new(HashMap::new()));

    match &cli.command {
        Commands::Dispatch { character_id } => {
            let mut cmd = "kdnd sync".to_string();
            if let Some(id) = character_id {
                cmd.push_str(&format!(" --character-id {}", id));
            }
            let status = std::process::Command::new("koad")
                .arg("dispatch")
                .arg(cmd)
                .status()?;
            if !status.success() {
                anyhow::bail!("Failed to dispatch command.");
            }
        },
        Commands::Sync { character_id, json, force } => {
            if !*json { println!("Starting Intelligent D&D Sync..."); }
            let pages = notion.query_database(db_id).await?;
            let mut tasks = Vec::new();
            for page in pages {
                let dnd_url = page.properties["DnD Beyond"]["url"].as_str().unwrap_or("");
                let id = extract_id_from_url(dnd_url);
                if let Some(target_id) = character_id {
                    if id != *target_id { continue; }
                }
                let notion_clone = Arc::clone(&notion);
                let db_clone = Arc::clone(&db);
                let cache_clone = Arc::clone(&name_cache);
                let force_val = *force;
                tasks.push(tokio::spawn(sync_character(notion_clone, db_clone, page, force_val, cache_clone)));
            }
            let results = join_all(tasks).await;
            let sync_results: Vec<SyncResult> = results.into_iter().filter_map(|r| r.ok()).collect();
            if *json {
                println!("{}", serde_json::to_string_pretty(&sync_results)?);
            } else {
                for res in sync_results {
                    println!("[{}] {}: {}", res.status, res.character, res.details.unwrap_or_default());
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_id_from_url() {
        assert_eq!(extract_id_from_url("https://www.dndbeyond.com/characters/12345"), "12345");
        assert_eq!(extract_id_from_url("12345"), "12345");
        assert_eq!(extract_id_from_url(""), "");
    }

    #[test]
    fn test_chunk_text() {
        let text = "Hello World";
        let chunks = chunk_text(text, 5);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "Hello");
        assert_eq!(chunks[1], " Worl");
        assert_eq!(chunks[2], "d");
    }
}
