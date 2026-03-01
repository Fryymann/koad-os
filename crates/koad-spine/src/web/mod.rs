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
use fred::interfaces::{PubsubInterface, EventInterface};
use serde_json::Value;
use futures::{StreamExt, SinkExt};

pub struct WebGateway {
    engine: Arc<Engine>,
}

impl WebGateway {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let app = Router::new()
            .nest_service("/", ServeDir::new("/home/ideans/.koad-os/web/deck/dist"))
            .route("/ws/fabric", get(ws_handler))
            .layer(CorsLayer::permissive())
            .with_state(self.engine.clone());

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        println!("WebGateway: Deck Dashboard active on http://localhost:3000");
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
    let redis = engine.redis.clone();
    let mut message_stream = redis.subscriber.message_rx();

    // Subscribe to all telemetry channels
    if redis.subscriber.subscribe(vec!["koad:telemetry", "koad:telemetry:stats"]).await.is_err() {
        return;
    }

    let (mut sender, mut receiver) = socket.split();

    // Task 1: Relay Redis -> WebSocket
    let relay_handle = tokio::spawn(async move {
        while let Ok(message) = message_stream.recv().await {
            let payload = message.value.as_string().unwrap_or_default();
            if sender.send(Message::Text(payload)).await.is_err() {
                break;
            }
        }
    });

    // Task 2: Handle WebSocket -> Redis (Commands)
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                if json["type"] == "COMMAND" {
                    if let Some(cmd) = json["payload"].as_str() {
                        let _: Result<(), _> = redis.client.publish("koad:commands", cmd).await;
                    }
                }
            }
        }
    }

    relay_handle.abort();
}
