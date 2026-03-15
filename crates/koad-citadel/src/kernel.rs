//! Citadel Kernel
//!
//! The Kernel is the central orchestration engine of the Citadel.

use crate::auth::interceptor::build_citadel_interceptor;
use crate::auth::hierarchy::HierarchyManager;
use crate::services::bay::PersonalBayService;
use crate::services::sector::SectorService;
use crate::services::session::CitadelSessionService;
use crate::services::signal::SignalService;
use crate::signal_corps::quota::QuotaValidator;
use koad_core::storage::StorageBridge;
use crate::signal_corps::streams::SignalCorps;
use crate::state::bay_store::BayStore;
use crate::state::storage_bridge::CitadelStorageBridge;
use crate::workspace::manager::WorkspaceManager;

use koad_core::config::KoadConfig;
use koad_core::utils::redis::RedisClient;
use koad_proto::citadel::v5::citadel_session_server::CitadelSessionServer;
use koad_proto::citadel::v5::personal_bay_server::PersonalBayServer;
use koad_proto::citadel::v5::sector_server::SectorServer;
use koad_proto::citadel::v5::signal_server::SignalServer;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tonic::transport::Server;
use tracing::{error, info};

/// The central nervous system of the Citadel.
pub struct Kernel {
    shutdown_tx: watch::Sender<bool>,
}

impl Kernel {
    pub async fn shutdown(self) {
        info!("Kernel: Initiating graceful shutdown...");
        let _ = self.shutdown_tx.send(true);
        tokio::time::sleep(Duration::from_millis(500)).await;
        info!("Kernel: Shutdown complete.");
    }
}

/// Builder for the [`Kernel`].
#[derive(Default)]
pub struct KernelBuilder {
    home_dir: Option<PathBuf>,
    tcp_addr: Option<String>,
    uds_path: Option<PathBuf>,
    admin_uds_path: Option<PathBuf>,
    config: Option<KoadConfig>,
}

impl KernelBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_home(mut self, path: PathBuf) -> Self {
        self.home_dir = Some(path);
        self
    }

    pub fn with_tcp(mut self, addr: &str) -> Self {
        self.tcp_addr = Some(addr.to_string());
        self
    }

    pub fn with_uds(mut self, path: PathBuf) -> Self {
        self.uds_path = Some(path);
        self
    }

    pub fn with_admin_uds(mut self, path: PathBuf) -> Self {
        self.admin_uds_path = Some(path);
        self
    }

    pub fn with_config(mut self, config: KoadConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub async fn start(self) -> anyhow::Result<Kernel> {
        let home_dir = self.home_dir.ok_or_else(|| anyhow::anyhow!("Home directory not specified"))?;
        let config = self.config.ok_or_else(|| anyhow::anyhow!("Config not specified"))?;
        let tcp_addr = self.tcp_addr.ok_or_else(|| anyhow::anyhow!("TCP address not specified"))?;

        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        info!("Kernel: Initializing at {}...", home_dir.display());

        let redis = Arc::new(RedisClient::new(&home_dir.to_string_lossy(), true).await?);
        let db_path = home_dir.join("citadel.db");
        let storage = Arc::new(CitadelStorageBridge::new(
            redis.clone(),
            &db_path.to_string_lossy(),
            config.storage.drain_interval_secs,
        )?);

        storage.enable_keyspace_notifications().await?;
        storage.hydrate_all().await?;

        let hierarchy = Arc::new(HierarchyManager::new(config.clone()));
        let bays_path = home_dir.join("bays");
        let bay_store = Arc::new(BayStore::new(bays_path));
        let identities_dir = home_dir.join("config/identities");
        bay_store.auto_provision_all(&identities_dir).await?;

        let signal_corps = Arc::new(SignalCorps::new(redis.clone(), "koad:stream:", 1000));
        let quota = Arc::new(QuotaValidator::new(redis.clone(), 60, 60));
        let workspace_mgr = Arc::new(WorkspaceManager::new(home_dir.join("workspaces"), home_dir.clone()));

        let session_svc_impl = CitadelSessionService::new(
            storage.clone(), 
            bay_store.clone(), 
            hierarchy.clone(),
            config.sessions.lease_duration_secs
        );
        let bay_svc_impl = PersonalBayService::new(bay_store.clone(), workspace_mgr.clone());
        let sector_svc_impl = SectorService::new(redis.clone());
        let signal_svc_impl = SignalService::new(signal_corps.clone(), quota.clone());

        let drain_storage = storage.clone();
        let mut rx_drain = shutdown_rx.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = drain_storage.start_drain_loop() => {},
                _ = rx_drain.changed() => { info!("Kernel: Drain loop stopping."); }
            }
        });

        let reaper_session = Arc::new(session_svc_impl.clone());
        let reaper_clone = reaper_session.clone();
        let dark_timeout = config.sessions.dark_timeout_secs;
        let purge_timeout = config.sessions.purge_timeout_secs;
        let reaper_interval = Duration::from_secs(config.sessions.reaper_interval_secs);
        let mut rx_reaper = shutdown_rx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(reaper_interval) => { reaper_clone.reap(dark_timeout, purge_timeout).await; }
                    _ = rx_reaper.changed() => { info!("Kernel: Reaper stopping."); break; }
                }
            }
        });

        let tcp_addr_parsed: std::net::SocketAddr = tcp_addr.parse()?;
        let auth_interceptor = build_citadel_interceptor(session_svc_impl.sessions_handle());

        let router = Server::builder()
            .add_service(CitadelSessionServer::with_interceptor(session_svc_impl.clone(), auth_interceptor.clone()))
            .add_service(PersonalBayServer::with_interceptor(bay_svc_impl.clone(), auth_interceptor.clone()))
            .add_service(SectorServer::with_interceptor(sector_svc_impl.clone(), auth_interceptor.clone()))
            .add_service(SignalServer::with_interceptor(signal_svc_impl.clone(), auth_interceptor));

        let mut rx_tcp = shutdown_rx.clone();
        tokio::spawn(async move {
            info!("Kernel: TCP listener active at {}", tcp_addr_parsed);
            if let Err(e) = router.serve_with_shutdown(tcp_addr_parsed, async move {
                let _ = rx_tcp.changed().await;
            }).await {
                error!("Kernel: TCP listener error: {}", e);
            }
        });

        Ok(Kernel { shutdown_tx })
    }
}
