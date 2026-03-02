        Commands::Boot { agent, project, task: _, compact } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let current_path_str = current_dir.to_string_lossy().to_string();
            let (pat_var, _) = get_gh_pat_for_path(&current_dir, &role);
            let (drive_var, _) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);

            let mut session_id = "BOOT".to_string();
            let mut mission_briefing = None;

            if !compact {
                // 1. Role Arbitration
                let conn = db.get_conn()?;
                let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                session_id = uuid::Uuid::new_v4().to_string();

                if role.to_lowercase() == "admin" {
                    // Check for active admin sessions in the last 2 minutes
                    let cutoff = (Local::now() - chrono::Duration::minutes(2)).format("%Y-%m-%d %H:%M:%S").to_string();
                    let mut stmt = conn.prepare("SELECT session_id, agent FROM sessions WHERE role = 'admin' AND last_heartbeat > ?1 AND status = 'active'")?;
                    let active_admin = stmt.query_row([cutoff], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)));

                    if let Ok((old_sid, old_agent)) = active_admin {
                        if old_agent == agent {
                            session_id = old_sid;
                        } else {
                            anyhow::bail!("Admin role is currently occupied by {} (Session: {}). Only one Admin allowed.", old_agent, old_sid);
                        }
                    }
                }

                // 2. Register/Refresh Session in Koad State Bus (Redis)
                let session_data = serde_json::json!({
                    "session_id": session_id,
                    "identity": {
                        "name": agent,
                        "rank": role,
                        "permissions": ["all"]
                    },
                    "environment": "wsl",
                    "context": {
                        "project_name": if project { "active" } else { "default" },
                        "root_path": current_path_str,
                        "allowed_paths": [],
                        "stack": []
                    },
                    "last_heartbeat": chrono::Utc::now().to_rfc3339(),
                    "metadata": {}
                });

                let redis_conn = KoadConfig::get_home()?.join("koad.sock");
                if redis_conn.exists() {
                    let mut client = redis::Client::open(format!("redis+unix://{}", redis_conn.display()))?;
                    if let Ok(mut con) = client.get_connection() {
                        let _: () = redis::cmd("HSET")
                            .arg("koad:state")
                            .arg(format!("koad:session:{}", session_id))
                            .arg(session_data.to_string())
                            .query(&mut con)?;
                        
                        let _: () = redis::cmd("PUBLISH")
                            .arg("koad:sessions")
                            .arg(serde_json::json!({
                                "type": "SESSION_UPDATE",
                                "payload": session_data
                            }).to_string())
                            .query(&mut con)?;

                        // Wait for Spine to hydrate
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        if let Ok(res) = redis::cmd("HGET")
                            .arg("koad:state")
                            .arg(format!("koad:session:{}", session_id))
                            .query::<Option<String>>(&mut con) 
                        {
                            if let Some(json_str) = res {
                                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                                    if let Some(briefing) = data.get("mission_briefing") {
                                        mission_briefing = Some(briefing.as_str().unwrap_or_default().to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                // 2.1 Direct SQL log
                conn.execute(
                    "INSERT INTO sessions (session_id, agent, role, status, last_heartbeat, pid)
                     VALUES (?1, ?2, ?3, 'active', ?4, ?5)
                     ON CONFLICT(session_id) DO UPDATE SET last_heartbeat = excluded.last_heartbeat, status = 'active'",
                    params![session_id, agent, role, now, std::process::id()],
                )?;

                // 3. Sidecar Booster
                if config.preferences.booster_enabled {
                    let booster_path = KoadConfig::get_home()?.join("bin/kbooster");
                    if booster_path.exists() {
                         let log_path = KoadConfig::get_home()?.join(format!("booster_{}.log", config.identity.name.to_lowercase()));
                         let log_file = std::fs::OpenOptions::new().append(true).create(true).open(&log_path)?;
                         let _ = Command::new(booster_path)
                            .arg("--agent-id").arg(&config.identity.name)
                            .arg("--role").arg(&config.identity.role)
                            .stdin(Stdio::null())
                            .stdout(Stdio::from(log_file.try_clone()?))
                            .stderr(Stdio::from(log_file))
                            .spawn();
                    }
                }
            }

            if compact {
                println!("I:{}|R:{}|G:{}|D:{}|T:{}|S:{}", config.identity.name, config.identity.role, pat_var, drive_var, tags.join(","), session_id);
            } else {
                println!("<koad_boot>");
                println!("Session:  {}", session_id);
                println!("Identity: {} ({})", config.identity.name, config.identity.role);
                if let Some(briefing) = mission_briefing {
                    println!("
[MISSION BRIEFING]");
                    println!("{}", briefing);
                }
                println!("Auth: GH={} | GD={}", pat_var, drive_var);
                
                if let Some(driver) = config.drivers.get(&agent) {
                    let b_path = driver.bootstrap.replace("~", &env::var("HOME").unwrap_or_default());
                    if let Ok(content) = std::fs::read_to_string(b_path) {
                        println!("
[BOOTSTRAP: {}]", agent);
                        println!("{}", content);
                    }
                }

                println!("
[CONTEXT: {}]", current_path_str);
                if !tags.is_empty() {
                    println!("Tags: {}", tags.join(", "));
                }

                if project {
                    if let Some(proj) = db.get_project_by_path(&current_path_str)? {
                        println!("Project: {} (Stack: {})", proj.name, proj.stack);
                    }
                }
                println!("</koad_boot>");
            }
        }
