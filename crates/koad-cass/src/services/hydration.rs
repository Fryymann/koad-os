//! Hydration Service Implementation
//!
//! Performs the Temporal Context Hydration (TCH) walk to bundle
//! relevant facts and episodes for agent boot.

use koad_core::hierarchy::HierarchyManager;
use koad_core::utils::tokens::count_tokens;
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
}

impl CassHydrationService {
    /// Creates a new `CassHydrationService`.
    pub fn new(storage: Arc<dyn crate::storage::Storage>, hierarchy: Arc<HierarchyManager>) -> Self {
        Self { storage, hierarchy }
    }
}

#[tonic::async_trait]
impl HydrationService for CassHydrationService {
    /// Bundles context for an agent based on their workspace level and token budget.
    async fn hydrate(&self, request: Request<HydrationRequest>) -> Result<Response<HydrationResponse>, Status> {
        let req = request.into_inner();
        let mut current_path = PathBuf::from(&req.project_root);
        let budget = req.token_budget as usize;
        
        info!(agent = %req.agent_name, path = %req.project_root, budget = %budget, "Hydration requested");

        let mut packet = format!("# Temporal Context Hydration: {}

", req.agent_name);
        let mut source_files = Vec::new();

        let mut layers = Vec::new();
        for _ in 0..10 {
            let agents_dir = current_path.join(".agents");
            if agents_dir.is_dir() {
                layers.push(agents_dir);
            }
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_path_buf();
                if current_path == Path::new("/") { break; }
            } else { break; }
        }

        for layer in layers.iter().rev() {
            let level = self.hierarchy.resolve_level(layer);
            let layer_info = format!("## Level: {:?}
Context path: {}

", level, layer.display());
            if count_tokens(&packet) + count_tokens(&layer_info) < budget {
                packet.push_str(&layer_info);
                source_files.push(layer.to_string_lossy().to_string());
            } else { break; }
        }

        let facts = self.storage.query_facts("architecture", &[], 5).await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !facts.is_empty() {
            let header = "## Ⅰ. Global Architecture Facts
";
            if count_tokens(&packet) + count_tokens(header) < budget {
                packet.push_str(header);
                for fact in facts {
                    let fact_line = format!("- {}: {}
", fact.domain, fact.content);
                    if count_tokens(&packet) + count_tokens(&fact_line) < budget {
                        packet.push_str(&fact_line);
                    } else { break; }
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
