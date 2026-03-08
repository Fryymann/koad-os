use crate::engine::Engine;
use crate::rpc::KoadSpine;
use koad_proto::spine::v1::spine_service_server::SpineServiceServer;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;
use std::process::{Child, Command};

use tokio::sync::watch;

/// The central nervous system of KoadOS.
pub struct Kernel {
    pub engine: Arc<Engine>,
    shutdown_tx: watch::Sender<bool>,
    asm_process: Option<Child>,
}

impl Kernel {
    pub async fn shutdown(mut self) {
        println!("Kernel: Initiating graceful shutdown signal...");
        let _ = self.shutdown_tx.send(true);

        if let Some(mut child) = self.asm_process.take() {
            println!("Kernel: Stopping koad-asm daemon...");
            let _ = child.kill();
        }

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

        // 2. Launch Standalone ASM Daemon
        let mut asm_process = None;
        let asm_bin = home_dir.join("bin/koad-asm");
        println!("Kernel: Checking for ASM daemon at {}...", asm_bin.display());
        if asm_bin.exists() {
            let abs_asm_bin = asm_bin.canonicalize().unwrap_or(asm_bin);
            let stderr_log = home_dir.join("logs/asm_spawn_error.log");
            let stderr_file = std::fs::File::create(&stderr_log).ok();

            println!("Kernel: Spawning ASM daemon from {}...", abs_asm_bin.display());
            let mut cmd = Command::new(&abs_asm_bin);
            cmd.env("KOAD_HOME", &home_dir);

            if let Some(file) = stderr_file {
                cmd.stderr(file);
            }

            match cmd.spawn() {
                Ok(child) => {
                    println!("Kernel: ASM daemon spawned successfully (PID: {}).", child.id());
                    asm_process = Some(child);
                },
                Err(e) => eprintln!("Kernel Error: Failed to spawn koad-asm ({}): {}", abs_asm_bin.display(), e),
            }
        }
 else {
            eprintln!("Kernel Warning: koad-asm binary not found at {}. ASM features will be limited.", asm_bin.display());
        }

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
                _ = rx_asm.changed() => { println!("Kernel: ASM watcher stopping."); }
            }
        });

        let diagnostics = engine.diagnostics.clone();
        let mut rx_diag = shutdown_rx.clone();
        tokio::spawn(async move {
            println!("Kernel: Autonomic Watchdog ACTIVE.");
            loop {
                let last_hb = diagnostics
                    .last_heartbeat
                    .load(std::sync::atomic::Ordering::SeqCst);
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
                .register_in_inventory("0.0.0.0", koad_core::constants::DEFAULT_SPINE_GRPC_PORT)
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
            asm_process,
        })
    }
}

impl Default for KernelBuilder {
    fn default() -> Self {
        Self::new()
    }
}
