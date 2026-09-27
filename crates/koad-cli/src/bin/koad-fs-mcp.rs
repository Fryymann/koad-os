//! koad-fs-mcp: per-agent filesystem MCP server (stdio).
//!
//! Resolves the directories the current agent may access (global
//! `[filesystem] allowed_directories`, the agent's own
//! `preferences.allowed_directories`, and its vault), then replaces this
//! process with the official `@modelcontextprotocol/server-filesystem`
//! restricted to those roots. Stdout belongs to the MCP protocol, so all
//! diagnostics go to stderr.
//!
//! Usage: KOAD_AGENT_NAME=<agent> koad-fs-mcp [--print-roots]

use anyhow::{bail, Context, Result};
use koad_core::config::KoadConfig;
use koad_core::fs_mcp::{resolve_roots, FILESYSTEM_SERVER_PACKAGE};
use std::os::unix::process::CommandExt;
use std::process::Command;

fn main() -> Result<()> {
    let print_only = std::env::args().any(|a| a == "--print-roots");
    let config = KoadConfig::load().context("Failed to load KoadOS config")?;
    let agent = std::env::var("KOAD_AGENT_NAME")
        .context("KOAD_AGENT_NAME is not set; koad-fs-mcp scopes access per agent")?
        .to_lowercase();
    let identity = config.identities.get(&agent);
    if identity.is_none() {
        eprintln!("koad-fs-mcp: no identity '{agent}'; using global directories only");
    }
    let agent_dirs = identity
        .and_then(|i| i.preferences.as_ref())
        .map(|p| p.allowed_directories.clone())
        .unwrap_or_default();
    let vault = identity.and_then(|i| i.vault.as_deref());
    let home = dirs::home_dir().context("Could not determine the home directory")?;

    let (roots, warnings) = resolve_roots(
        &home,
        &config.filesystem.allowed_directories,
        &agent_dirs,
        vault,
    );
    for w in &warnings {
        eprintln!("koad-fs-mcp: {w}");
    }
    for protected in &config.filesystem.protected_paths {
        let protected = home.join(protected.trim_start_matches("~/"));
        if roots.iter().any(|r| protected.starts_with(r)) {
            eprintln!(
                "koad-fs-mcp: warning: protected path {} is inside an allowed root",
                protected.display()
            );
        }
    }
    if roots.is_empty() {
        bail!(
            "no accessible directories for agent '{agent}'; check [filesystem] allowed_directories"
        );
    }

    if print_only {
        for r in &roots {
            println!("{}", r.display());
        }
        return Ok(());
    }

    let err = Command::new("npx")
        .arg("-y")
        .arg(FILESYSTEM_SERVER_PACKAGE)
        .args(&roots)
        .exec();
    Err(err).context("Failed to start the filesystem MCP server via npx")
}
