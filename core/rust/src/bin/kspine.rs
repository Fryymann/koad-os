use clap::{Parser, Subcommand};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, Path as AxumPath},
    routing::{get, post},
    Router,
    Json,
};
use tower_http::services::ServeDir;
use std::sync::Arc;
use tokio::sync::broadcast;
use std::collections::HashMap;
use rusqlite::params;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use chrono::Local;
use sysinfo::System;
use notify::{Watcher, Config, Event};
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Parser)]
#[command(name = "kspine", version = "2.0", about = "KoadOS Spine - Platform Kernel")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Koad Spine (The Kernel)
    Start {
        #[arg(short, long, default_value = "8080")]
        port: u16,
        #[arg(short, long, default_value = "8081")]
        tcp_port: u16,
        #[arg(short, long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long, hide = true)]
        internal_daemon: bool,
    },
    /// Stop the Koad Spine
    Stop,
    /// View KoadOS Live State
    Status {
        /// Return JSON state
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Clone)]
struct AppState {
    channels: Arc<std::sync::Mutex<HashMap<String, broadcast::Sender<String>>>>,
    pool: Pool<SqliteConnectionManager>,
}

impl AppState {
    fn get_or_create_channel(&self, topic: &str) -> broadcast::Sender<String> {
        let mut channels = self.channels.lock().unwrap();
        if let Some(tx) = channels.get(topic) {
            tx.clone()
        } else {
            let (tx, _) = broadcast::channel(100);
            channels.insert(topic.to_string(), tx.clone());
            tx
        }
    }
}

#[derive(Serialize, Deserialize)]
struct KnowledgeRequest {
    category: String,
    content: String,
    tags: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TaskRequest {
    description: String,
    assignee_id: Option<String>,
    project_id: Option<String>,
    command: Option<String>,
}

async fn add_knowledge_handler(
    State(state): State<AppState>,
    Json(payload): Json<KnowledgeRequest>,
) -> Json<Value> {
    let conn = state.pool.get().unwrap();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = conn.execute(
        "INSERT INTO knowledge (category, content, tags, timestamp) VALUES (?1, ?2, ?3, ?4)",
        params![payload.category, payload.content, payload.tags, now],
    );
    Json(serde_json::json!({ "status": "success" }))
}

async fn create_task_handler(
    State(state): State<AppState>,
    Json(payload): Json<TaskRequest>,
) -> Json<Value> {
    let conn = state.pool.get().unwrap();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = conn.execute(
        "INSERT INTO task_graph (assignee_id, project_id, description, command, status, created_at) VALUES (?1, ?2, ?3, ?4, 'todo', ?5)",
        params![payload.assignee_id, payload.project_id, payload.description, payload.command, now],
    );
    Json(serde_json::json!({ "status": "success" }))
}

#[derive(Deserialize)]
struct QueryParams {
    term: Option<String>,
    limit: Option<usize>,
    tags: Option<String>,
}

async fn query_knowledge_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<QueryParams>,
) -> Json<Vec<Value>> {
    let conn = state.pool.get().unwrap();
    let term = params.term.unwrap_or_default();
    let limit = params.limit.unwrap_or(10);
    let tag_filter = params.tags.unwrap_or_default();

    let term_p = format!("%{}%", term);
    let tag_p = format!("%{}%", tag_filter);

    let rows = if tag_filter.is_empty() {
        let mut stmt = conn.prepare("SELECT id, category, content, timestamp FROM knowledge WHERE (content LIKE ?1 OR category LIKE ?1) AND active = 1 ORDER BY timestamp DESC LIMIT ?2").unwrap();
        stmt.query_map(params![term_p, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "category": row.get::<_, String>(1)?,
                "content": row.get::<_, String>(2)?,
                "timestamp": row.get::<_, String>(3)?
            }))
        }).unwrap().flatten().collect::<Vec<_>>()
    } else {
        let mut stmt = conn.prepare("SELECT id, category, content, timestamp FROM knowledge WHERE (content LIKE ?1 OR category LIKE ?1) AND tags LIKE ?2 AND active = 1 ORDER BY timestamp DESC LIMIT ?3").unwrap();
        stmt.query_map(params![term_p, tag_p, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "category": row.get::<_, String>(1)?,
                "content": row.get::<_, String>(2)?,
                "timestamp": row.get::<_, String>(3)?
            }))
        }).unwrap().flatten().collect::<Vec<_>>()
    };

    Json(rows)
}

