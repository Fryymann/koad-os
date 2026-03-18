use crate::cli::{BridgeAction, NotionAction, StreamAction};
use crate::db::KoadDB;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use koad_bridge_notion::NotionClient;
use koad_core::config::KoadConfig;
use koad_proto::citadel::v5::admin_client::AdminClient;
use koad_proto::citadel::v5::{EventSeverity, SystemEvent};
use std::env;
use uuid::Uuid;

pub async fn handle_bridge_action(
    action: BridgeAction,
    config: &KoadConfig,
    _db: &KoadDB,
) -> Result<()> {
    let agent_name = config.get_agent_name();
    match action {
        BridgeAction::Notion { action } => {
            let api_key = env::var("NOTION_API_KEY")
                .map_err(|_| anyhow!("NOTION_API_KEY environment variable not set"))?;
            let client = NotionClient::new(api_key)?;

            match action {
                NotionAction::Read { id } => {
                    let markdown = client.get_page_content_markdown(&id).await?;
                    println!("{}", markdown);
                }
                NotionAction::Stream {
                    message,
                    target,
                    priority,
                } => {
                    let db_id = env::var("NOTION_STREAM_DB_ID").or_else(|_| {
                        config
                            .integrations
                            .notion
                            .as_ref()
                            .and_then(|n| n.index.get("stream_db").cloned())
                            .ok_or_else(|| {
                                anyhow!("Notion stream_db ID not found in config or env")
                            })
                    })?;
                    client
                        .post_to_stream(&db_id, &agent_name, &target, &message, &priority)
                        .await?;
                    println!(">>> [UPLINK] Message posted to Notion KoadStream.");
                }
            }
        }
        BridgeAction::Stream { action } => match action {
            StreamAction::Post {
                topic,
                message,
                msg_type,
            } => {
                let mut client = AdminClient::connect(config.network.citadel_grpc_addr.clone())
                    .await
                    .context("Failed to connect to Citadel gRPC")?;

                let severity = match msg_type.to_uppercase().as_str() {
                    "DEBUG" => EventSeverity::Debug,
                    "INFO" => EventSeverity::Info,
                    "WARN" => EventSeverity::Warn,
                    "ERROR" => EventSeverity::Error,
                    "CRITICAL" => EventSeverity::Critical,
                    _ => EventSeverity::Info,
                };

                let context = Some(crate::utils::get_trace_context(&agent_name, 3));

                let event = SystemEvent {
                    event_id: Uuid::new_v4().to_string(),
                    source: topic,
                    severity: severity as i32,
                    message,
                    metadata_json: "{}".to_string(),
                    timestamp: Some(prost_types::Timestamp {
                        seconds: Utc::now().timestamp(),
                        nanos: Utc::now().timestamp_subsec_nanos() as i32,
                    }),
                    context,
                };

                client.post_system_event(event).await?;
                println!(">>> [UPLINK] Message broadcast to KoadStream.");
            }
        },
        BridgeAction::Skill { action } => match action {
            crate::cli::SkillAction::List => {
                println!("Listing skills...");
            }
            crate::cli::SkillAction::Run { name, args } => {
                println!("Running skill '{}' with args: {:?}", name, args);
            }
        },
        _ => {
            println!("Bridge action placeholder.");
        }
    }
    Ok(())
}
