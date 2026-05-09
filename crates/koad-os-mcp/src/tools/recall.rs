use anyhow::Result;
use async_trait::async_trait;
use koad_mcp::{McpContent, McpTool, McpToolCallResponse, McpToolHandler};
use koad_proto::cass::v1::memory_service_client::MemoryServiceClient;
use koad_proto::cass::v1::FactQuery;
use serde_json::{json, Value};

pub struct RecallTool {
    cass_url: String,
    partition: String,
}

impl RecallTool {
    pub fn new(cass_url: String, partition: String) -> Self {
        Self { cass_url, partition }
    }
}

#[async_trait]
impl McpToolHandler for RecallTool {
    fn definition(&self) -> McpTool {
        McpTool {
            name: "memory.recall".to_string(),
            description: "Fetch recent memory cards from prior sessions".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Max number of cards to return (default 10)",
                        "default": 10
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, params: Value) -> Result<McpToolCallResponse> {
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as u32;

        let mut client = MemoryServiceClient::connect(self.cass_url.clone()).await?;
        let resp = client
            .query_facts(FactQuery {
                domain: self.partition.clone(),
                tags: vec![],
                limit,
                min_level: 0,
            })
            .await?
            .into_inner();

        let text = if resp.facts.is_empty() {
            "No memory cards found for this session partition.".to_string()
        } else {
            resp.facts
                .iter()
                .map(|f| format!("**[{}]** {}\n_{}_", f.domain, f.content, f.id))
                .collect::<Vec<_>>()
                .join("\n\n---\n\n")
        };

        Ok(McpToolCallResponse {
            content: vec![McpContent::Text { text }],
            is_error: None,
        })
    }
}
