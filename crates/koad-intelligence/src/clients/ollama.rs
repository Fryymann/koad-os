//! Ollama Local Inference Client

use crate::InferenceClient;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, error};

/// A client for the local Ollama API (http://localhost:11434).
pub struct OllamaClient {
    client: Client,
    model: String,
    base_url: String,
}

impl OllamaClient {
    /// Create a new Ollama client with the specified model (default: "mistral").
    pub fn new(model: Option<&str>, base_url: Option<&str>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .connect_timeout(Duration::from_secs(1))
                .build()
                .unwrap_or_default(),
            model: model.unwrap_or("mistral").to_string(),
            base_url: base_url.unwrap_or("http://localhost:11434").to_string(),
        }
    }
}

#[async_trait]
impl InferenceClient for OllamaClient {
    async fn chat(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        debug!(model = %self.model, "Ollama: Sending request to {}", url);

        let response = self
            .client
            .post(&url)
            .json(&json!({
                "model": self.model,
                "prompt": prompt,
                "stream": false
            }))
            .send()
            .await
            .context(
                "Failed to connect to Ollama. Ensure it is running at http://localhost:11434.",
            )?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            error!(status = %status, error = %err_text, "Ollama API Error");
            return Err(anyhow::anyhow!("Ollama API returned error: {}", err_text));
        }

        let body: serde_json::Value = response.json().await?;
        Ok(body["response"]
            .as_str()
            .unwrap_or_default()
            .trim()
            .to_string())
    }

    async fn summarize(&self, text: &str) -> Result<String> {
        let prompt = format!(
            "Summarize the following session history for an AI agent. Focus on key decisions, task status, and technical findings. Output ONLY a concise markdown summary.\n\n### TEXT TO SUMMARIZE:\n{}",
            text
        );
        self.chat(&prompt).await
    }

    async fn score_significance(&self, text: &str) -> Result<f32> {
        let prompt = format!(
            "Analyze the following technical content and score its 'significance' for long-term memory on a scale of 0.0 to 1.0. 
            0.0 means noise (errors, greetings, trivialities). 
            1.0 means critical (architectural decisions, resolved bugs, user preferences).
            Output ONLY the numeric score.\n\n### CONTENT:\n{}",
            text
        );

        let response = self.chat(&prompt).await?;
        let score: f32 = response.parse().unwrap_or_else(|_| {
            debug!(
                "Ollama: Failed to parse significance score from '{}', defaulting to 0.5",
                response
            );
            0.5
        });

        Ok(score.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    // Integration test placeholder (Requires live Ollama)
    // In a real project, we'd use mockito/wiremock here.
}
