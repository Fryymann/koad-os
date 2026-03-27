//! CASS Binary Entry Point

use anyhow::Result;
use koad_bridge_notion::NotionClient;
use koad_cass::services::eow::EndOfWatchPipeline;
use koad_cass::services::hydration::CassHydrationService;
use koad_cass::services::memory::CassMemoryService;
use koad_cass::services::stream::CassStreamService;
use koad_cass::services::symbol::CassSymbolService;
use koad_cass::services::tool_registry::CassToolRegistryService;
use koad_cass::storage::CassStorage;
use koad_codegraph::CodeGraph;
use koad_core::config::KoadConfig;
use koad_core::hierarchy::HierarchyManager;
use koad_core::signal::SignalCorps;
use koad_core::utils::redis::RedisClient;
use koad_intelligence::router::InferenceRouter;
use koad_plugins::registry::PluginRegistry;

use koad_proto::cass::v1::hydration_service_server::HydrationServiceServer;
use koad_proto::cass::v1::memory_service_server::MemoryServiceServer;
use koad_proto::cass::v1::stream_service_server::StreamServiceServer;
use koad_proto::cass::v1::symbol_service_server::SymbolServiceServer;
use koad_proto::cass::v1::tool_registry_service_server::ToolRegistryServiceServer;

use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("CASS: Initializing Support Systems...");

    let config = KoadConfig::load()?;
    let redis = Arc::new(RedisClient::new(&config.home.to_string_lossy(), true).await?);
    let storage = Arc::new(CassStorage::new(
        &config.home.join("data/db/cass.db").to_string_lossy(),
    )?);
    let hierarchy = Arc::new(HierarchyManager::new(config.clone()));
    let signal_corps = Arc::new(SignalCorps::new(redis.clone(), "koad:stream:", 1000));
    let codegraph = Arc::new(CodeGraph::new(&config.home.join("data/db/codegraph.db"))?);
    let plugin_registry = PluginRegistry::new()?;

    let notion_key = std::env::var("KOADOS_PAT_NOTION_MAIN").unwrap_or_default();
    let notion_client = Arc::new(NotionClient::new(notion_key)?);
    let stream_db = config
        .integrations
        .notion
        .as_ref()
        .and_then(|n| n.index.get("stream"))
        .cloned()
        .unwrap_or_default();

    let intelligence = Arc::new(InferenceRouter::new_default()?);

    // Services
    let memory_svc = CassMemoryService::new(storage.clone(), intelligence.clone());
    let hydration_svc = CassHydrationService::new(
        storage.clone(),
        hierarchy.clone(),
        codegraph.clone(),
        intelligence.clone(),
    );
    let stream_svc = CassStreamService::new(notion_client.clone(), stream_db);
    let symbol_svc = CassSymbolService::new(codegraph.clone());
    let tool_svc = CassToolRegistryService::new(plugin_registry);

    // Pipelines
    let eow_pipeline = Arc::new(EndOfWatchPipeline::new(
        storage.clone(),
        signal_corps.clone(),
        intelligence.clone(),
    ));
    tokio::spawn(async move {
        eow_pipeline.start_listener().await;
    });

    let addr = "127.0.0.1:50052".parse()?;
    info!("CASS: gRPC server listening on {}", addr);

    Server::builder()
        .add_service(MemoryServiceServer::new(memory_svc))
        .add_service(HydrationServiceServer::new(hydration_svc))
        .add_service(StreamServiceServer::new(stream_svc))
        .add_service(SymbolServiceServer::new(symbol_svc))
        .add_service(ToolRegistryServiceServer::new(tool_svc))
        .serve(addr)
        .await?;

    Ok(())
}
