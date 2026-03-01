use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use crate::engine::Engine;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use fred::interfaces::{PubsubInterface, EventInterface, HashesInterface};
use serde_json::{json, Value};
use futures::{StreamExt, SinkExt};
use chrono::Utc;

pub struct WebGateway {
    engine: Arc<Engine>,
}

impl WebGateway {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let home_dir = std::env::var("KOAD_HOME")
            .unwrap_or_else(|_| format!("{}/.koad-os", std::env::var("HOME").unwrap_or_default()));
        
        // Ensure static dist directory exists for Vite Dashboard
        let dist_path = format!("{}/web/deck/dist", home_dir);
        let _ = std::fs::create_dir_all(&dist_path);

        let app = Router::new()
            .nest_service("/", ServeDir::new(dist_path))
            .route("/ws/fabric", get(ws_handler))
            .layer(CorsLayer::permissive())
            .with_state(self.engine.clone());

        // BINDING: 0.0.0.0:3000 for Windows-to-WSL bridge visibility
        let addr = "0.0.0.0:3000";
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        println!("EdgeGateway: Web Deck & WebSocket active on http://{}", addr);
        
        // Register Service in Inventory
        let service_entry = json!({
            "name": "web-deck",
            "host": "0.0.0.0",
            "port": 3000,
            "protocol": "http/ws",
            "status": "UP",
            "last_seen": Utc::now().timestamp()
        });
        let _: () = self.engine.redis.client.hset::<(), _, _>("koad:services", ("web-deck", service_entry.to_string())).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;

        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(engine): axum::extract::State<Arc<Engine>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, engine))
}

async fn handle_socket(socket: WebSocket, engine: Arc<Engine>) {
    let (mut sender, mut receiver) = socket.split();
    let redis = engine.redis.clone();
    
    // Relay Task: Redis PubSub (Real-time Stats) -> WebSocket
    let mut message_stream = redis.subscriber.message_rx();
    let _ = redis.subscriber.subscribe(vec!["koad:telemetry", "koad:telemetry:stats"]).await;

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
                        // WRAP: Convert to Sandbox-ready Intent Payload
                        let intent = json!({
                            "identity": "admin", // Default for Web Deck for now
                            "command": cmd
                        });
                        let _: Result<(), _> = redis.client.publish("koad:commands", intent.to_string()).await;
                    }
                }
            }
        }
    }

    relay_handle.abort();
}
