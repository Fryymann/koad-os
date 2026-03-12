use crate::cli::BoardAction;
use anyhow::Result;
use koad_board::sync::BoardSyncer;
use koad_board::GitHubClient;
use koad_core::config::KoadConfig;
use tracing::{info, warn};

pub async fn handle_board(action: BoardAction, config: &KoadConfig) -> Result<()> {
    // 1. Resolve Project Context from path
    let current_dir = std::env::current_dir().unwrap_or_default();
    let project_ctx = config.resolve_project_context(&current_dir);
    let project = project_ctx.as_ref().map(|(_, p)| p);

    // 2. Resolve GitHub credentials and metadata
    let token = config.resolve_gh_token(project)?;
    let owner = config.get_github_owner(project);
    let repo = config.get_github_repo(project);
    let project_num = project
        .and_then(|p| p.default_project)
        .unwrap_or(config.github.default_project_number) as i32;

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
                    info!(
                        "[{}] #{} {}",
                        item.status,
                        item.number.unwrap_or(0),
                        item.title
                    );
                }
            }
        }
        BoardAction::Done { id } => {
            // 1. Move on Project Board
            client.update_item_status(project_num, id, "Done").await?;
            // 2. Close on GitHub
            client.close_issue(id).await?;
            info!("[OK] Issue #{} marked as Done and closed.", id);
        }
        _ => {
            warn!("Subcommand not yet fully implemented for SGP.");
        }
    }

    Ok(())
}
