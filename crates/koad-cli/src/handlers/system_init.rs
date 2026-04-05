use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub fn run(config: &KoadConfig, force: bool) -> Result<()> {
    let home = &config.home;
    println!("\x1b[1;36m[KoadOS]\x1b[0m Initializing KoadOS environment at: {}" , home.display());

    // 1. Directory Structure
    let dirs = [
        "bin",
        "logs",
        "cache",
        "data/db",
        "data/redis",
        "run",
        "config/integrations",
        "config/identities",
        "config/projects",
    ];

    for dir in dirs {
        let path = home.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path).with_context(|| format!("Failed to create directory: {:?}", path))?;
            println!("  \x1b[32m✓\x1b[0m Created {}", dir);
        }
    }

    // 2. .env file
    let env_path = home.join(".env");
    if !env_path.exists() || force {
        println!("\n\x1b[1m[Environment Setup]\x1b[0m");
        let mut env_content = String::new();
        
        // Try to read existing template from current dir or home
        let template_path = PathBuf::from(".env.template");
        if template_path.exists() {
            env_content = fs::read_to_string(template_path)?;
        }

        println!("Setting up AI Provider keys (press Enter to skip):");
        
        let providers = [
            ("GOOGLE_AI_API_KEY", "Google AI (Gemini)"),
            ("ANTHROPIC_API_KEY", "Anthropic (Claude)"),
            ("OPENAI_API_KEY", "OpenAI (Codex)"),
        ];

        for (key, label) in providers {
            print!("  {} API Key: ", label);
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if !input.is_empty() {
                if env_content.contains(&format!("{}=", key)) {
                    // Update existing line
                    let lines: Vec<String> = env_content.lines().map(|line| {
                        if line.starts_with(&format!("{}=", key)) {
                            format!("{}={}", key, input)
                        } else {
                            line.to_string()
                        }
                    }).collect();
                    env_content = lines.join("\n");
                } else {
                    // Append new line
                    env_content.push_str(&format!("\n{}={}", key, input));
                }
            }
        }

        fs::write(&env_path, env_content)?;
        println!("  \x1b[32m✓\x1b[0m Environment saved to {}", env_path.display());
    }

    // 3. kernel.toml
    let kernel_path = home.join("config/kernel.toml");
    if !kernel_path.exists() || force {
        let template = PathBuf::from("config/defaults/kernel.toml");
        if template.exists() {
            fs::copy(&template, &kernel_path)?;
            println!("  \x1b[32m✓\x1b[0m Initialized config/kernel.toml from default template");
        } else {
            println!("  \x1b[33m⚠\x1b[0m Default kernel.toml template not found. Please create one manually.");
        }
    }

    // 4. redis.conf (with path hydration)
    let redis_path = home.join("config/redis.conf");
    if !redis_path.exists() || force {
        let template_path = PathBuf::from("config/defaults/redis.conf");
        if template_path.exists() {
            let content = fs::read_to_string(template_path)?;
            let hydrated = content.replace("{{KOAD_HOME}}", &home.to_string_lossy());
            fs::write(&redis_path, hydrated)?;
            println!("  \x1b[32m✓\x1b[0m Initialized config/redis.conf with hydrated paths");
        }
    }

    println!("\n\x1b[1;32m[SUCCESS]\x1b[0m System initialization complete.");
    println!("Next: Run 'koad system doctor' to verify provider connectivity.");

    Ok(())
}
