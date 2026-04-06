//! Sandbox handler — run commands in isolated Docker/Podman containers.

use anyhow::Result;
use koad_sandbox::container::{ContainerConfig, ContainerSandbox};
use std::time::Duration;

pub async fn handle_sandbox_run(
    command: Vec<String>,
    image: String,
    network: bool,
    memory: String,
    podman: bool,
) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("No command specified. Usage: koad sandbox <command>");
    }

    let runtime = if podman { "podman" } else { "docker" }.to_string();
    let cmd = command.join(" ");

    println!("\x1b[1;34m[SANDBOX]\x1b[0m Starting isolated container...");
    println!("  Runtime: {}", runtime);
    println!("  Image:   {}", image);
    println!("  Network: {}", if network { "enabled" } else { "isolated" });
    println!("  Command: {}", cmd);
    println!();

    let config = ContainerConfig {
        image,
        runtime,
        memory_limit: memory,
        cpu_limit: "0.5".to_string(),
        allow_network: network,
        read_only_mounts: vec![],
        timeout: Duration::from_secs(60),
    };

    match ContainerSandbox::new(config).execute(&cmd).await {
        Ok(result) => {
            if !result.stdout.is_empty() {
                print!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            if result.exit_code == 0 {
                println!("\x1b[32m[OK]\x1b[0m Sandbox complete in {}ms.", result.duration_ms);
            } else {
                println!("\x1b[31m[FAIL]\x1b[0m Exit code: {}", result.exit_code);
            }
        }
        Err(e) => {
            // Graceful degradation — Docker/Podman may not be available
            println!("\x1b[33m[DEGRADED]\x1b[0m Sandbox unavailable: {}", e);
            println!("  Ensure Docker or Podman is running and in PATH.");
        }
    }
    Ok(())
}
