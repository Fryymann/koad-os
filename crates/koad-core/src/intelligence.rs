use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait IntelligenceRouter: Send + Sync {
    async fn summarize(&self, text: &str) -> Result<String>;
    async fn analyze(&self, text: &str) -> Result<String>;
}
