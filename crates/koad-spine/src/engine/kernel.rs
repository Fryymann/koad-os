
use crate::engine::Engine;
use crate::rpc::KoadSpine;
use koad_proto::spine::v1::spine_service_server::SpineServiceServer;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

use tokio::sync::watch;

/// The central nervous system of KoadOS.
pub struct Kernel {
    pub engine: Arc<Engine>,
    shutdown_tx: watch::Sender<bool>,
}

impl Kernel {
    pub async fn shutdown(self) {
        println!("Kernel: Initiating graceful shutdown signal...");
        let _ = self.shutdown_tx.send(true);
        // Wait a moment for background tasks to notice
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        println!("Kernel: Engine Room shutdown complete.");
    }
}

/// A builder for the KoadOS Spine Kernel.
pub struct KernelBuilder {
    home_dir: Option<PathBuf>,
    tcp_addr: Option<String>,
    uds_path: Option<PathBuf>,
}

impl KernelBuilder {
    pub fn new() -> Self {
        Self {
            home_dir: None,
            tcp_addr: None,
            uds_path: None,
        }
    }

    /// Sets the KoadOS home directory.
    pub fn with_home(mut self, path: PathBuf) -> Self {
        self.home_dir = Some(path);
        self
    }

    /// Configures the gRPC bridge (TCP + UDS).
    pub fn with_grpc(mut self, tcp_addr: &str, uds_path: PathBuf) -> Self {
        self.tcp_addr = Some(tcp_addr.to_string());
        self.uds_path = Some(uds_path);
        self
    }

    /// Asynchronously starts all kernel systems.
    pub async fn start(self) -> anyhow::Result<Kernel> {
        let home_dir = self
            .home_dir
            .ok_or_else(|| anyhow::anyhow!("Home directory not specified"))?;
        let db_path = home_dir.join("koad.db");

        println!(
            "Kernel: Initializing Engine Room at {}...",
            home_dir.display()
        );

        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        // 1. Engine & State Initialization (Handles Skill Discovery internally now)
        let engine =
            Arc::new(Engine::new(&home_dir.to_string_lossy(), &db_path.to_string_lossy()).await?);

        // 3. Launch Core Background Loops
        let storage_drain = engine.storage.clone();
        let mut rx_drain = shutdown_rx.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = storage_drain.start_drain_loop() => {},
                _ = rx_drain.changed() => { println!("Kernel: Storage drain loop stopping."); }
            }
        });

        let asm = engine.asm.clone();
        let mut rx_asm = shutdown_rx.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = asm.start_session_monitor() => {},
                _ = rx_asm.changed() => { println!("Kernel: ASM monitor stopping."); }
            }
        });

        // 4. Session Reaper (Cleanup Dark Agents)
        let reaper_asm = engine.asm.clone();
        let mut rx_reaper = shutdown_rx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = reaper_asm.prune_sessions(30).await {
                            eprintln!("Kernel Error: Session reaper failed: {}", e);
                        }
                    },
                    _ = rx_reaper.changed() => {
                        println!("Kernel: Session reaper stopping.");
                        break;
                    }
                }
            }
        });

        let diagnostics = engine.diagnostics.clone();
        let mut rx_diag = shutdown_rx.clone();
        tokio::spawn(async move {
            println!("Kernel: Autonomic Watchdog ACTIVE.");
            loop {
                let last_hb = diagnostics.last_heartbeat.load(std::sync::atomic::Ordering::SeqCst);
                let now = chrono::Utc::now().timestamp();
                let gap = now - last_hb;

                if gap > 15 {
                    eprintln!("\x1b[31mCRITICAL: ShipDiagnostics loop stalled (Gap: {}s). Recovery required.\x1b[0m", gap);
                }

                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {},
                    _ = rx_diag.changed() => { break; }
                }
            }
            println!("Kernel: Watchdog stopping.");
        });

        let diag_inner = engine.diagnostics.clone();
        let mut rx_diag_task = shutdown_rx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = diag_inner.start_health_monitor() => {
                        eprintln!("\x1b[33mSHIP ALERT: Health monitor task exited unexpectedly. Restarting...\x1b[0m");
                    },
                    _ = rx_diag_task.changed() => {
                        println!("Kernel: Health monitor stopping.");
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        let directive_router = crate::engine::router::DirectiveRouter::new(engine.clone());
        let mut rx_router = shutdown_rx.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = directive_router.start() => {},
                _ = rx_router.changed() => { println!("Kernel: Directive router stopping."); }
            }
        });

        // 5. Start gRPC Bridges if configured
        if let (Some(tcp_addr_str), Some(uds_path)) = (self.tcp_addr, self.uds_path) {
            let tcp_addr: std::net::SocketAddr = tcp_addr_str.parse()?;

            // Cleanup UDS socket if it exists
            if std::fs::metadata(&uds_path).is_ok() {
                std::fs::remove_file(&uds_path)?;
            }

            let spine_service = KoadSpine::new(engine.clone());
            let spine_service_arc = Arc::new(spine_service);

            // Register gRPC service in Redis inventory
            spine_service_arc
                .register_in_inventory("0.0.0.0", 50051)
                .await?;

            // TCP Server
            let tcp_spine = spine_service_arc.clone();
            let mut rx_tcp = shutdown_rx.clone();
            tokio::spawn(async move {
                println!("Kernel: Launching Bridge gRPC (TCP) on {}...", tcp_addr);
                let _ = Server::builder()
                    .add_service(SpineServiceServer::from_arc(tcp_spine))
                    .serve_with_shutdown(tcp_addr, async move {
                        let _ = rx_tcp.changed().await;
                    })
                    .await;
            });

            // UDS Server
            let uds_spine = spine_service_arc.clone();
            let mut rx_uds = shutdown_rx.clone();
            let uds_listener = UnixListener::bind(&uds_path).expect("Failed to bind UDS");
            let uds_stream = UnixListenerStream::new(uds_listener);
            tokio::spawn(async move {
                println!(
                    "Kernel: Launching Bridge gRPC (UDS) on {}...",
                    uds_path.display()
                );
                let _ = Server::builder()
                    .add_service(SpineServiceServer::from_arc(uds_spine))
                    .serve_with_incoming_shutdown(uds_stream, async move {
                        let _ = rx_uds.changed().await;
                    })
                    .await;
            });
        }

        println!("Kernel: Engine Room stable.");
        println!("{}", engine.diagnostics.get_morning_report());

        Ok(Kernel {
            engine,
            shutdown_tx,
        })
    }
}

impl Default for KernelBuilder {
    fn default() -> Self {
        Self::new()
    }
}
