use fred::prelude::*;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

pub struct RedisClient {
    pub client: RedisClientInner,
    pub subscriber: RedisClientInner,
    pub _process: Option<Child>,
}

pub type RedisClientInner = fred::clients::RedisClient;

impl RedisClient {
    pub async fn new(koad_home: &str) -> anyhow::Result<Self> {
        let home_path = PathBuf::from(koad_home);

        let socket_path = if let Ok(env_socket) = std::env::var("REDIS_SOCKET") {
            PathBuf::from(env_socket)
        } else {
            home_path.join("koad.sock")
        };

        let pid_path = home_path.join("redis.pid");
        let log_path = home_path.join("redis.log");
        let data_dir = home_path.join("data/redis");

        let mut process = None;

        // 1. Start Redis Process ONLY if socket doesn't exist (Self-managed)
        if !socket_path.exists() {
            println!(
                "Starting Koad-managed Redis server at {}...",
                socket_path.display()
            );
            std::fs::create_dir_all(&data_dir)?;
            process = Some(
                Command::new("redis-server")
                    .arg("--port")
                    .arg("0")
                    .arg("--unixsocket")
                    .arg(&socket_path)
                    .arg("--pidfile")
                    .arg(&pid_path)
                    .arg("--logfile")
                    .arg(&log_path)
                    .arg("--dir")
                    .arg(&data_dir)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()?,
            );

            // Give it a moment to create the socket
            sleep(Duration::from_millis(500)).await;
        } else {
            println!(
                "Connecting to existing Redis socket at {}...",
                socket_path.display()
            );
        }

        // 2. Connect via UDS
        let config = RedisConfig {
            server: ServerConfig::Unix { path: socket_path },
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
            _process: process,
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
