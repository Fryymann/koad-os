use crate::engine::storage_bridge::KoadStorageBridge;
use chrono::Utc;
use fred::interfaces::StreamsInterface;
use koad_core::intent::{GovernanceAction, GovernanceIntent};

use serde_json::json;
use std::sync::Arc;
use tokio::process::Command;

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
            GovernanceAction::Clean => self.run_repo_clean().await,
            GovernanceAction::Audit => self.run_audit().await,
            GovernanceAction::Sync => self.sync_board().await,
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

        let _: () = self
            .storage
            .redis
            .pool
            .xadd(
                "koad:events:stream",
                false,
                None,
                "*",
                vec![
                    ("source", "engine:kcm"),
                    ("severity", if result.is_ok() { "INFO" } else { "ERROR" }),
                    ("message", "GOVERNANCE_EXECUTION"),
                    ("metadata", &event.to_string()),
                    ("timestamp", &timestamp.to_string()),
                ],
            )
            .await?;

        result
    }

    async fn run_repo_clean(&self) -> anyhow::Result<()> {
        let koad_home = std::env::var("KOAD_HOME").expect("KOAD_HOME not set");
        let script_path = format!("{}/doodskills/repo-clean.py", koad_home);

        // Try to find python in venv first
        let venv_python = format!("{}/venv/bin/python3", koad_home);
        let python_exe = if std::path::Path::new(&venv_python).exists() {
            venv_python
        } else {
            "python3".to_string()
        };

        println!(
            "KCM: Running Repository Cleanup at {} using {}...",
            script_path, python_exe
        );
        let output = Command::new(python_exe).arg(&script_path).output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("KCM: Repo Clean Failed: {}", stderr);
            anyhow::bail!("Repo clean failed: {}", stderr);
        }
        Ok(())
    }

    async fn run_audit(&self) -> anyhow::Result<()> {
        let koad_home = std::env::var("KOAD_HOME").expect("KOAD_HOME not set");
        let koad_bin = format!("{}/bin/koad", koad_home);

        println!("KCM: Running System Audit via {}...", koad_bin);
        let output = Command::new(&koad_bin)
            .arg("doctor")
            .env("KOAD_HOME", &koad_home)
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
        let koad_home = std::env::var("KOAD_HOME").expect("KOAD_HOME not set");
        let koad_bin = format!("{}/bin/koad", koad_home);

        println!("KCM: Synchronizing Project Board via {}...", koad_bin);
        let output = Command::new(&koad_bin)
            .arg("board")
            .arg("sync")
            .env("KOAD_HOME", &koad_home)
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
