//! # KoadOS Vault Command Handler
//!
//! Implements the command-based access layer for agent identity,
//! config, and credentials.

use anyhow::{Context, Result};
use koad_core::config::{KoadConfig, SkillBlueprint, SkillInstance};
use koad_core::skills::SkillScanner;
use std::env;
use std::path::Path;
use tokio::fs;

use crate::cli::{VaultAction, VaultSkillAction};

/// Entry point for all `koad vault` commands.
pub async fn handle_vault_action(action: VaultAction, config: &KoadConfig) -> Result<()> {
    let agent_name = env::var("KOAD_AGENT_NAME").unwrap_or_else(|_| "Admiral".to_string());
    let vault_uri = config
        .resolve_vault_uri(&agent_name)
        .context("Could not resolve vault URI for current agent.")?;
    let vault_path = config.resolve_vault_path(&vault_uri)?;

    match action {
        VaultAction::Whoami => handle_whoami(&agent_name, &vault_uri, &vault_path).await,
        VaultAction::Env => handle_env(&agent_name, config).await,
        VaultAction::Get { path } => handle_get(&vault_path, &path).await,
        VaultAction::Ls { path } => handle_ls(&vault_path, path.as_deref()).await,
        VaultAction::Secret { action } => handle_secret_action(action, config).await,
        VaultAction::Skill { action } => {
            handle_vault_skill_action(action, &agent_name, config, &vault_path).await
        }
        _ => {
            println!(
                "\x1b[33m[STUB]\x1b[0m This vault action is not yet implemented in Dark Mode."
            );
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
        anyhow::bail!(
            "Item is a directory. Use `koad vault ls {}` to list contents.",
            rel_path
        );
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
        let suffix = if entry.file_type().await?.is_dir() {
            "/"
        } else {
            ""
        };
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

async fn handle_vault_skill_action(
    action: VaultSkillAction,
    agent_name: &str,
    config: &KoadConfig,
    vault_path: &Path,
) -> Result<()> {
    match action {
        VaultSkillAction::List => skill_list(agent_name, config).await,
        VaultSkillAction::Info { id } => skill_info(&id, agent_name, config).await,
        VaultSkillAction::Search => skill_search(agent_name, config).await,
        VaultSkillAction::Equip { id } => skill_equip(&id, agent_name, config, vault_path).await,
        VaultSkillAction::Sync => skill_sync(agent_name, config).await,
    }
}

async fn skill_list(agent_name: &str, config: &KoadConfig) -> Result<()> {
    let key = agent_name.to_lowercase();
    let identity = config
        .identities
        .get(&key)
        .with_context(|| format!("Agent '{}' not found in config.", agent_name))?;

    if identity.skills.is_empty() {
        println!("No skills equipped for agent '{}'.", agent_name);
        println!("Run `koad vault skill search` to discover available blueprints.");
        return Ok(());
    }

    println!("{:<20} {:<8} {:<6}", "Skill ID", "Level", "XP");
    println!("{}", "-".repeat(38));
    for inst in &identity.skills {
        println!(
            "{:<20} {:<8} {:<6}",
            inst.blueprint_id, inst.level, inst.current_xp
        );
    }
    Ok(())
}

async fn skill_info(id: &str, agent_name: &str, config: &KoadConfig) -> Result<()> {
    let key = agent_name.to_lowercase();
    let identity = config
        .identities
        .get(&key)
        .with_context(|| format!("Agent '{}' not found in config.", agent_name))?;

    let instance = identity
        .skills
        .iter()
        .find(|s| s.blueprint_id == id)
        .with_context(|| {
            format!(
                "Skill '{}' is not equipped. Use `koad vault skill equip {}`.",
                id, id
            )
        })?;

    // Try to load the blueprint for full details
    let scanner = SkillScanner::new(&config.home);
    let blueprints = scanner.scan()?;
    let blueprint = blueprints.iter().find(|b| b.id == id);

    println!("=== Skill: {} ===", id);
    println!("Level:    {}", instance.level);
    println!("XP:       {}", instance.current_xp);

    if let Some(bp) = blueprint {
        println!("Name:     {}", bp.name);
        println!("Desc:     {}", bp.description);
        println!("Version:  {}", bp.version);
        println!("Runtime:  {:?}", bp.runtime);
        println!(
            "Entry:    {}",
            if bp.entry_point.is_empty() {
                "(builtin)"
            } else {
                &bp.entry_point
            }
        );
        if bp.capabilities.is_empty() {
            println!("Caps:     (none)");
        } else {
            println!("Caps:     {}", bp.capabilities.join(", "));
        }
    } else {
        println!("(Blueprint not found in skills/ directory — showing instance data only)");
    }

    if !instance.settings.is_empty() {
        println!("Settings:");
        for (k, v) in &instance.settings {
            println!("  {} = {}", k, v);
        }
    }
    Ok(())
}

async fn skill_search(agent_name: &str, config: &KoadConfig) -> Result<()> {
    let key = agent_name.to_lowercase();
    let equipped_ids: std::collections::HashSet<String> = config
        .identities
        .get(&key)
        .map(|id| id.skills.iter().map(|s| s.blueprint_id.clone()).collect())
        .unwrap_or_default();

    let scanner = SkillScanner::new(&config.home);
    let blueprints = scanner.scan()?;

    let available: Vec<&SkillBlueprint> = blueprints
        .iter()
        .filter(|b| !equipped_ids.contains(&b.id))
        .collect();

    if available.is_empty() {
        println!(
            "No new skill blueprints available. All discovered blueprints are already equipped."
        );
        return Ok(());
    }

    println!("{:<20} {:<25} {:<10}", "ID", "Name", "Runtime");
    println!("{}", "-".repeat(58));
    for bp in &available {
        println!("{:<20} {:<25} {:<10?}", bp.id, bp.name, bp.runtime);
    }
    println!(
        "\nFound {} available blueprint(s). Use `koad vault skill equip <id>` to equip.",
        available.len()
    );
    Ok(())
}

async fn skill_equip(
    id: &str,
    agent_name: &str,
    config: &KoadConfig,
    vault_path: &Path,
) -> Result<()> {
    // Load blueprint
    let scanner = SkillScanner::new(&config.home);
    let blueprints = scanner.scan()?;
    let blueprint = blueprints.iter().find(|b| b.id == id).with_context(|| {
        format!(
            "Blueprint '{}' not found. Run `koad vault skill search` to see available skills.",
            id
        )
    })?;

    // Check not already equipped
    let key = agent_name.to_lowercase();
    if let Some(identity) = config.identities.get(&key) {
        if identity.skills.iter().any(|s| s.blueprint_id == id) {
            println!(
                "Skill '{}' is already equipped by agent '{}'.",
                id, agent_name
            );
            return Ok(());
        }
    }

    // Validate capabilities
    koad_core::skills::validate_capabilities(&blueprint.capabilities)?;

    // Confirmation step
    if blueprint.capabilities.is_empty() {
        println!(
            "Skill '{}' requires no special capabilities.",
            blueprint.name
        );
    } else {
        println!(
            "Skill '{}' requires the following capabilities:",
            blueprint.name
        );
        for cap in &blueprint.capabilities {
            println!("  - {}", cap);
        }
    }
    println!("\nProceed with equip? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
        println!("Equip cancelled.");
        return Ok(());
    }

    // Scaffold vault skill directory
    let skill_dir = vault_path.join("skills").join(id);
    tokio::fs::create_dir_all(&skill_dir)
        .await
        .with_context(|| {
            format!(
                "Failed to scaffold vault skill dir: {}",
                skill_dir.display()
            )
        })?;

    // Atomic write to identity TOML
    let identity_path = config.home.join(format!("config/identities/{}.toml", key));
    anyhow::ensure!(
        identity_path.exists(),
        "Identity TOML not found: {}. Cannot equip skill.",
        identity_path.display()
    );

    let raw = std::fs::read_to_string(&identity_path)
        .with_context(|| format!("Reading {}", identity_path.display()))?;
    let mut doc: toml::Value =
        toml::from_str(&raw).with_context(|| format!("Parsing {}", identity_path.display()))?;

    // Navigate to identities.{name}.skills array
    let instance_value = toml::Value::try_from(SkillInstance {
        blueprint_id: id.to_string(),
        level: 0,
        current_xp: 0,
        settings: std::collections::HashMap::new(),
    })?;

    if let Some(identities) = doc.get_mut("identities") {
        if let Some(agent_table) = identities.get_mut(&key) {
            let skills_arr = agent_table
                .as_table_mut()
                .context("identity entry is not a TOML table")?
                .entry("skills")
                .or_insert_with(|| toml::Value::Array(vec![]));
            if let toml::Value::Array(arr) = skills_arr {
                arr.push(instance_value);
            }
        } else {
            anyhow::bail!("Identity '{}' not found inside TOML identities table.", key);
        }
    } else {
        anyhow::bail!("TOML has no [identities] section.");
    }

    // Serialize and atomic write
    let new_content =
        toml::to_string_pretty(&doc).context("Failed to serialize updated identity TOML")?;
    let tmp_path = identity_path.with_extension("toml.tmp");
    std::fs::write(&tmp_path, &new_content).context("Failed to write temp identity file")?;
    std::fs::rename(&tmp_path, &identity_path)
        .context("Failed to atomically replace identity TOML")?;

    println!("Skill '{}' equipped for agent '{}'.", id, agent_name);
    println!("Vault directory scaffolded: {}", skill_dir.display());
    Ok(())
}

async fn skill_sync(agent_name: &str, _config: &KoadConfig) -> Result<()> {
    // Dark-mode: operate locally only; gRPC sync is a future Phase 5 feature
    println!("[skill sync] Agent: {}", agent_name);
    println!("Dark Mode: Citadel sync is not available offline.");
    println!(
        "XP state is persisted locally in config/identities/{}.toml.",
        agent_name.to_lowercase()
    );
    println!("Tip: Use `koad vault skill equip` to add skills and `koad vault skill list` to review them.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use koad_core::config::{AgentIdentityConfig, SkillInstance};
    use std::collections::HashMap;

    fn make_identity(skills: Vec<SkillInstance>) -> AgentIdentityConfig {
        AgentIdentityConfig {
            name: "test-agent".to_string(),
            role: "Tester".to_string(),
            rank: "Crew".to_string(),
            bio: "Test bio".to_string(),
            vault: None,
            vault_uri: None,
            bootstrap: None,
            preferences: None,
            runtime: None,
            tier: 3,
            xp: 0,
            skills,
        }
    }

    #[test]
    fn test_skill_list_empty_is_coherent() {
        let identity = make_identity(vec![]);
        assert!(
            identity.skills.is_empty(),
            "fresh identity should have no skills"
        );
    }

    #[test]
    fn test_skill_instance_fields_roundtrip() {
        let inst = SkillInstance {
            blueprint_id: "hello-world".to_string(),
            level: 2,
            current_xp: 150,
            settings: HashMap::from([("mode".to_string(), "verbose".to_string())]),
        };
        let identity = make_identity(vec![inst]);
        let found = identity
            .skills
            .iter()
            .find(|s| s.blueprint_id == "hello-world");
        assert!(
            found.is_some(),
            "equipped skill should be findable by blueprint_id"
        );
        let s = found.unwrap();
        assert_eq!(s.level, 2);
        assert_eq!(s.current_xp, 150);
        assert_eq!(s.settings.get("mode").map(String::as_str), Some("verbose"));
    }

    #[test]
    fn test_skill_search_filter_logic() {
        // Simulate the search filter: blueprints not in equipped_ids should appear
        use std::collections::HashSet;
        let equipped: HashSet<String> = vec!["hello-world".to_string()].into_iter().collect();
        let all_ids = ["hello-world", "notion-sync", "git-assist"];
        let available: Vec<&str> = all_ids
            .iter()
            .copied()
            .filter(|id| !equipped.contains(*id))
            .collect();
        assert_eq!(available.len(), 2);
        assert!(!available.contains(&"hello-world"));
        assert!(available.contains(&"notion-sync"));
    }
}
