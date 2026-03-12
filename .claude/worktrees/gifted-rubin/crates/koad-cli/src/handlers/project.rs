use crate::cli::ProjectAction;
use anyhow::Result;
use koad_core::config::KoadConfig;

pub async fn handle_project(action: ProjectAction, _config: &KoadConfig) -> Result<()> {
    match action {
        ProjectAction::List => {
            println!("Listing registered projects (from Memory Bank)...");
            // Placeholder for SQLite project lookup
        }
        _ => {
            println!("Project command not yet fully implemented in v4.1.");
        }
    }
    Ok(())
}
