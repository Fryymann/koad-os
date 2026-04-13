//! # Hydration Service Implementation
//!
//! Performs the Temporal Context Hydration (TCH) walk to bundle
//! relevant facts and episodes for agent boot.
//! This service integrates structural code maps and intelligent history
//! distillation into a single, high-density markdown packet.

use crate::storage::PulseTier;
use koad_codegraph::CodeGraph;
use koad_core::hierarchy::HierarchyManager;
use koad_core::utils::tokens::count_tokens;
use koad_intelligence::router::InferenceRouter;
use koad_proto::cass::v1::hydration_service_server::HydrationService;
use koad_proto::cass::v1::{HydrationRequest, HydrationResponse};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

/// Service implementation for the `HydrationService` gRPC interface.
pub struct CassHydrationService {
    storage: Arc<dyn crate::storage::Storage>,
    hierarchy: Arc<HierarchyManager>,
    codegraph: Arc<CodeGraph>,
    intelligence: Arc<InferenceRouter>,
    pulse_store: Option<Arc<dyn PulseTier>>,
}

impl CassHydrationService {
    /// Creates a new `CassHydrationService`.
    pub fn new(
        storage: Arc<dyn crate::storage::Storage>,
        hierarchy: Arc<HierarchyManager>,
        codegraph: Arc<CodeGraph>,
        intelligence: Arc<InferenceRouter>,
    ) -> Self {
        Self {
            storage,
            hierarchy,
            codegraph,
            intelligence,
            pulse_store: None,
        }
    }

    /// Attaches an optional pulse store for Global Pulses section in TCH.
    pub fn with_pulse_store(mut self, store: Arc<dyn PulseTier>) -> Self {
        self.pulse_store = Some(store);
        self
    }
}

