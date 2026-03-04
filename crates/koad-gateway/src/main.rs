pub mod deck;
use crate::deck::DeckManager;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Utc;
use clap::Parser;
use fred::interfaces::{HashesInterface, PubsubInterface, ClientLike};
use fred::prelude::*;
use fred::clients::RedisPool;
use futures::{SinkExt, StreamExt};
use koad_board::GitHubClient;
use koad_core::config::KoadConfig as CoreConfig;
use koad_core::intent::{ExecuteIntent, Intent};
use koad_core::logging::init_logging;

use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::*;
use rusqlite::Connection;
use serde_json::{json, Value};


use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::{info, warn, error};

#[derive(Parser)]
struct Cli {
    /// Listen address
    #[arg(long)]
    addr: Option<String>,
}

struct GatewayState {
    pub pool: RedisPool,
    pub subscriber: fred::clients::RedisClient,
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

    info!(
        "Gateway: Connecting to Redis via UDS at {}...",
        socket_path.display()
    );

    let redis_config = RedisConfig {
        server: ServerConfig::Unix { path: socket_path },
        ..Default::default()
    };

    // Connection Pool for commands
    let pool = Builder::from_config(redis_config.clone())
        .with_connection_config(|c| {
            c.connection_timeout = std::time::Duration::from_secs(5);
        })
        .build_pool(4)?;

    // Dedicated subscriber for PubSub
    let subscriber = Builder::from_config(redis_config).build()?;

    pool.init().await?;
    subscriber.init().await?;

    let state = Arc::new(GatewayState {
        pool,
        subscriber,
        gh_client,
        config: config.clone(),
    });

    // 1. Initialize and Start Deck Manager
    let deck_path = config.home.join("web/deck").to_string_lossy().into_owned();
    let deck_manager = DeckManager::new(&deck_path).start().await?;

    let dist_path = config.home.join("web/deck/dist");
    let _ = std::fs::create_dir_all(&dist_path);

    let app = Router::new()
        .nest_service("/", ServeDir::new(dist_path))
        .route("/ws/fabric", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    info!(
        "EdgeGateway: Web Deck & WebSocket active on http://{}",
        config.gateway_addr
    );

    // Register Service in Inventory
    let service_entry = json!({
        "name": "web-deck",
        "host": "0.0.0.0",
        "port": 3000,
        "protocol": "http/ws",
        "status": "UP",
        "last_seen": Utc::now().timestamp()
    });
    let _: () = state
        .pool
        .next()
        .hset::<(), _, _>("koad:services", ("web-deck", service_entry.to_string()))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let listener = tokio::net::TcpListener::bind(&config.gateway_addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Gateway: Shutdown complete.");
    drop(deck_manager);
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    info!("Gateway: Termination signal received. Commencing graceful shutdown...");
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
    let pool = state.pool.clone();

    let (outbox_tx, mut outbox_rx) = tokio::sync::mpsc::channel::<Message>(64);

    // 1. Initial Sync
    let mut agents = vec![];
    if let Ok(all_state) = pool.next()
        .hgetall::<std::collections::HashMap<String, String>, _>("koad:state")
        .await
    {
        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                if let Ok(raw_json) = serde_json::from_str::<Value>(&val) {
                    let data = if let Some(inner) = raw_json.get("data") { inner } else { &raw_json };
                    let status = data["status"].as_str().unwrap_or("unknown");
                    if status == "active" || status == "idle" {
                        agents.push(data.clone());
                    }
                }
            }
        }
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
                for r in rows { if let Ok(p) = r { projects.push(p); } }
            }
        }
    }

    let sync_msg = json!({
        "type": "SYSTEM_SYNC",
        "payload": { "agents": agents, "issues": issues, "projects": projects }
    });
    let _ = outbox_tx.send(Message::Text(sync_msg.to_string())).await;

    // Telemetry Relay: Redis PubSub (Non-blocking) -> Outbox
    let mut telemetry_subscriber = state.subscriber.clone();
    let tx_telemetry = outbox_tx.clone();
    let _telemetry_handle = tokio::spawn(async move {
        let mut message_stream = telemetry_subscriber.message_rx();
        let _ = telemetry_subscriber.subscribe(vec!["koad:telemetry:stats", "koad:telemetry:manifest"]).await;
        
        while let Ok(msg) = message_stream.recv().await {
            let channel = msg.channel.to_string();
            let payload = msg.value.as_string().unwrap_or_default();
            
            let ws_msg = match channel.as_str() {
                "koad:telemetry:stats" => json!({ "type": "SYSTEM_STATS", "payload": serde_json::from_str::<Value>(&payload).unwrap_or_default() }),
                "koad:telemetry:manifest" => json!({ "type": "CREW_MANIFEST", "payload": serde_json::from_str::<Value>(&payload).unwrap_or_default() }),
                _ => continue,
            };
            
            if tx_telemetry.send(Message::Text(ws_msg.to_string())).await.is_err() { break; }
        }
    });

    // Relay Task: Spine gRPC (Real-time Events) -> Outbox
    let spine_addr = state.config.spine_grpc_addr.clone();
    let tx_relay = outbox_tx.clone();
    let relay_handle = tokio::spawn(async move {
        if let Ok(mut client) = SpineServiceClient::connect(spine_addr).await {
            if let Ok(response) = client.stream_system_events(StreamSystemEventsRequest { filter_sources: vec![] }).await {
                let mut stream = response.into_inner();
                while let Ok(Some(event)) = stream.message().await {
                    let payload_str = event.message;
                    if let Ok(mut json) = serde_json::from_str::<Value>(&payload_str) {
                        let msg_type = json["type"].as_str().unwrap_or_default().to_lowercase();
                        if msg_type == "session_update" {
                            let normalized = json!({ "type": "SESSION_UPDATE", "payload": json["data"].take() });
                            if tx_relay.send(Message::Text(normalized.to_string())).await.is_err() { break; }
                            continue;
                        }
                    }
                    if tx_relay.send(Message::Text(payload_str)).await.is_err() { break; }
                }
            }
        }
    });

    // Outbox Sender
    let outbox_sender_handle = tokio::spawn(async move {
        while let Some(msg) = outbox_rx.recv().await {
            if sender.send(msg).await.is_err() { break; }
        }
    });

    // Receiver: WebSocket -> Redis
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    if json["type"] == "COMMAND" {
                        if let Some(cmd) = json["payload"].as_str() {
                            let intent = Intent::Execute(ExecuteIntent {
                                identity: "admin".into(),
                                command: cmd.into(),
                                args: vec![],
                                working_dir: None,
                                env_vars: std::collections::HashMap::new(),
                            });
                            if let Ok(intent_str) = serde_json::to_string(&intent) {
                                let _: Result<(), _> = pool.next().publish("koad:commands", intent_str).await;
                            }
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    relay_handle.abort();
    outbox_sender_handle.abort();
}
