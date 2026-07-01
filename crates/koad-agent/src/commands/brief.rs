use anyhow::{Context, Result};
use koad_core::config::KoadConfig;

pub async fn handle_brief(config: &KoadConfig, task: &str) -> Result<()> {
    let agent_name = std::env::var("KOAD_AGENT_NAME").unwrap_or_else(|_| config.get_agent_name());
    let vault_uri = config
        .resolve_vault_uri(&agent_name)
        .context("Could not resolve vault URI for current agent.")?;
    let vault_path = config.resolve_vault_path(&vault_uri)?;

    println!("\x1b[34m[BRIEFING]\x1b[0m Synthesizing tactical context for task...");

    let collector = crate::handlers::abc::AbcCollector::new(
        config.clone(),
        vault_path.to_path_buf(),
        agent_name.clone(),
    );
    let raw_data = collector.collect_raw_data().await?;

    let generator = crate::handlers::abc::AbcGenerator::new()?;
    let brief = generator.generate_task_brief(&raw_data, &agent_name, task).await?;

    println!("\n# 📋 TASK BRIEF: {}\n", task);
    println!("{}", brief);
    println!("\n\x1b[2m--- Briefing Complete ---\x1b[0m");

    Ok(())
}
