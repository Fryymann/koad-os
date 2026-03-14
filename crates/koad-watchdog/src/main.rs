use koad_core::config::KoadConfig;
use koad_core::logging::init_logging;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::Empty;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = KoadConfig::load()?;
    let _guard = init_logging("koad-watchdog", Some(config.home.clone()));

    info!("KoadOS Autonomic Watchdog started. (Config-Driven)");

    let mut failures = 0;
    let max_failures = config.watchdog.max_failures;
    let check_interval = Duration::from_secs(config.watchdog.check_interval_secs);

    loop {
        let mut health_ok = true;

        // 1. Check Spine gRPC
        if let Err(e) = check_spine(&config).await {
            warn!("Spine health check failed: {}", e);
            health_ok = false;
        }

        if health_ok {
            if failures > 0 {
                info!("System health restored. Resetting failure counter.");
            }
            failures = 0;
        } else {
            failures += 1;
            warn!(
                "System health check failed ({}/{}).",
                failures, max_failures
            );

            if failures >= max_failures {
                error!("CRITICAL: System unresponsive. Initiating autonomic reboot...");
                reboot_spine(&config).await;
                failures = 0;
                sleep(Duration::from_secs(10)).await;
            }
        }

        sleep(check_interval).await;
    }
}

async fn check_spine(config: &KoadConfig) -> anyhow::Result<()> {
    let mut client = SpineServiceClient::connect(config.network.spine_grpc_addr.clone()).await?;
    let mut request = tonic::Request::new(Empty {});
    // Use system key for health check
    request
        .metadata_mut()
        .insert("x-system-key", "citadel-core".parse().unwrap());
    let _ = client.heartbeat(request).await?;
    Ok(())
}

async fn reboot_spine(config: &KoadConfig) {
    info!("Watchdog: Attempting graceful drain before reboot...");
    if let Ok(mut client) =
        SpineServiceClient::connect(config.network.spine_grpc_addr.clone()).await
    {
        let mut request = tonic::Request::new(Empty {});
        request
            .metadata_mut()
            .insert("x-system-key", "citadel-core".parse().unwrap());
        let _ = client.drain_all(request).await;
    }

    info!("Watchdog: Terminating Spine process...");
    let _ = Command::new("pkill").arg("koad-spine").status();

    // Wait for graceful exit
    sleep(Duration::from_secs(5)).await;

    // Fallback to SIGKILL
    let _ = Command::new("pkill").arg("-9").arg("koad-spine").status();

    let bin_path = config.home.join("bin/koad-spine");
    let log_path = config.home.join("logs/watchdog_recovery.log");

    info!("Watchdog: Respawning Spine...");
    if let Ok(file) = std::fs::File::create(&log_path) {
        let _ = Command::new("nohup")
            .arg(&bin_path)
            .env("KOAD_HOME", &config.home)
            .stdout(std::process::Stdio::from(file.try_clone().unwrap()))
            .stderr(std::process::Stdio::from(file))
            .spawn();
        info!("Watchdog: Spine respawned. Recovery log at {:?}", log_path);
    } else {
        let _ = Command::new("nohup")
            .arg(&bin_path)
            .env("KOAD_HOME", &config.home)
            .spawn();
        warn!("Watchdog: Spine respawned (log file creation failed).");
    }
}
