use fred::prelude::*;
use std::process::{Child, Command};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

pub struct RedisClient {
    pub client: RedisClientInner,
    pub subscriber: RedisClientInner,
    _process: Option<Child>,
}

pub type RedisClientInner = fred::clients::RedisClient;

impl RedisClient {
    pub async fn new(config_path: &str) -> anyhow::Result<Self> {
        // 1. Start Redis Process (Local-managed)
        println!("Starting Koad-managed Redis server...");
        let process = Command::new("redis-server")
            .arg(config_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        // Give it a moment to create the socket
        sleep(Duration::from_millis(500)).await;

        // 2. Connect via UDS
        let config = RedisConfig {
            server: ServerConfig::Unix {
                path: PathBuf::from("/home/ideans/.koad-os/koad.sock"),
            },
            ..Default::default()
        };

        // Primary client for commands
        let client = Builder::from_config(config.clone())
            .with_connection_config(|c| {
                c.connection_timeout = Duration::from_secs(5);
            })
            .build()?;

        // Subscriber client for PubSub
        let subscriber = Builder::from_config(config)
            .with_connection_config(|c| {
                c.connection_timeout = Duration::from_secs(5);
            })
            .build()?;

        client.init().await?;
        subscriber.init().await?;
        
        println!("Connected to Redis via UDS (Primary + Subscriber).");

        Ok(Self {
            client,
            subscriber,
            _process: Some(process),
        })
    }
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        if let Some(mut process) = self._process.take() {
            println!("Stopping Redis server...");
            let _ = process.kill();
        }
    }
}
