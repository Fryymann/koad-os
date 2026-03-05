pub mod discovery;
pub mod engine;
pub mod rpc;

use crate::engine::kernel::KernelBuilder;
use koad_core::config::KoadConfig;
use koad_core::constants::DEFAULT_SPINE_PID;
use koad_core::logging::init_logging;
use koad_core::utils::pid::PidGuard;

use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = KoadConfig::load()?;
    let koad_home = config.home.to_string_lossy().to_string();
    std::env::set_var("KOAD_HOME", &koad_home);

    // Acquire PID lock immediately
    let pid_path = config.home.join(DEFAULT_SPINE_PID);
    let _pid_guard = PidGuard::new(pid_path)?;

    // Initialize Structured Logging
    let _guard = init_logging("kspine", Some(config.home.clone()));

    info!("KoadOS Spine starting up...");

    // Initialize and Start the Kernel using the Builder pattern
    let kernel = KernelBuilder::new()
        .with_home(config.home.clone())
        .with_grpc(
            &config.spine_grpc_addr.replace("http://", ""),
            config.spine_socket.clone(),
        )
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

    info!("Koad-Spine: Server stopping. Cleaning up...");
    kernel.shutdown().await;

    Ok(())
}
