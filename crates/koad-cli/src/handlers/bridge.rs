use anyhow::Result;
use koad_core::config::KoadConfig;
use crate::cli::BridgeAction;
use crate::db::KoadDB;

pub async fn handle_bridge_action(
    action: BridgeAction,
    _config: &KoadConfig,
    _db: &KoadDB,
) -> Result<()> {
    match action {
        BridgeAction::Skill { action } => match action {
            crate::cli::SkillAction::List => { println!("Listing skills..."); }
            crate::cli::SkillAction::Run { name, args } => { println!("Running skill '{}' with args: {:?}", name, args); }
        }
        _ => { println!("Bridge action placeholder."); }
    }
    Ok(())
}
