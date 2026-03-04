use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

pub mod actions;
pub mod issue;
pub mod project;
pub mod sync;

pub struct GitHubClient {
    client: reqwest::Client,
    owner: String,
    repo: String,
}

impl GitHubClient {
    pub fn new(token: String, owner: String, repo: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("token {}", token))?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("KoadOS-Board-Bridge"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            owner,
            repo,
        })
    }

    /// Execute a GraphQL query.
    pub async fn graphql<T>(&self, query: &str, variables: serde_json::Value) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        let response = self
            .client
            .post(format!("{}/graphql", koad_core::constants::GITHUB_API_BASE))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("GraphQL request failed: {}", error_text);
        }

        let json: serde_json::Value = response.json().await?;
        if let Some(errors) = json.get("errors") {
            anyhow::bail!("GraphQL errors: {}", errors);
        }

        let data = json
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?
            .clone();
        Ok(serde_json::from_value(data)?)
    }

    /// Execute a REST API request (GET).
    pub async fn get_rest<T>(&self, path: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!(
            "{}/repos/{}/{}/{}",
            koad_core::constants::GITHUB_API_BASE,
            self.owner, self.repo, path
        );
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("REST GET request failed: {}", error_text);
        }

        Ok(response.json().await?)
    }

    /// Synchronize all open issues with the project board.
    pub async fn sync_issues(&self, project_number: i32) -> Result<()> {
        println!(
            "Syncing repository issues with Project #{}...",
            project_number
        );

        // 1. Get Project ID
        let project_id = self.get_project_id(project_number).await?;

        // 2. List current project items to avoid duplicates
        let current_items = self.list_project_items(project_number).await?;
        let existing_numbers: std::collections::HashSet<i32> =
            current_items.iter().filter_map(|i| i.number).collect();

        // 3. List open repository issues
        let open_issues = self.list_open_issues().await?;

        // 4. Add missing issues to project
        for (content_id, number, title, _milestone) in open_issues {
            if !existing_numbers.contains(&number) {
                println!("Adding Issue #{}: {} to project...", number, title);
                self.add_item_to_project(&project_id, &content_id).await?;
            }
        }

        println!("Sync complete.");
        Ok(())
    }

    /// Update the status of a project item.
    pub async fn update_item_status(
        &self,
        project_number: i32,
        issue_number: i32,
        status: &str,
    ) -> Result<()> {
        println!("Moving Issue #{} to {}...", issue_number, status);

        // 1. Get Project and Status IDs
        let project_id = self.get_project_id(project_number).await?;
        let status_field_id = self.get_status_field_id(&project_id).await?;
        let status_option_id = self.get_status_option_id(&project_id, status).await?;

        // 2. Find Item ID for the issue
        let items = self.list_project_items(project_number).await?;
        let item_id = items
            .iter()
            .find(|i| i.number == Some(issue_number))
            .map(|i| i.id.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Issue #{} not found in Project #{}",
                    issue_number,
                    project_number
                )
            })?;

        // 3. Update the item
        let query = r#"
            mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $optionId: String!) {
                updateProjectV2ItemFieldValue(
                    input: {
                        projectId: $projectId,
                        itemId: $itemId,
                        fieldId: $fieldId,
                        value: { singleSelectOptionId: $optionId }
                    }
                ) {
                    projectV2Item { id }
                }
            }
        "#;

        let variables = serde_json::json!({
            "projectId": project_id,
            "itemId": item_id,
            "fieldId": status_field_id,
            "optionId": status_option_id
        });

        let _: serde_json::Value = self.graphql(query, variables).await?;
        println!("Issue #{} successfully moved to {}.", issue_number, status);
        Ok(())
    }
}
