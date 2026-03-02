use std::sync::Arc;
use crate::engine::storage_bridge::KoadStorageBridge;
use koad_core::intent::{GovernanceAction, GovernanceIntent};
use koad_core::storage::StorageBridge;
use tokio::process::Command;
use chrono::Utc;
use serde_json::json;
use fred::interfaces::StreamsInterface;

pub struct KoadComplianceManager {
    storage: Arc<KoadStorageBridge>,
}

impl KoadComplianceManager {
    pub fn new(storage: Arc<KoadStorageBridge>) -> Self {
        Self { storage }
    }

    pub async fn handle_intent(&self, intent: GovernanceIntent) -> anyhow::Result<()> {
        let timestamp = Utc::now().timestamp();
        println!("KCM: Handling Governance Action: {:?}", intent.action);

        let result = match intent.action {
            GovernanceAction::Clean => {
                self.run_repo_clean().await
            }
            GovernanceAction::Audit => {
                self.run_audit().await
            }
            GovernanceAction::Sync => {
                self.sync_board().await
            }
        };

        let status = if result.is_ok() { "SUCCESS" } else { "FAILED" };
        let error_msg = result.as_ref().err().map(|e| e.to_string());

        // Log the governance event to the event stream
        let event = json!({
            "action": intent.action,
            "target": intent.target,
            "status": status,
            "error": error_msg,
            "timestamp": timestamp
        });
        
        let _: () = self.storage.redis.client.xadd(
            "koad:events:stream", false, None, "*", 
            vec![
                ("source", "engine:kcm"),
                ("severity", if result.is_ok() { "INFO" } else { "ERROR" }),
                ("message", "GOVERNANCE_EXECUTION"),
                ("metadata", &event.to_string()),
                ("timestamp", &timestamp.to_string())
            ]
        ).await?;

        result
    }

    async fn run_repo_clean(&self) -> anyhow::Result<()> {
        println!("KCM: Running Repository Cleanup...");
        let output = Command::new("python3")
            .arg("/home/ideans/.koad-os/doodskills/repo-clean.py")
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("KCM: Repo Clean Failed: {}", stderr);
            anyhow::bail!("Repo clean failed: {}", stderr);
        }
        Ok(())
    }

    async fn run_audit(&self) -> anyhow::Result<()> {
        println!("KCM: Running System Audit...");
        let output = Command::new("/home/ideans/.koad-os/bin/koad")
            .arg("doctor")
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("KCM: Audit Failed: {}", stderr);
            anyhow::bail!("Audit failed: {}", stderr);
        }
        Ok(())
    }

    async fn sync_board(&self) -> anyhow::Result<()> {
        println!("KCM: Synchronizing Project Board...");
        let output = Command::new("/home/ideans/.koad-os/bin/koad")
            .arg("board")
            .arg("sync")
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("KCM: Board Sync Failed: {}", stderr);
            anyhow::bail!("Board sync failed: {}", stderr);
        }
        Ok(())
    }
}
