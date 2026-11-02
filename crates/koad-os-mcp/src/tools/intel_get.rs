use anyhow::Result;
use async_trait::async_trait;
use koad_mcp::{McpContent, McpTool, McpToolCallResponse, McpToolHandler};
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
            description: "Retrieve memory cards by topic or domain within this agent's partition".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "domain": {
                        "type": "string",
                        "description": "Topic or full domain to retrieve. Limited to this agent's partition."
                    }
                },
                "required": ["domain"]
            }),
        }
    }

    async fn call(&self, params: Value) -> Result<McpToolCallResponse> {
        let requested = params.get("domain").and_then(|v| v.as_str()).unwrap_or("");
        let domain = scoped_domain(&self.partition, requested);

        let mut client = super::cass::memory(&self.cass_url).await?;
        let resp = super::cass::call(client.query_facts(FactQuery {
            domain,
            tags: vec![],
            limit: 5,
            min_level: 0,
        }))
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

/// Confines a caller's domain to this server's partition.
///
/// CASS treats `FactQuery.domain` as the partition and has no access control of
/// its own, so this is the check that keeps one agent out of another's memory.
/// Anything that isn't the partition or a domain inside it becomes a topic within
/// it, so a request naming another partition simply finds nothing.
fn scoped_domain(partition: &str, requested: &str) -> String {
    let inside = requested
        .strip_prefix(partition)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(':'));
    if inside {
        requested.to_string()
    } else if requested.is_empty() {
        partition.to_string()
    } else {
        format!("{partition}:{requested}")
    }
}

#[cfg(test)]
mod tests {
    use super::scoped_domain;

    const PARTITION: &str = "clyde_Jupiter_ideans";

    #[test]
    fn own_partition_is_kept() {
        assert_eq!(scoped_domain(PARTITION, PARTITION), PARTITION);
    }

    #[test]
    fn own_full_domain_is_kept() {
        let d = "clyde_Jupiter_ideans:learning";
        assert_eq!(scoped_domain(PARTITION, d), d);
    }

    #[test]
    fn bare_topic_is_scoped_to_partition() {
        assert_eq!(
            scoped_domain(PARTITION, "learning"),
            "clyde_Jupiter_ideans:learning"
        );
    }

    #[test]
    fn empty_request_means_the_partition() {
        assert_eq!(scoped_domain(PARTITION, ""), PARTITION);
    }

    /// Every result must stay inside the partition, whatever the caller sends.
    #[test]
    fn nothing_escapes_the_partition() {
        for hostile in [
            "hermes_jupiter_ideans",
            "hermes_jupiter_ideans:system-check",
            "clyde_Jupiter_ideans_evil",
            "clyde_Jupiter_ideans_evil:notes",
            "clyde_jupiter_ideans:wrong-case",
            "%",
            "%:%",
            "_",
        ] {
            let scoped = scoped_domain(PARTITION, hostile);
            assert!(
                scoped.starts_with("clyde_Jupiter_ideans:"),
                "{hostile:?} escaped as {scoped:?}"
            );
        }
    }
}
