use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use koad_mcp::{JsonRpcRequest, McpServer};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tools::commit::CommitTool;
use tools::intel_get::IntelGetTool;
use tools::list_topics::ListTopicsTool;
use tools::recall::RecallTool;
use tools::search_semantic::SearchSemanticTool;
use tools::status::StatusTool;

mod tools;

#[derive(Clone)]
struct AppState {
    server: Arc<McpServer>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Logs go to stderr: in stdio mode stdout is the JSON-RPC channel.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let cass_url = std::env::var("CASS_URL").unwrap_or_else(|_| "http://localhost:50052".to_string());
    let agent_name = std::env::var("AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("AGENT_NAME is required (e.g. clyde)"))?;
    let partition = resolve_partition(std::env::var("AGENT_PARTITION").ok(), &agent_name);
    let mcp_mode = std::env::var("MCP_MODE").unwrap_or_else(|_| "read_only".to_string());
    let transport = std::env::var("MCP_TRANSPORT").unwrap_or_else(|_| "http".to_string());
    let port: u16 = std::env::var("MCP_PORT")
        .unwrap_or_else(|_| "9742".to_string())
        .parse()
        .unwrap_or(9742);

    tracing::info!(agent = %agent_name, partition = %partition, mode = %mcp_mode, transport = %transport, "KoadOS MCP bridge starting");

    let mut server = McpServer::new(&agent_name, "0.1.0");
    server.register_tool(RecallTool::new(cass_url.clone(), partition.clone()));
    server.register_tool(SearchSemanticTool::new(cass_url.clone(), partition.clone()));
    server.register_tool(ListTopicsTool::new(cass_url.clone(), partition.clone()));
    server.register_tool(IntelGetTool::new(cass_url.clone(), partition.clone()));
    server.register_tool(StatusTool::new(cass_url.clone(), partition.clone()));

    if mcp_mode == "read_write" {
        tracing::info!("MCP_MODE=read_write: memory.commit tool enabled");
        server.register_tool(CommitTool::new(cass_url.clone(), partition.clone(), agent_name.clone()));
    }

    match transport.as_str() {
        "stdio" => {
            tracing::info!("Transport: stdio");
            server.run().await?;
        }
        _ => {
            let state = AppState { server: Arc::new(server) };
            let app = Router::new()
                .route("/mcp", post(handle_mcp))
                .route("/health", axum::routing::get(|| async { "ok" }))
                .with_state(state);

            let addr = bind_addr(std::env::var("MCP_BIND").ok().as_deref(), port)?;
            tracing::info!(%addr, "Transport: HTTP");
            let listener = TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}

async fn handle_mcp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    tracing::info!(method = %req.method, id = ?req.id, "MCP request");
    if req.method.starts_with("notifications/") {
        return (StatusCode::ACCEPTED, "").into_response();
    }

    let resp = state.server.handle_request(req).await;
    let json_body = serde_json::to_string(&resp).unwrap_or_default();

    let accept = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if accept.contains("text/event-stream") {
        let sse_body = format!("data: {}\n\n", json_body);
        (
            StatusCode::OK,
            [
                ("content-type", "text/event-stream"),
                ("cache-control", "no-cache"),
            ],
            sse_body,
        )
            .into_response()
    } else {
        (
            StatusCode::OK,
            [("content-type", "application/json")],
            json_body,
        )
            .into_response()
    }
}

/// Address the HTTP transport listens on. `MCP_BIND` overrides the default
/// loopback address; it must be an IP address.
fn bind_addr(bind: Option<&str>, port: u16) -> anyhow::Result<SocketAddr> {
    let ip: std::net::IpAddr = match bind {
        Some(b) => b
            .parse()
            .map_err(|_| anyhow::anyhow!("MCP_BIND must be an IP address, got {b:?}"))?,
        None => std::net::Ipv4Addr::LOCALHOST.into(),
    };
    Ok(SocketAddr::new(ip, port))
}

/// Partition from `AGENT_PARTITION`, or derived from the agent name with the
/// canonical `partition_key` when unset. Keeping one derivation avoids shell
/// copies of the rule drifting and writing memory to a partition no one reads.
fn resolve_partition(explicit: Option<String>, agent_name: &str) -> String {
    explicit
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| koad_core::utils::partition::partition_key(agent_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression guard: the listener was hardcoded to 0.0.0.0, exposing
    /// unauthenticated agent memory to every network the host joins
    /// (including a tailnet when Tailscale runs in the same WSL instance).
    #[test]
    fn defaults_to_loopback() {
        assert_eq!(
            bind_addr(None, 9744).unwrap(),
            "127.0.0.1:9744".parse().unwrap()
        );
    }

    #[test]
    fn honours_an_explicit_bind_address() {
        assert_eq!(
            bind_addr(Some("0.0.0.0"), 9745).unwrap(),
            "0.0.0.0:9745".parse().unwrap()
        );
        assert_eq!(
            bind_addr(Some("::1"), 9745).unwrap(),
            "[::1]:9745".parse().unwrap()
        );
    }

    #[test]
    fn rejects_a_non_ip_bind_address() {
        assert!(bind_addr(Some("localhost"), 9744).is_err());
    }

    #[test]
    fn partition_defaults_to_the_canonical_key() {
        let canonical = koad_core::utils::partition::partition_key("clyde");
        assert_eq!(resolve_partition(None, "Clyde"), canonical);
        assert_eq!(resolve_partition(Some(String::new()), "clyde"), canonical);
        assert_eq!(
            resolve_partition(Some("custom_p".into()), "clyde"),
            "custom_p"
        );
    }
}
