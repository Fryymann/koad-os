use crate::cli::{BridgeAction, NotionAction, StreamAction};
use crate::db::KoadDB;
use crate::utils::get_spine_client;
use anyhow::{anyhow, Result};
use chrono::Utc;
use koad_bridge_notion::NotionClient;
use koad_core::config::KoadConfig;
use koad_proto::spine::v1::{EventSeverity, SystemEvent};
use std::env;
use uuid::Uuid;

pub async fn handle_bridge_action(
    action: BridgeAction,
    config: &KoadConfig,
    _db: &KoadDB,
) -> Result<()> {
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
                    let db_id = env::var("NOTION_STREAM_DB_ID")
                        .map_err(|_| anyhow!("NOTION_STREAM_DB_ID environment variable not set"))?;
                    client
                        .post_to_stream(&db_id, "Koad", &target, &message, &priority)
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
                let mut client = get_spine_client(config).await?;

                let severity = match msg_type.to_uppercase().as_str() {
                    "DEBUG" => EventSeverity::Debug,
                    "INFO" => EventSeverity::Info,
                    "WARN" => EventSeverity::Warn,
                    "ERROR" => EventSeverity::Error,
                    "CRITICAL" => EventSeverity::Critical,
                    _ => EventSeverity::Info,
                };

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
