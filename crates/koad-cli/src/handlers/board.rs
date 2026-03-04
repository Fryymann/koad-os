use anyhow::Result;
use koad_board::GitHubClient;
use koad_board::sync::BoardSyncer;
use koad_core::config::KoadConfig;
use crate::cli::BoardAction;
use tracing::{info, warn};

pub async fn handle_board(action: BoardAction, config: &KoadConfig) -> Result<()> {
    // Resolve GitHub credentials from KoadConfig
    let token = config.resolve_gh_token()?;
    let owner = config.get_github_owner()?;
    let repo = config.get_github_repo()?;
    let project_num = config.github_project_number as i32;

    let client = GitHubClient::new(token, owner, repo)?;

    match action {
        BoardAction::Sync { dry_run } => {
            let syncer = BoardSyncer::new(&client, project_num, dry_run);
            syncer.run().await?;
        }
        BoardAction::Status { active } => {
            info!("Project Board Status (Project #{}):", project_num);
            let items = client.list_project_items(project_num).await?;
            for item in items {
                if !active || item.status == "In Progress" || item.status == "Todo" {
                    info!("[{}] #{} {}", item.status, item.number.unwrap_or(0), item.title);
                }
            }
        }
        _ => {
            warn!("Subcommand not yet fully implemented for SGP.");
        }
    }

    Ok(())
}
