use crate::cli::FleetAction;
use crate::db::KoadDB;
use anyhow::Result;
use koad_core::config::KoadConfig;

pub async fn handle_fleet_action(
    action: FleetAction,
    config: &KoadConfig,
    _db: &KoadDB,
) -> Result<()> {
    match action {
        FleetAction::Board { action } => {
            crate::handlers::board::handle_board(action, config).await?;
        }
        _ => {
            println!("Fleet action placeholder.");
        }
    }
    Ok(())
}
