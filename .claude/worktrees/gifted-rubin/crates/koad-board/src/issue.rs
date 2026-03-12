use super::GitHubClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Issue {
    pub number: i32,
    pub title: String,
    pub state: String,
    pub body: Option<String>,
}

impl GitHubClient {
    pub async fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: Vec<String>,
    ) -> Result<Issue> {
        let url = format!(
            "{}/repos/{}/{}/issues",
            koad_core::constants::GITHUB_API_BASE,
            self.owner,
            self.repo
        );
        let payload = serde_json::json!({
            "title": title,
            "body": body,
            "labels": labels,
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let err = response.text().await?;
            anyhow::bail!("Failed to create issue: {}", err);
        }

        Ok(response.json().await?)
    }

    pub async fn list_issues(&self, state: &str) -> Result<Vec<Issue>> {
        let path = format!("issues?state={}", state);
        self.get_rest(&path).await
    }

    pub async fn close_issue(&self, number: i32) -> Result<()> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            koad_core::constants::GITHUB_API_BASE,
            self.owner,
            self.repo,
            number
        );
        let payload = serde_json::json!({
            "state": "closed",
        });

        let response = self.client.patch(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let err = response.text().await?;
            anyhow::bail!("Failed to close issue #{}: {}", number, err);
        }

        println!("Issue #{} closed on GitHub.", number);
        Ok(())
    }
}
