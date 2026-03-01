#[cfg(test)]
mod tests {
    use crate::engine::redis::RedisClient;
    use crate::engine::persistence::PersistenceManager;
    use std::sync::Arc;
    use std::path::Path;

    #[tokio::test]
    async fn test_redis_lifecycle() {
        let config_path = "/home/ideans/.koad-os/config/redis.conf";
        // This will start and stop the redis server via Drop
        let redis = RedisClient::new(config_path).await;
        assert!(redis.is_ok(), "RedisClient should initialize and connect");
    }

    #[tokio::test]
    async fn test_persistence_initialization() {
        let config_path = "/home/ideans/.koad-os/config/redis.conf";
        let redis = Arc::new(RedisClient::new(config_path).await.unwrap());
        let db_path = "/home/ideans/.koad-os/test_koad.db";
        
        let pm = PersistenceManager::new(redis, db_path);
        if let Err(ref e) = pm {
            eprintln!("PersistenceManager Init Error: {:?}", e);
        }
        assert!(pm.is_ok(), "PersistenceManager should initialize SQLite with WAL");
        
        // Cleanup
        if Path::new(db_path).exists() {
            let _ = std::fs::remove_file(db_path);
            let _ = std::fs::remove_file(format!("{}-shm", db_path));
            let _ = std::fs::remove_file(format!("{}-wal", db_path));
        }
    }
}
