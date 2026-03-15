//! Symbol Service Implementation
//!
//! Handles RPC calls for querying the code knowledge graph.

use koad_codegraph::CodeGraph;
use koad_proto::cass::v1::symbol_service_server::SymbolService;
use koad_proto::cass::v1::{IndexRequest, Symbol as ProtoSymbol, SymbolQuery, SymbolResponse};
use koad_proto::citadel::v5::{StatusResponse, TraceContext, WorkspaceLevel};
use std::path::Path;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};

/// Service implementation for the `SymbolService` gRPC interface.
pub struct CassSymbolService {
    graph: Arc<CodeGraph>,
}

impl CassSymbolService {
    /// Creates a new `CassSymbolService`.
    pub fn new(graph: Arc<CodeGraph>) -> Self {
        Self { graph }
    }
}

#[tonic::async_trait]
impl SymbolService for CassSymbolService {
    /// Query the code graph for symbols.
    async fn query(
        &self,
        request: Request<SymbolQuery>,
    ) -> Result<Response<SymbolResponse>, Status> {
        let req = request.into_inner();
        let trace_id = req.context.as_ref().map(|c| c.trace_id.as_str()).unwrap_or("UNKNOWN");
        
        info!(trace_id = %trace_id, symbol = %req.name, "SymbolService: Querying graph");

        let symbols = self
            .graph
            .query_symbol(&req.name)
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_symbols = symbols
            .into_iter()
            .map(|s| ProtoSymbol {
                name: s.name,
                kind: s.kind,
                path: s.path,
                start_line: s.start_line as u32,
                end_line: s.end_line as u32,
            })
            .collect();

        let context = req.context.map(|c| TraceContext {
            trace_id: c.trace_id,
            origin: "CASS".to_string(),
            actor: "cass".to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
            level: WorkspaceLevel::LevelCitadel as i32,
        });

        Ok(Response::new(SymbolResponse {
            symbols: proto_symbols,
            context,
        }))
    }

    /// Trigger a project indexing.
    async fn index_project(
        &self,
        request: Request<IndexRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let trace_id = req.context.as_ref().map(|c| c.trace_id.as_str()).unwrap_or("UNKNOWN");
        let root = Path::new(&req.project_root);

        info!(trace_id = %trace_id, path = ?root, "CASS: Starting project re-index");

        let graph = self.graph.clone();
        let root_path = root.to_path_buf();
        let trace_id_clone = trace_id.to_string();

        // Run indexing in a blocking thread to avoid stalling the executor
        tokio::task::spawn_blocking(move || {
            if let Err(e) = graph.index_project(&root_path) {
                error!(trace_id = %trace_id_clone, "CASS: Project indexing failed: {}", e);
            } else {
                info!(trace_id = %trace_id_clone, "CASS: Project indexing complete.");
            }
        });

        let context = req.context.map(|c| TraceContext {
            trace_id: c.trace_id,
            origin: "CASS".to_string(),
            actor: "cass".to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
            level: WorkspaceLevel::LevelCitadel as i32,
        });

        Ok(Response::new(StatusResponse {
            success: true,
            message: "Indexing started".to_string(),
            context,
        }))
    }
}
