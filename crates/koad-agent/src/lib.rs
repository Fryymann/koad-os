use anyhow::Result;
use clap::{Parser, Subcommand};
use koad_core::config::KoadConfig;
use std::path::PathBuf;

pub mod handlers { pub mod abc; }
pub mod commands;

#[derive(Parser)]
#[command(name = "koad-agent")]
#[command(about = "KoadOS Agent Bootstrap Tool: Neural link hydration and pre-flight.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Bootstrap an agent session with identity and environment hydration.
    Boot {
        /// The name of the agent to boot.
        #[arg(short, long)]
        agent: Option<String>,
        /// Position name of the agent to boot (legacy compatibility).
        name: Option<String>,
        /// Generate output suitable for shell evaluation.
        #[arg(short, long, default_value_t = true)]
        shell: bool,
    },
    /// Verify the integrity of an agent's personal vault (KAPV).
    Verify {
        /// The name of the agent vault to verify.
        agent: String,
    },
    /// Display summary information for an agent identity.
    Info {
        /// The name of the agent to inspect.
        agent: String,
    },
    /// Generate a tactical brief for a sub-agent task.
    Brief {
        /// Description of the task or path to a task manifest.
        task: String,
    },
    /// Generate a high-density context packet for a crate.
    Context {
        /// Name of the crate (e.g. "koad-core", "koad-cass").
        crate_name: String,
        /// Output path (defaults to ./<crate_name>.context.md).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Validate and register a task manifest for the current agent.
    Task {
        /// Path to the task manifest (.md or .toml file).
        manifest: PathBuf,
        /// Release the current active task (mark as complete).
        #[arg(long)]
        done: bool,
    },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = KoadConfig::load()?;

    match cli.command {
        Commands::Boot { agent, name, shell } => {
            commands::handle_boot(&config, agent, name, shell).await?;
        }
        Commands::Verify { agent } => {
            commands::handle_verify(agent, &config).await?;
        }
        Commands::Info { agent } => {
            commands::handle_info(agent).await?;
        }
        Commands::Brief { task } => {
            commands::handle_brief(&config, &task).await?;
        }
        Commands::Context { crate_name, output } => {
            commands::handle_context(&config, &crate_name, output).await?;
        }
        Commands::Task { manifest, done } => {
            commands::handle_task(&config, manifest, done).await?;
        }
    }
    Ok(())
}
