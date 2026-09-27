//! Citadel Kernel Entry Point

use anyhow::{Context, Result};
use koad_citadel::kernel::wait_for_shutdown;
use koad_citadel::KernelBuilder;
use koad_core::config::KoadConfig;

use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Citadel: Igniting Kernel...");

    let config = KoadConfig::load().context("Failed to load Citadel config")?;

    let kernel = KernelBuilder::new()
        .with_home(config.home.clone())
        .with_tcp(&format!("127.0.0.1:{}", config.network.citadel_grpc_port))
        .with_admin_uds(config.get_admin_socket())
        .with_config(config.clone())
        .start()
        .await?;

    let reason = wait_for_shutdown(kernel.shutdown_requested()).await?;
    info!(
        ?reason,
        "Citadel: Shutdown requested, draining and stopping..."
    );
    kernel.shutdown().await;

    Ok(())
}