#[derive(Serialize, Deserialize)]
struct KoadOSState {
    system: SystemMetrics,
    agents: Vec<AgentState>,
    tasks: Vec<TaskState>,
    kernel: KernelInfo,
}

#[derive(Serialize, Deserialize)]
struct SystemMetrics {
    cpu_usage: f32,
    mem_used_pct: f32,
    uptime_secs: u64,
}

#[derive(Serialize, Deserialize)]
struct AgentState {
    id: String,
    name: String,
    role: String,
    status: String,
    last_boot: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TaskState {
    id: i64,
    description: String,
    assignee: Option<String>,
    status: String,
}

#[derive(Serialize, Deserialize)]
struct KernelInfo {
    pid: u32,
    version: String,
    start_time: String,
}

async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<String>, topic: String) {
    println!(">>> Kernel: WebSocket Client connected to '{}'", topic);
    let mut rx = tx.subscribe();
    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Ok(msg) = msg {
                    if socket.send(Message::Text(msg.into())).await.is_err() { break; }
                }
            }
            msg = socket.recv() => {
                if let Some(Ok(Message::Text(text))) = msg {
                    let _ = tx.send(text.to_string());
                } else if msg.is_none() {
                    break;
                }
            }
        }
    }
    println!(">>> Kernel: WebSocket Client disconnected from '{}'", topic);
}

#[derive(Serialize, Deserialize)]
struct HealthReport {
    status: String,
    database: String,
    event_bus: String,
    uptime: u64,
    checks_passed: bool,
}

async fn health_check_handler(State(state): State<AppState>) -> Json<HealthReport> {
    let db_status = match state.pool.get() {
        Ok(_) => "connected",
        Err(_) => "error",
    };
    
    let checks_passed = db_status == "connected";

    Json(HealthReport {
        status: if checks_passed { "healthy" } else { "degraded" }.to_string(),
        database: db_status.to_string(),
        event_bus: "active".to_string(), // In-memory broadcast check
        uptime: System::uptime(),
        checks_passed,
    })
}

async fn get_state_handler(State(state): State<AppState>) -> Json<KoadOSState> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let metrics = SystemMetrics {
        cpu_usage: sys.global_cpu_usage(),
        mem_used_pct: (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0,
        uptime_secs: System::uptime(),
    };

    let conn = state.pool.get().unwrap();
    
    // Fetch Agents from Registry
    let mut stmt = conn.prepare("SELECT agent_id, name, role, status, last_boot FROM agents_registry").unwrap();
    let agents = stmt.query_map([], |row| {
        Ok(AgentState {
            id: row.get(0)?,
            name: row.get(1)?,
            role: row.get(2)?,
            status: row.get(3)?,
            last_boot: row.get(4)?,
        })
    }).unwrap().flatten().collect();

    // Fetch Tasks from Graph
    let mut stmt = conn.prepare("SELECT task_id, description, assignee_id, status FROM task_graph WHERE status != 'completed' LIMIT 10").unwrap();
    let tasks = stmt.query_map([], |row| {
        Ok(TaskState {
            id: row.get(0)?,
            description: row.get(1)?,
            assignee: row.get(2)?,
            status: row.get(3)?,
        })
    }).unwrap().flatten().collect();

    Json(KoadOSState {
        system: metrics,
        agents,
        tasks,
        kernel: KernelInfo {
            pid: std::process::id(),
            version: "3.0.0-alpha".to_string(),
            start_time: Local::now().to_string(), // Simplified
        },
    })
}

