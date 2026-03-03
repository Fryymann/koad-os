pub mod engine;
pub mod discovery;
pub mod rpc;

use crate::engine::kernel::KernelBuilder;
use std::path::PathBuf;
use tokio::signal;
use koad_core::logging::init_logging;
use koad_core::config::KoadConfig;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = KoadConfig::load()?;

    // Initialize Structured Logging
    let _guard = init_logging("kspine", Some(config.home.clone()));
    
    info!("KoadOS Spine starting up...");

    // Initialize and Start the Kernel using the Builder pattern
    let kernel = KernelBuilder::new()
        .with_home(config.home.clone())
        .with_grpc(&config.spine_grpc_addr.replace("http://", ""), config.home.join("kspine.sock"))
        .start()
        .await?;

    info!("KoadOS Spine: Engine Room energized and stable.");

    // Shutdown Signal Handler
    let shutdown_signal = async {
        let _ = signal::ctrl_c().await;
        info!("Koad-Spine: Shutdown signal received. Commencing graceful teardown...");
    };

    tokio::select! {
        _ = shutdown_signal => {},
        _ = tokio::time::sleep(std::time::Duration::from_secs(3600)) => {
            error!("Koad-Spine: Safety limit reached (1 hour). Forced shutdown.");
        },
    }

    info!("Koad-Spine: Server stopped. Cleaning up child processes...");
    // The drop of kernel will trigger engine drop, which kills Redis.
    drop(kernel);
    
    Ok(())
}
