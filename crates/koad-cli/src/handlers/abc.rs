//! # KoadOS Automated Boot Cognition (ABC)
//!
//! Implements the three-stage pipeline for automated agent briefing:
//! 1. **Collect**: Deterministic data gathering (Filesystem, Updates, SQLite).
//! 2. **Extract**: Signal extraction via local Gemma 3:4B.
//! 3. **Synthesize**: Tactical brief generation via local Qwen 2.5:14B.

use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_intelligence::clients::OllamaClient;
use koad_intelligence::InferenceClient;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, debug, instrument};

/// Collects raw system and session data for the ABC pipeline.
pub struct AbcCollector {
    config: KoadConfig,
    vault_path: PathBuf,
    agent_name: String,
}

impl AbcCollector {
    /// Create a new collector with the given configuration and vault path.
    pub fn new(config: KoadConfig, vault_path: PathBuf, agent_name: String) -> Self {
        Self { config, vault_path, agent_name }
    }

    /// Gathers deterministic state from the environment, filesystem, and databases.
    ///
    /// # Errors
    /// Returns an error if filesystem reads or database queries fail.
    pub async fn collect_raw_data(&self) -> Result<String> {
        let mut raw = String::new();
        let identity = self.config.identities.get(&self.agent_name.to_lowercase());
        
        raw.push_str("--- ENVIRONMENT ---\n");
        raw.push_str(&format!("Agent: {} | Rank: {} | Tier: {}\n", 
            self.agent_name,
            identity.map(|id| id.rank.clone()).unwrap_or_else(|| "unknown".to_string()),
            identity.map(|id| id.tier).unwrap_or(0)
        ));
        raw.push_str(&format!("Date: {}\n\n", chrono::Utc::now()));

        // 1. Filesystem Map (Root)
        raw.push_str("--- FILESYSTEM MAP (Root) ---\n");
        if let Ok(mut entries) = fs::read_dir(&self.config.home).await {
            let mut count = 0;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if count >= 15 { break; }
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') && name != ".koad-os" { continue; }
                let suffix = if entry.file_type().await?.is_dir() { "/" } else { "" };
                raw.push_str(&format!("{}{}\n", name, suffix));
                count += 1;
            }
        }
        raw.push_str("\n");

        // 2. Recent Updates
        raw.push_str("--- RECENT UPDATES ---\n");
        let updates_dir = self.config.home.join("updates");
        if updates_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&updates_dir).await {
                let mut update_files = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(meta) = entry.metadata().await {
                        if let Ok(modified) = meta.modified() {
                            update_files.push((entry.path(), modified));
                        }
                    }
                }
                update_files.sort_by(|a, b| b.1.cmp(&a.1));
                for (path, _) in update_files.into_iter().take(3) {
                    if let Ok(content) = fs::read_to_string(path).await {
                        raw.push_str(&content);
                        raw.push_str("\n---\n");
                    }
                }
            }
        }
        raw.push_str("\n");

        // 3. Working Memory
        raw.push_str("--- WORKING MEMORY ---\n");
        let wm_path = self.vault_path.join("memory/WORKING_MEMORY.md");
        if wm_path.exists() {
            if let Ok(content) = fs::read_to_string(&wm_path).await {
                raw.push_str(&content.lines().take(20).collect::<Vec<_>>().join("\n"));
            }
        }
        raw.push_str("\n\n");

        // 4. Notion Missions (Active Quests)
        raw.push_str("--- ACTIVE MISSIONS (DB) ---\n");
        let db_path = self.config.home.join("data/db/notion-sync.db");
        if db_path.exists() {
            let db_path_buf = db_path.to_path_buf();
            let missions = tokio::task::spawn_blocking(move || -> Result<Vec<String>> {
                let conn = rusqlite::Connection::open(db_path_buf)?;
                let mut stmt = conn.prepare("SELECT title FROM pages WHERE is_deleted = 0 LIMIT 5")?;
                let missions: Vec<String> = stmt.query_map([], |row| row.get(0))?
                    .filter_map(Result::ok)
                    .collect();
                Ok(missions)
            }).await??;

            for m in missions {
                raw.push_str(&format!("- [QUEST] {}\n", m));
            }
        }

        Ok(raw)
    }
}

