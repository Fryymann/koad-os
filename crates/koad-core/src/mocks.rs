use mockall::mock;
use crate::intelligence::IntelligenceRouter;
use anyhow::Result;
use async_trait::async_trait;

mock! {
    pub IntelligenceRouter {}
    #[async_trait]
    impl IntelligenceRouter for IntelligenceRouter {
        async fn summarize(&self, text: &str) -> Result<String>;
        async fn analyze(&self, text: &str) -> Result<String>;
    }
}
