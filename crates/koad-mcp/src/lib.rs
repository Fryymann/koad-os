use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Protocol revisions this server implements, oldest first. 2026-07-28 is
/// deliberately absent: it removes the initialize handshake and requires
/// server/discover, neither of which this server supports yet.
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] =
    &["2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25"];

pub const LATEST_PROTOCOL_VERSION: &str = "2025-11-25";

/// Per the MCP lifecycle rules, echo the client's version only when we support
/// it; otherwise offer our latest and let the client decide whether to proceed.
fn negotiate_protocol_version(requested: Option<&str>) -> &'static str {
    requested
        .and_then(|r| SUPPORTED_PROTOCOL_VERSIONS.iter().find(|v| **v == r).copied())
        .unwrap_or(LATEST_PROTOCOL_VERSION)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    #[serde(default)]
    pub id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    pub id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema", alias = "input_schema")]
    pub input_schema: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolList {
    pub tools: Vec<McpTool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolCallResponse {
    pub content: Vec<McpContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContent {
    #[serde(rename = "text")]
    Text { text: String },
}

pub struct McpServer {
    name: String,
    version: String,
    tools: HashMap<String, Box<dyn McpToolHandler + Send + Sync>>,
}

#[async_trait::async_trait]
pub trait McpToolHandler {
    fn definition(&self) -> McpTool;
    async fn call(&self, params: Value) -> Result<McpToolCallResponse>;
}

impl McpServer {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            tools: HashMap::new(),
        }
    }

    pub fn register_tool<T: McpToolHandler + 'static + Send + Sync>(&mut self, handler: T) {
        let def = handler.definition();
        self.tools.insert(def.name.clone(), Box::new(handler));
    }

    pub async fn run(&self) -> Result<()> {
        let mut lines = BufReader::new(tokio::io::stdin()).lines();
        let mut stdout = tokio::io::stdout();

        while let Some(line) = lines.next_line().await? {
            let req: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    tracing::error!("Failed to parse request: {}", e);
                    continue;
                }
            };

            let response = self.handle_request(req).await;
            let res_str = serde_json::to_string(&response)?;
            stdout.write_all(res_str.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let result = match req.method.as_str() {
            "initialize" => {
                let requested = req.params.as_ref()
                    .and_then(|p| p.get("protocolVersion"))
                    .and_then(|v| v.as_str());
                Some(serde_json::json!({
                    "protocolVersion": negotiate_protocol_version(requested),
                    // Must advertise every capability we actually serve. Clients
                    // that honour the capability map skip tools/list entirely when
                    // `tools` is absent, so omitting it presents a connected
                    // server with zero usable tools. listChanged is false because
                    // the tool set is fixed at registration time.
                    "capabilities": {
                        "tools": { "listChanged": false }
                    },
                    "serverInfo": {
                        "name": self.name,
                        "version": self.version,
                    }
                }))
            }
            "tools/list" => {
                let tools: Vec<McpTool> = self.tools.values().map(|h| h.definition()).collect();
                Some(serde_json::to_value(McpToolList { tools }).unwrap())
            }
            "tools/call" => {
                if let Some(params) = req.params {
                    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                    let args = params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));
                    
                    if let Some(handler) = self.tools.get(name) {
                        match handler.call(args).await {
                            Ok(res) => Some(serde_json::to_value(res).unwrap()),
                            Err(e) => Some(serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": format!("Error: {}", e)
                                }],
                                "isError": true
                            })),
                        }
                    } else {
                        Some(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Tool not found: {}", name)
                            }],
                            "isError": true
                        }))
                    }
                } else {
                    Some(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": "Missing params for tools/call"
                        }],
                        "isError": true
                    }))
                }
            }
            _ => None,
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: result.clone(),
            error: if result.is_none() {
                Some(serde_json::json!({
                    "code": -32601,
                    "message": format!("Method not found: {}", req.method)
                }))
            } else {
                None
            },
            id: req.id.unwrap_or(Value::Null),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopTool;

    #[async_trait::async_trait]
    impl McpToolHandler for NoopTool {
        fn definition(&self) -> McpTool {
            McpTool {
                name: "noop".to_string(),
                description: "does nothing".to_string(),
                input_schema: serde_json::json!({"type": "object", "properties": {}}),
            }
        }

        async fn call(&self, _params: Value) -> Result<McpToolCallResponse> {
            Ok(McpToolCallResponse {
                content: vec![McpContent::Text { text: "ok".to_string() }],
                is_error: None,
            })
        }
    }

    fn request(method: &str, params: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params),
            id: Some(serde_json::json!(1)),
        }
    }

    fn server_with_tool() -> McpServer {
        let mut server = McpServer::new("test-server", "0.1.0");
        server.register_tool(NoopTool);
        server
    }

    /// Regression guard: a server that implements tools/list MUST advertise the
    /// tools capability during initialize. Clients that honour the capability
    /// map (Claude Code among them) never call tools/list otherwise, so the
    /// connection succeeds while exposing zero tools.
    #[tokio::test]
    async fn initialize_advertises_tools_capability() {
        let server = server_with_tool();
        let res = server
            .handle_request(request("initialize", serde_json::json!({"protocolVersion": "2025-06-18"})))
            .await;

        let result = res.result.expect("initialize must return a result");
        assert!(
            result["capabilities"].get("tools").is_some(),
            "initialize must advertise the tools capability, got capabilities = {}",
            result["capabilities"]
        );
    }

    async fn negotiated_version(params: Value) -> Value {
        let res = server_with_tool()
            .handle_request(request("initialize", params))
            .await;
        res.result.expect("initialize must return a result")["protocolVersion"].clone()
    }

    #[tokio::test]
    async fn initialize_echoes_a_supported_client_protocol_version() {
        for version in SUPPORTED_PROTOCOL_VERSIONS {
            let got = negotiated_version(serde_json::json!({"protocolVersion": version})).await;
            assert_eq!(got, *version);
        }
    }

    /// Regression guard: echoing an unsupported version claims support for a
    /// protocol we do not implement. 2026-07-28 drops initialize entirely and
    /// requires server/discover, so a client believing that claim breaks.
    #[tokio::test]
    async fn initialize_counters_an_unsupported_version_with_our_latest() {
        for version in ["2026-07-28", "1999-01-01", "garbage"] {
            let got = negotiated_version(serde_json::json!({"protocolVersion": version})).await;
            assert_eq!(got, LATEST_PROTOCOL_VERSION, "requested {version}");
        }
    }

    #[tokio::test]
    async fn initialize_without_a_version_offers_our_latest() {
        let got = negotiated_version(serde_json::json!({})).await;
        assert_eq!(got, LATEST_PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn initialize_reports_server_identity() {
        let server = server_with_tool();
        let res = server
            .handle_request(request("initialize", serde_json::json!({})))
            .await;

        let result = res.result.expect("initialize must return a result");
        assert_eq!(result["serverInfo"]["name"], "test-server");
        assert_eq!(result["serverInfo"]["version"], "0.1.0");
    }

    /// The advertised capability must match reality: tools/list has to return
    /// the registered tools, otherwise the advertisement is a lie in the other
    /// direction.
    #[tokio::test]
    async fn tools_list_returns_registered_tools() {
        let server = server_with_tool();
        let res = server
            .handle_request(request("tools/list", serde_json::json!({})))
            .await;

        let result = res.result.expect("tools/list must return a result");
        let tools = result["tools"].as_array().expect("tools must be an array");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "noop");
    }

    #[tokio::test]
    async fn unknown_method_returns_method_not_found() {
        let server = server_with_tool();
        let res = server
            .handle_request(request("nope/nope", serde_json::json!({})))
            .await;

        assert!(res.result.is_none());
        assert_eq!(res.error.expect("expected an error")["code"], -32601);
    }
}
