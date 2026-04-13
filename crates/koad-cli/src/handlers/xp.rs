//! XP and Skill Command Handlers

use crate::cli::XpCommands;
use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_proto::citadel::v5::xp_service_client::XpServiceClient;
use koad_proto::citadel::v5::{XpAwardRequest, XpSource, XpStatusRequest};

/// Main entry point for XP-related CLI actions.
pub async fn handle_xp_command(command: XpCommands, config: &KoadConfig) -> Result<()> {
    let mut client = XpServiceClient::connect(config.network.citadel_grpc_addr.clone())
        .await
        .context("Failed to connect to Citadel XP Service")?;

    match command {
        XpCommands::Status { agent } => {
            let agent_name = agent.unwrap_or_else(|| config.get_agent_name());
            let context = Some(crate::utils::get_trace_context(&agent_name, 1));
            let resp = client
                .get_status(XpStatusRequest {
                    context,
                    agent_name: agent_name.clone(),
                })
                .await?
                .into_inner();

            println!("\n\x1b[1;34m--- KoadOS Experience Status ---\x1b[0m");
            println!("\x1b[1mAgent:\x1b[0m      {}", resp.agent_name);
            println!(
                "\x1b[1mTier:\x1b[0m       \x1b[32m{}\x1b[0m (Level {})",
                resp.tier_name, resp.level
            );
            println!("\x1b[1mCurrent XP:\x1b[0m {}", resp.total_xp);

            // Progress Bar Logic
            // In a real curve, we'd need the XP at the START of the current level.
            // For now, we'll use a simple percentage of the next level target.
            let next_total = resp.next_level_xp;
            let progress = (resp.total_xp as f32 / next_total as f32).min(1.0);
            let bars = (progress * 20.0) as usize;
            let bar_str = format!(
                "\x1b[32m{}\x1b[0m{}",
                "█".repeat(bars),
                "░".repeat(20 - bars)
            );

            println!(
                "\x1b[1mProgress:\x1b[0m   {}  {:.0}%",
                bar_str,
                progress * 100.0
            );
            println!("\x1b[1mNext Level:\x1b[0m {} XP", next_total);
            println!();
        }
        XpCommands::Award {
            agent,
            amount,
            reason,
            source,
        } => {
            let xp_source = match source.to_lowercase().as_str() {
                "task" => XpSource::Task,
                "skill" => XpSource::Skill,
                _ => XpSource::System,
            };

            let context = Some(crate::utils::get_trace_context(&agent, 3));
            let resp = client
                .award_xp(XpAwardRequest {
                    context,
                    agent_name: agent,
                    amount,
                    reason,
                    source: xp_source as i32,
                    source_id: String::new(),
                })
                .await?
                .into_inner();

            if resp.success {
                println!("\x1b[32m[OK]\x1b[0m {}", resp.message);
            } else {
                println!("\x1b[31m[ERROR]\x1b[0m {}", resp.message);
            }
        }
    }

    Ok(())
}
