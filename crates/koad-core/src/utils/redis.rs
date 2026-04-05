use anyhow::Result;
use fred::clients::RedisPool;
use fred::prelude::*;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

pub struct RedisClient {
    pub pool: RedisPool,
    pub subscriber: fred::clients::RedisClient,
    pub process: Option<Child>,
}

impl RedisClient {
    /// Initialize a new Redis client.
    /// If `manage_process` is true, it will attempt to start a local Redis server if not already running.
    pub async fn new(koad_home: &str, manage_process: bool) -> Result<Self> {
        let home_path = PathBuf::from(koad_home);

        let socket_path = if let Ok(env_socket) = std::env::var("REDIS_SOCKET") {
            PathBuf::from(env_socket)
        } else {
            home_path.join(crate::constants::DEFAULT_REDIS_SOCK)
        };

        let pid_path = home_path.join("run/redis.pid");
        let log_path = home_path.join("logs/redis.log");
        let data_dir = home_path.join("data/redis");

        let mut process = None;

        // Socket Hygiene Check
        if socket_path.exists() {
            let config = RedisConfig {
                server: ServerConfig::Unix {
                    path: socket_path.clone(),
                },
                ..Default::default()
            };

            let test_client = Builder::from_config(config)
                .with_connection_config(|c| c.connection_timeout = Duration::from_millis(500))
                .build()?;

            let is_alive = match tokio::time::timeout(Duration::from_millis(1000), async {
                test_client.init().await?;
                test_client.wait_for_connect().await?;
                let _: () = test_client.ping().await?;
                Ok::<(), anyhow::Error>(())
            })
            .await
            {
                Ok(Ok(())) => {
                    let _ = test_client.quit().await;
                    true
                }
                _ => false,
            };

            if !is_alive && manage_process {
                let _ = std::fs::remove_file(&socket_path);
                let _ = std::fs::remove_file(&pid_path);
            }
        }

        // 1. Start Redis Process if managed and socket doesn't exist
        if manage_process && !socket_path.exists() {
            std::fs::create_dir_all(&data_dir)?;
            std::fs::create_dir_all(home_path.join("run"))?;
            std::fs::create_dir_all(home_path.join("logs"))?;
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

            sleep(Duration::from_millis(500)).await;
        }

        // 2. Connect via UDS
        let config = RedisConfig {
            server: ServerConfig::Unix { path: socket_path },
            ..Default::default()
        };

        // Primary Connection Pool (8 connections)
        let pool = Builder::from_config(config.clone())
            .with_connection_config(|c| {
                c.connection_timeout = Duration::from_secs(5);
            })
            .build_pool(8)?;

        // Subscriber client for PubSub
        let subscriber = Builder::from_config(config)
            .with_connection_config(|c| {
                c.connection_timeout = Duration::from_secs(5);
            })
            .build()?;

        pool.init().await?;
        subscriber.init().await?;

        Ok(Self {
            pool,
            subscriber,
            process,
        })
    }
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
    }
}
