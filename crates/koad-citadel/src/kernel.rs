//! Citadel Kernel
//!
//! The Kernel is the central orchestration engine of the Citadel. It handles
//! the initialization of all sub-services (Session, Bay, Sector, Signal)
//! and manages the lifecycle of both TCP and Unix Domain Socket (UDS) listeners.

use crate::auth::interceptor::trace_context_interceptor;
use crate::services::bay::PersonalBayService;
use crate::services::sector::SectorService;
use crate::services::session::CitadelSessionService;
use crate::services::signal::SignalService;
use crate::signal_corps::quota::QuotaValidator;
use crate::signal_corps::streams::SignalCorps;
use crate::state::bay_store::BayStore;
use crate::state::storage_bridge::CitadelStorageBridge;
use crate::workspace::manager::WorkspaceManager;

use koad_core::config::KoadConfig;
use koad_core::storage::StorageBridge;
use koad_core::utils::redis::RedisClient;
use koad_proto::citadel::v5::citadel_session_server::CitadelSessionServer;
use koad_proto::citadel::v5::personal_bay_server::PersonalBayServer;
use koad_proto::citadel::v5::sector_server::SectorServer;
use koad_proto::citadel::v5::signal_server::SignalServer;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::sync::watch;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;
use tracing::{error, info};

/// The central nervous system of the Citadel.
///
/// Manages background tasks and provides a graceful shutdown mechanism.
pub struct Kernel {
    shutdown_tx: watch::Sender<bool>,
}

impl Kernel {
    /// Initiates a graceful shutdown of all Citadel services and background tasks.
    pub async fn shutdown(self) {
        info!("Kernel: Initiating graceful shutdown...");
        let _ = self.shutdown_tx.send(true);
        // Allow brief window for connections to drain
        tokio::time::sleep(Duration::from_millis(500)).await;
        info!("Kernel: Shutdown complete.");
    }
}

/// Builder for the [`Kernel`].
///
/// Provides a fluent interface for configuring the Citadel's home directory,
/// network interfaces, and system configuration.
#[derive(Default)]
pub struct KernelBuilder {
    home_dir: Option<PathBuf>,
    tcp_addr: Option<String>,
    uds_path: Option<PathBuf>,
    admin_uds_path: Option<PathBuf>,
    config: Option<KoadConfig>,
}

impl KernelBuilder {
    /// Creates a new, empty [`KernelBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the home directory for the Citadel (where databases and bays are stored).
    pub fn with_home(mut self, path: PathBuf) -> Self {
        self.home_dir = Some(path);
        self
    }

    /// Sets the TCP address for the primary gRPC listener.
    pub fn with_tcp(mut self, addr: &str) -> Self {
        self.tcp_addr = Some(addr.to_string());
        self
    }

    /// Sets the path for the standard Unix Domain Socket (UDS) listener.
    pub fn with_uds(mut self, path: PathBuf) -> Self {
        self.uds_path = Some(path);
        self
    }

    /// Sets the path for the privileged Admin UDS listener.
    pub fn with_admin_uds(mut self, path: PathBuf) -> Self {
        self.admin_uds_path = Some(path);
        self
    }

