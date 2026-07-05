use anyhow::Result;
use async_trait::async_trait;
use koad_mcp::{McpContent, McpTool, McpToolCallResponse, McpToolHandler};
use koad_proto::cass::v1::memory_service_client::MemoryServiceClient;
use koad_proto::cass::v1::FactQuery;
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub struct ListTopicsTool {
    cass_url: String,
    partition: String,
}

impl ListTopicsTool {
    pub fn new(cass_url: String, partition: String) -> Self {
        Self { cass_url, partition }
    }
}

#[async_trait]
impl McpToolHandler for ListTopicsTool {
    fn definition(&self) -> McpTool {
        McpTool {
            name: "memory.list_topics".to_string(),
            description: "List known topics and domains in this partition's memory".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _params: Value) -> Result<McpToolCallResponse> {
        let mut client = MemoryServiceClient::connect(self.cass_url.clone()).await?;
        let resp = client
            .query_facts(FactQuery {
                domain: self.partition.clone(),
                tags: vec![],
                limit: 200,
                min_level: 0,
            })
            .await?
            .into_inner();

        let topics: BTreeSet<String> = resp.facts.iter().map(|f| f.domain.clone()).collect();

        let text = if topics.is_empty() {
            "No topics found in memory.".to_string()
        } else {
            format!(
                "Known topics ({}):\n\n{}",
                topics.len(),
                topics.iter().map(|t| format!("- {t}")).collect::<Vec<_>>().join("\n")
            )
        };

        Ok(McpToolCallResponse {
            content: vec![McpContent::Text { text }],
            is_error: None,
        })
    }
}
