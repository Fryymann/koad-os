use crate::cli::IntelAction;
use koad_core::db::KoadDB;
use crate::utils::errors::map_connect_err;
use crate::utils::{detect_model_tier, feature_gate};
use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_proto::citadel::v5::admin_client::AdminClient;
use koad_proto::citadel::v5::*;
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
            println!(
                "
\x1b[1m--- INTEL: Knowledge Query [{}] ---\x1b[0m",
                term
            );

            // CASS-first: semantic recall under this agent's partition.
            // Bounded connect so an unreachable CASS degrades fast, not a hang.
            let cass_connect = match tonic::transport::Endpoint::from_shared(
                config.network.cass_grpc_addr.clone(),
            ) {
                Ok(ep) => ep
                    .connect_timeout(std::time::Duration::from_secs(3))
                    .timeout(std::time::Duration::from_secs(10))
                    .connect()
                    .await
                    .map_err(anyhow::Error::from),
                Err(e) => Err(anyhow::Error::from(e)),
            };
            match cass_connect
                .map(koad_proto::cass::v1::memory_service_client::MemoryServiceClient::new)
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

            println!("\x1b[1m--- Local Archive ---\x1b[0m");
            let results = db.query_knowledge(&term, limit, agent.as_deref())?;
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
    }
    Ok(())
}
