use crate::cli::SignalAction;
use koad_core::config::KoadConfig;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::*;
use anyhow::Result;
use std::collections::HashMap;
use std::env;

pub async fn handle_signal_action(
    action: SignalAction,
    config: &KoadConfig,
    agent_name: &str,
) -> Result<()> {
    let mut client = SpineServiceClient::connect(config.spine_grpc_addr.clone()).await?;
    let session_id = env::var("KOAD_SESSION_ID").unwrap_or_else(|_| agent_name.to_string());

    match action {
        SignalAction::Send { target, message, priority } => {
            let p = match priority.to_lowercase().as_str() {
                "low" => SignalPriority::Low,
                "high" => SignalPriority::High,
                "critical" => SignalPriority::Critical,
                _ => SignalPriority::Standard,
            };

            let mut req = tonic::Request::new(SendSignalRequest {
                target_agent: target.clone(),
                message,
                priority: p as i32,
                metadata: HashMap::new(),
            });
            req.metadata_mut().insert("x-session-id", session_id.parse()?);

            client.send_signal(req).await?;
            println!("Signal dispatched to {}.", target);
        }
        SignalAction::List { all: _all } => {
            let mut req = tonic::Request::new(GetSignalsRequest {
                agent_name: agent_name.to_string(),
                filter_status: SignalStatus::Pending as i32,
            });
            req.metadata_mut().insert("x-session-id", session_id.parse()?);

            let res = client.get_signals(req).await?.into_inner();
            if res.signals.is_empty() {
                println!("No pending signals for {}.", agent_name);
            } else {
                println!("--- Pending Signals for {} ---", agent_name);
                for sig in res.signals {
                    let status = if sig.status == SignalStatus::Pending as i32 { "[PENDING]" } else { "[READ]" };
                    println!("{} {} from {}: {}", &sig.id[..8], status, sig.source_agent, sig.message);
                }
            }
        }
        SignalAction::Read { id } => {
            let mut req = tonic::Request::new(GetSignalsRequest {
                agent_name: agent_name.to_string(),
                filter_status: SignalStatus::Pending as i32,
            });
            req.metadata_mut().insert("x-session-id", session_id.clone().parse()?);
            
            let res = client.get_signals(req).await?.into_inner();
            
            if let Some(sig) = res.signals.into_iter().find(|s| s.id.starts_with(&id)) {
                println!("--- Signal {} ---", sig.id);
                println!("From:     {}", sig.source_agent);
                println!("Priority: {:?}", sig.priority);
                println!("Message:  {}", sig.message);
                
                // Mark as read
                let mut update_req = tonic::Request::new(UpdateSignalStatusRequest {
                    signal_id: sig.id,
                    status: SignalStatus::Read as i32,
                });
                update_req.metadata_mut().insert("x-session-id", session_id.parse()?);
                client.update_signal_status(update_req).await?;
            } else {
                println!("Signal not found.");
            }
        }
        SignalAction::Archive { id } => {
             let mut update_req = tonic::Request::new(UpdateSignalStatusRequest {
                signal_id: id,
                status: SignalStatus::Archived as i32,
            });
            update_req.metadata_mut().insert("x-session-id", session_id.parse()?);
            client.update_signal_status(update_req).await?;
            println!("Signal archived.");
        }
    }

    Ok(())
}
