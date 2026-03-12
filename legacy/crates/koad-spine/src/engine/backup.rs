use koad_core::config::KoadConfig;
use koad_core::constants::{BACKUP_DIR_REDIS, BACKUP_DIR_SQLITE};
use std::path::Path;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use chrono::Local;
use tracing::{info, error, warn};
use std::process::Command;

pub struct KoadBackupManager {
    config: Arc<KoadConfig>,
}

impl KoadBackupManager {
    pub fn new(config: Arc<KoadConfig>) -> Self {
        Self { config }
    }

    pub async fn start_backup_loop(&self) {
        info!("BackupManager: Persistence loop active.");
        
        // Initial delay to avoid collision with startup
        sleep(Duration::from_secs(60)).await;

        loop {
            if let Err(e) = self.perform_full_backup().await {
                error!("BackupManager Error: {}", e);
            }
            
            // Backup every 6 hours
            sleep(Duration::from_secs(6 * 3600)).await;
        }
    }

    pub async fn perform_full_backup(&self) -> anyhow::Result<()> {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let home = &self.config.home;
        
        // 1. SQLite Backup
        self.backup_sqlite(home, &timestamp).await?;
        
        // 2. Redis Backup
        self.backup_redis(home, &timestamp).await?;
        
        // 3. Prune Old Backups (Keep last 10 of each)
        self.prune_backups(&home.join(BACKUP_DIR_SQLITE), 10)?;
        self.prune_backups(&home.join(BACKUP_DIR_REDIS), 10)?;

        Ok(())
    }

    async fn backup_sqlite(&self, home: &Path, ts: &str) -> anyhow::Result<()> {
        let dest_dir = home.join(BACKUP_DIR_SQLITE);
        std::fs::create_dir_all(&dest_dir)?;
        
        let src = home.join(&self.config.storage.db_name);
        let dest = dest_dir.join(format!("koad-{}.db", ts));
        
        if src.exists() {
            info!("BackupManager: Sector SQLITE -> {:?}", dest);
            // Use sqlite3 .backup for safety (handles WAL/locking)
            let output = Command::new("sqlite3")
                .arg(&src)
                .arg(format!(".backup '{}'", dest.to_string_lossy()))
                .output()?;
                
            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                error!("SQLite backup failed: {}", err);
                // Fallback to simple copy if sqlite3 command fails
                std::fs::copy(&src, &dest)?;
            }
        }
        Ok(())
    }

    async fn backup_redis(&self, home: &Path, ts: &str) -> anyhow::Result<()> {
        let dest_dir = home.join(BACKUP_DIR_REDIS);
        std::fs::create_dir_all(&dest_dir)?;
        
        // Attempt to trigger a Redis SAVE first
        let socket = home.join(&self.config.network.redis_socket);
        let _ = Command::new("redis-cli")
            .arg("-s")
            .arg(&socket)
            .arg("SAVE")
            .status();

        let src = home.join("dump.rdb");
        let dest = dest_dir.join(format!("koad-{}.rdb", ts));
        
        if src.exists() {
            info!("BackupManager: Sector REDIS -> {:?}", dest);
            std::fs::copy(&src, &dest)?;
        } else {
            warn!("BackupManager: No Redis dump.rdb found at {:?}", src);
        }
        Ok(())
    }

    fn prune_backups(&self, dir: &Path, keep: usize) -> anyhow::Result<()> {
        if !dir.exists() { return Ok(()); }
        
        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(Result::ok)
            .collect();
            
        if entries.len() <= keep { return Ok(()); }
        
        // Sort by modification time
        entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
        
        let to_remove = entries.len() - keep;
        for i in 0..to_remove {
            let path = entries[i].path();
            info!("BackupManager: Pruning stale backup {:?}", path);
            let _ = std::fs::remove_file(path);
        }
        
        Ok(())
    }
}
