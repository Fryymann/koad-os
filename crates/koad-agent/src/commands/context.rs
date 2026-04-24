use anyhow::Result;
use koad_core::config::KoadConfig;
use std::path::PathBuf;
use tokio::fs;

pub async fn handle_context(
    config: &KoadConfig,
    crate_name: &str,
    output: Option<PathBuf>,
) -> Result<()> {
    use std::process::Command as StdCommand;

    let out_path = output.unwrap_or_else(|| PathBuf::from(format!("{}.context.md", crate_name)));

    // Locate crate directory
    let crate_dir = config.home.join("crates").join(crate_name);
    let crate_exists = crate_dir.exists();

    let mut packet = format!(
        "# Context Packet: {}\nGenerated: {}\n\n",
        crate_name,
        chrono::Utc::now().format("%Y-%m-%d")
    );

    // 1. Crate purpose from lib.rs or main.rs doc comment
    if crate_exists {
        let candidates = ["src/lib.rs", "src/main.rs"];
        for candidate in &candidates {
            let candidate_path = crate_dir.join(candidate);
            if candidate_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&candidate_path) {
                    let doc_lines: String = content
                        .lines()
                        .take_while(|l| l.starts_with("//!") || l.trim().is_empty())
                        .map(|l| l.trim_start_matches("//!").trim())
                        .filter(|l| !l.is_empty())
                        .take(10)
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !doc_lines.is_empty() {
                        packet.push_str(&format!("## Purpose\n{}\n\n", doc_lines));
                    }
                }
                break;
            }
        }
    }

    // 2. Public symbols via koad-codegraph
    if crate_exists {
        match koad_codegraph::CodeGraph::new_with_memory() {
            Ok(graph) => {
                if let Ok(()) = graph.index_project(&crate_dir) {
                    if let Ok(summary) = graph.get_crate_summary(&crate_dir.to_string_lossy()) {
                        if !summary.is_empty() {
                            packet.push_str("## Public API\n");
                            packet.push_str(&summary);
                            packet.push('\n');
                        }
                    }
                }
            }
            Err(e) => {
                packet.push_str(&format!(
                    "## Public API\n_Symbol extraction unavailable: {}_\n\n",
                    e
                ));
            }
        }
    } else {
        packet.push_str(&format!(
            "## Note\nCrate directory not found at `{}`. Generating from git log only.\n\n",
            crate_dir.display()
        ));
    }

    // 3. Recent git history for this crate
    let git_log = StdCommand::new("git")
        .args([
            "log",
            "--oneline",
            "-10",
            "--",
            &format!("crates/{}/", crate_name),
        ])
        .current_dir(&config.home)
        .output();

    match git_log {
        Ok(out) if out.status.success() => {
            let log_str = String::from_utf8_lossy(&out.stdout);
            if !log_str.trim().is_empty() {
                packet.push_str("## Recent Git Activity\n```\n");
                packet.push_str(log_str.trim());
                packet.push_str("\n```\n\n");
            }
        }
        _ => {
            packet.push_str("## Recent Git Activity\n_Git log unavailable._\n\n");
        }
    }

    // 4. Cargo.toml dependencies (key deps, not all)
    let cargo_toml_path = crate_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        if let Ok(toml_content) = std::fs::read_to_string(&cargo_toml_path) {
            let dep_lines: Vec<&str> = toml_content
                .lines()
                .skip_while(|l| !l.contains("[dependencies]"))
                .skip(1)
                .take_while(|l| !l.starts_with('['))
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .take(15)
                .collect();
            if !dep_lines.is_empty() {
                packet.push_str("## Key Dependencies\n```toml\n");
                packet.push_str(&dep_lines.join("\n"));
                packet.push_str("\n```\n");
            }
        }
    }

    fs::write(&out_path, &packet).await?;
    println!(
        "\x1b[32m[OK]\x1b[0m Context packet written to: {}",
        out_path.display()
    );
    println!("     Crate:  {}", crate_name);
    println!("     Size:   {} bytes", packet.len());
    Ok(())
}
