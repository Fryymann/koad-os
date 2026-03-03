use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::*;
use crate::cli::IntelAction;
use crate::db::KoadDB;
use crate::utils::{detect_model_tier, feature_gate};

pub async fn handle_intel_action(
    action: IntelAction,
    config: &KoadConfig,
    db: &KoadDB,
) -> Result<()> {
    let model_tier = detect_model_tier();
    match action {
        IntelAction::Query { term, limit, tags } => {
            let results = db.query_knowledge(&term, limit)?;
            println!("
\x1b[1m--- INTEL: Knowledge Query [{}] ---\x1b[0m", term);
            for (cat, content, t) in results {
                if let Some(ref filter_tags) = tags {
                    if !t.contains(filter_tags) { continue; }
                }
                println!("[{}] ({}) {}", cat, t, content);
            }
            println!("\x1b[1m---------------------------------------------------\x1b[0m
");
        }
        IntelAction::Remember { category } => {
            let (cat_str, text, tags) = match category {
                crate::cli::MemoryCategory::Fact { text, tags } => ("fact", text, tags),
                crate::cli::MemoryCategory::Learning { text, tags } => ("learning", text, tags)
            };
            db.remember(cat_str, &text, tags, model_tier)?;
            println!("Memory updated.");
        }
        IntelAction::Ponder { text, tags } => {
            db.remember("pondering", &text, Some(format!("persona-journal,{}", tags.unwrap_or_default())), model_tier)?;
            println!("Reflection recorded.");
        }
        IntelAction::Guide { topic: _ } => { feature_gate("koad guide", None); }
        IntelAction::Scan { path: _ } => { feature_gate("koad scan", None); }
        IntelAction::Mind { action } => match action {
            crate::cli::MindAction::Status => { println!("Mind status checked."); }
            _ => { println!("Mind action placeholder."); }
        }
        IntelAction::Snippet { path, start, end, bypass } => {
            println!(">>> [UPLINK] Connecting to Spine at {}...", config.spine_grpc_addr);
            let mut client = SpineServiceClient::connect(config.spine_grpc_addr.clone()).await.context("Connect failed.")?;
            let resp = client.get_file_snippet(GetFileSnippetRequest {
                path: path.to_string_lossy().to_string(),
                start_line: start,
                end_line: end,
                bypass_cache: bypass,
            }).await.map_err(|e| anyhow::anyhow!("Snippet Retrieval Failed: [{:?}] {}", e.code(), e.message()))?;
            
            let package = resp.into_inner();
            println!("
\x1b[1m--- SNIPPET: {:?} (Lines {}-{}, Source: {}) ---\x1b[0m", path, start, end, package.source);
            println!("{}", package.content);
            println!("\x1b[1m---------------------------------------------------\x1b[0m
");
        }
    }
    Ok(())
}