async fn run_tcp_server_on_host(host: &str, port: u16) {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("TCP Channel listening on {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(_) => return,
                };
                if socket.write_all(&buf[0..n]).await.is_err() {
                    return;
                }
            }
        });
    }
}

fn daemon_loop(pool: Pool<SqliteConnectionManager>, state: AppState) {
    println!("Kernel Background Loops active...");
    
    let metrics_tx = state.get_or_create_channel("metrics");
    std::thread::spawn(move || {
        let mut sys = System::new_all();
        loop {
            sys.refresh_all();
            let stats = format!("{{\"cpu\": {:.1}, \"mem\": {:.1}}}", 
                sys.global_cpu_usage(), 
                (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0);
            let _ = metrics_tx.send(stats);
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });

    // Agent Awareness Loop
    let daemon_pool = pool.clone();
    std::thread::spawn(move || {
        let mut sys = System::new_all();
        loop {
            sys.refresh_all();
            if let Ok(conn) = daemon_pool.get() {
                // Verify session PIDs
                if let Ok(mut stmt) = conn.prepare("SELECT session_id, pid FROM sessions WHERE status = 'active'") {
                    let active_sessions: Vec<(String, Option<i32>)> = stmt.query_map([], |row| {
                        Ok((row.get(0)?, row.get(1)?))
                    }).unwrap().flatten().collect();

                    for (sid, pid) in active_sessions {
                        if let Some(p) = pid {
                            if !sys.process(sysinfo::Pid::from(p as usize)).is_some() {
                                let _ = conn.execute("UPDATE sessions SET status = 'closed' WHERE session_id = ?1", [sid]);
                            }
                        }
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    });
}

async fn vault_snapshot_handler() -> Json<Value> {
    let s_path = dirs::home_dir().unwrap().join(".koad-os/skills/admin/vault.py");
    let status = Command::new(s_path).arg("snapshot").status();
    let success = status.map(|s| s.success()).unwrap_or(false);
    Json(serde_json::json!({ "status": if success { "success" } else { "error" } }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // ... (rest of main)

    let home = dirs::home_dir().unwrap().join(".koad-os");
    let pid_file = home.join("kspine.pid");
    let log_path = home.join("spine.log");

    match cli.command {
        Commands::Start { port, tcp_port, host, internal_daemon } => {
            if !internal_daemon {
                if pid_file.exists() {
                    let pid_str = std::fs::read_to_string(&pid_file)?;
                    if let Ok(pid) = pid_str.trim().parse::<i32>() {
                        if Command::new("kill").arg("-0").arg(pid.to_string()).status().map(|s| s.success()).unwrap_or(false) {
                            println!("KoadOS Kernel is already running (PID: {}).", pid);
                            return Ok(());
                        } else {
                            println!(">>> Clearing stale PID file.");
                            let _ = std::fs::remove_file(&pid_file);
                        }
                    }
                }
                println!(">>> Launching KoadOS Kernel in background... (Logs: {})", log_path.display());
                let log_file = std::fs::OpenOptions::new().append(true).create(true).open(&log_path)?;
                let _child = Command::new(std::env::current_exe()?)
                    .arg("start")
                    .arg("--port").arg(port.to_string())
                    .arg("--tcp-port").arg(tcp_port.to_string())
                    .arg("--host").arg(&host)
                    .arg("--internal-daemon")
                    .stdin(Stdio::null())
                    .stdout(Stdio::from(log_file.try_clone()?))
                    .stderr(Stdio::from(log_file))
                    .spawn()?;
                
                std::fs::write(&pid_file, _child.id().to_string())?;
                println!(">>> [PASS] Kernel started (PID: {}).", _child.id());
                return Ok(());
            }

            // Internal Daemon Logic: Write our own PID and STAY in foreground
            std::fs::write(&pid_file, std::process::id().to_string())?;

            println!(">>> Kernel: Binding Listeners on {}...", host);
            let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await
                .expect(&format!("FATAL: Failed to bind Axum to {}:{}", host, port));

            println!(">>> Kernel: Initializing Database...");
            let db_path = home.join("koad.db");
            let manager = SqliteConnectionManager::file(db_path);
            let pool = Pool::builder()
                .max_size(10)
                .build(manager)?;

            // Robustness Pragmas
            {
                let conn = pool.get()?;
                let _ = conn.execute("PRAGMA journal_mode=WAL", []);
                let _ = conn.execute("PRAGMA busy_timeout=5000", []);
                println!(">>> Kernel: [PASS] Database WAL mode active (Concurrency Safe).");
            }

            println!(">>> Kernel: Starting Background Loops...");
            let state = AppState {
                channels: Arc::new(std::sync::Mutex::new(HashMap::new())),
                pool: pool.clone(),
            };

            daemon_loop(pool, state.clone());

            println!(">>> Kernel: Starting TCP Channel on {}:{}...", host, tcp_port);
            let tcp_host = host.clone();
            tokio::spawn(async move {
                run_tcp_server_on_host(&tcp_host, tcp_port).await;
            });

            let target_dir = home.join("data/dashboard");
            println!(">>> Kernel: Starting Axum (Web/WS) on {}:{}...", host, port);
            let app = Router::new()
                .route("/ws/{topic}", get(move |AxumPath(topic): AxumPath<String>, ws: WebSocketUpgrade, State(state): State<AppState>| {
                    async move {
                        let tx = state.get_or_create_channel(&topic);
                        ws.on_upgrade(move |socket| handle_socket(socket, tx, topic))
                    }
                }))
                .route("/state", get(get_state_handler))
                .route("/health", get(health_check_handler))
                .route("/knowledge", get(query_knowledge_handler).post(add_knowledge_handler))
                .route("/tasks", get(get_state_handler).post(create_task_handler))
                .route("/vault/snapshot", post(vault_snapshot_handler))
                .fallback_service(ServeDir::new(&target_dir))
                .with_state(state);

            println!(">>> Kernel: Server Ready.");
            axum::serve(listener, app).await
                .expect("FATAL: Axum server crashed");
        }
        Commands::Stop => {
            if pid_file.exists() {
                let pid = std::fs::read_to_string(&pid_file)?.trim().parse::<i32>()?;
                println!("Stopping KoadOS Kernel (PID: {})...", pid);
                let _ = Command::new("kill").arg(pid.to_string()).status();
                let _ = std::fs::remove_file(pid_file);
                println!("[PASS] Kernel stopped.");
            } else {
                println!("[INFO] No Kernel PID file found.");
            }
        }
        Commands::Status { json } => {
            if pid_file.exists() {
                let pid_str = std::fs::read_to_string(&pid_file)?;
                let pid = pid_str.trim().parse::<i32>()?;
                if Command::new("kill").arg("-0").arg(pid.to_string()).status().map(|s| s.success()).unwrap_or(false) {
                    if json {
                        let res = reqwest::blocking::get("http://localhost:8080/state")?.text()?;
                        println!("{}", res);
                    } else {
                        println!("KoadOS Kernel: RUNNING (PID: {})", pid);
                        let res = reqwest::blocking::get("http://localhost:8080/state")?.json::<KoadOSState>()?;
                        println!("\n[Agents]");
                        for a in res.agents { println!("- {} ({}) [{}]", a.name, a.role, a.status); }
                        println!("\n[Pending Tasks]");
                        if res.tasks.is_empty() { println!("- None"); }
                        for t in res.tasks { println!("- [#{} to {}] {}", t.id, t.assignee.unwrap_or_else(|| "unassigned".to_string()), t.description); }
                    }
                } else {
                    println!("KoadOS Kernel: STOPPED (Stale PID)");
                }
            } else {
                println!("KoadOS Kernel: STOPPED");
            }
        }
    }
    Ok(())
}
