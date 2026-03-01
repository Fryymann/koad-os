use clap::Parser;
use std::process::{Command, Stdio};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use chrono::Local;
use std::path::PathBuf;
use tungstenite::{connect, Message};
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[command(name = "kbooster", version = "3.0", about = "KoadOS Context Sidecar (v3)")]
struct Cli {
    /// Persistent Identity Handle (e.g. 'koad', 'SWS-Manager')
    #[arg(short, long)]
    agent_id: String,

    /// Role (e.g. 'Admin', 'PM')
    #[arg(short, long)]
    role: String,
}

#[derive(Serialize, Deserialize)]
struct KoadOSState {
    agents: Vec<AgentState>,
    tasks: Vec<TaskState>,
}

#[derive(Serialize, Deserialize)]
struct AgentState {
    id: String,
    status: String,
}

#[derive(Serialize, Deserialize)]
struct TaskState {
    id: i64,
    assignee: Option<String>,
    status: String,
}

fn record_context_delta(pool: &Pool<SqliteConnectionManager>, agent_id: &str, category: &str, content: &str) -> anyhow::Result<()> {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let conn = pool.get()?;
    let tags = format!("booster,agent_id:{},context_warmup", agent_id);
    
    conn.execute(
        "INSERT INTO knowledge (category, content, tags, timestamp) VALUES (?1, ?2, ?3, ?4)",
        params![category, content, tags, now],
    )?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!(">>> KoadOS Booster (v3): Starting for Identity '{}' (Role: {})", cli.agent_id, cli.role);
    
    let home = dirs::home_dir().unwrap().join(".koad-os");
    let db_path = home.join("koad.db");
    
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder().max_size(5).build(manager)?;

    // Identity Registration Check
    {
        let conn = pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO agents_registry (agent_id, name, role, status) VALUES (?1, ?1, ?2, 'idle')",
            params![cli.agent_id, cli.role],
        )?;
    }

    // Connect to Kernel State Bus (WebSocket)
    let hub_url = "ws://localhost:8080/ws/events";
    println!(">>> Connecting to Kernel: {}", hub_url);
    
    let (mut socket, _) = match connect(hub_url) {
        Ok(s) => s,
        Err(e) => {
            println!(">>> [ERROR] Could not connect to Koad Kernel (kspine). Is it running? Error: {}", e);
            return Err(anyhow::anyhow!(e));
        }
    };

    println!(">>> [PASS] Booster attached to Kernel bus. Monitoring context...");

    // Sidecar Loop
    loop {
        // 1. Listen for Live State Events
        if let Ok(msg) = socket.read() {
            if let Message::Text(text) = msg {
                // If the event relates to our agent or project, process it
                if text.contains(&cli.agent_id) || text.contains("Event:") {
                    let _ = record_context_delta(&pool, &cli.agent_id, "live_event", &text);
                }
            }
        }

        // 2. Proactive "Review Required" Check (Only for Admin/Koad)
        if cli.role.to_lowercase() == "admin" {
            if let Ok(conn) = pool.get() {
                if let Ok(mut stmt) = conn.prepare("SELECT task_id, description FROM task_graph WHERE status = 'review_required'") {
                    let pending: Vec<(i64, String)> = stmt.query_map([], |row| {
                        Ok((row.get(0)?, row.get(1)?))
                    }).unwrap().flatten().collect();

                    for (tid, desc) in pending {
                        let _ = record_context_delta(&pool, &cli.agent_id, "review_alert", &format!("Task #{} ({}) is ready for your review.", tid, desc));
                    }
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
