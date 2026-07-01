use anyhow::Result;
use koad_core::config::KoadConfig;
use std::path::PathBuf;
use tokio::fs;

pub async fn handle_task(config: &KoadConfig, manifest: PathBuf, done: bool) -> Result<()> {
    let run_dir = config.home.join("run");
    fs::create_dir_all(&run_dir).await?;
    let tasks_file = run_dir.join("tasks.json");

    // Load existing tasks state
    let mut tasks: serde_json::Value = if tasks_file.exists() {
        let raw = fs::read_to_string(&tasks_file).await?;
        serde_json::from_str(&raw).unwrap_or(serde_json::json!({"active": []}))
    } else {
        serde_json::json!({"active": []})
    };

    let agent_name = std::env::var("KOAD_AGENT_NAME").unwrap_or_else(|_| config.get_agent_name());
    let agent_role = std::env::var("KOAD_AGENT_ROLE").unwrap_or_else(|_| "unknown".to_string());
    let worktree = std::env::current_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if done {
        // Release active task for this agent
        if let Some(arr) = tasks["active"].as_array_mut() {
            arr.retain(|t| t["agent"].as_str() != Some(&agent_name));
        }
        fs::write(&tasks_file, serde_json::to_string_pretty(&tasks)?).await?;
        println!(
            "\x1b[32m[DONE]\x1b[0m Task released for agent '{}'.",
            agent_name
        );
        return Ok(());
    }

    // Validate manifest exists
    if !manifest.exists() {
        println!(
            "\x1b[31m[BLOCKED]\x1b[0m Manifest not found: {}",
            manifest.display()
        );
        return Ok(());
    }

    // Parse manifest — extract key fields from markdown frontmatter or content
    let content = fs::read_to_string(&manifest).await?;

    // Extract task ID (look for "Task:" or filename stem)
    let task_id = manifest
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Extract required role from manifest content (look for "**Agent:**" or "## ASSIGNMENT")
    let required_agent = content
        .lines()
        .find(|l| l.contains("**Agent:**") || l.contains("Agent:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().trim_matches('*').trim().to_lowercase())
        .unwrap_or_default();

    // Check worktree collision: any active task in the same worktree?
    let mut blockers: Vec<String> = vec![];

    if let Some(active) = tasks["active"].as_array() {
        for t in active {
            let t_worktree = t["worktree"].as_str().unwrap_or("");
            let t_agent = t["agent"].as_str().unwrap_or("");
            let t_id = t["task_id"].as_str().unwrap_or("");

            if t_worktree == worktree && t_agent != agent_name {
                blockers.push(format!(
                    "Worktree collision: agent '{}' is working on '{}' in this worktree.",
                    t_agent, t_id
                ));
            }
            if t_agent == agent_name {
                blockers.push(format!(
                    "Agent '{}' already has active task '{}'. Use --done to release.",
                    agent_name, t_id
                ));
            }
        }
    }

    // Role check (fuzzy — check if agent name or role is in the required field)
    if !required_agent.is_empty() {
        let name_match = agent_name.to_lowercase().contains(&required_agent)
            || required_agent.contains(&agent_name.to_lowercase());
        let role_match = agent_role.to_lowercase().contains(&required_agent);
        if !name_match && !role_match && required_agent != "unknown" {
            blockers.push(format!(
                "Role mismatch: manifest requires '{}', agent is '{}' ({}).",
                required_agent, agent_name, agent_role
            ));
        }
    }

    // Validate referenced files exist (look for file paths in the manifest)
    let missing_files: Vec<String> = content
        .lines()
        .filter(|l| l.contains("src/") && (l.contains(".rs") || l.contains(".toml")))
        .filter_map(|l| {
            // Extract a path-like token
            l.split_whitespace()
                .find(|t| t.contains("src/") && t.contains(".rs"))
                .map(|t| {
                    t.trim_matches(
                        &['`', '*', '(', ')', '[', ']', '\'', '"', ':', ',', '.'] as &[char]
                    )
                    .to_string()
                })
        })
        .filter(|p| {
            let full = config
                .home
                .join("crates")
                .join(p.trim_start_matches("crates/"));
            !p.is_empty() && !full.exists()
        })
        .collect();

    if !missing_files.is_empty() {
        for f in &missing_files {
            blockers.push(format!("Referenced file not found: {}", f));
        }
    }

    if blockers.is_empty() {
        // Register active task
        let entry = serde_json::json!({
            "task_id": task_id,
            "agent": agent_name,
            "role": agent_role,
            "worktree": worktree,
            "manifest": manifest.to_string_lossy(),
            "registered_at": chrono::Utc::now().to_rfc3339(),
        });

        if let Some(arr) = tasks["active"].as_array_mut() {
            arr.push(entry);
        }
        fs::write(&tasks_file, serde_json::to_string_pretty(&tasks)?).await?;

        println!(
            "\x1b[32m[READY]\x1b[0m Task '{}' validated and registered.",
            task_id
        );
        println!("       Agent:    {}", agent_name);
        println!("       Worktree: {}", worktree);
    } else {
        println!(
            "\x1b[31m[BLOCKED]\x1b[0m Task '{}' cannot proceed:",
            task_id
        );
        for reason in &blockers {
            println!("  - {}", reason);
        }
    }

    Ok(())
}
