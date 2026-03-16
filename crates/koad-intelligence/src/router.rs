//! Inference Routing & Task Management

use crate::clients::OllamaClient;
use crate::InferenceClient;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

/// High-level categories for intelligence tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceTask {
    /// Summarization and history distillation (Local preferred).
    Distillation,
    /// Significance scoring and fact extraction (Local preferred).
    Evaluation,
    /// Complex multi-step reasoning or technical architecture (Cloud preferred).
    Reasoning,
}

/// A router that selects the appropriate [`InferenceClient`] based on task and availability.
pub struct InferenceRouter {
    local_client: Arc<dyn InferenceClient>,
    // cloud_client: Option<Arc<GeminiClient>>, // Reserved for Phase 3.5
}

impl InferenceRouter {
    /// Create a new router with the specified clients.
    pub fn new(local_client: Arc<dyn InferenceClient>) -> Self {
        Self { local_client }
    }

    /// Create a new router with default clients (Local Ollama).
    /// 
    /// # Errors
    /// Returns an error if the default Ollama client cannot be built.
    pub fn new_default() -> Result<Self> {
        info!("InferenceRouter: Initializing with default Ollama client.");
        Ok(Self::new(Arc::new(OllamaClient::new(None, None)?)))
    }

    /// Select a client for the given task.
    /// Currently defaults to Ollama for all tasks until Gemini is wired.
    pub fn select(&self, _task: InferenceTask) -> Arc<dyn InferenceClient> {
        self.local_client.clone()
    }

    /// Convenience: Route a summarization request.
    pub async fn summarize(&self, text: &str) -> Result<String> {
        self.select(InferenceTask::Distillation)
            .summarize(text)
            .await
    }

    /// Convenience: Route a significance scoring request.
    pub async fn score(&self, text: &str) -> Result<f32> {
        self.select(InferenceTask::Evaluation)
            .score_significance(text)
            .await
    }
}
