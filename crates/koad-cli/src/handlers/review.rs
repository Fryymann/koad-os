//! # KoadOS Automated Code Review (ACR)
//!
//! Implements a three-stage audit pipeline for Rust codebases to ensure
//! compliance with the KoadOS RUST_CANON and Zero-Panic policies.
//!
//! 1. **Deterministic**: Linting and Zero-Panic (no unwraps/expects) checks.
//! 2. **Technical**: Documentation and pattern audit via Gemma 3:4B.
//! 3. **Architectural**: Canon compliance and async safety via Qwen 2.5:14B.

use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_intelligence::clients::OllamaClient;
use koad_intelligence::InferenceClient;
use std::path::Path;
use tokio::fs;
use tracing::{info, debug, instrument};

/// Performs an automated code review on a single file.
pub struct Reviewer {
    config: KoadConfig,
    llama: OllamaClient,
    qwen: OllamaClient,
}

impl Reviewer {
    /// Create a new reviewer using local Ollama models.
    ///
    /// # Errors
    /// Returns an error if the Ollama client cannot be initialized for Llama or Qwen models.
    pub fn new(config: KoadConfig) -> Result<Self> {
        Ok(Self {
            config,
            llama: OllamaClient::new(Some("llama3.2:1b"), None)?,
            qwen: OllamaClient::new(Some("qwen2.5-coder:7b"), None)?,
        })
    }

    /// Runs the full review pipeline for the given file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or if the AI inference pipeline fails.
    #[instrument(skip(self, file_path), fields(file = %file_path.display()))]
    pub async fn review_file(&self, file_path: &Path) -> Result<()> {
        info!("Starting KoadOS Code Review");
        
        let content = fs::read_to_string(file_path).await
            .with_context(|| format!("Failed to read file for review: {}", file_path.display()))?;

        println!("🔍 Starting KoadOS Code Review for: {}", file_path.display());
        println!("═══════════════════════════════════════════════");

        // --- STAGE 1: DETERMINISTIC CHECKS ---
        self.run_deterministic_checks(&content);

        // --- STAGE 2: GEMMA 3 (Technical & Documentation Audit) ---
        self.run_technical_audit(&content).await?;

        // --- STAGE 3: QWEN 2.5 (Architectural Alignment) ---
        self.run_architectural_audit(&content).await?;

        println!("\n═══════════════════════════════════════════════");
        println!("✅ Review Complete.");
        Ok(())
    }

    /// Performs deterministic regex-based checks for Zero-Panic policy violations.
    #[instrument(skip(self, content))]
    fn run_deterministic_checks(&self, content: &str) {
        debug!("Running deterministic checks");
        println!("📦 Running Deterministic Checks...");
        let unwrap_count = content.matches(".unwrap()").count();
        let expect_count = content.matches(".expect(").count();

        if unwrap_count > 0 || expect_count > 0 {
            println!("⚠️  WARNING: Found {} unwraps and {} expects. (Violates Zero-Panic Policy)", 
                unwrap_count, expect_count);
        } else {
            println!("✅ Zero-Panic Check: PASSED");
        }
    }

    /// Triggers the Technical Audit stage using Llama 3.2:1B.
    ///
    /// # Errors
    /// Returns an error if the local model inference fails.
    #[instrument(skip(self, content))]
    async fn run_technical_audit(&self, content: &str) -> Result<()> {
        info!("🧠 ABC: Technical Audit via Llama 3.2:1B...");
        println!("\n🧠 Llama 3.2:1B: Auditing Documentation & Patterns...");
        
        // Focus Llama on the first 100 lines for efficiency (headers/docs)
        let snippet = content.lines().take(100).collect::<Vec<_>>().join("\n");
        
        let llama_prompt = format!(
            "You are a KoadOS Technical Auditor. Review the following Rust code for compliance with these rules:\n\
            1. Every file must have a //! module header.\n\
            2. Every public function must have /// doc comments and a '# Errors' section.\n\
            3. No usage of .unwrap() or .expect().\n\
            List only the violations found. If none, say 'DOCUMENTATION: CLEAN'.\n\n\
            ### CODE:\n{}",
            snippet
        );

        let report = self.llama.chat(&llama_prompt).await
            .context("Local technical audit via Llama 3.2:1B failed.")?;
            
        println!("{}", report.trim());
        Ok(())
    }

    /// Triggers the Architectural Audit stage using Qwen 2.5-Coder:7B.
    ///
    /// # Errors
    /// Returns an error if the local model inference fails or if the Canon file cannot be read.
    #[instrument(skip(self, content))]
    async fn run_architectural_audit(&self, content: &str) -> Result<()> {
        info!("♟️ ABC: Architectural Audit via Qwen 2.5-Coder:7B...");
        println!("\n♟️ Qwen 2.5:7B: Auditing Architectural Alignment...");
        
        let canon_path = self.config.home.join("docs/protocols/CONTRIBUTOR_CANON.md");
        let canon = if canon_path.exists() {
            fs::read_to_string(&canon_path).await
                .context("Failed to read CONTRIBUTOR_CANON.md")?
        } else {
            "No Canon file found. Use general senior Rust standards: Async Safety, Observability (tracing), Structural Error Handling (anyhow).".to_string()
        };

        let qwen_prompt = format!(
            "You are the KoadOS Chief Engineer. Compare the PROVIDED CODE against the RUST_CANON standards.\n\
            Focus on:\n\
            1. Async Safety: Are blocking calls (std::fs, rusqlite) used inside async functions without spawn_blocking?\n\
            2. Observability: Is the 'tracing' crate used with structured fields and #[instrument]?\n\
            3. Structural: Are errors propagated correctly using anyhow::Result and .context()?\n\n\
            ### CANON:\n{}\n\n\
            ### CODE:\n{}\n\n\
            Output a concise 'Compliance Report' with 'PASSED' or 'ACTION REQUIRED' for each focus area.",
            canon, content
        );

        let report = self.qwen.chat(&qwen_prompt).await
            .context("Local architectural audit via Qwen 2.5-Coder:7B failed.")?;
            
        println!("{}", report.trim());
        Ok(())
    }
}

/// Main entry point for the review handler.
///
/// # Errors
/// Returns an error if the reviewer cannot be initialized or the file review fails.
#[instrument(skip(config))]
pub async fn handle_review(file_path: &Path, config: &KoadConfig) -> Result<()> {
    let reviewer = Reviewer::new(config.clone())?;
    reviewer.review_file(file_path).await
}
