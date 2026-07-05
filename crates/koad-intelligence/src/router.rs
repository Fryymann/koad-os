//! Inference Routing & Task Management

use koad_core::intelligence::IntelligenceRouter;
use crate::clients::OllamaClient;
use crate::InferenceClient;
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

/// High-level categories for intelligence tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceTask {
    /// Summarization and history distillation (Local preferred).
    Distillation,
    /// Significance scoring and fact extraction (Local preferred).
    Evaluation,
    /// Complex multi-step reasoning or technical architecture (Cloud preferred).
    Reasoning,
    /// Dense vector embedding generation (dedicated local embedding model).
    Embedding,
}

/// A router that selects the appropriate [`InferenceClient`] based on task and availability.
pub struct InferenceRouter {
    local_client: Arc<dyn InferenceClient>,
    embed_client: Arc<dyn InferenceClient>,
}

impl InferenceRouter {
    /// Create a new router with the specified client for all tasks.
    pub fn new(local_client: Arc<dyn InferenceClient>) -> Self {
        Self {
            embed_client: local_client.clone(),
            local_client,
        }
    }

    /// Override the client used for `InferenceTask::Embedding`.
    pub fn with_embed_client(mut self, embed_client: Arc<dyn InferenceClient>) -> Self {
        self.embed_client = embed_client;
        self
    }

    /// Create a new router with default clients (Local Ollama).
    ///
    /// Chat/enrichment model from `KOADOS_INTEL_MODEL` (default `granite3.3:2b`).
    /// Embedding model from `KOADOS_EMBED_MODEL` (default `nomic-embed-text`).
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be built.
    pub fn new_default() -> Result<Self> {
        let model =
            std::env::var("KOADOS_INTEL_MODEL").unwrap_or_else(|_| "granite3.3:2b".to_string());
        let embed_model =
            std::env::var("KOADOS_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());
        info!(model = %model, embed_model = %embed_model, "InferenceRouter: Initializing Ollama clients.");
        Ok(
            Self::new(Arc::new(OllamaClient::new(Some(&model), None)?))
                .with_embed_client(Arc::new(OllamaClient::new(Some(&embed_model), None)?)),
        )
    }

    /// Select a client for the given task.
    pub fn select(&self, task: InferenceTask) -> Arc<dyn InferenceClient> {
        match task {
            InferenceTask::Embedding => self.embed_client.clone(),
            _ => self.local_client.clone(),
        }
    }

    /// Convenience: Route a summarization request.
    /// Falls back to returning the original text if the model is unavailable.
    pub async fn summarize(&self, text: &str) -> Result<String> {
        match self.select(InferenceTask::Distillation).summarize(text).await {
            Ok(summary) => Ok(summary),
            Err(e) => {
                warn!("Inference unavailable for summarize ({}), returning raw text.", e);
                Ok(text.to_string())
            }
        }
    }

    /// Convenience: Route a significance scoring request.
    /// Falls back to 1.0 (store everything) if the model is unavailable.
    pub async fn score(&self, text: &str) -> Result<f32> {
        match self.select(InferenceTask::Evaluation).score_significance(text).await {
            Ok(score) => Ok(score),
            Err(e) => {
                warn!("Inference unavailable for score ({}), defaulting to 1.0.", e);
                Ok(1.0)
            }
        }
    }

    /// Generate a dense vector embedding. Local-only by design: cloud embedding
    /// models emit different dimensions than the local Qdrant collections, so a
    /// cloud fallback vector would be unusable. Errors propagate to the caller
    /// (the enrichment worker retries).
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.select(InferenceTask::Embedding).embed(text).await
    }
}

#[async_trait::async_trait]
impl IntelligenceRouter for InferenceRouter {
    async fn summarize(&self, text: &str) -> Result<String> {
        self.summarize(text).await
    }
    async fn analyze(&self, text: &str) -> Result<String> {
        self.analyze(text).await
    }
}
