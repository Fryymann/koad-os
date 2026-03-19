//! CASS Tool Registry Service
//!
//! Exposes the `koad-plugins` WASM registry over gRPC, allowing agents to
//! dynamically register and invoke tools.

use koad_plugins::registry::PluginRegistry;
use koad_proto::cass::v1::tool_registry_service_server::ToolRegistryService;
use koad_proto::cass::v1::{
    DeregisterToolRequest, InvokeToolRequest, InvokeToolResponse, ListToolsRequest,
    ListToolsResponse, RegisterToolRequest,
};
use koad_proto::citadel::v5::StatusResponse;

use std::path::PathBuf;
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct CassToolRegistryService {
    registry: PluginRegistry,
}

impl CassToolRegistryService {
    pub fn new(registry: PluginRegistry) -> Self {
        Self { registry }
    }
}

#[tonic::async_trait]
impl ToolRegistryService for CassToolRegistryService {
    async fn register_tool(
        &self,
        request: Request<RegisterToolRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let name = req.name;
        let path = PathBuf::from(req.component_path);

        if !path.exists() {
            return Err(Status::not_found(format!(
                "WASM component not found at {:?}",
                path
            )));
        }

        info!(tool = %name, path = ?path, "ToolRegistry: Registering tool");
        self.registry.register(&name, path).await;

        Ok(Response::new(StatusResponse {
            success: true,
            message: format!("Tool '{}' registered successfully", name),
            context: req.context,
        }))
    }

    async fn deregister_tool(
        &self,
        request: Request<DeregisterToolRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let name = req.name;

        info!(tool = %name, "ToolRegistry: Deregistering tool");
        let found = self.registry.deregister(&name).await;

        if !found {
            return Err(Status::not_found(format!("Tool '{}' not found", name)));
        }

        Ok(Response::new(StatusResponse {
            success: true,
            message: format!("Tool '{}' deregistered successfully", name),
            context: req.context,
        }))
    }

    async fn invoke_tool(
        &self,
        request: Request<InvokeToolRequest>,
    ) -> Result<Response<InvokeToolResponse>, Status> {
        let req = request.into_inner();
        let name = req.name;
        let topic = req.topic;
        let payload = req.payload;

        info!(tool = %name, topic = %topic, "ToolRegistry: Invoking tool");

        match self.registry.invoke(&name, &topic, &payload).await {
            Ok(result) => Ok(Response::new(InvokeToolResponse {
                output: result.output,
                duration_ms: result.metrics.duration_ms,
                memory_bytes: result.metrics.memory_bytes,
                context: req.context,
            })),
            Err(e) => {
                error!(tool = %name, error = %e, "ToolRegistry: Invocation failed");
                Err(Status::internal(format!("Tool invocation failed: {}", e)))
            }
        }
    }

    async fn list_tools(
        &self,
        request: Request<ListToolsRequest>,
    ) -> Result<Response<ListToolsResponse>, Status> {
        let req = request.into_inner();
        let tool_names = self.registry.list().await;

        Ok(Response::new(ListToolsResponse {
            tool_names,
            context: req.context,
        }))
    }
}
