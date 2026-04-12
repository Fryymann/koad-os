use anyhow::{bail, Context, Result};
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

    // 4. redis.active.conf (runtime-hydrated from template)
    let active_conf_path = home.join("run/redis.active.conf");
    if !active_conf_path.exists() || force {
        let template_path = home.join("config/defaults/redis.conf.template");
        if template_path.exists() {
            fs::create_dir_all(home.join("run"))?;
            let content = fs::read_to_string(&template_path)?;
            let hydrated = content.replace("{{KOAD_HOME}}", &home.to_string_lossy());
            fs::write(&active_conf_path, hydrated)?;
            println!("  \x1b[32m✓\x1b[0m Initialized run/redis.active.conf with hydrated paths");
        } else {
            println!("  \x1b[33m⚠\x1b[0m redis.conf.template not found, skipping Redis config generation");
        }
    }

    // 5. Interactive Captain Creation
    println!("\n\x1b[1m[Captain Identity Setup]\x1b[0m");
    println!("Every Citadel needs a Captain. Let's create yours.");
    
    print!("  Captain's Name (e.g. Tyr): ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim();

    if !name.is_empty() {
        print!("  Short Bio / Mission: ");
        io::stdout().flush()?;
        let mut bio = String::new();
        io::stdin().read_line(&mut bio)?;
        let bio = bio.trim();

        println!("  Select Runtime Body:");
        println!("    1. Claude Code (claude)");
        println!("    2. Gemini CLI (gemini)");
        println!("    3. Codex CLI (codex)");
        print!("  Selection [1-3]: ");
        io::stdout().flush()?;
        let mut rt_choice = String::new();
        io::stdin().read_line(&mut rt_choice)?;
        let runtime = match rt_choice.trim() {
            "2" => "gemini",
            "3" => "codex",
            _ => "claude",
        };

        println!("\n  \x1b[1;34m→\x1b[0m Provisioning Captain {}...", name);
        
        // We use a simplified version of handle_new_agent logic here
        // or we could potentially call it if it was public and async-aware.
        // For init, we'll do a basic scaffold.
        if let Err(e) = provision_captain(name, bio, runtime, config) {
            println!("  \x1b[31m✗\x1b[0m Failed to provision Captain: {}", e);
        } else {
            println!("  \x1b[32m✓\x1b[0m Captain {} is ready for duty.", name);
        }
    } else {
        println!("  \x1b[33m⚠\x1b[0m No name provided. Skipping Captain creation. You can run 'koad agent new' later.");
    }

    println!("\n\x1b[1;32m[SUCCESS]\x1b[0m System initialization complete.");
    println!("Next: Run 'koad system doctor' to verify provider connectivity.");

    Ok(())
}

fn provision_captain(name: &str, bio: &str, runtime: &str, config: &KoadConfig) -> Result<()> {
    let key = name.to_lowercase();
    let identities_dir = config.home.join("config/identities");
    let identity_toml_path = identities_dir.join(format!("{}.toml", key));

    if identity_toml_path.exists() {
        bail!("Identity for '{}' already exists at {}.", name, identity_toml_path.display());
    }

    let vault_str = format!("~/.koad-os/agents/{}", key);
    let access_keys_toml = "access_keys = [\"GITHUB_PAT\"]"; // Default for Captain

    let toml_content = format!(
        r#"[identities.{key}]
name = "{name}"
role = "Captain and Systems Architect"
rank = "Captain"
tier = 4
xp = 0
bio = "{bio}"
vault = "{vault_str}"
bootstrap = "{vault_str}/identity/IDENTITY.md"
runtime = "{runtime}"

[identities.{key}.preferences]
{access_keys_toml}

[identities.{key}.session_policy]
mode = "proactive"
timeout_minutes = 240
auto_saveup = true
"#,
        key = key,
        name = name,
        bio = bio,
        vault_str = vault_str,
        runtime = runtime,
        access_keys_toml = access_keys_toml,
    );

    fs::create_dir_all(&identities_dir)?;
    fs::write(&identity_toml_path, toml_content)?;
    
    // However, for a good UX, we should at least create the vault dir.
    let _home_dir = dirs::home_dir().context("Could not determine home directory.")?;
    let vault_path = config.home.join(format!("agents/{}", key));
    fs::create_dir_all(&vault_path)?;

    Ok(())
}
