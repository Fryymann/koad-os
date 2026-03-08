use fred::clients::RedisPool;
use fred::prelude::*;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

pub struct RedisClient {
    pub pool: RedisPool,
    pub subscriber: fred::clients::RedisClient,
    pub _process: Option<Child>,
}

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

        // Socket Hygiene Check
        if socket_path.exists() {
            println!(
                "Testing existing Redis socket at {}...",
                socket_path.display()
            );
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

            if !is_alive {
                println!("Socket is stale or unresponsive. Purging ghost files...");
                let _ = std::fs::remove_file(&socket_path);
                let _ = std::fs::remove_file(&pid_path);
            } else {
                println!("Socket is active and responding to PING.");
            }
        }

        // 1. Start Redis Process ONLY if socket doesn't exist
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

        // 3. Setup event listeners
        pool.next().on_error(|e| {
            eprintln!("Redis Pool Error: {:?}", e);
            Ok(())
        });

        println!("Connected to Redis Pool (8 connections) + Subscriber.");

        Ok(Self {
            pool,
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
