pub mod discovery;
pub mod engine;
pub mod rpc;

use crate::engine::kernel::KernelBuilder;
use koad_core::config::KoadConfig;
use koad_core::constants::DEFAULT_SPINE_PID;
use koad_core::logging::init_logging;
use koad_core::utils::pid::PidGuard;

use tokio::signal;
use tracing::info;

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
            &config.network.spine_grpc_addr.replace("http://", ""),
            config.home.join(&config.network.spine_socket),
        )
        .start()
        .await?;

    info!("KoadOS Spine: Engine Room energized and stable.");

    // Shutdown Signal Handler
    let shutdown_signal = async {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).expect("Failed to register SIGTERM handler");
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt()).expect("Failed to register SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => { info!("Koad-Spine: SIGTERM received."); },
            _ = sigint.recv() => { info!("Koad-Spine: SIGINT (Ctrl+C) received."); },
        }
        info!("Koad-Spine: Commencing graceful teardown...");
    };

    shutdown_signal.await;

    info!("Koad-Spine: Server stopping. Cleaning up...");
    kernel.shutdown().await;

    Ok(())
}
