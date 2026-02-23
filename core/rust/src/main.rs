use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;
use std::process::Command;
use chrono::Local;
use std::io::{BufRead, BufReader};

#[derive(Debug, Serialize, Deserialize)]
pub struct KoadConfig {
    pub version: String,
    pub identity: Identity,
    pub preferences: Preferences,
    pub memory: Memory,
    pub drivers: HashMap<String, DriverConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub role: String,
    pub bio: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preferences {
    pub languages: Vec<String>,
    pub style: String,
    pub principles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Memory {
    pub global_facts: Vec<Fact>,
    #[serde(default)]
    pub learnings: Vec<Fact>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fact {
    pub id: String,
    pub text: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverConfig {
    pub bootstrap: String,
    #[serde(default)]
    pub mcp_enabled: bool,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Parser)]
#[command(name = "koad")]
#[command(version = "2.0.0")]
#[command(about = "The KoadOS Control Plane")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Boot {
        #[arg(short, long, default_value = "gemini")]
        agent: String,
        #[arg(short, long)]
        project: bool,
    },
    Auth,
    Query {
        term: String,
    },
    Remember {
        #[command(subcommand)]
        category: MemoryCategory,
    },
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    Init {
        #[arg(short, long)]
        force: bool,
    },
    Harvest {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum MemoryCategory {
    Fact { text: String },
    Learning { text: String },
}

#[derive(Subcommand)]
enum SkillAction {
    List,
    Run {
        name: String,
        #[arg(last = true)]
        args: Vec<String>,
    },
}

impl KoadConfig {
    pub fn get_path() -> Result<PathBuf> {
        let base = env::var("KOAD_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".koad-os"));
        Ok(base.join("koad.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;
        if !path.exists() {
            return Ok(Self::default_initial());
        }
        let content = std::fs::read_to_string(path)?;
        let mut cfg: Self = serde_json::from_str(&content).context("Failed to parse koad.json")?;
        
        // Override with Env Vars for Anonymization/Public Safety
        if let Ok(val) = env::var("KOAD_NAME") { cfg.identity.name = val; }
        if let Ok(val) = env::var("KOAD_ROLE") { cfg.identity.role = val; }
        if let Ok(val) = env::var("KOAD_BIO") { cfg.identity.bio = val; }
        
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content).context("Failed to write koad.json")
    }

    pub fn default_initial() -> Self {
        Self {
            version: "2.0".to_string(),
            identity: Identity {
                name: env::var("KOAD_NAME").unwrap_or_else(|_| "Koad".into()),
                role: env::var("KOAD_ROLE").unwrap_or_else(|_| "AI Persona".into()),
                bio: env::var("KOAD_BIO").unwrap_or_else(|_| "Agnostic AI coding framework.".into()),
            },
            preferences: Preferences {
                languages: vec!["Rust".into(), "Node.js".into(), "Python".into()],
                style: "programmatic-first".to_string(),
                principles: vec!["Simplicity first".into(), "Plan before build".into()],
            },
            memory: Memory { global_facts: vec![], learnings: vec![] },
            drivers: HashMap::new(),
        }
    }

    pub fn add_learning(&mut self, text: String) {
        let id = format!("l_{}", self.memory.learnings.len() + 1);
        let timestamp = Local::now().format("%Y-%m-%d").to_string();
        self.memory.learnings.push(Fact { id, text, timestamp });
    }
}

fn get_gh_pat_for_path(path: &Path) -> (&'static str, &'static str) {
    let path_str = path.to_string_lossy();
    if path_str.contains("skylinks") {
        ("GITHUB_SKYLINKS_PAT", "Work (Skylinks)")
    } else {
        ("GITHUB_PERSONAL_PAT", "Personal")
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = KoadConfig::load()?;

    match cli.command {
        Commands::Boot { agent: _, project } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (pat_var, pat_desc) = get_gh_pat_for_path(&current_dir);
            
            println!("<koad_boot>");
            println!("Identity: {} ({})", config.identity.name, config.identity.role);
            println!("Bio: {}", config.identity.bio);
            println!("Auth: {} ({})", pat_var, pat_desc);
            
            println!("\n[Recent Memory]");
            for fact in config.memory.global_facts.iter().rev().take(5) {
                println!("- [Fact] {}", fact.text);
            }
            for learning in config.memory.learnings.iter().rev().take(5) {
                println!("- [Learning] {}", learning.text);
            }

            if project {
                let progress_path = current_dir.join("PROJECT_PROGRESS.md");
                if progress_path.exists() {
                    println!("\n[Project Progress Snapshot]");
                    let progress = std::fs::read_to_string(progress_path)?;
                    if let Some(start) = progress.find("## Snapshot") {
                        let end = progress.find("## Roadmap Alignment").unwrap_or(progress.len());
                        println!("{}", progress[start..end].trim());
                    }
                }
            }
            println!("</koad_boot>");
        }
        Commands::Auth => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (pat_var, pat_desc) = get_gh_pat_for_path(&current_dir);
            println!("Context: {} | Env: {}", pat_desc, pat_var);
        }
        Commands::Query { term } => {
            let term = term.to_lowercase();
            for fact in &config.memory.global_facts {
                if fact.text.to_lowercase().contains(&term) {
                    println!("- [Fact] [{}] {}", fact.timestamp, fact.text);
                }
            }
            for learning in &config.memory.learnings {
                if learning.text.to_lowercase().contains(&term) {
                    println!("- [Learning] [{}] {}", learning.timestamp, learning.text);
                }
            }
        }
        Commands::Remember { category } => {
            let timestamp = Local::now().format("%Y-%m-%d").to_string();
            match category {
                MemoryCategory::Fact { text } => {
                    let id = format!("f_{}", config.memory.global_facts.len() + 1);
                    config.memory.global_facts.push(Fact { id, text: text.clone(), timestamp });
                }
                MemoryCategory::Learning { text } => {
                    let id = format!("l_{}", config.memory.learnings.len() + 1);
                    config.memory.learnings.push(Fact { id, text: text.clone(), timestamp });
                }
            }
            config.save()?;
            println!("Memory updated.");
        }
        Commands::Skill { action } => {
             let base = env::var("KOAD_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".koad-os"));
             let skills_dir = base.join("skills");
             match action {
                 SkillAction::List => {
                     if skills_dir.exists() {
                        for entry in std::fs::read_dir(skills_dir)? {
                            let entry = entry?;
                            if entry.path().is_dir() {
                                let category = entry.file_name().to_string_lossy().to_string();
                                for skill in std::fs::read_dir(entry.path())? {
                                    let skill = skill?;
                                    println!("- {}/{}", category, skill.file_name().to_string_lossy());
                                }
                            }
                        }
                     }
                 },
                 SkillAction::Run { name, args } => {
                     let skill_path = skills_dir.join(&name);
                     let mut child = Command::new(&skill_path).args(args).spawn()?;
                     child.wait()?;
                 }
             }
        }
        Commands::Init { force } => {
            let path = KoadConfig::get_path()?;
            if path.exists() && !force { anyhow::bail!("Exists."); }
            KoadConfig::default_initial().save()?;
            println!("Initialized.");
        }
        Commands::Harvest { path } => {
            let file = std::fs::File::open(&path)?;
            let reader = BufReader::new(file);
            let mut in_discovery_section = false;
            let mut count = 0;
            for line in reader.lines() {
                let line = line?;
                let trimmed = line.trim();
                if trimmed.starts_with("## Discoveries") || trimmed.starts_with("## Learnings") {
                    in_discovery_section = true;
                    continue;
                } else if trimmed.starts_with("## ") && in_discovery_section {
                    break;
                }
                if in_discovery_section && (trimmed.starts_with("- ") || trimmed.starts_with("* ")) {
                    let learning = trimmed[2..].to_string();
                    if !learning.is_empty() {
                        config.add_learning(learning);
                        count += 1;
                    }
                }
            }
            if count > 0 {
                config.save()?;
                println!("Harvested {} learnings.", count);
            }
        }
    }

    Ok(())
}
