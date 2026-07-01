use crate::cli::IntelAction;
use koad_core::db::KoadDB;
use crate::utils::errors::map_connect_err;
use crate::utils::{detect_model_tier, feature_gate};
use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_proto::citadel::v5::admin_client::AdminClient;
use koad_proto::citadel::v5::*;
use rusqlite::params;
use std::env;

pub async fn handle_intel_action(
    action: IntelAction,
    config: &KoadConfig,
    db: &KoadDB,
    agent_name: &str,
) -> Result<()> {
    let _model_tier = detect_model_tier();
    let context = Some(crate::utils::get_trace_context(agent_name, 3)); // Level 3 = Citadel scope
    match action {
        IntelAction::Query {
            term,
            limit,
            tags,
            agent,
        } => {
            let results = db.query_knowledge(&term, limit, agent.as_deref())?;
            println!(
                "
\x1b[1m--- INTEL: Knowledge Query [{}] ---\x1b[0m",
                term
            );
            for (cat, content, t, origin) in results {
                if let Some(ref filter_tags) = tags {
                    if !t.contains(filter_tags) {
                        continue;
                    }
                }
                println!("[{}] ({}) [{}] {}", cat, t, origin, content);
            }
            println!(
                "\x1b[1m---------------------------------------------------\x1b[0m
"
            );
        }
        IntelAction::Remember { category } => {
            let (cat_str, text, tags) = match category {
                crate::cli::MemoryCategory::Fact { text, tags } => ("fact", text, tags),
                crate::cli::MemoryCategory::Learning { text, tags } => ("learning", text, tags),
            };

            let session_id = env::var("KOAD_SESSION_ID")
                .context("KOAD_SESSION_ID not set. Please boot an agent first.")?;
            let mut client = AdminClient::connect(config.network.citadel_grpc_addr.clone())
                .await
                .map_err(|e| {
                    map_connect_err("KoadOS Citadel", &config.network.citadel_grpc_addr, e)
                })
                .map_err(anyhow::Error::from)?;

            client
                .commit_knowledge(crate::utils::authenticated_request(
                    CommitKnowledgeRequest {
                        context: context.clone(),
                        session_id,
                        category: cat_str.to_string(),
                        content: text,
                        tags: tags.unwrap_or_default(),
                    },
                ))
                .await
                .context("Commit failed")?;

            println!("Memory updated via Citadel.");
        }
        IntelAction::Ponder { text, tags } => {
            let session_id = env::var("KOAD_SESSION_ID")
                .context("KOAD_SESSION_ID not set. Please boot an agent first.")?;
            let mut client = AdminClient::connect(config.network.citadel_grpc_addr.clone())
                .await
                .map_err(|e| {
                    map_connect_err("KoadOS Citadel", &config.network.citadel_grpc_addr, e)
                })
                .map_err(anyhow::Error::from)?;

            client
                .commit_knowledge(crate::utils::authenticated_request(
                    CommitKnowledgeRequest {
                        context: context.clone(),
                        session_id,
                        category: "pondering".to_string(),
                        content: text,
                        tags: format!("persona-journal,{}", tags.unwrap_or_default()),
                    },
                ))
                .await
                .context("Commit failed")?;

            println!("Reflection recorded via Citadel.");
        }
        IntelAction::Guide { topic } => {
            crate::handlers::guide::handle_guide_action(topic, config).await?;
        }
        IntelAction::Scan { path: _ } => {
            feature_gate("koad scan", None);
        }
        IntelAction::Mind { action } => {
            let conn = db.get_conn()?;
            match action {
                crate::cli::MindAction::Status => {
                    println!("Mind status checked.");
                }
                crate::cli::MindAction::Learn {
                    domain,
                    summary,
                    detail,
                } => {
                    conn.execute("INSERT INTO learnings (domain, summary, detail, source, status, origin_agent) VALUES (?1, ?2, ?3, 'cli', 'active', ?4)", 
                        params![domain, summary, detail, agent_name])?;
                    println!(
                        "\x1b[32m[LEARNED]\x1b[0m New {} insight integrated into mind.",
                        domain
                    );
                }
                _ => {
                    println!("Mind action placeholder.");
                }
            }
        }
        IntelAction::Snippet {
            path,
            start,
            end,
            bypass,
        } => {
            println!(
                ">>> [UPLINK] Connecting to Citadel at {}...",
                config.network.citadel_grpc_addr
            );
            let mut client = AdminClient::connect(config.network.citadel_grpc_addr.clone())
                .await
                .map_err(|e| {
                    map_connect_err("KoadOS Citadel", &config.network.citadel_grpc_addr, e)
                })
                .map_err(anyhow::Error::from)?;
            let resp = client
                .get_file_snippet(crate::utils::authenticated_request(GetFileSnippetRequest {
                    context: context.clone(),
                    path: path.to_string_lossy().to_string(),
                    start_line: start,
                    end_line: end,
                    bypass_cache: bypass,
                }))
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Snippet Retrieval Failed: [{:?}] {}", e.code(), e.message())
                })?;

            let package = resp.into_inner();
            println!(
                "
\x1b[1m--- SNIPPET: {:?} (Lines {}-{}, Source: {}) ---\x1b[0m",
                path, start, end, package.source
            );
            println!("{}", package.content);
            println!(
                "\x1b[1m---------------------------------------------------\x1b[0m
"
            );
        }
    }
    Ok(())
}