/// Orchestrates local AI models to extract signal and synthesize a brief.
pub struct AbcGenerator {
    llama: OllamaClient,
    qwen: OllamaClient,
}

impl AbcGenerator {
    /// Create a new generator using local Ollama models.
    ///
    /// # Errors
    /// Returns an error if the Ollama client cannot be initialized.
    pub fn new() -> Result<Self> {
        Ok(Self {
            llama: OllamaClient::new(Some("llama3.2:1b"), None)?,
            qwen: OllamaClient::new(Some("qwen2.5-coder:7b"), None)?,
        })
    }

    /// Runs the AI pipeline: Llama (Extract) -> Qwen (Synthesize).
    ///
    /// # Errors
    /// Returns an error if the local LLM inference fails.
    pub async fn generate_brief(&self, raw_data: &str, agent_name: &str) -> Result<String> {
        info!("🧠 ABC: Extracting signal via Llama 3.2:1B...");
        let llama_prompt = format!(
            "You are a KoadOS Intelligence Agent. From the following raw system data, extract ONLY the 3 most recent accomplishments, the 3 most critical active missions, and the current 'Next Action' from working memory. Be extremely concise. Use bullet points.\n\n### RAW DATA:\n{}",
            raw_data
        );
        let signal = self.llama.chat(&llama_prompt).await
            .context("Local extraction via Llama 3.2:1B failed. Ensure Ollama is running and llama3.2:1b is pulled.")?;
        
        debug!(signal_len = signal.len(), "ABC Signal Extracted");

        info!("♟️ ABC: Synthesizing brief via Qwen 2.5-Coder:7B...");
        let qwen_prompt = format!(
            "You are the Citadel Chief of Staff. Based on the following extracted project signals, draft a 3-paragraph Tactical Brief for the Captain ({:?}). 
            Paragraph 1: Summary of the current state of the Citadel.
            Paragraph 2: The critical delta between Notion Missions and implementation progress.
            Paragraph 3: Recommended immediate first step for this session.
            Maintain a professional, senior military tone.\n\n### SIGNALS:\n{}",
            agent_name,
            signal
        );
        let brief = self.qwen.chat(&qwen_prompt).await
            .context("Local synthesis via Qwen 2.5-Coder:7B failed. Ensure Ollama is running and qwen2.5-coder:7b is pulled.")?;
        
        Ok(brief)
    }
}

/// Main entry point for the ABC pipeline.
/// Returns the generated brief string and caches it to disk.
///
/// # Errors
/// Returns an error if data collection or inference fails.
#[instrument(skip(config))]
pub async fn run_abc(config: &KoadConfig, vault_path: &Path, agent_name: &str) -> Result<String> {
    let agent_key = agent_name.to_lowercase();
    debug!(agent = %agent_key, "Starting ABC pipeline");

    let collector = AbcCollector::new(config.clone(), vault_path.to_path_buf(), agent_name.to_string());
    let raw_data = collector.collect_raw_data().await?;

    let generator = AbcGenerator::new()?;
    let brief = generator.generate_brief(&raw_data, agent_name).await?;

    // Cache the brief for future turns/sessions
    let cache_dir = config.home.join("cache");
    let _ = fs::create_dir_all(&cache_dir).await;
    let brief_path = cache_dir.join(format!("tactical-brief-{}.md", agent_key));
    
    fs::write(brief_path, &brief).await
        .context("Failed to write tactical brief to cache")?;

    Ok(brief)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abc_collector_new() {
        let config = KoadConfig::load().unwrap_or_else(|_| {
            // Provide a dummy config if load fails during CI/Test
            serde_json::from_str("{}").unwrap_or_else(|_| {
                KoadConfig::from_json("{}").expect("Failed to parse empty config")
            })
        });
        let collector = AbcCollector::new(config, PathBuf::from("/tmp"));
        assert_eq!(collector.vault_path, PathBuf::from("/tmp"));
    }
}
