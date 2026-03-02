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
use clap::Parser;
use std::path::PathBuf;
use crate::deck::DeckManager;

#[derive(Parser)]
struct Cli {
    /// Koad home directory
    #[arg(long)]
    home: Option<String>,

    /// Listen address
    #[arg(long, default_value = "0.0.0.0:3000")]
    addr: String,
}

struct GatewayState {
    pub client: RedisClient,
    pub subscriber: RedisClient,
    pub gh_client: Option<GitHubClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let home_dir = cli.home.clone().unwrap_or_else(|| {
        std::env::var("KOAD_HOME")
            .unwrap_or_else(|_| format!("{}/.koad-os", std::env::var("HOME").unwrap_or_default()))
    });

    // Resolve GitHub Token
    let gh_token = std::env::var("GITHUB_ADMIN_PAT").ok()
        .or_else(|| std::env::var("GITHUB_PERSONAL_PAT").ok());
    
    let gh_client = if let Some(token) = gh_token {
        println!("Gateway: GitHub integration active.");
        GitHubClient::new(token, "Fryymann".to_string(), "koad-os".to_string()).ok()
    } else {
        println!("Gateway: GitHub integration DISABLED (No PAT found).");
        None
    };

    let socket_path = PathBuf::from(&home_dir).join("koad.sock");
    println!("Gateway: Connecting to Redis via UDS at {}...", socket_path.display());

    let config = RedisConfig {
        server: ServerConfig::Unix {
            path: socket_path,
        },
        ..Default::default()
    };

    let client = Builder::from_config(config.clone()).build()?;
    let subscriber = Builder::from_config(config).build()?;

    client.init().await?;
    subscriber.init().await?;

    let state = Arc::new(GatewayState {
        client,
        subscriber,
        gh_client,
    });

    // 1. Initialize and Start Deck Manager (Vite Dev Server if needed)
    let deck_path = format!("{}/web/deck", home_dir);
    let deck_manager = DeckManager::new(&deck_path).start().await?;

    // Ensure static dist directory exists for Vite Dashboard
    let dist_path = format!("{}/web/deck/dist", home_dir);
    let _ = std::fs::create_dir_all(&dist_path);

    let app = Router::new()
        .nest_service("/", ServeDir::new(dist_path))
        .route("/ws/fabric", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    println!("EdgeGateway: Web Deck & WebSocket active on http://{}", cli.addr);
    
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

    let listener = tokio::net::TcpListener::bind(&cli.addr).await?;
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
    let (mut sender, mut receiver) = socket.split();
    let client = state.client.clone();
    let subscriber = state.subscriber.clone();
    
    // 1. Initial Sync (Agents + Issues)
    let mut agents = vec![];
    if let Ok(all_state) = client.hgetall::<std::collections::HashMap<String, String>, _>("koad:state").await {
        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                if let Ok(session) = serde_json::from_str::<AgentSession>(&val) {
                    agents.push(session);
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

    let sync_msg = json!({
        "type": "SYSTEM_SYNC",
        "payload": {
            "agents": agents,
            "issues": issues
        }
    });

    let _ = sender.send(Message::Text(sync_msg.to_string())).await;

    // Relay Task: Redis PubSub (Real-time Stats) -> WebSocket
    let mut message_stream = subscriber.message_rx();
    let _ = subscriber.subscribe(vec!["koad:telemetry", "koad:telemetry:stats"]).await;

    let relay_handle = tokio::spawn(async move {
        while let Ok(message) = message_stream.recv().await {
            let payload = message.value.as_string().unwrap_or_default();
            if sender.send(Message::Text(payload)).await.is_err() {
                break;
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