    /// Provides the [`KoadConfig`] for the Citadel.
    pub fn with_config(mut self, config: KoadConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Starts the Citadel kernel and all background services.
    ///
    /// # Errors
    /// Returns an error if mandatory fields (home, config, tcp) are missing,
    /// or if sub-service initialization fails (e.g. Redis connection, SQLite setup).
    pub async fn start(self) -> anyhow::Result<Kernel> {
        let home_dir = self
            .home_dir
            .ok_or_else(|| anyhow::anyhow!("Home directory not specified"))?;
        let config = self
            .config
            .ok_or_else(|| anyhow::anyhow!("Config not specified"))?;
        let tcp_addr = self
            .tcp_addr
            .ok_or_else(|| anyhow::anyhow!("TCP address not specified"))?;

        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        info!("Kernel: Initializing at {}...", home_dir.display());

        // --- 1. Redis connection ---
        let redis = Arc::new(RedisClient::new(&home_dir.to_string_lossy(), true).await?);

        // --- 2. Storage Bridge (CQRS) ---
        let db_path = home_dir.join("citadel.db");
        let storage = Arc::new(CitadelStorageBridge::new(
            redis.clone(),
            &db_path.to_string_lossy(),
            config.storage.drain_interval_secs,
        )?);

        // Enable keyspace notifications
        storage.enable_keyspace_notifications().await?;

        // Hydrate from SQLite
        storage.hydrate_all().await?;

        // --- 3. Bay Store ---
        let bays_path = home_dir.join("bays");
        let bay_store = Arc::new(BayStore::new(bays_path));

        // Auto-provision bays for registered agents
        let identities_dir = home_dir.join("config/identities");
        bay_store.auto_provision_all(&identities_dir).await?;

        // --- 4. Signal Corps ---
        let signal_corps = Arc::new(SignalCorps::new(redis.clone(), "koad:stream:", 1000));
        let quota = Arc::new(QuotaValidator::new(
            redis.clone(),
            60, // max signals per window
            60, // window in seconds
        ));

        // --- 5. Workspace Manager ---
        let workspace_mgr = Arc::new(WorkspaceManager::new(
            home_dir.join("workspaces"),
            home_dir.clone(),
        ));

        // --- 6. Build gRPC service implementations ---
        let session_svc_impl = CitadelSessionService::new(
            storage.clone(),
            bay_store.clone(),
            config.sessions.lease_duration_secs,
        );
        let bay_svc_impl = PersonalBayService::new(bay_store.clone(), workspace_mgr.clone());
        let sector_svc_impl = SectorService::new(redis.clone());
        let signal_svc_impl = SignalService::new(signal_corps.clone(), quota.clone());

        // --- 7. Background tasks ---

        // Storage drain loop
        let drain_storage = storage.clone();
        let mut rx_drain = shutdown_rx.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = drain_storage.start_drain_loop() => {},
                _ = rx_drain.changed() => {
                    info!("Kernel: Drain loop stopping.");
                }
            }
        });

        // Session reaper
        let reaper_session = Arc::new(session_svc_impl.clone());
        let reaper_clone = reaper_session.clone();
        let dark_timeout = config.sessions.dark_timeout_secs;
        let purge_timeout = config.sessions.purge_timeout_secs;
        let reaper_interval = Duration::from_secs(config.sessions.reaper_interval_secs);
        let mut rx_reaper = shutdown_rx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(reaper_interval) => {
                        reaper_clone.reap(dark_timeout, purge_timeout).await;
                    }
                    _ = rx_reaper.changed() => {
                        info!("Kernel: Reaper stopping.");
                        break;
                    }
                }
            }
        });

        // --- 8. gRPC Router (TCP) ---
        let tcp_addr_parsed: std::net::SocketAddr = tcp_addr.parse()?;
        let router = Server::builder()
            .add_service(CitadelSessionServer::with_interceptor(
                session_svc_impl.clone(),
                trace_context_interceptor,
            ))
            .add_service(PersonalBayServer::new(bay_svc_impl.clone()))
            .add_service(SectorServer::new(sector_svc_impl.clone()))
            .add_service(SignalServer::new(signal_svc_impl.clone()));

        // --- 9. Start TCP listener ---
        let mut rx_tcp = shutdown_rx.clone();
        tokio::spawn(async move {
            info!("Kernel: TCP listener starting on {}", tcp_addr_parsed);
            if let Err(e) = router
                .serve_with_shutdown(tcp_addr_parsed, async {
                    let _ = rx_tcp.changed().await;
                })
                .await
            {
                error!("Kernel: TCP server error: {}", e);
            }
        });

        // --- 10. Start UDS listener (standard) ---
        if let Some(uds_path) = self.uds_path {
            let session_uds = session_svc_impl.clone();
            let bay_uds = bay_svc_impl.clone();
            let sector_uds = sector_svc_impl.clone();
            let signal_uds = signal_svc_impl.clone();
            let mut rx_uds = shutdown_rx.clone();

            tokio::spawn(async move {
                let _ = std::fs::remove_file(&uds_path);
                match UnixListener::bind(&uds_path) {
                    Ok(uds) => {
                        info!("Kernel: UDS listener at {:?}", uds_path);
                        let incoming = UnixListenerStream::new(uds);

                        let router = Server::builder()
                            .add_service(CitadelSessionServer::new(session_uds))
                            .add_service(PersonalBayServer::new(bay_uds))
                            .add_service(SectorServer::new(sector_uds))
                            .add_service(SignalServer::new(signal_uds));

                        if let Err(e) = router
                            .serve_with_incoming_shutdown(incoming, async {
                                let _ = rx_uds.changed().await;
                            })
                            .await
                        {
                            error!("Kernel: UDS server error: {}", e);
                        }
                    }
                    Err(e) => error!("Kernel: UDS bind failed: {}", e),
                }
            });
        }

        // --- 11. Start Admin UDS listener ---
        if let Some(admin_path) = self.admin_uds_path {
            let session_admin = session_svc_impl;
            let bay_admin = bay_svc_impl;
            let sector_admin = sector_svc_impl;
            let signal_admin = signal_svc_impl;
            let mut rx_admin = shutdown_rx.clone();

            tokio::spawn(async move {
                let _ = std::fs::remove_file(&admin_path);
                match UnixListener::bind(&admin_path) {
                    Ok(uds) => {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let perms = std::fs::Permissions::from_mode(0o600);
                            let _ = std::fs::set_permissions(&admin_path, perms);
                        }

                        info!("Kernel: Admin UDS at {:?} (privileged)", admin_path);
                        let incoming = UnixListenerStream::new(uds);

                        let router = Server::builder()
                            .add_service(CitadelSessionServer::new(session_admin))
                            .add_service(PersonalBayServer::new(bay_admin))
                            .add_service(SectorServer::new(sector_admin))
                            .add_service(SignalServer::new(signal_admin));

                        if let Err(e) = router
                            .serve_with_incoming_shutdown(incoming, async {
                                let _ = rx_admin.changed().await;
                            })
                            .await
                        {
                            error!("Kernel: Admin UDS error: {}", e);
                        }
                    }
                    Err(e) => error!("Kernel: Admin UDS bind failed: {}", e),
                }
            });
        }

        info!("Kernel: All systems online.");

        Ok(Kernel { shutdown_tx })
    }
}
