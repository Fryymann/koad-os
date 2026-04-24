use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;

pub async fn handle_verify(agent_name: String, config: &koad_core::config::KoadConfig) -> Result<()> {
    let vault_uri = config
        .resolve_vault_uri(&agent_name)
        .context("Could not resolve vault URI for current agent.")?;
    let vault_path = config.resolve_vault_path(&vault_uri)?;
    verify_kapv(&vault_path).await?;
    println!("\x1b[32m[OK]\x1b[0m Vault for '{}' is valid.", agent_name);
    Ok(())
}

pub async fn verify_kapv(path: &Path) -> Result<()> {
    let dirs = [
        "bank",
        "config",
        "identity",
        "instructions",
        "memory",
        "sessions",
        "tasks",
    ];
    for d in dirs {
        let dir_path = path.join(d);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .await
                .context(format!("Failed to auto-heal missing KAPV dir: {}", d))?;
        } else if !dir_path.is_dir() {
            anyhow::bail!("KAPV entry '{}' exists but is not a directory.", d);
        }
    }
    Ok(())
}
