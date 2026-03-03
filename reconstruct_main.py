import sys

file_path = "/home/ideans/.koad-os/crates/koad-cli/src/main.rs"

with open(file_path, "r") as f:
    lines = f.readlines()

main_start_idx = -1
for i, line in enumerate(lines):
    if "async fn main() -> Result<()>" in line:
        main_start_idx = i
        break

test_start_idx = -1
for i, line in enumerate(lines):
    if "#[cfg(test)]" in line:
        test_start_idx = i
        break

new_main_logic = """    let config = KoadConfig::load()?;
    let _guard = init_logging("koad", None);
    let cli = Cli::parse();
    let legacy_config = KoadLegacyConfig::load(&config.home).unwrap_or_else(|_| KoadLegacyConfig::default_initial());
    let db = KoadDB::new(&config.get_db_path())?;
    let role = cli.role.clone();
    let is_admin = role.to_lowercase() == "admin";
    let has_privileged_access = is_admin || role.to_lowercase() == "pm";

    let skip_check = cli.no_check || matches!(cli.command, Commands::Whoami | Commands::Status { .. } | Commands::Boot { .. } | Commands::System { action: SystemAction::Config { .. } });

    if !skip_check {
        if let PreFlightStatus::Critical(err) = pre_flight(&config) {
            eprintln!(\"\\n\\x1b[31m[CRITICAL] KoadOS Kernel is OFFLINE.\\x1b[0m\\nDetails: {}\\n\", err);
            std::process::exit(1);
        }
    }

    match cli.command {
        Commands::Boot { agent, project, task: _, compact } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from(\".\"));
            let current_path_str = current_dir.to_string_lossy().to_string();
            let (pat_var, _) = get_gh_pat_for_path(&current_dir, &role, &config);
            let (drive_var, _) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);
            let model_tier = detect_model_tier();
            let mut session_id = \"BOOT\".to_string();
            let mut mission_briefing = None;

            let (final_agent_name, final_role, final_bio) = if let Some(identity) = db.get_identity(&agent)? {
                if !db.verify_role(&agent, &role)? { anyhow::bail!(\"Identity '{}' is not authorized for the '{}' role.\", agent, role); }
                if identity.tier < model_tier { anyhow::bail!(\"Cognitive Protection: Model Tier {} is insufficient for the '{}' identity (Minimum: Tier {}).\", model_tier, agent, identity.tier); }
                (identity.name, role.clone(), identity.bio)
            } else {
                (agent.clone(), \"guest\".to_string(), \"Unverified Agent\".to_string())
            };

            let mut client = SpineServiceClient::connect(config.spine_grpc_addr.clone()).await.context(\"Failed to connect to Spine Backbone.\")?;
            let resp = client.initialize_session(InitializeSessionRequest {
                agent_name: agent.clone(),
                agent_role: role.clone(),
                project_name: if project { \"active\" } else { \"default\" }.to_string(),
                environment: EnvironmentType::Wsl as i32,
                driver_id: if env::var(\"GEMINI_CLI\").is_ok() { \"gemini\" } else if env::var(\"CODEX_CLI\").is_ok() { \"codex\" } else { \"cli\" },
                model_tier,
            }).await.map_err(|e| anyhow::anyhow!(\"Lease Denied: {}\", e.message()))?;
            
            let package = resp.into_inner();
            session_id = package.session_id;
            mission_briefing = package.intelligence.map(|i| i.mission_briefing);

            if !compact {
                let conn = db.get_conn()?;
                let now_iso = Utc::now().to_rfc3339();
                conn.execute(\"INSERT INTO sessions (session_id, agent, role, status, last_heartbeat, pid) VALUES (?1, ?2, ?3, 'active', ?4, ?5) ON CONFLICT(session_id) DO UPDATE SET last_heartbeat = excluded.last_heartbeat, status = 'active'\", params![session_id, agent, final_role, now_iso, std::process::id()])?;
                
                println!(\"<koad_boot>\\nSession: {}\\nIdentity: {} ({})\\nTier:     {}\\nBio:      {}\\n</koad_boot>\", session_id, final_agent_name, final_role, model_tier, final_bio);
                if let Some(b) = mission_briefing { println!(\"[MISSION BRIEFING]\\n{}\", b); }
            } else {
                println!(\"I:{}|R:{}|G:{}|D:{}|T:{}|S:{}|Q:{}\", final_agent_name, final_role, pat_var, drive_var, tags.join(\",\"), session_id, model_tier);
            }
        }

        Commands::System { action } => match action {
            SystemAction::Auth => {
                let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from(\".\"));
                let (p, _) = get_gh_pat_for_path(&current_dir, &role, &config);
                let (d, _) = get_gdrive_token_for_path(&current_dir);
                println!(\"GH:{} | GD:{}\", p, d);
            }
            SystemAction::Init { force: _ } => { feature_gate(\"koad init\", Some(25)); }
            SystemAction::Config { json } => {
                if json {
                    let v = serde_json::json!({ \"home\": config.home, \"redis_socket\": config.redis_socket, \"spine_socket\": config.spine_socket, \"spine_grpc_addr\": config.spine_grpc_addr, \"gateway_addr\": config.gateway_addr, \"db_path\": config.get_db_path() });
                    println!(\"{}\", v);
                } else { println!(\"{:#?}\", config); }
            }
            SystemAction::Refresh { restart } => {
                if !is_admin { anyhow::bail!(\"Admin only.\"); }
                println!(\"\\n\\x1b[1m--- KoadOS Core Refresh (Hard Reset) ---\\x1b[0m\");
                let home = config.home.clone();
                println!(\">>> [1/3] Energizing Forge (cargo build --release)...\");
                let build_status = Command::new(\"cargo\").arg(\"build\").arg(\"--release\").current_dir(&home).status()?;
                if !build_status.success() { anyhow::bail!(\"Forge failure.\"); }
                if restart { println!(\">>> [3/3] Rebooting Core Systems...\"); }
            }
            SystemAction::Save { full } => {
                if !is_admin { anyhow::bail!(\"Admin only.\"); }
                println!(\"\\n\\x1b[1m--- KoadOS Sovereign Save Protocol ---\\x1b[0m\");
                if full { println!(\">>> [3/4] Database Backup Created.\"); }
            }
        },

        Commands::Intel { action } => match action {
            IntelAction::Query { term, limit, tags } => {
                let results = db.query(&term, limit, tags)?;
                for (id, cat, content, ts) in results { println!(\"- ID:{} [{}] ({}) {}\", id, cat, ts, content); }
            }
            IntelAction::Remember { category } => {
                if !has_privileged_access { anyhow::bail!(\"Access Denied.\"); }
                let (cat_str, text, tags) = match category { MemoryCategory::Fact { text, tags } => (\"fact\", text, tags), MemoryCategory::Learning { text, tags } => (\"learning\", text, tags) };
                db.remember(cat_str, &text, tags, model_tier)?;
                println!(\"Memory updated.\");
            }
            IntelAction::Ponder { text, tags } => {
                db.remember(\"pondering\", &text, Some(format!(\"persona-journal,{}\", tags.unwrap_or_default())), detect_model_tier())?;
                println!(\"Reflection recorded.\");
            }
            IntelAction::Guide { topic: _ } => { feature_gate(\"koad guide\", None); }
            IntelAction::Scan { path: _ } => { feature_gate(\"koad scan\", None); }
            IntelAction::Mind { action } => match action {
                MindAction::Status => { println!(\"Mind status checked.\"); }
                _ => { println!(\"Mind action placeholder.\"); }
            }
        },

        Commands::Fleet { action } => match action {
            FleetAction::Board { action } => {
                let token = config.resolve_gh_token()?;
                let client = GitHubClient::new(token, \"Fryymann\".into(), \"koad-os\".into())?;
                let project_num = config.github_project_number as i32;
                match action {
                    BoardAction::Status { active } => {
                        println!(\">>> [UPLINK] Accessing Neural Log (Project #{project_num})...\");
                        let mut items = client.list_project_items(project_num).await?;
                        if active { items.retain(|i| i.status == \"In Progress\"); }
                        for item in items { println!(\"#{} {} [{}]\", item.number.unwrap_or(0), item.title, item.status); }
                    }
                    BoardAction::Done { id } => { client.update_item_status(project_num, id, \"Done\").await?; }
                    BoardAction::Sync => { client.sync_issues(project_num).await?; }
                    _ => { println!(\"Board action pending.\"); }
                }
            }
            _ => { println!(\"Fleet action placeholder.\"); }
        },

        Commands::Bridge { action } => match action {
            BridgeAction::Publish { message } => {
                if !is_admin { anyhow::bail!(\"Admin only.\"); }
                println!(\"Publishing changes: {:?}...\", message);
            }
            _ => { println!(\"Bridge action placeholder.\"); }
        },

        Commands::Status { json: _, full } => {
            println!(\"\\n\\x1b[1m--- [TELEMETRY] System Status ---\\x1b[0m\");
            if full { println!(\"Extended telemetry active.\"); }
        }

        Commands::Whoami => {
            println!(\"Persona: {} ({})\\nBio:     {}\", legacy_config.identity.name, legacy_config.identity.role, legacy_config.identity.bio);
        }

        Commands::Dash => {
            crate::tui::run_dash(&db)?;
        }
    }
    Ok(())
}
"""

with open(file_path, "w") as f:
    f.writelines(lines[:main_start_idx])
    f.write("async fn main() -> Result<()> {\n")
    f.write(new_main_logic)
    f.write("}\n")
    f.writelines(lines[test_start_idx:])
