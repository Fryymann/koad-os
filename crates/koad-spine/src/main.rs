pub mod engine;
pub mod discovery;
pub mod rpc;

use crate::engine::kernel::KernelBuilder;
use std::path::PathBuf;
use tokio::signal;
use tracing::{info, error};
use koad_core::logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let home_dir = std::env::var("KOAD_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".koad-os"));

    // Initialize Structured Logging
    let _guard = init_logging("kspine", Some(home_dir.clone()));
    
    info!("KoadOS Spine starting up...");

    // Initialize and Start the Kernel using the Builder pattern
    let kernel = KernelBuilder::new()
        .with_home(home_dir.clone())
        .with_grpc("0.0.0.0:50051", home_dir.join("kspine.sock"))
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
