use anyhow::Result;
use async_trait::async_trait;
use koad_mcp::{McpContent, McpTool, McpToolCallResponse, McpToolHandler};
use koad_proto::cass::v1::memory_service_client::MemoryServiceClient;
use koad_proto::cass::v1::FactQuery;
use serde_json::{json, Value};

pub struct IntelGetTool {
    cass_url: String,
    partition: String,
}

impl IntelGetTool {
    pub fn new(cass_url: String, partition: String) -> Self {
        Self { cass_url, partition }
    }
}

#[async_trait]
impl McpToolHandler for IntelGetTool {
    fn definition(&self) -> McpTool {
        McpTool {
            name: "intel.get".to_string(),
            description: "Retrieve a specific memory card by ID or domain".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "domain": {
                        "type": "string",
                        "description": "Domain or topic to retrieve"
                    }
                },
                "required": ["domain"]
            }),
        }
    }

    async fn call(&self, params: Value) -> Result<McpToolCallResponse> {
        let domain = params
            .get("domain")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut client = MemoryServiceClient::connect(self.cass_url.clone()).await?;
        let resp = client
            .query_facts(FactQuery {
                domain,
                tags: vec![],
                limit: 5,
                min_level: 0,
            })
            .await?
            .into_inner();

        let text = if resp.facts.is_empty() {
            "No memory card found for that domain.".to_string()
        } else {
            resp.facts
                .iter()
                .map(|f| {
                    format!(
                        "**{}** (confidence: {:.0}%)\n{}\n_id: {}_",
                        f.domain,
                        f.confidence * 100.0,
                        f.content,
                        f.id
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        Ok(McpToolCallResponse {
            content: vec![McpContent::Text { text }],
            is_error: None,
        })
    }
}
