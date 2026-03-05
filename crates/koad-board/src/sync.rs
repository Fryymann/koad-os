use crate::GitHubClient;
use anyhow::Result;
use std::collections::HashSet;
use tracing::info;

pub struct BoardSyncer<'a> {
    client: &'a GitHubClient,
    project_number: i32,
    dry_run: bool,
}

impl<'a> BoardSyncer<'a> {
    pub fn new(client: &'a GitHubClient, project_number: i32, dry_run: bool) -> Self {
        Self {
            client,
            project_number,
            dry_run,
        }
    }

    /// Perform a full reconciliation of the repository issues and the project board.
    pub async fn run(&self) -> Result<()> {
        info!("--- [SGP] Sovereign GitHub Protocol Sync Starting ---");
        if self.dry_run {
            info!("[DRY RUN] No changes will be persisted.");
        }

        // 1. Fetch the Project ID and Field Metadata
        let project_id = self.client.get_project_id(self.project_number).await?;
        let _ = self.client.get_status_field_id(&project_id).await?;
        let _ = self
            .client
            .get_status_option_id(&project_id, "Todo")
            .await?;

        // Task Weight metadata
        let weight_field_id = self.client.get_field_id(&project_id, "Task Weight").await?;

        // 2. Fetch current Project Items to avoid duplicates and track existing state
        let current_items = self.client.list_project_items(self.project_number).await?;
        let existing_numbers: HashSet<i32> =
            current_items.iter().filter_map(|i| i.number).collect();

        // 3. Fetch all open issues in the repository
        // We need the body for Task Weight extraction
        let query = r#"
            query($owner: String!, $repo: String!) {
              repository(owner: $owner, name: $repo) {
                issues(first: 100, states: OPEN) {
                  nodes {
                    id
                    number
                    title
                    body
                    milestone {
                      title
                    }
                  }
                }
              }
            }
        "#;
        let variables = serde_json::json!({
            "owner": self.client.owner,
            "repo": self.client.repo,
        });

        let data: serde_json::Value = self.client.graphql(query, variables).await?;
        let nodes = data["repository"]["issues"]["nodes"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No issues found"))?;

        // 4. Reconciliation Loop
        for node in nodes {
            let content_id = node["id"].as_str().unwrap_or_default().to_string();
            let number = node["number"].as_i64().unwrap_or_default() as i32;
            let title = node["title"].as_str().unwrap_or_default().to_string();
            let body = node["body"].as_str().unwrap_or_default().to_string();
            let milestone = node["milestone"]["title"].as_str();

            // Extract Task Weight from body
            // Format: "## Task Weight\n[trivial | standard | complex]" or "## Task Weight\ntrivial"
            let weight_choice = if body.contains("trivial") {
                Some("Trivial")
            } else if body.contains("complex") {
                Some("Complex")
            } else if body.contains("standard") {
                Some("Standard")
            } else {
                None
            };

            // Check if the issue is already in the project
            let item_id = if !existing_numbers.contains(&number) {
                info!("[SYNC] Adding Issue #{} '{}' to Backlog...", number, title);
                if !self.dry_run {
                    let res = self
                        .client
                        .add_item_to_project(&project_id, &content_id)
                        .await?;

                    // Rule: If milestone is present, move to 'Todo' immediately
                    if milestone.is_some() {
                        info!("[SYNC] Milestone detected. Moving #{} to Todo...", number);
                        self.client
                            .update_item_status(self.project_number, number, "Todo")
                            .await?;
                    }
                    Some(res)
                } else {
                    None
                }
            } else {
                // If it's already in, check if it needs to move to 'Todo'
                if let Some(item) = current_items.iter().find(|i| i.number == Some(number)) {
                    if (item.status == "Unknown" || item.status == "Backlog") && milestone.is_some()
                    {
                        info!("[SYNC] Existing issue #{} has Milestone but is in Backlog. Moving to Todo...", number);
                        if !self.dry_run {
                            self.client
                                .update_item_status(self.project_number, number, "Todo")
                                .await?;
                        }
                    }
                    Some(item.id.clone())
                } else {
                    None
                }
            };

            // Sync Task Weight
            if let (Some(item_id), Some(weight)) = (item_id, weight_choice) {
                if !self.dry_run {
                    let option_id = self
                        .client
                        .get_single_select_option_id(&project_id, "Task Weight", weight)
                        .await?;
                    self.client
                        .update_item_field(
                            &project_id,
                            &item_id,
                            &weight_field_id,
                            serde_json::json!({ "singleSelectOptionId": option_id }),
                        )
                        .await?;
                }
            }
        }

        info!("--- [SGP] Sync Complete. Foundation Stable. ---");
        Ok(())
    }
}
