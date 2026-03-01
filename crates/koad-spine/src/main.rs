pub mod engine;
pub mod discovery;
pub mod rpc;

use crate::engine::kernel::KernelBuilder;
use std::path::PathBuf;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let home_dir = std::env::var("KOAD_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".koad-os"));

    // Initialize and Start the Kernel using the Builder pattern
    let kernel = KernelBuilder::new()
        .with_home(home_dir.clone())
        .with_grpc("0.0.0.0:50051", home_dir.join("kspine.sock"))
        .start()
        .await?;

    // Shutdown Signal Handler
    let shutdown_signal = async {
        let _ = signal::ctrl_c().await;
        println!("\nKoad-Spine: Shutdown signal received. Commencing graceful teardown...");
    };

    tokio::select! {
        _ = shutdown_signal => {},
        _ = tokio::time::sleep(std::time::Duration::from_secs(3600)) => {}, // 1 hour safety limit
    }

    println!("Koad-Spine: Server stopped. Cleaning up child processes...");
    // The drop of kernel will trigger engine drop, which kills Redis.
    drop(kernel);
    
    Ok(())
}
