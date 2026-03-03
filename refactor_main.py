import sys

file_path = "/home/ideans/.koad-os/crates/koad-cli/src/main.rs"

with open(file_path, "r") as f:
    lines = f.readlines()

# 1. Identify the new match block start
start_index = -1
for i, line in enumerate(lines):
    if "match cli.command {" in line:
        start_index = i
        break

# 2. Identify the end of the main function (the first Ok(()) followed by a closing brace)
end_index = -1
for i in range(start_index, len(lines)):
    if "Ok(())" in lines[i] and i + 1 < len(lines) and "}" in lines[i+1]:
        end_index = i + 2
        break

if start_index == -1 or end_index == -1:
    print(f"Error: Could not find match boundaries. Start: {start_index}, End: {end_index}")
    sys.exit(1)

# 3. Construct the new implementation
# We will use the implementation logic we read earlier.
# This script will perform the structural replacement.

new_impl = """    match cli.command {
        Commands::Boot { agent, project, task: _, compact } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let current_path_str = current_dir.to_string_lossy().to_string();
            let (pat_var, _) = get_gh_pat_for_path(&current_dir, &role, &config);
            let (drive_var, _) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);
            let model_tier = detect_model_tier();
            let mut session_id = "BOOT".to_string();
            let mut mission_briefing = None;

            let (final_agent_name, final_role, final_bio) = if let Some(identity) = db.get_identity(&agent)? {
                if !db.verify_role(&agent, &role)? { anyhow::bail!("Identity '{}' is not authorized for the '{}' role.", agent, role); }
                if identity.tier < model_tier { anyhow::bail!("Cognitive Protection: Model Tier {} is insufficient for the '{}' identity (Minimum: Tier {}).", model_tier, agent, identity.tier); }
                (identity.name, role.clone(), identity.bio)
            } else {
                warn!("Identity '{}' not found in registry. Defaulting to Guest (restricted).", agent);
                (agent.clone(), "guest".to_string(), "Unverified Agent".to_string())
            };

            if !compact { println!(">>> [UPLINK] Acquiring Identity Lease for KAI '{}'...", agent); }
            let driver_id = if env::var("GEMINI_CLI").is_ok() { "gemini" } else if env::var("CODEX_CLI").is_ok() { "codex" } else { "cli" };
            let mut client = SpineServiceClient::connect(config.spine_grpc_addr.clone()).await.context("Failed to connect to Spine Backbone. Is the kernel awake?")?;
            let resp = client.initialize_session(InitializeSessionRequest {
                agent_name: agent.clone(),
                agent_role: role.clone(),
                project_name: if project { "active" } else { "default" }.to_string(),
                environment: EnvironmentType::Wsl as i32,
                driver_id: driver_id.to_string(),
                model_tier,
            }).await.map_err(|e| anyhow::anyhow!("Lease Denied: {}", e.message()))?;
            let package = resp.into_inner();
            session_id = package.session_id;
            mission_briefing = package.intelligence.map(|i| i.mission_briefing);

            if !compact {
                let conn = db.get_conn()?;
                let now_iso = Utc::now().to_rfc3339();
                let session_data = serde_json::json!({
                    "session_id": session_id,
                    "identity": { "name": final_agent_name.clone(), "rank": final_role.clone(), "permissions": if final_role.to_lowercase() == "admin" { vec!["all"] } else { vec!["limited"] }, "tier": model_tier },
                    "environment": "wsl",
                    "context": { "project_name": if project { "active" } else { "default" }, "root_path": current_path_str, "allowed_paths": [], "stack": [] },
                    "status": "active",
                    "last_heartbeat": now_iso,
                    "metadata": { "bio": final_bio.clone(), "model_tier": model_tier }
                });
                conn.execute("INSERT INTO sessions (session_id, agent, role, status, last_heartbeat, pid) VALUES (?1, ?2, ?3, 'active', ?4, ?5) ON CONFLICT(session_id) DO UPDATE SET last_heartbeat = excluded.last_heartbeat, status = 'active'", params![session_id, agent, final_role, now_iso, std::process::id()])?;
                let hb_session_id = session_id.clone();
                let hb_config = config.clone();
                let hb_db_path = db_path.clone();
                let mut hb_session_data = session_data.clone();
                tokio::spawn(async move {
                    let mut hb_client = match SpineServiceClient::connect(hb_config.spine_grpc_addr.clone()).await { Ok(c) => Some(c), Err(_) => None };
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                        let now_hb = Utc::now();
                        let now_hb_str = now_hb.to_rfc3339();
                        if let Some(ref mut c) = hb_client {
                            let mut req = tonic::Request::new(Empty {});
                            req.metadata_mut().insert("x-session-id", hb_session_id.parse().unwrap_or("".parse().unwrap()));
                            let _ = c.heartbeat(req).await;
                        }
                        if hb_config.redis_socket.exists() {
                            if let Ok(client) = redis::Client::open(format!("redis+unix://{}", hb_config.redis_socket.display())) {
                                if let Ok(mut con) = client.get_connection() {
                                    if let Some(obj) = hb_session_data.as_object_mut() { obj.insert("last_heartbeat".to_string(), serde_json::json!(now_hb_str)); }
                                    let _: Result<(), _> = redis::cmd("HSET").arg("koad:state").arg(format!("koad:session:{}", hb_session_id)).arg(hb_session_data.to_string()).query(&mut con);
                                }
                            }
                        }
                        if let Ok(conn) = rusqlite::Connection::open(&hb_db_path) { let _ = conn.execute("UPDATE sessions SET last_heartbeat = ?1 WHERE session_id = ?2", params![now_hb_str, hb_session_id]); }
                    }
                });
                println!("
[SESSIONS] KAI '{}' is now WAKE. Heartbeat active.", agent);
                println!("Press Ctrl+C to terminate session and go DARK.");
                tokio::signal::ctrl_c().await?;
                println!("
[SESSIONS] Terminating KAI session: {}...", agent);
                let now_iso = Utc::now().to_rfc3339();
                if let Ok(conn) = rusqlite::Connection::open(&db_path) { let _ = conn.execute("UPDATE sessions SET status = 'dark', last_heartbeat = ?1 WHERE session_id = ?2", params![now_iso, session_id]); }
                if config.redis_socket.exists() {
                    if let Ok(client) = redis::Client::open(format!("redis+unix://{}", config.redis_socket.display())) {
                        if let Ok(mut con) = client.get_connection() { let _: Result<(), _> = redis::cmd("HSET").arg("koad:state").arg(format!("koad:session:{}", session_id)).arg(serde_json::json!({"status": "dark", "last_heartbeat": now_iso}).to_string()).query(&mut con); }
                    }
                }
            }

            if compact {
                println!("I:{}|R:{}|G:{}|D:{}|T:{}|S:{}|Q:{}", final_agent_name, final_role, pat_var, drive_var, tags.join(","), session_id, model_tier);
            } else {
                println!("<koad_boot>
Session:  {}
Identity: {} ({})
Tier:     {}", session_id, final_agent_name, final_role, model_tier);
                println!("Bio:      {}", final_bio);
                if let Some(briefing) = mission_briefing { println!("
[MISSION BRIEFING]
{}", briefing); }
                println!("Auth: GH={} | GD={}", pat_var, drive_var);
                if let Some(driver) = legacy_config.drivers.get(&agent) {
                    let b_path = driver.bootstrap.replace("~", &env::var("HOME").unwrap_or_default());
                    if let Ok(content) = std::fs::read_to_string(b_path) { println!("
[BOOTSTRAP: {}]
{}", agent, content); }
                }
                println!("
[CONTEXT: {}]
Tags: {}", current_path_str, tags.join(", "));
                for (cat, content) in db.get_contextual(8, tags)? { println!("- [{}] {}", cat, content); }
                println!("
[Persona Reflections]");
                let ponders = db.get_ponderings(3)?;
                if ponders.is_empty() { println!("- No active reflections."); }
                for p in ponders { println!("- {}", p); }
                if project {
                    if let Some(proj) = db.get_project_by_path(&current_path_str)? {
                        println!("
[Project: {} (Stack: {})]", proj.name, proj.stack);
                        let progress_path = current_dir.join("PROJECT_PROGRESS.md");
                        if progress_path.exists() {
                            let p = std::fs::read_to_string(progress_path)?;
                            if let Some(s) = p.find("## Snapshot") { println!("
[Project Progress]
{}", p[s..].trim()); }
                        }
                    }
                }
                println!("</koad_boot>");
            }
        }

        Commands::System { action } => match action {
            SystemAction::Auth => {
                let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let (p, _) = get_gh_pat_for_path(&current_dir, &role, &config);
                let (d, _) = get_gdrive_token_for_path(&current_dir);
                println!("GH:{} | GD:{}", p, d);
            }
            SystemAction::Init { force: _ } => { feature_gate("koad init", Some(25)); }
            SystemAction::Config { json } => {
                if json {
                    let v = serde_json::json!({ "home": config.home, "redis_socket": config.redis_socket, "spine_socket": config.spine_socket, "spine_grpc_addr": config.spine_grpc_addr, "gateway_addr": config.gateway_addr, "db_path": config.get_db_path() });
                    println!("{}", v);
                } else { println!("{:#?}", config); }
            }
            SystemAction::Refresh { restart } => {
                if !is_admin { anyhow::bail!("Admin only."); }
                println!("
\x1b[1m--- KoadOS Core Refresh (Hard Reset) ---\x1b[0m");
                let home = config.home.clone();
                println!(">>> [1/3] Energizing Forge (cargo build --release)...");
                let build_status = Command::new("cargo").arg("build").arg("--release").current_dir(&home).status()?;
                if !build_status.success() { anyhow::bail!("Forge failure: Build failed with exit code {}", build_status.code().unwrap_or(-1)); }
                println!(">>> [2/3] Recalibrating Matrix (Redeploying binaries)...");
                let bin_dir = home.join("bin");
                let target_dir = home.join("target/release");
                let bins = vec![("koad", "koad"), ("koad-spine", "kspine"), ("koad-gateway", "kgateway"), ("koad-tui", "kdash")];
                for (src, dest) in bins {
                    let src_path = target_dir.join(src);
                    let dest_path = bin_dir.join(dest);
                    if src_path.exists() {
                        let old_path = dest_path.with_extension("old");
                        if dest_path.exists() { let _ = std::fs::rename(&dest_path, &old_path); }
                        if let Err(e) = std::fs::copy(&src_path, &dest_path) {
                            warn!("Failed to copy binary {}: {}", src, e);
                            if old_path.exists() { let _ = std::fs::rename(&old_path, &dest_path); }
                        } else {
                            println!("  [OK] Deployed: {}", dest);
                            if old_path.exists() { let _ = std::fs::remove_file(old_path); }
                        }
                    }
                }
                if restart {
                    println!(">>> [3/3] Rebooting Core Systems...");
                    let _ = Command::new("pkill").arg("-9").arg("kspine").status();
                    let _ = Command::new("pkill").arg("-9").arg("kgateway").status();
                    let _ = Command::new("nohup").arg(bin_dir.join("kspine")).arg("--home").arg(&home).stdout(Stdio::from(std::fs::File::create(home.join("spine.log"))?)).stderr(Stdio::from(std::fs::File::create(home.join("spine.log"))?)).spawn()?;
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let _ = Command::new("nohup").arg(bin_dir.join("kgateway")).arg("--addr").arg("0.0.0.0:3000").stdout(Stdio::from(std::fs::File::create(home.join("gateway.log"))?)).stderr(Stdio::from(std::fs::File::create(home.join("gateway.log"))?)).spawn()?;
                    println!("
\x1b[32m[CONDITION GREEN] KoadOS has been refreshed and rebooted.\x1b[0m");
                } else { println!("
\x1b[32m[DONE] Core binaries updated. Restart manually or use --restart.\x1b[0m"); }
            }
            SystemAction::Save { full } => {
                if !is_admin { anyhow::bail!("Admin only."); }
                println!("
\x1b[1m--- KoadOS Sovereign Save Protocol ---\x1b[0m");
                let home = config.home.clone();
                let now_ts = Local::now().format("%Y%m%d-%H%M%S").to_string();
                println!(">>> [1/4] Neuronal Flush (Spine Drain)...");
                match SpineServiceClient::connect(config.spine_grpc_addr.clone()).await {
                    Ok(mut client) => { if let Err(e) = client.drain_all(Empty {}).await { warn!("  [FAIL] Neuronal flush failed: {}. Continuing with local save.", e); } else { println!("  [OK] Hot-stream drained to durable memory."); } }
                    Err(_) => warn!("  [SKIP] Spine offline. Skipping hot-stream drain."),
                }
                println!(">>> [2/4] Archiving Identity (Mind Snapshot)...");
                let conn = db.get_conn()?;
                conn.execute("INSERT INTO identity_snapshots (trigger, notes, created_at) VALUES ('sovereign-save', 'Full system checkpoint.', ?1)", params![Local::now().to_rfc3339()])?;
                println!("  [OK] Persona state captured.");
                if full {
                    println!(">>> [3/4] Fortifying Memory (Database Backup)...");
                    let backup_dir = home.join("backups");
                    std::fs::create_dir_all(&backup_dir)?;
                    let backup_path = backup_dir.join(format!("koad-{}.db", now_ts));
                    std::fs::copy(home.join("koad.db"), &backup_path)?;
                    println!("  [OK] Database archived to: {}", backup_path.display());
                    println!(">>> [4/4] Finalizing Timeline (Git Checkpoint)...");
                    let m = format!("Sovereign Save: {}", now_ts);
                    let _ = Command::new("git").arg("-C").arg(&home).arg("add").arg(".").status();
                    let _ = Command::new("git").arg("-C").arg(&home).arg("commit").arg("-m").arg(&m).status();
                    println!("  [OK] System checkpoint committed to git.");
                } else {
                    println!(">>> [3/4] Skipping full backup (use --full for DB/Git checkpoint).");
                    println!(">>> [4/4] Log synchronization complete.");
                }
                println!("
\x1b[32m[CONDITION GREEN] Sovereign Save Complete.\x1b[0m");
            }
        },

        Commands::Intel { action } => match action {
            IntelAction::Query { term, limit, tags } => {
                let results = db.query(&term, limit, tags)?;
                for (id, cat, content, ts) in results { println!("- ID:{} [{}] ({}) {}", id, cat, ts, content); }
            }
            IntelAction::Remember { category } => {
                if !has_privileged_access { anyhow::bail!("Access Denied."); }
                let model_tier = detect_model_tier();
                let (cat_str, text, tags) = match category {
                    MemoryCategory::Fact { text, tags } => ("fact", text, tags),
                    MemoryCategory::Learning { text, tags } => ("learning", text, tags),
                };
                db.remember(cat_str, &text, tags, model_tier)?;
                println!("Memory updated in local KoadDB.");
            }
            IntelAction::Ponder { text, tags } => {
                let model_tier = detect_model_tier();
                db.remember("pondering", &text, Some(format!("persona-journal,{}", tags.unwrap_or_default())), model_tier)?;
                println!("Reflection recorded.");
            }
            IntelAction::Guide { topic: _ } => { feature_gate("koad guide", None); }
            IntelAction::Scan { path } => {
                let t = path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let output = Command::new("fdfind").arg(".koad").arg("--type").arg("d").arg("--hidden").arg("--absolute-path").arg(&t).output();
                match output {
                    Ok(out) if out.status.success() => {
                        let mut count = 0;
                        for line in String::from_utf8_lossy(&out.stdout).lines() {
                            if let Some(project_root) = PathBuf::from(line).parent() {
                                let name = project_root.file_name().unwrap_or_default().to_string_lossy();
                                if db.register_project(&name, &project_root.to_string_lossy()).is_ok() {
                                    println!("[PASS] Registered: {}", name);
                                    count += 1;
                                }
                            }
                        }
                        println!("Scan complete. {} projects registered.", count);
                    }
                    _ => {
                        if t.join(".koad").exists() {
                            let name = t.file_name().unwrap_or_default().to_string_lossy();
                            db.register_project(&name, &t.to_string_lossy())?;
                            println!("Project '{}' registered.", name);
                        }
                    }
                }
            }
            IntelAction::Mind { action } => {
                let conn = db.get_conn()?;
                match action {
                    MindAction::Status => {
                        println!("
\x1b[1m--- [INTROSPECT] Cognitive Health Status ---\x1b[0m");
                        let learn_count: i32 = conn.query_row("SELECT count(*) FROM learnings WHERE status = 'active'", [], |r| r.get(0))?;
                        let decision_count: i32 = conn.query_row("SELECT count(*) FROM decisions", [], |r| r.get(0))?;
                        let skill_count: i32 = conn.query_row("SELECT count(*) FROM skills", [], |r| r.get(0))?;
                        let last_snapshot: String = conn.query_row("SELECT created_at FROM identity_snapshots ORDER BY created_at DESC LIMIT 1", [], |r| r.get(0)).unwrap_or_else(|_| "Never".to_string());
                        println!("{:<25} {:<10}", "Active Learnings:", learn_count);
                        println!("{:<25} {:<10}", "Decisions Logged:", decision_count);
                        println!("{:<25} {:<10}", "Proven Skills:", skill_count);
                        println!("{:<25} {:<10}", "Last Identity Snapshot:", last_snapshot);
                        println!("
\x1b[1mTop Domains:\x1b[0m");
                        let mut stmt = conn.prepare("SELECT domain, count(*) as c FROM learnings GROUP BY domain ORDER BY c DESC LIMIT 5")?;
                        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i32>(1)?)))?;
                        for row in rows { let (domain, count) = row?; println!("  - {:<15} ({})", domain, count); }
                    }
                    MindAction::Snapshot => {
                        if !is_admin { anyhow::bail!("Admin only."); }
                        let now = Local::now().to_rfc3339();
                        conn.execute("INSERT INTO identity_snapshots (trigger, notes, created_at) VALUES ('manual', 'Manual session snapshot.', ?1)", params![now])?;
                        println!("\x1b[32m[SNAPSHOT]\x1b[0m Identity state archived.");
                    }
                    MindAction::Learn { domain, summary, detail } => {
                        let model_tier = detect_model_tier();
                        if model_tier > 1 { anyhow::bail!("Cognitive Protection: Model Tier {} is not authorized to add structured learnings.", model_tier); }
                        conn.execute("INSERT INTO learnings (domain, summary, detail, source, status) VALUES (?1, ?2, ?3, 'cli', 'active')", params![domain, summary, detail])?;
                        println!("\x1b[32m[LEARNED]\x1b[0m New {} insight integrated into mind.", domain);
                    }
                }
            }
        },

        Commands::Fleet { action } => match action {
            FleetAction::Board { action } => {
                let token = config.resolve_gh_token()?;
                let client = GitHubClient::new(token, "Fryymann".into(), "koad-os".into())?;
                let project_num = config.github_project_number as i32;
                match action {
                    BoardAction::Status { active } => {
                        println!(">>> [UPLINK] Accessing Neural Log: Tactical Overlay (Project #{})...", project_num);
                        let mut items = client.list_project_items(project_num).await?;
                        if active { items.retain(|i| i.status == "In Progress"); }
                        println!("
{:<5} {:<50} {:<15} {:<15}", "NODE", "DATA FRAGMENT", "STATUS", "VERSION");
                        println!("{:-<90}", "");
                        for item in items {
                            let num = item.number.map(|n| format!("#{}", n)).unwrap_or_default();
                            println!("{:<5} {:<50} {:<15} {:<15}", num, if item.title.len() > 48 { format!("{}...", &item.title[..45]) } else { item.title.clone() }, item.status, item.target_version.clone().unwrap_or_default());
                        }
                    }
                    BoardAction::Sync => { if !is_admin { anyhow::bail!("Admin Auth Required for Board Sync."); } client.sync_issues(project_num).await?; }
                    BoardAction::Done { id } => { if !is_admin { anyhow::bail!("Admin Auth Required to Close Nodes."); } client.update_item_status(project_num, id, "Done").await?; }
                    BoardAction::Todo { id } => { if !is_admin { anyhow::bail!("Admin Auth Required to Reopen Nodes."); } client.update_item_status(project_num, id, "Todo").await?; }
                    BoardAction::Verify { id } => {
                        println!(">>> [VERIFY] Cross-referencing Node #{} with Command Deck...", id);
                        let items = client.list_project_items(project_num).await?;
                        if let Some(item) = items.iter().find(|i| i.number == Some(id)) { println!("  [PASS] Node #{} verified. Current Status: {}", id, item.status); }
                        else { anyhow::bail!("Node #{} not found on Project Board. Manual sync required.", id); }
                    }
                    BoardAction::Sdr => { feature_gate("koad board sdr", None); }
                }
            }
            FleetAction::Project { action } => match action {
                ProjectAction::List => {
                    let projects = db.list_projects()?;
                    println!("
\x1b[1m--- KoadOS Master Project Map ---\x1b[0m");
                    println!("{:<4} {:<20} {:<15} {:<10} {}", "ID", "NAME", "BRANCH", "HEALTH", "PATH");
                    println!("{}", "-".repeat(80));
                    for (id, name, path, branch, health) in projects {
                        let health_color = match health.as_str() { "green" => "\x1b[32m", "yellow" => "\x1b[33m", "red" => "\x1b[31m", _ => "\x1b[0m" };
                        println!("{:<4} {:<20} {:<15} {}{:<10}\x1b[0m {}", id, name, branch, health_color, health, path);
                    }
                }
                ProjectAction::Register { name, path } => {
                    let p = path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                    let abs_path = std::fs::canonicalize(p)?;
                    db.register_project(&name, &abs_path.to_string_lossy())?;
                    println!("Project '{}' registered at {}.", name, abs_path.display());
                }
                ProjectAction::Sync { id } => {
                    let project_id = match id { Some(i) => i, None => {
                        let current_dir = env::current_dir()?;
                        let mut project_id = None;
                        let projects = db.list_projects()?;
                        for (id, _, path, _, _) in projects { if current_dir.to_string_lossy().starts_with(&path) { project_id = Some(id); break; } }
                        project_id.ok_or_else(|| anyhow::anyhow!("Not inside a registered project. Provide an ID."))?
                    }};
                    let (_, path, _, _, _) = db.get_project(project_id)?.ok_or_else(|| anyhow::anyhow!("Project not found"))?;
                    let branch_out = Command::new("git").arg("-C").arg(&path).arg("rev-parse").arg("--abbrev-ref").arg("HEAD").output();
                    let branch = if let Ok(out) = branch_out { if out.status.success() { Some(String::from_utf8_lossy(&out.stdout).trim().to_string()) } else { None } } else { None };
                    let health = if Path::new(&path).join("package.json").exists() || Path::new(&path).join("Cargo.toml").exists() || Path::new(&path).join("koad.json").exists() { Some("green".into()) } else { Some("unknown".into()) };
                    db.update_project_status(project_id, branch, health)?;
                    println!("Project #{} status updated.", project_id);
                }
                ProjectAction::Info { id } => {
                    if let Some((name, path, branch, health, last_sync)) = db.get_project(id)? {
                        println!("
\x1b[1m--- Project Info: {} ---\x1b[0m", name);
                        println!("{:<15} {}", "Path:", path);
                        println!("{:<15} {}", "Branch:", branch.unwrap_or_else(|| "unknown".into()));
                        println!("{:<15} {}", "Health:", health.unwrap_or_else(|| "unknown".into()));
                        println!("{:<15} {}", "Last Sync:", last_sync.unwrap_or_else(|| "never".into()));
                    } else { println!("Project #{} not found.", id); }
                }
                ProjectAction::Retire { id } => { db.retire_project(id)?; println!("Project #{} retired.", id); }
            }
            FleetAction::Issue { action } => {
                let conn = db.get_conn()?;
                match action {
                    IssueAction::Track { number, description } => {
                        let now = Local::now().to_rfc3339();
                        conn.execute("INSERT OR REPLACE INTO task_graph (description, created_at, status, canon_step, issue_number) VALUES (?1, ?2, 'todo', 1, ?3)", params![description, now, number])?;
                        println!("\x1b[32m[TRACKED]\x1b[0m Node #{} is now under Sovereignty tracking.", number);
                    }
                    IssueAction::Move { number, step } => {
                        if step < 1 || step > 8 { anyhow::bail!("Invalid step."); }
                        if step == 5 { anyhow::bail!("Access Denied: Step 5 requires Approval."); }
                        let mut stmt = conn.prepare("SELECT canon_step FROM task_graph WHERE issue_number = ?1")?;
                        let current_step: i32 = stmt.query_row([number], |row| row.get(0)).context("Issue not tracked.")?;
                        if step != current_step + 1 && step != current_step { anyhow::bail!("Protocol Violation: Sequence must be incremental."); }
                        conn.execute("UPDATE task_graph SET canon_step = ?1, updated_at = ?2 WHERE issue_number = ?3", params![step, Local::now().to_rfc3339(), number])?;
                        println!("\x1b[34m[MOVE]\x1b[0m Node #{} advanced to Step {}.", number, step);
                    }
                    IssueAction::Approve { number } => {
                        if !is_admin { anyhow::bail!("Access Denied."); }
                        let mut stmt = conn.prepare("SELECT canon_step FROM task_graph WHERE issue_number = ?1")?;
                        let current_step: i32 = stmt.query_row([number], |row| row.get(0))?;
                        let (new_step, label) = if current_step == 4 { (5, "Authorized for Implementation") } else if current_step == 8 { (9, "Verified and Authorized for Closure") } else { anyhow::bail!("Invalid State."); };
                        conn.execute("UPDATE task_graph SET canon_step = ?1, updated_at = ?2 WHERE issue_number = ?3", params![new_step, Local::now().to_rfc3339(), number])?;
                        println!("\x1b[35m[APPROVED]\x1b[0m Node #{} {}.", number, label);
                    }
                    IssueAction::Close { number } => {
                        let mut stmt = conn.prepare("SELECT canon_step FROM task_graph WHERE issue_number = ?1")?;
                        let current_step: i32 = stmt.query_row([number], |row| row.get(0)).context("Issue not tracked.")?;
                        if current_step != 9 { anyhow::bail!("LOCKED."); }
                        let token = config.resolve_gh_token()?;
                        let client = GitHubClient::new(token, "Fryymann".into(), "koad-os".into())?;
                        client.update_item_status(config.github_project_number as i32, number, "Done").await?;
                        conn.execute("UPDATE task_graph SET status = 'completed', updated_at = ?1 WHERE issue_number = ?2", params![Local::now().to_rfc3339(), number])?;
                        println!("\x1b[32m[FINALIZED]\x1b[0m Node #{} is closed.", number);
                    }
                    IssueAction::Status { number } => {
                        let mut stmt = conn.prepare("SELECT canon_step, description, status FROM task_graph WHERE issue_number = ?1")?;
                        let (step, desc, status): (i32, String, String) = stmt.query_row([number], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
                        println!("
\x1b[1m--- Sovereignty Status: Node #{} ---\x1b[0m", number);
                        println!("Description: {}
Status:      {}
Canon Step:  {} / 9", desc, status, step);
                    }
                }
            }
        },

        Commands::Bridge { action } => match action {
            BridgeAction::Gcloud { .. } => { feature_gate("koad bridge gcloud", None); }
            BridgeAction::Airtable { .. } => { feature_gate("koad bridge airtable", None); }
            BridgeAction::Sync { .. } => { feature_gate("koad bridge sync", None); }
            BridgeAction::Drive { .. } => { feature_gate("koad bridge drive", None); }
            BridgeAction::Stream { .. } => { feature_gate("koad bridge stream", None); }
            BridgeAction::Skill { .. } => { feature_gate("koad bridge skill", None); }
            BridgeAction::Publish { message } => {
                if !is_admin { anyhow::bail!("Admin only."); }
                let h = config.home.clone();
                let m = message.unwrap_or_else(|| format!("KoadOS Sync - {}", Local::now().format("%Y-%m-%d %H:%M")));
                Command::new("git").arg("-C").arg(&h).arg("add").arg(".").spawn()?.wait()?;
                Command::new("git").arg("-C").arg(&h).arg("commit").arg("-m").arg(&m).spawn()?.wait()?;
                Command::new("git").arg("-C").arg(&h).arg("push").arg("origin").spawn()?.wait()?;
                println!("Published.");
            }
        },

        Commands::Status { json: _, full } => {
            println!("
\x1b[1m--- [TELEMETRY] Neural Link & Grid Integrity ---\x1b[0m");
            print!("{:<30}", "Engine Room (Redis):");
            if config.redis_socket.exists() {
                match redis::Client::open(format!("redis+unix://{}", config.redis_socket.display())) {
                    Ok(client) => {
                        if let Ok(mut con) = client.get_connection() {
                            let _: String = redis::cmd("PING").query(&mut con).unwrap_or_else(|_| "FAIL".into());
                            println!("\x1b[32m[PASS]\x1b[0m Hot-stream energized.");
                        } else { println!("\x1b[31m[FAIL]\x1b[0m Ghost Socket Detected."); }
                    }
                    Err(_) => println!("\x1b[31m[FAIL]\x1b[0m Client initialization failed."),
                }
            } else { println!("\x1b[31m[FAIL]\x1b[0m Neural Bus missing."); }

            print!("{:<30}", "Backbone (Spine):");
            if config.home.join("kspine.sock").exists() { println!("\x1b[32m[PASS]\x1b[0m Neural bus active."); }
            else { println!("\x1b[33m[WARN]\x1b[0m Orchestrator link severed."); }

            if full {
                let ghosts = find_ghosts(&config.home);
                if !ghosts.is_empty() {
                    println!("
\x1b[33m[WARN] Ghost Processes Detected ({}):\x1b[0m", ghosts.len());
                    for (pid, info) in ghosts { println!("  - PID {}: {}", pid, info); }
                }
                if config.redis_socket.exists() {
                    if let Ok(mut con) = redis::Client::open(format!("redis+unix://{}", config.redis_socket.display()))?.get_connection() {
                        let res: Option<String> = redis::cmd("HGET").arg("koad:state").arg("system_stats").query(&mut con)?;
                        if let Some(s) = res {
                            let v: serde_json::Value = serde_json::from_str(&s)?;
                            println!("
\x1b[1m--- Resource Allocation ---\x1b[0m");
                            println!("CPU Usage: {:.1}%
Memory:    {} MB", v["cpu_usage"].as_f64().unwrap_or(0.0), v["memory_usage"].as_u64().unwrap_or(0));
                        }
                    }
                }
            }
            println!("\x1b[1m---------------------------------------------------\x1b[0m
");
        }

        Commands::Whoami => {
            println!("Persona: {} ({})
Bio:     {}", legacy_config.identity.name, legacy_config.identity.role, legacy_config.identity.bio);
        }

        Commands::Dash => { crate::tui::run_dash(&db)?; }
    }
    Ok(())
}
"""

lines_to_keep_before = lines[:start_index]
lines_to_keep_after = lines[end_index:]

# Filter out dangling legacy arms from lines_to_keep_after
filtered_after = []
in_legacy_block = True
for line in lines_to_keep_after:
    if "#[cfg(test)]" in line:
        in_legacy_block = False
    if not in_legacy_block:
        filtered_after.append(line)

with open(file_path, "w") as f:
    f.writelines(lines_to_keep_before)
    f.write(new_impl)
    f.writelines(filtered_after)
