//! Citadel Kernel Entry Point

use anyhow::{Context, Result};
use koad_citadel::services::bay::PersonalBayService;
use koad_citadel::services::sector::SectorService;
use koad_citadel::services::session::CitadelSessionService;
use koad_citadel::services::signal::SignalService;
use koad_citadel::signal_corps::quota::QuotaValidator;
use koad_citadel::state::bay_store::BayStore;
use koad_citadel::state::storage_bridge::CitadelStorageBridge;
use koad_citadel::workspace::manager::WorkspaceManager;
use koad_core::config::KoadConfig;
use koad_core::hierarchy::HierarchyManager;
use koad_core::signal::SignalCorps;
use koad_core::utils::redis::RedisClient;
use koad_sandbox::Sandbox;

use koad_proto::citadel::v5::citadel_session_server::CitadelSessionServer;
use koad_proto::citadel::v5::personal_bay_server::PersonalBayServer;
use koad_proto::citadel::v5::sector_server::SectorServer;
use koad_proto::citadel::v5::signal_server::SignalServer;

use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Citadel: Igniting Kernel...");

    let config = KoadConfig::load().context("Failed to load Citadel config")?;
    let redis = Arc::new(RedisClient::new(&config.home.to_string_lossy(), true).await?);
    let storage = Arc::new(CitadelStorageBridge::new(
        redis.clone(),
        &config.home.join("koad.db").to_string_lossy(),
        30,
    )?);
    let hierarchy = Arc::new(HierarchyManager::new(config.clone()));
    let signal_corps = Arc::new(SignalCorps::new(redis.clone(), "koad:stream:", 1000));
    let bay_store = Arc::new(BayStore::new(config.home.join("bays")));
    let workspace_manager = Arc::new(WorkspaceManager::new(
        config.home.join("workspaces"),
        config.home.clone(),
    ));
    let quota = Arc::new(QuotaValidator::new(redis.clone(), 100, 60));
    let sandbox = Arc::new(Sandbox::new(config.clone()));

    // Services
    let session_svc = CitadelSessionService::new(
        signal_corps.clone(),
        storage.clone(),
        bay_store.clone(),
        hierarchy.clone(),
        90,
    );
    let sector_svc = SectorService::new(redis.clone(), sandbox.clone());
    let signal_svc = SignalService::new(signal_corps.clone(), quota.clone());
    let bay_svc = PersonalBayService::new(bay_store.clone(), workspace_manager.clone());

    let addr = "127.0.0.1:50051".parse()?;
    info!("Citadel: gRPC server listening on {}", addr);

    Server::builder()
        .add_service(CitadelSessionServer::new(session_svc))
        .add_service(SectorServer::new(sector_svc))
        .add_service(SignalServer::new(signal_svc))
        .add_service(PersonalBayServer::new(bay_svc))
        .serve(addr)
        .await?;

    Ok(())
}
