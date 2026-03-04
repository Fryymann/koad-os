use anyhow::Result;
use koad_core::config::KoadConfig;
use crate::cli::{BridgeAction, StreamAction};
use crate::db::KoadDB;
use crate::utils::get_spine_client;
use koad_proto::spine::v1::{SystemEvent, EventSeverity};
use chrono::Utc;
use uuid::Uuid;

pub async fn handle_bridge_action(
    action: BridgeAction,
    config: &KoadConfig,
    _db: &KoadDB,
) -> Result<()> {
    match action {
        BridgeAction::Stream { action } => match action {
            StreamAction::Post { topic, message, msg_type } => {
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
            _ => { println!("Stream action not fully implemented."); }
        },
        BridgeAction::Skill { action } => match action {
            crate::cli::SkillAction::List => { println!("Listing skills..."); }
            crate::cli::SkillAction::Run { name, args } => { println!("Running skill '{}' with args: {:?}", name, args); }
        }
        _ => { println!("Bridge action placeholder."); }
    }
    Ok(())
}
