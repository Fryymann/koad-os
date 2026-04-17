//! Pulse handler — broadcast short-lived agent signals via CASS PulseService.

use anyhow::Result;
use koad_core::config::KoadConfig;
use koad_proto::cass::v1::pulse_service_client::PulseServiceClient;
use koad_proto::cass::v1::{AddPulseRequest, GetPulsesRequest};
use std::env;

/// Broadcast a pulse message to CASS.
pub async fn handle_pulse_add(
    message: String,
    role: Option<String>,
    config: &KoadConfig,
) -> Result<()> {
    let agent_name = env::var("KOAD_AGENT_NAME").unwrap_or_else(|_| "unknown".to_string());
    let role_str = role.unwrap_or_else(|| "global".to_string());

    match PulseServiceClient::connect(config.network.cass_grpc_addr.clone()).await {
        Ok(mut client) => {
            let req = AddPulseRequest {
                context: Some(crate::utils::get_trace_context(&agent_name, 3)),
                author: agent_name.clone(),
                role: role_str.clone(),
                message: message.clone(),
                ttl_seconds: 3600,
            };
            match client.add_pulse(req).await {
                Ok(_) => {
                    println!("\x1b[32m[PULSE]\x1b[0m Signal broadcast.");
                    println!("       Author: {}", agent_name);
                    println!("       Role:   {}", role_str);
                    println!("       Msg:    {}", message);
                }
                Err(e) => {
                    println!("\x1b[33m[WARN]\x1b[0m CASS rejected pulse: {}", e);
                }
            }
        }
        Err(_) => {
            println!("\x1b[33m[DEGRADED]\x1b[0m CASS offline — pulse not broadcast.");
        }
    }
    Ok(())
}

/// List active pulses from CASS.
pub async fn handle_pulse_list(role: Option<String>, config: &KoadConfig) -> Result<()> {
    let agent_name = env::var("KOAD_AGENT_NAME").unwrap_or_else(|_| "unknown".to_string());
    let role_str = role.unwrap_or_else(|| "global".to_string());

    match PulseServiceClient::connect(config.network.cass_grpc_addr.clone()).await {
        Ok(mut client) => {
            let req = GetPulsesRequest {
                context: Some(crate::utils::get_trace_context(&agent_name, 3)),
                role: role_str.clone(),
            };
            match client.get_pulses(req).await {
                Ok(resp) => {
                    let pulses = resp.into_inner().pulses;
                    if pulses.is_empty() {
                        println!(
                            "\x1b[33m[EMPTY]\x1b[0m No active pulses for role '{}'.",
                            role_str
                        );
                    } else {
                        println!("\x1b[1;34m--- Active Pulses ({}) ---\x1b[0m", pulses.len());
                        for p in &pulses {
                            println!(
                                "  \x1b[36m[{}]\x1b[0m {} \x1b[2m— {}\x1b[0m",
                                p.role, p.message, p.author
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("\x1b[33m[WARN]\x1b[0m Failed to fetch pulses: {}", e);
                }
            }
        }
        Err(_) => {
            println!("\x1b[33m[DEGRADED]\x1b[0m CASS offline — cannot fetch pulses.");
        }
    }
    Ok(())
}
