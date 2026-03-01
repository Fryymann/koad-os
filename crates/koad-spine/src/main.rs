pub mod engine;
pub mod discovery;
pub mod rpc;
pub mod web;
pub mod deck;

use crate::engine::Engine;
use crate::discovery::SkillRegistry;
use crate::rpc::KoadSpine;
use crate::web::WebGateway;
use koad_proto::spine::v1::spine_service_server::SpineServiceServer;
use tonic::transport::Server;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Koad-Spine: Initializing Engine Room...");

    let home_dir = std::env::var("KOAD_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".koad-os"));

    // 1. Initialize State Engine (Redis + SQLite)
    let db_path = home_dir.join("koad.db");
    let engine = Arc::new(Engine::new(&home_dir.to_string_lossy(), &db_path.to_string_lossy()).await?);
    
    // 2. Discover Skills & Agents
    let mut registry = SkillRegistry::new();
    let _ = registry.scan_directory(&home_dir.join("skills").to_string_lossy());
    let _ = registry.scan_directory(&home_dir.join("doodskills").to_string_lossy());

    // 3. Start Core Engine Services (Draining, Heartbeat, Commands)
    let storage_drain = engine.storage.clone();
    tokio::spawn(async move { storage_drain.start_drain_loop().await; });

    let diagnostics = engine.diagnostics.clone();
    tokio::spawn(async move { diagnostics.start_health_monitor().await; });

    let command_processor = crate::engine::commands::CommandProcessor::new(engine.clone());
    tokio::spawn(async move { command_processor.start().await; });

    // 4. Start Web Gateway (Dashboard & WebDeck)
    let web_gateway = WebGateway::new(engine.clone());
    tokio::spawn(async move {
        let _ = web_gateway.start().await;
    });

    // 5. Start gRPC Bridge (Dual Binding: UDS + TCP 50051)
    let tcp_addr: std::net::SocketAddr = "0.0.0.0:50051".parse()?;
    let uds_path = home_dir.join("kspine.sock");
    
    if std::fs::metadata(&uds_path).is_ok() {
        std::fs::remove_file(&uds_path)?;
    }

    let spine_service = KoadSpine::new(engine.clone());
    let spine_service_arc = Arc::new(spine_service);
    
    // Populate Service Inventory
    spine_service_arc.register_in_inventory("0.0.0.0", 50051).await?;

    // Start TCP gRPC Server
    let tcp_spine = spine_service_arc.clone();
    tokio::spawn(async move {
        println!("Koad-Spine: Launching Bridge gRPC (TCP) on {}...", tcp_addr);
        if let Err(e) = Server::builder()
            .add_service(SpineServiceServer::from_arc(tcp_spine))
            .serve(tcp_addr)
            .await {
                eprintln!("gRPC TCP Error: {}", e);
            }
    });

    // Start UDS gRPC Server (Local WSL)
    let uds_spine = spine_service_arc.clone();
    let uds_stream = tokio::net::UnixListener::bind(&uds_path)?;
    let uds_stream = tokio_stream::wrappers::UnixListenerStream::new(uds_stream);
    
    tokio::spawn(async move {
        println!("Koad-Spine: Launching Bridge gRPC (UDS) on {}...", uds_path.display());
        if let Err(e) = Server::builder()
            .add_service(SpineServiceServer::from_arc(uds_spine))
            .serve_with_incoming(uds_stream)
            .await {
                eprintln!("gRPC UDS Error: {}", e);
            }
    });

    println!("{}", engine.diagnostics.get_morning_report());
    println!("Koad-Spine: Engine Room stable.");

    // Shutdown Signal Handler
    let shutdown_signal = async {
        let _ = signal::ctrl_c().await;
        println!("\nKoad-Spine: Shutdown signal received. Commencing graceful teardown...");
    };

    tokio::select! {
        _ = shutdown_signal => {},
        _ = tokio::time::sleep(std::time::Duration::from_secs(3600)) => {}, // 1 hour test limit
    }

    println!("Koad-Spine: Server stopped. Cleaning up child processes...");
    drop(engine);
    
    Ok(())
}