#[tonic::async_trait]
impl HydrationService for CassHydrationService {
    /// Bundles context for an agent based on their workspace level and token budget.
    ///
    /// # Errors
    /// Returns a `tonic::Status` if storage queries or intelligence distillation fail.
    async fn hydrate(
        &self,
        request: Request<HydrationRequest>,
    ) -> Result<Response<HydrationResponse>, Status> {
        let req = request.into_inner();
        let current_path = PathBuf::from(&req.project_root);
        let budget = req.token_budget as usize;
        let agent = &req.agent_name;
        let task_id = if req.task_id.is_empty() {
            None
        } else {
            Some(req.task_id.as_str())
        };

        info!(agent = %agent, path = %req.project_root, budget = %budget, task = ?task_id, "TCH: Hydration requested");

        let mut packet = format!(
            "# Temporal Context Hydration: {}
Date: {}

",
            agent,
            chrono::Utc::now().format("%Y-%m-%d")
        );
        let mut source_files = Vec::new();

        // 1. Agent History Distillation (Upgrade 2 + 3)
        let episodes = self
            .storage
            .query_recent_episodes(agent, 5, task_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !episodes.is_empty() {
            let mut raw_history = String::new();
            for ep in &episodes {
                raw_history.push_str(&format!("Session {}: {}\n", ep.session_id, ep.summary));
            }

            // Distill history into a State of the Union paragraph
            let prompt = format!(
                "Synthesize the following recent agent session reports into a single, high-density paragraph \
                 describing the current 'State of the Union' for the project. Focus on completed work, \
                 active blockers, and next steps. Reports:\n\n{}", 
                raw_history
            );

            let distilled = self
                .intelligence
                .summarize(&prompt)
                .await
                .unwrap_or_else(|_| {
                    "History distillation failed. Review raw episodes.".to_string()
                });

            let ep_section = format!(
                "## Ⅰ. State of the Union (Distilled History)\n{}\n\n",
                distilled
            );

            if count_tokens(&packet) + count_tokens(&ep_section) < budget {
                packet.push_str(&ep_section);
            }
        }

        // 2. High-Signal Facts (Upgrade 3)
        let facts = self
            .storage
            .query_agent_facts(agent, 10, task_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !facts.is_empty() {
            let mut fact_section = "## Ⅱ. Active Fact Cards\n".to_string();
            for fact in facts {
                fact_section.push_str(&format!(
                    "- [{}] (Conf: {:.2}): {}\n",
                    fact.domain, fact.confidence, fact.content
                ));
            }

            if count_tokens(&packet) + count_tokens(&fact_section) < budget {
                packet.push_str(&fact_section);
            }
        }

        // 3. Hierarchy Walk (Layers)
        let mut layers = Vec::new();
        let mut temp_path = current_path.clone();
        for _ in 0..5 {
            let agents_dir = temp_path.join("agents");
            if agents_dir.is_dir() {
                layers.push(temp_path.clone());
            }
            if let Some(parent) = temp_path.parent() {
                temp_path = parent.to_path_buf();
                if current_path == Path::new("/") {
                    break;
                }
            } else {
                break;
            }
        }

        if !layers.is_empty() {
            let home_dir = std::env::var("HOME").unwrap_or_default();
            packet.push_str("## Ⅲ. Workspace Hierarchy\n");
            for layer in layers.iter().rev() {
                let level = self.hierarchy.resolve_level(layer);
                let display_path = layer.to_string_lossy();
                let sanitized = if !home_dir.is_empty() {
                    display_path.replacen(&home_dir, "~", 1)
                } else {
                    display_path.into_owned()
                };
                let layer_info = format!("### Level: {:?}\nPath: {}\n\n", level, sanitized);
                if count_tokens(&packet) + count_tokens(&layer_info) < budget {
                    packet.push_str(&layer_info);
                    source_files.push(layer.to_string_lossy().to_string());
                } else {
                    break;
                }
            }
        }

        // 4. Ghost API Summaries (Upgrade 1)
        let mut api_section = "## Ⅳ. Crate API Maps (Ghost Summaries)\n".to_string();
        api_section.push_str("The following public items are available in your current workspace members. Use these to find symbols without reading files.\n");

        // We'll summarize the current crate and core
        let crate_list = vec![
            current_path.to_string_lossy().to_string(),
            current_path
                .join("crates/koad-core")
                .to_string_lossy()
                .to_string(),
        ];

        for c_path in crate_list {
            if let Ok(summary) = self.codegraph.get_crate_summary(&c_path) {
                if !summary.is_empty() {
                    let c_name = Path::new(&c_path)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    let block = format!("\n### Crate: {}\n{}\n", c_name, summary);
                    if count_tokens(&packet) + count_tokens(&api_section) + count_tokens(&block)
                        < budget
                    {
                        api_section.push_str(&block);
                    }
                }
            }
        }

        if api_section.len() > 150 {
            // Only add if we actually found something
            packet.push_str(&api_section);
        }

        // 5. Global Pulses
        if let Some(pulse_store) = &self.pulse_store {
            let agent_role = "global"; // In future, derive from agent identity
            if let Ok(pulses) = pulse_store.get_active_pulses(agent_role).await {
                if !pulses.is_empty() {
                    let mut pulse_section = "\n## Ⅴ. Global Pulses\n".to_string();
                    for p in &pulses {
                        pulse_section
                            .push_str(&format!("- [{}] {}: {}\n", p.role, p.author, p.message));
                    }
                    if count_tokens(&packet) + count_tokens(&pulse_section) < budget {
                        packet.push_str(&pulse_section);
                    }
                }
            }
        }

        let tokens = count_tokens(&packet);
        Ok(Response::new(HydrationResponse {
            markdown_packet: packet,
            estimated_tokens: tokens as u32,
            source_files,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::mock::{MockPulseStore, MockStorage};
    use koad_proto::cass::v1::Pulse;

    #[tokio::test]
    async fn test_hydration_bundles_sections() -> anyhow::Result<()> {
        let storage = Arc::new(MockStorage::new());
        let config = koad_core::config::KoadConfig::load()?;
        let hierarchy = Arc::new(HierarchyManager::new(config));
        let codegraph = Arc::new(CodeGraph::new_with_memory()?);
        let intelligence = Arc::new(InferenceRouter::new_default()?);

        let service = CassHydrationService::new(storage, hierarchy, codegraph, intelligence);

        let request = Request::new(HydrationRequest {
            agent_name: "test-agent".to_string(),
            project_root: "/tmp".to_string(),
            level: 0,
            token_budget: 10000,
            task_id: "".to_string(),
        });

        let response = service.hydrate(request).await?;
        let packet = response.into_inner().markdown_packet;

        assert!(packet.contains("# Temporal Context Hydration"));
        Ok(())
    }

    #[tokio::test]
    async fn test_hydration_includes_pulse_section() -> anyhow::Result<()> {
        let storage = Arc::new(MockStorage::new());
        let config = koad_core::config::KoadConfig::load()?;
        let hierarchy = Arc::new(HierarchyManager::new(config));
        let codegraph = Arc::new(CodeGraph::new_with_memory()?);
        let intelligence = Arc::new(InferenceRouter::new_default()?);

        let pulse_store = Arc::new(MockPulseStore::new());
        pulse_store
            .seed(Pulse {
                id: "test-pulse-1".to_string(),
                author: "test-agent".to_string(),
                role: "global".to_string(),
                message: "Test pulse active".to_string(),
                ttl_seconds: 3600,
                created_at: None,
            })
            .await;

        let service = CassHydrationService::new(storage, hierarchy, codegraph, intelligence)
            .with_pulse_store(pulse_store);

        let request = Request::new(HydrationRequest {
            agent_name: "test-agent".to_string(),
            project_root: "/tmp".to_string(),
            level: 0,
            token_budget: 10000,
            task_id: "".to_string(),
        });

        let response = service.hydrate(request).await?;
        let packet = response.into_inner().markdown_packet;

        assert!(
            packet.contains("Global Pulses"),
            "TCH packet missing Global Pulses section"
        );
        assert!(
            packet.contains("Test pulse active"),
            "TCH packet missing pulse message"
        );
        Ok(())
    }
}
