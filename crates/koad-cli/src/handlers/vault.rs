//! # KoadOS Vault Command Handler
//!
//! Implements the command-based access layer for agent identity,
//! config, and credentials.

use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use std::env;
use std::path::Path;
use tokio::fs;

use crate::cli::VaultAction;

/// Entry point for all `koad vault` commands.
pub async fn handle_vault_action(action: VaultAction, config: &KoadConfig) -> Result<()> {
    let agent_name = env::var("KOAD_AGENT_NAME").unwrap_or_else(|_| "Admiral".to_string());
    let vault_uri = config.resolve_vault_uri(&agent_name)
        .context("Could not resolve vault URI for current agent.")?;
    let vault_path = config.resolve_vault_path(&vault_uri)?;

    match action {
        VaultAction::Whoami => handle_whoami(&agent_name, &vault_uri, &vault_path).await,
        VaultAction::Env => handle_env(&agent_name, config).await,
        VaultAction::Get { path } => handle_get(&vault_path, &path).await,
        VaultAction::Ls { path } => handle_ls(&vault_path, path.as_deref()).await,
        VaultAction::Secret { action } => handle_secret_action(action, config).await,
        _ => {
            println!("\x1b[33m[STUB]\x1b[0m This vault action is not yet implemented in Dark Mode.");
            Ok(())
        }
    }
}

async fn handle_whoami(agent: &str, uri: &str, path: &Path) -> Result<()> {
    println!("Agent: {}", agent);
    println!("Vault URI: {}", uri);
    println!("Vault Root: {}", path.display());
    Ok(())
}

async fn handle_env(agent: &str, config: &KoadConfig) -> Result<()> {
    let vault_uri = config.resolve_vault_uri(agent).unwrap_or_default();
    println!("export KOAD_VAULT_URI=\"{}\"", vault_uri);
    
    if let Some(id) = config.identities.get(&agent.to_lowercase()) {
        println!("export KOAD_AGENT_ROLE=\"{}\"", id.role);
        println!("export KOAD_AGENT_RANK=\"{}\"", id.rank);
        println!("export KOAD_AGENT_TIER={}", id.tier);
    }
    Ok(())
}

async fn handle_get(vault_root: &Path, rel_path: &str) -> Result<()> {
    let target = vault_root.join(rel_path);
    if !target.starts_with(vault_root) {
        anyhow::bail!("Access denied: Path is outside the vault sanctuary.");
    }
    if !target.exists() {
        anyhow::bail!("Vault item not found: {}", rel_path);
    }
    if target.is_dir() {
        anyhow::bail!("Item is a directory. Use `koad vault ls {}` to list contents.", rel_path);
    }
    let content = fs::read_to_string(target).await?;
    print!("{}", content);
    Ok(())
}

async fn handle_ls(vault_root: &Path, rel_path: Option<&str>) -> Result<()> {
    let target = if let Some(p) = rel_path {
        vault_root.join(p)
    } else {
        vault_root.to_path_buf()
    };

    if !target.starts_with(vault_root) {
        anyhow::bail!("Access denied: Path is outside the vault sanctuary.");
    }
    if !target.exists() {
        anyhow::bail!("Vault path not found: {}", rel_path.unwrap_or("."));
    }

    let mut entries = fs::read_dir(target).await?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        let suffix = if entry.file_type().await?.is_dir() { "/" } else { "" };
        println!("{}{}", name, suffix);
    }
    Ok(())
}

use crate::cli::VaultSecretAction;
async fn handle_secret_action(action: VaultSecretAction, config: &KoadConfig) -> Result<()> {
    match action {
        VaultSecretAction::Get { key } => {
            let val = config.resolve_secret(&key, None);
            if val.is_empty() {
                anyhow::bail!("Secret key '{}' not found in any scope.", key);
            }
            println!("{}", val);
        }
        VaultSecretAction::Ls => {
            println!("Available Secret Scopes (Local Environment):");
            // This is a partial list based on known KOADOS_ env vars
            for (k, _) in env::vars() {
                if k.starts_with("KOADOS_") || k == "GITHUB_PAT" {
                    println!("- {}", k);
                }
            }
        }
        VaultSecretAction::Set { .. } => {
            println!("\x1b[31m[DENIED]\x1b[0m `vault secret set` is an operator-only command. Please set environment variables manually.");
        }
    }
    Ok(())
}
