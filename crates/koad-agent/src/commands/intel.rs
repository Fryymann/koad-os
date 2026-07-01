use anyhow::{Context, Result};
use koad_core::config::KoadConfig;

/// Handles the 'intel' command, executing the heavy ABC synthesis pipeline on-demand.
pub async fn handle_intel(config: &KoadConfig, agent_name: &str) -> Result<()> {
    let agent_name = agent_name.to_lowercase();
    
    // Resolve the vault path for the agent to access its localized memory
    let vault_uri = config
        .resolve_vault_uri(&agent_name)
        .context("Could not resolve vault URI for agent.")?;
    let vault_path = config
        .resolve_vault_path(&vault_uri)
        .context("Could not resolve vault path.")?;

    println!("\x1b[1;34m[INTEL]\x1b[0m Gathering situational awareness for '{}'...", agent_name);
    
    // Execute the full ABC pipeline
    let brief = crate::handlers::abc::run_abc(config, &vault_path, &agent_name).await?;
    
    println!("\n\x1b[1;32m--- Tactical Brief (Citadel ABC) ---\x1b[0m");
    println!("{}", brief);
    
    Ok(())
}
