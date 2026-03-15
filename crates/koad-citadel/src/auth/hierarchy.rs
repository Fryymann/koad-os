//! Hierarchy Manager
//!
//! Resolves physical filesystem paths to logical Workspace Levels
//! based on the Citadel configuration. This is a core component of 
//! the Level-Aware Workspace Hierarchy (v3.2).

use koad_core::config::KoadConfig;
use std::path::Path;
use koad_proto::citadel::v5::WorkspaceLevel;

/// Manages the resolution and validation of Workspace Levels.
pub struct HierarchyManager {
    config: KoadConfig,
}

impl HierarchyManager {
    /// Creates a new `HierarchyManager` with the provided configuration.
    pub fn new(config: KoadConfig) -> Self {
        Self { config }
    }

    /// Resolves an absolute path to its corresponding [`WorkspaceLevel`].
    pub fn resolve_level(&self, path: &Path) -> WorkspaceLevel {
        let home = &self.config.home;
        
        // 1. Check if it's the Citadel (Platform Core)
        if path.starts_with(home) {
            return WorkspaceLevel::LevelCitadel;
        }

        // 2. Check registered Stations/Projects
        let mut best_match: Option<(usize, WorkspaceLevel)> = None;

        for (_, project) in &self.config.projects {
            let project_path = Path::new(&project.path);
            if path.starts_with(project_path) {
                let depth = project_path.components().count();
                let level = match project.level.as_deref() {
                    Some("station") => WorkspaceLevel::LevelStation,
                    Some("citadel") => WorkspaceLevel::LevelCitadel,
                    Some("system") => WorkspaceLevel::LevelSystem,
                    _ => WorkspaceLevel::LevelOutpost,
                };

                if best_match.is_none() || depth > best_match.unwrap().0 {
                    best_match = Some((depth, level));
                }
            }
        }

        if let Some((_, level)) = best_match {
            return level;
        }

        WorkspaceLevel::LevelSystem
    }

    /// Validates if an agent has permission to operate at a specific [`WorkspaceLevel`].
    pub fn validate_access(&self, agent_rank: &str, requested_level: WorkspaceLevel) -> bool {
        match requested_level {
            WorkspaceLevel::LevelSystem => agent_rank == "Admiral" || agent_rank == "Captain",
            WorkspaceLevel::LevelCitadel => agent_rank == "Admiral" || agent_rank == "Captain" || agent_rank == "Officer",
            WorkspaceLevel::LevelStation => agent_rank != "Crew",
            WorkspaceLevel::LevelOutpost => true,
            WorkspaceLevel::LevelUnspecified => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use koad_core::config::{ProjectConfig, StorageConfig, SessionsConfig, NetworkConfig, WatchdogConfig, IntegrationsConfig, FilesystemConfig, InterfaceConfig};
    use std::collections::HashMap;

    fn mock_config() -> KoadConfig {
        let mut projects = HashMap::new();
        projects.insert("test-repo".to_string(), ProjectConfig {
            path: "/home/user/repos/test-repo".to_string(),
            github_owner: None,
            github_repo: None,
            default_project: None,
            level: Some("outpost".to_string()),
            credential_key: None,
        });

        KoadConfig {
            home: PathBuf::from("/home/user/.koad-os"),
            system: None,
            network: NetworkConfig { 
                gateway_port: 0, gateway_addr: "".into(), spine_grpc_port: 0, 
                spine_grpc_addr: "".into(), redis_socket: "".into(), spine_socket: "".into() 
            },
            storage: StorageConfig { db_name: "".into(), drain_interval_secs: 0 },
            sessions: SessionsConfig { 
                deadman_timeout_secs: 0, dark_timeout_secs: 0, purge_timeout_secs: 0, 
                lease_duration_secs: 0, reaper_interval_secs: 0 
            },
            watchdog: WatchdogConfig { check_interval_secs: 0, max_failures: 0, monitor_asm: false },
            integrations: IntegrationsConfig::default(),
            filesystem: FilesystemConfig::default(),
            projects,
            identities: HashMap::new(),
            interfaces: HashMap::new(),
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_resolve_citadel_level() {
        let hm = HierarchyManager::new(mock_config());
        let level = hm.resolve_level(Path::new("/home/user/.koad-os/config"));
        assert_eq!(level, WorkspaceLevel::LevelCitadel);
    }

    #[test]
    fn test_resolve_outpost_level() {
        let hm = HierarchyManager::new(mock_config());
        let level = hm.resolve_level(Path::new("/home/user/repos/test-repo/src"));
        assert_eq!(level, WorkspaceLevel::LevelOutpost);
    }

    #[test]
    fn test_validate_access_rank_gating() {
        let hm = HierarchyManager::new(mock_config());
        assert!(hm.validate_access("Crew", WorkspaceLevel::LevelOutpost));
        assert!(hm.validate_access("Captain", WorkspaceLevel::LevelSystem));
    }
}
