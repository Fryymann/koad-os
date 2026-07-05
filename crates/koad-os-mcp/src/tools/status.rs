use anyhow::Result;
use async_trait::async_trait;
use koad_mcp::{McpContent, McpTool, McpToolCallResponse, McpToolHandler};
use koad_proto::cass::v1::memory_service_client::MemoryServiceClient;
use koad_proto::cass::v1::pulse_service_client::PulseServiceClient;
use koad_proto::cass::v1::{FactQuery, GetPulsesRequest};
use serde_json::{json, Value};

pub struct StatusTool {
    cass_url: String,
    partition: String,
}

impl StatusTool {
    pub fn new(cass_url: String, partition: String) -> Self {
        Self { cass_url, partition }
    }
}

#[async_trait]
impl McpToolHandler for StatusTool {
    fn definition(&self) -> McpTool {
        McpTool {
            name: "status.citadel".to_string(),
            description: "Check CASS health and partition statistics".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _params: Value) -> Result<McpToolCallResponse> {
        let memory_status = match MemoryServiceClient::connect(self.cass_url.clone()).await {
            Ok(mut client) => {
                match client
                    .query_facts(FactQuery {
                        domain: self.partition.clone(),
                        tags: vec![],
                        limit: 1000,
                        min_level: 0,
                    })
                    .await
                {
                    Ok(resp) => format!("ONLINE — {} cards in partition", resp.into_inner().facts.len()),
                    Err(e) => format!("DEGRADED — {e}"),
                }
            }
            Err(e) => format!("OFFLINE — {e}"),
        };

        let pulse_status = match PulseServiceClient::connect(self.cass_url.clone()).await {
            Ok(mut client) => {
                match client
                    .get_pulses(GetPulsesRequest { role: "global".to_string(), context: None })
                    .await
                {
                    Ok(resp) => format!("{} active pulses", resp.into_inner().pulses.len()),
                    Err(_) => "pulse unavailable".to_string(),
                }
            }
            Err(_) => "pulse unavailable".to_string(),
        };

        let text = format!(
            "**CASS Status**\n- Memory: {memory_status}\n- Pulse: {pulse_status}\n- Partition: `{}`",
            self.partition
        );

        Ok(McpToolCallResponse {
            content: vec![McpContent::Text { text }],
            is_error: None,
        })
    }
}
