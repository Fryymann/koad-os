use std::sync::Arc;
use std::path::PathBuf;
use crate::engine::Engine;
use crate::discovery::SkillRegistry;
use crate::rpc::KoadSpine;
use koad_proto::spine::v1::spine_service_server::SpineServiceServer;
use tonic::transport::Server;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;

/// The central nervous system of KoadOS.
pub struct Kernel {
    pub engine: Arc<Engine>,
    pub skill_registry: SkillRegistry,
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
        let home_dir = self.home_dir.ok_or_else(|| anyhow::anyhow!("Home directory not specified"))?;
        let db_path = home_dir.join("koad.db");

        println!("Kernel: Initializing Engine Room at {}...", home_dir.display());

        // 1. Engine & State Initialization
        let engine = Arc::new(Engine::new(&home_dir.to_string_lossy(), &db_path.to_string_lossy()).await?);

        // 2. Skill Discovery
        let mut skill_registry = SkillRegistry::new();
        let _ = skill_registry.scan_directory(&home_dir.join("skills").to_string_lossy());
        let _ = skill_registry.scan_directory(&home_dir.join("doodskills").to_string_lossy());

        // 3. Launch Core Background Loops
        let storage_drain = engine.storage.clone();
        tokio::spawn(async move { storage_drain.start_drain_loop().await; });

        let diagnostics = engine.diagnostics.clone();
        tokio::spawn(async move { diagnostics.start_health_monitor().await; });

        let command_processor = crate::engine::commands::CommandProcessor::new(engine.clone());
        tokio::spawn(async move { command_processor.start().await; });

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
            spine_service_arc.register_in_inventory("0.0.0.0", 50051).await?;

            // TCP Server
            let tcp_spine = spine_service_arc.clone();
            tokio::spawn(async move {
                println!("Kernel: Launching Bridge gRPC (TCP) on {}...", tcp_addr);
                if let Err(e) = Server::builder()
                    .add_service(SpineServiceServer::from_arc(tcp_spine))
                    .serve(tcp_addr)
                    .await {
                        eprintln!("Kernel: gRPC TCP Server Error: {}", e);
                    }
            });

            // UDS Server
            let uds_spine = spine_service_arc.clone();
            let uds_listener = UnixListener::bind(&uds_path)?;
            let uds_stream = UnixListenerStream::new(uds_listener);
            tokio::spawn(async move {
                println!("Kernel: Launching Bridge gRPC (UDS) on {}...", uds_path.display());
                if let Err(e) = Server::builder()
                    .add_service(SpineServiceServer::from_arc(uds_spine))
                    .serve_with_incoming(uds_stream)
                    .await {
                        eprintln!("Kernel: gRPC UDS Server Error: {}", e);
                    }
            });
        }

        println!("Kernel: Engine Room stable.");
        println!("{}", engine.diagnostics.get_morning_report());

        Ok(Kernel {
            engine,
            skill_registry,
        })
    }
}

impl Default for KernelBuilder {
    fn default() -> Self {
        Self::new()
    }
}
