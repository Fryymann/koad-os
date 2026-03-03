import sys

file_path = "/home/ideans/.koad-os/crates/koad-cli/src/main.rs"

with open(file_path, "r") as f:
    lines = f.readlines()

start_idx = -1
for i, line in enumerate(lines):
    if "match cli.command {" in line:
        start_idx = i
        break

end_idx = -1
for i in range(start_idx, len(lines)):
    if "#[cfg(test)]" in line: 
        end_idx = i
        break

if start_idx == -1 or end_idx == -1:
    print(f"Error: Boundaries not found. Start: {start_idx}, End: {end_idx}")
    sys.exit(1)

new_match_block = """    match cli.command {
        Commands::Boot { agent, project, task: _, compact } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (pat_var, _) = get_gh_pat_for_path(&current_dir, &role, &config);
            let (drive_var, _) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);
            let model_tier = detect_model_tier();
            let mut session_id = "BOOT".to_string();

            let (final_agent_name, final_role, final_bio) = if let Some(identity) = db.get_identity(&agent)? {
                if !db.verify_role(&agent, &role)? { anyhow::bail!("Access Denied."); }
                (identity.name, role.clone(), identity.bio)
            } else {
                (agent.clone(), "guest".to_string(), "Unverified Agent".to_string())
            };

            let mut client = SpineServiceClient::connect(config.spine_grpc_addr.clone()).await.context("Connect failed.")?;
            let resp = client.initialize_session(InitializeSessionRequest {
                agent_name: agent.clone(),
                agent_role: role.clone(),
                project_name: "default".into(),
                environment: EnvironmentType::Wsl as i32,
                driver_id: "cli".into(),
                model_tier,
            }).await.map_err(|e| anyhow::anyhow!("Denied: {}", e.message()))?;
            
            session_id = resp.into_inner().session_id;

            if !compact {
                println!("<koad_boot>\\nSession: {}\\nIdentity: {} ({})\\n</koad_boot>", session_id, final_agent_name, final_role);
            } else {
                println!("I:{}|R:{}|S:{}", final_agent_name, final_role, session_id);
            }
        }

        Commands::System { action } => match action {
            SystemAction::Auth => { println!("Auth requested."); }
            SystemAction::Init { force: _ } => { feature_gate("koad init", None); }
            SystemAction::Config { json: _ } => { println!("Config requested."); }
            SystemAction::Refresh { restart: _ } => { println!("Refresh requested."); }
            SystemAction::Save { full: _ } => { println!("Save requested."); }
        },

        Commands::Intel { action } => match action {
            IntelAction::Query { term, limit: _, tags: _ } => { println!("Querying for {}...", term); }
            _ => { println!("Intel action placeholder."); }
        },

        Commands::Fleet { action } => match action {
            _ => { println!("Fleet action placeholder."); }
        },

        Commands::Bridge { action } => match action {
            _ => { println!("Bridge action placeholder."); }
        },

        Commands::Status { json: _, full: _ } => {
            println!("--- [TELEMETRY] KoadOS System Health ---");
        }

        Commands::Whoami => {
            println!("Persona: {} ({})\\nBio: {}", legacy_config.identity.name, legacy_config.identity.role, legacy_config.identity.bio);
        }

        Commands::Dash => { crate::tui::run_dash(&db)?; }
    }
    Ok(())
}
"""

with open(file_path, "w") as f:
    f.writelines(lines[:start_idx])
    f.write(new_match_block)
    f.write("\n")
    f.writelines(lines[end_idx:])
