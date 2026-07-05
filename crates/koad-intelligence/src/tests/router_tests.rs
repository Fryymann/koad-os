use crate::router::InferenceRouter;
use crate::InferenceClient;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

struct MockInferenceClient {
    pub reply: String,
    pub score: f32,
}

#[async_trait]
impl InferenceClient for MockInferenceClient {
    async fn chat(&self, _prompt: &str) -> Result<String> {
        Ok(self.reply.clone())
    }
    async fn summarize(&self, _text: &str) -> Result<String> {
        Ok(self.reply.clone())
    }
    async fn score_significance(&self, _text: &str) -> Result<f32> {
        Ok(self.score)
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.0f32; 32])
    }
}

#[tokio::test]
async fn test_router_selects_and_summarizes() -> Result<()> {
    let mock = Arc::new(MockInferenceClient {
        reply: "Summary complete".to_string(),
        score: 0.9,
    });

    let router = InferenceRouter::new(mock.clone());

    let summary = router.summarize("test input").await?;
    assert_eq!(summary, "Summary complete");

    let score = router.score("test content").await?;
    assert_eq!(score, 0.9);

    Ok(())
}

struct DimMockClient {
    dim: usize,
}

#[async_trait]
impl InferenceClient for DimMockClient {
    async fn chat(&self, _prompt: &str) -> Result<String> {
        Ok("chat".to_string())
    }
    async fn summarize(&self, _text: &str) -> Result<String> {
        Ok("summary".to_string())
    }
    async fn score_significance(&self, _text: &str) -> Result<f32> {
        Ok(0.5)
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.5f32; self.dim])
    }
}

#[tokio::test]
async fn test_embedding_task_routes_to_embed_client() -> Result<()> {
    let chat_client = Arc::new(DimMockClient { dim: 4 });
    let embed_client = Arc::new(DimMockClient { dim: 8 });

    let router = InferenceRouter::new(chat_client).with_embed_client(embed_client);

    // embed() must use the dedicated embed client (dim 8), not the chat client (dim 4)
    let vec = router.embed("hello").await?;
    assert_eq!(vec.len(), 8);
    Ok(())
}

#[tokio::test]
async fn test_embed_defaults_to_local_client_without_embed_client() -> Result<()> {
    let chat_client = Arc::new(DimMockClient { dim: 4 });
    let router = InferenceRouter::new(chat_client);
    let vec = router.embed("hello").await?;
    assert_eq!(vec.len(), 4);
    Ok(())
}
