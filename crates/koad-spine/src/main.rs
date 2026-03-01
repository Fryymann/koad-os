pub mod engine;
pub mod discovery;
pub mod rpc;

use crate::engine::Engine;
use crate::discovery::SkillRegistry;
use crate::rpc::KoadKernel;
use koad_proto::kernel::kernel_service_server::KernelServiceServer;
use tonic::transport::Server;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use std::sync::Arc;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Koad-Spine: Initializing Engine Room...");

    // 1. Initialize State Engine
    let config_path = "/home/ideans/.koad-os/config/redis.conf";
    let engine = Arc::new(Engine::new(config_path, "/home/ideans/.koad-os/koad.db").await?);
    
    // 2. Discover Skills
    let mut registry = SkillRegistry::new();
    registry.scan_directory("/home/ideans/.koad-os/skills")?;
    registry.scan_directory("/home/ideans/.koad-os/doodskills")?;

    // 3. Start background tasks
    let persistence = engine.persistence.clone();
    tokio::spawn(async move {
        persistence.start_drain_loop().await;
    });

    let diagnostics = engine.diagnostics.clone();
    tokio::spawn(async move {
        diagnostics.start_health_monitor().await;
    });

    println!("{}", engine.diagnostics.get_morning_report());

    // 4. Start gRPC Server (Bridge Interface)
    let uds_path = "/home/ideans/.koad-os/kspine.sock";
    if fs::metadata(uds_path).is_ok() {
        fs::remove_file(uds_path)?;
    }

    let uds = UnixListener::bind(uds_path)?;
    let uds_stream = UnixListenerStream::new(uds);

    let kernel = KoadKernel::new(engine);
    
    println!("Koad-Spine: Engine Room stable. Launching Bridge gRPC on {}...", uds_path);

    Server::builder()
        .add_service(KernelServiceServer::new(kernel))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}
