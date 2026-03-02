use serde::Deserialize;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use anyhow::Result;

pub mod project;
pub mod issue;
pub mod actions;

pub struct GitHubClient {
    client: reqwest::Client,
    owner: String,
    repo: String,
}

impl GitHubClient {
    pub fn new(token: String, owner: String, repo: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("token {}", token))?);
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
    where T: for<'de> Deserialize<'de> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        let response = self.client.post("https://api.github.com/graphql")
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

        let data = json.get("data").ok_or_else(|| anyhow::anyhow!("No data in response"))?.clone();
        Ok(serde_json::from_value(data)?)
    }

    /// Execute a REST API request (GET).
    pub async fn get_rest<T>(&self, path: &str) -> Result<T>
    where T: for<'de> Deserialize<'de> {
        let url = format!("https://api.github.com/repos/{}/{}/{}", self.owner, self.repo, path);
        let response = self.client.get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("REST GET request failed: {}", error_text);
        }

        Ok(response.json().await?)
    }
}
