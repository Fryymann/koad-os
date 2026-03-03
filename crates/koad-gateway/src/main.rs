pub mod deck;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use fred::interfaces::{PubsubInterface, EventInterface, HashesInterface};
use fred::prelude::*;
use serde_json::{json, Value};
use futures::{StreamExt, SinkExt};
use chrono::Utc;
use koad_core::intent::{Intent, ExecuteIntent};
use koad_core::session::AgentSession;
use koad_board::GitHubClient;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::*;
use clap::Parser;
use std::path::PathBuf;
use rusqlite::Connection;
use crate::deck::DeckManager;
use tracing::{info, error, warn};
use koad_core::logging::init_logging;
use koad_core::config::KoadConfig as CoreConfig;

#[derive(Parser)]
struct Cli {
    /// Listen address
    #[arg(long)]
    addr: Option<String>,
}

struct GatewayState {
    pub client: RedisClient,
    pub subscriber: RedisClient,
    pub gh_client: Option<GitHubClient>,
    pub config: CoreConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut config = CoreConfig::load()?;
    
    if let Some(addr) = cli.addr {
        config.gateway_addr = addr;
    }

    // Initialize Structured Logging
    let _guard = init_logging("kgateway", Some(config.home.clone()));
    
    info!("KoadOS Gateway starting up...");

    // Resolve GitHub Token
    let gh_client = if let Ok(token) = config.resolve_gh_token() {
        info!("Gateway: GitHub integration active.");
        GitHubClient::new(token, "Fryymann".to_string(), "koad-os".to_string()).ok()
    } else {
        warn!("Gateway: GitHub integration DISABLED (No PAT found).");
        None
    };

    let socket_path = config.redis_socket.clone();
    
    info!("Gateway: Connecting to Redis via UDS at {}...", socket_path.display());

    let redis_config = RedisConfig {
        server: ServerConfig::Unix {
            path: socket_path,
        },
        ..Default::default()
    };

    let client = Builder::from_config(redis_config.clone()).build()?;
    let subscriber = Builder::from_config(redis_config).build()?;

    client.init().await?;
    subscriber.init().await?;

    let state = Arc::new(GatewayState {
        client,
        subscriber,
        gh_client,
        config: config.clone(),
    });

    // 1. Initialize and Start Deck Manager (Vite Dev Server if needed)
    let deck_path = config.home.join("web/deck").to_string_lossy().into_owned();
    let deck_manager = DeckManager::new(&deck_path).start().await?;

    // Ensure static dist directory exists for Vite Dashboard
    let dist_path = config.home.join("web/deck/dist");
    let _ = std::fs::create_dir_all(&dist_path);

    let app = Router::new()
        .nest_service("/", ServeDir::new(dist_path))
        .route("/ws/fabric", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    info!("EdgeGateway: Web Deck & WebSocket active on http://{}", config.gateway_addr);
    
    // Register Service in Inventory
    let service_entry = json!({
        "name": "web-deck",
        "host": "0.0.0.0",
        "port": 3000,
        "protocol": "http/ws",
        "status": "UP",
        "last_seen": Utc::now().timestamp()
    });
    let _: () = state.client.hset::<(), _, _>("koad:services", ("web-deck", service_entry.to_string())).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let listener = tokio::net::TcpListener::bind(&config.gateway_addr).await?;
    axum::serve(listener, app).await?;

    drop(deck_manager);
    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<GatewayState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<GatewayState>) {
    info!("Gateway: WebSocket client connected.");
    let (mut sender, mut receiver) = socket.split();
    let client = state.client.clone();
    let _subscriber = state.subscriber.clone();
    
    // 1. Initial Sync (Agents + Issues + Projects)
    let mut agents = vec![];
    info!("Gateway: Syncing agents from Redis...");
    if let Ok(all_state) = client.hgetall::<std::collections::HashMap<String, String>, _>("koad:state").await {
        info!("Gateway: Found {} items in koad:state", all_state.len());
        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                // Try strict parsing first, then fallback to raw JSON if it has session_id
                if let Ok(session) = serde_json::from_str::<AgentSession>(&val) {
                    agents.push(json!(session));
                } else if let Ok(mut raw_json) = serde_json::from_str::<Value>(&val) {
                    // Check if it's wrapped in a 'data' field (from Spine hydration)
                    if let Some(inner_data) = raw_json.get("data") {
                        if inner_data["session_id"].is_string() {
                            agents.push(inner_data.clone());
                            continue;
                        }
                    }
                    
                    if raw_json["session_id"].is_string() {
                        agents.push(raw_json);
                    }
                }
            }
        }
    } else {
        println!("Gateway: Failed to HGETALL koad:state");
    }

    let mut issues = vec![];
    if let Some(gh) = &state.gh_client {
        if let Ok(gh_items) = gh.list_project_items(2).await {
            issues = gh_items;
        }
    }

    let mut projects = vec![];
    if let Ok(conn) = Connection::open(state.config.get_db_path()) {
        if let Ok(mut stmt) = conn.prepare("SELECT id, name, path, branch, health FROM projects WHERE active = 1") {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, i32>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "path": row.get::<_, String>(2)?,
                    "branch": row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "unknown".into()),
                    "health": row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "unknown".into()),
                }))
            }) {
                for r in rows {
                    if let Ok(p) = r {
                        projects.push(p);
                    }
                }
            }
        }
    }

    let sync_msg = json!({
        "type": "SYSTEM_SYNC",
        "payload": {
            "agents": agents,
            "issues": issues,
            "projects": projects
        }
    });

    let _ = sender.send(Message::Text(sync_msg.to_string())).await;

    // Relay Task: Spine gRPC (Real-time Events) -> WebSocket
    let spine_addr = state.config.spine_grpc_addr.clone();
    let relay_handle = tokio::spawn(async move {
        if let Ok(mut client) = SpineServiceClient::connect(spine_addr).await {
            if let Ok(response) = client.stream_system_events(StreamSystemEventsRequest { filter_sources: vec![] }).await {
                let mut stream = response.into_inner();
                while let Ok(Some(event)) = stream.message().await {
                    let payload_str = event.message;
                    
                    // Normalize messages for the Frontend
                    if let Ok(mut json) = serde_json::from_str::<Value>(&payload_str) {
                        let msg_type = json["type"].as_str().unwrap_or_default().to_lowercase();
                        if msg_type == "session_update" {
                            let normalized = json!({
                                "type": "SESSION_UPDATE",
                                "payload": json["data"].take()
                            });
                            if sender.send(Message::Text(normalized.to_string())).await.is_err() {
                                break;
                            }
                            continue;
                        }
                    }

                    if sender.send(Message::Text(payload_str)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Command Task: WebSocket -> Redis (Sandbox-aware Command Intents)
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                if json["type"] == "COMMAND" {
                    if let Some(cmd) = json["payload"].as_str() {
                        // WRAP: Convert to strongly-typed Intent
                        let intent = Intent::Execute(ExecuteIntent {
                            identity: "admin".to_string(),
                            command: cmd.to_string(),
                            args: vec![],
                            working_dir: None,
                            env_vars: std::collections::HashMap::new(),
                        });
                        
                        if let Ok(intent_str) = serde_json::to_string(&intent) {
                            let _: Result<(), _> = client.publish("koad:commands", intent_str).await;
                        }
                    }
                }
            }
        }
    }

    relay_handle.abort();
}
