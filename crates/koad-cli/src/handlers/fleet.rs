use anyhow::Result;
use koad_core::config::KoadConfig;
use crate::cli::FleetAction;
use crate::db::KoadDB;
use koad_board::GitHubClient;
use koad_board::project::ProjectItem;
use std::env;

pub async fn handle_fleet_action(
    action: FleetAction,
    _config: &KoadConfig,
    _db: &KoadDB,
) -> Result<()> {
    let token = env::var("GITHUB_ADMIN_PAT").unwrap_or_else(|_| "GITHUB_PERSONAL_PAT".to_string());
    let owner = "Fryymann".to_string();
    let repo = "koad-os".to_string();

    match action {
        FleetAction::Board { action } => match action {
            crate::cli::BoardAction::Sync => {
                println!("Syncing repository issues with Project #2...");
                let board = GitHubClient::new(token, owner, repo)?;
                board.sync_issues(2).await?;
                println!("Sync complete.");
            }
            crate::cli::BoardAction::Status { active } => {
                println!(">>> [UPLINK] Accessing Neural Log (Project #2)...");
                let board = GitHubClient::new(token, owner, repo)?;
                let items: Vec<ProjectItem> = board.list_project_items(2).await?;
                println!("Debug: Found {} total items in project board.", items.len());
                for item in items {
                    if active && item.status == "Done" { continue; }
                    println!("#{} {} [{}]", item.number.unwrap_or(0), item.title, item.status);
                }
            }
            crate::cli::BoardAction::Done { id } => {
                println!("Moving Issue #{} to Done...", id);
                let board = GitHubClient::new(token, owner, repo)?;
                board.update_item_status(id, 2, "Done").await?;
                println!("Issue #{} successfully moved to Done.", id);
            }
            _ => { println!("Board action placeholder."); }
        },
        _ => { println!("Fleet action placeholder."); }
    }
    Ok(())
}
