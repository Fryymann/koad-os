//! KoadOS Citadel Binary
//!
//! Entry point for the persistent Citadel gRPC service.

use anyhow::Result;
use koad_citadel::KernelBuilder;
use koad_core::config::KoadConfig;
use koad_core::logging::init_logging;
use koad_core::utils::pid::PidGuard;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let config = KoadConfig::load()?;
    let koad_home = config.home.to_string_lossy().to_string();
    std::env::set_var("KOAD_HOME", &koad_home);

    // Acquire PID lock to prevent multiple instances
    let pid_path = config.home.join("kcitadel.pid");
    let _pid_guard = PidGuard::new(pid_path)?;

    // Initialize structured logging
    let _guard = init_logging("koad-citadel", Some(config.home.clone()));

    info!("KoadOS Citadel starting up...");

    // Build and start the kernel
    let kernel = KernelBuilder::new()
        .with_home(config.home.clone())
        .with_tcp("127.0.0.1:50051")
        .with_uds(config.home.join("kcitadel.sock"))
        .with_admin_uds(std::path::PathBuf::from("/tmp/koad.admin.sock"))
        .with_config(config)
        .start()
        .await?;

    info!("KoadOS Citadel: Engine Room energized and stable.");

    // Shutdown signal handler for graceful teardown
    let shutdown_signal = async {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");
        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("Failed to register SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => { info!("Citadel: SIGTERM received."); },
            _ = sigint.recv() => { info!("Citadel: SIGINT (Ctrl+C) received."); },
        }
        info!("Citadel: Commencing graceful teardown...");
    };

    shutdown_signal.await;

    info!("Citadel: Server stopping. Cleaning up...");
    kernel.shutdown().await;

    Ok(())
}
