use crate::cli::{BridgeAction, FsAction, NotionAction, SkillAction, StreamAction};
use crate::db::KoadDB;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use koad_bridge_notion::{NotionClient, NotionMcpProxy};
use koad_core::config::KoadConfig;
use koad_proto::cass::v1::tool_registry_service_client::ToolRegistryServiceClient;
use koad_proto::cass::v1::{DeregisterToolRequest, InvokeToolRequest, ListToolsRequest, RegisterToolRequest};
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
        BridgeAction::Fs { action } => match action {
            FsAction::Serve => {
                let fs_mcp_bin = config.home.join("bin/koad-fs-mcp");
                let mut child = tokio::process::Command::new(fs_mcp_bin)
                    .env("KOAD_AGENT_NAME", &agent_name)
                    .spawn()
                    .context("Failed to launch koad-fs-mcp supervisor")?;
                
                let _ = child.wait().await?;
            }
        },
        BridgeAction::Notion { action } => {
            let api_key = env::var("KOADOS_PAT_NOTION_MAIN")
                .or_else(|_| env::var("KOADOS_MAIN_NOTION_TOKEN"))
                .or_else(|_| env::var("NOTION_API_KEY"))
                .or_else(|_| env::var("NOTION_TOKEN"))
                .map_err(|_| anyhow!("Notion token not set. Expected KOADOS_PAT_NOTION_MAIN in environment."))?;
            
            let db_path = config.home.join("data/db/notion-sync.db");
            let proxy = NotionMcpProxy::new(api_key.clone(), db_path)?;

            match action {
                NotionAction::Read { id } => {
                    let markdown = proxy.get_page_content(&id, false).await?;
                    println!("{}", markdown);
                }
                NotionAction::Sync { id } => {
                    println!(">>> [SYNC] Synchronizing Notion database: {}", id);
                    let count = proxy.sync_database(&id).await?;
                    println!(">>> [SUCCESS] Synced {} pages to local datastore.", count);
                }
                NotionAction::Export { id, output } => {
                    println!(">>> [EXPORT] Exporting database {} to {}", id, output.display());
                    let count = proxy.export_database(&id, &output)?;
                    println!(">>> [SUCCESS] Exported {} Markdown files.", count);
                }
                NotionAction::UpdateStatus { id, status } => {
                    println!(">>> [UPLINK] Updating status of {} to '{}'...", id, status);
                    proxy.update_status(&id, &status).await?;
                    println!(">>> [SUCCESS] Notion status updated.");
                }
                NotionAction::Stream {
                    message,
                    target,
                    priority,
                } => {
                    let client = NotionClient::new(api_key)?;
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
        BridgeAction::Skill { action } => {
            let mut client = ToolRegistryServiceClient::connect(config.network.cass_grpc_addr.clone())
                .await
                .context("Failed to connect to CASS gRPC (ToolRegistry)")?;
            let trace_ctx = Some(crate::utils::get_trace_context(&agent_name, 3));
            match action {
                SkillAction::List => {
                    let req = ListToolsRequest { context: trace_ctx };
                    let resp = client.list_tools(req).await?.into_inner();
                    if resp.tool_names.is_empty() {
                        println!("No skills registered.");
                    } else {
                        println!("Registered skills:");
                        for name in &resp.tool_names {
                            println!("  - {}", name);
                        }
                    }
                }
                SkillAction::Register { name, path } => {
                    let req = RegisterToolRequest {
                        context: trace_ctx,
                        name: name.clone(),
                        component_path: path,
                        container_image: String::new(),
                    };
                    let resp = client.register_tool(req).await?.into_inner();
                    println!(">>> [OK] Skill '{}' registered. Status: {}", name, resp.message);
                }
                SkillAction::Deregister { name } => {
                    let req = DeregisterToolRequest {
                        context: trace_ctx,
                        name: name.clone(),
                    };
                    let resp = client.deregister_tool(req).await?.into_inner();
                    println!(">>> [OK] Skill '{}' deregistered. Status: {}", name, resp.message);
                }
                SkillAction::Run { name, topic, payload } => {
                    let req = InvokeToolRequest {
                        context: trace_ctx,
                        name: name.clone(),
                        topic,
                        payload,
                    };
                    let resp = client.invoke_tool(req).await?.into_inner();
                    println!(">>> [OK] Skill '{}' executed in {}ms ({} bytes).", name, resp.duration_ms, resp.memory_bytes);
                    println!("{}", resp.output);
                }
            }
        }
        _ => {
            println!("Bridge action placeholder.");
        }
    }
    Ok(())
}
