use anyhow::{Context, Result};
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use crate::constants::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitadelSubsystem {
    pub id: String,
    pub name: String,
    pub subsystem: String,
    pub enabled: bool,
    pub stub: bool,
    pub probe_type: String,
    pub probe_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotdConfig {
    pub enabled: bool,
    pub show_citadel_snapshot: bool,
    pub show_agent_identity: bool,
    pub show_stats: bool,
    pub show_intelligence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBoardConfig {
    pub refresh_interval_secs: u64,
    pub color_mode: String,
    pub systems: Vec<CitadelSubsystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitadelStatusRegistry {
    pub motd: MotdConfig,
    pub status_board: StatusBoardConfig,
}

/// The global configuration for the KoadOS environment.
///
/// This structure aggregates all system-level settings, including network paths,
/// storage configurations, active identities, and project registries.
/// It is typically loaded from `kernel.toml` and augmented by identity-specific
/// TOML files in the `config/identities/` directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoadConfig {
    /// Canonical path to the KoadOS home directory.
    pub home: PathBuf,
    /// High-level system metadata (version, repository info).
    pub system: SystemConfig,
    /// Network and socket configuration for inter-service communication.
    pub network: NetworkConfig,
    /// Persistence settings for the primary SQLite database.
    pub storage: StorageConfig,
    /// Optional registry for MOTD and status board visualization.
    #[serde(default)]
    pub status_registry: Option<CitadelStatusRegistry>,
    /// Session lifecycle and timeout policies.
    #[serde(default = "default_sessions")]
    pub sessions: SessionsConfig,
    /// Security sandboxing rules for agent execution.
    #[serde(default = "default_sandbox")]
    pub sandbox: SandboxConfig,
    /// Experience Point (XP) and leveling progression curves.
    #[serde(default)]
    pub xp: XpConfig,
    /// Registry of globally available skill blueprints.
    #[serde(default)]
    pub skills: HashMap<String, SkillBlueprint>,
    /// External service integration settings (GitHub, Notion, etc.).
    #[serde(default)]
    pub integrations: IntegrationsConfig,
    /// Filesystem access control and workspace protection rules.
    #[serde(default)]
    pub filesystem: FilesystemConfig,
    /// Map of all recognized agent identities and their configurations.
    #[serde(default)]
    pub identities: HashMap<String, AgentIdentityConfig>,
    /// Driver and bootstrap configuration for model interfaces.
    #[serde(default)]
    pub interfaces: HashMap<String, InterfaceConfig>,
    /// Registry of managed projects and their metadata.
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
    /// Index of project directories for fast resolution.
    #[serde(default)]
    pub project_dirs: HashMap<String, ProjectDirConfig>,
    /// Catch-all for unplanned or extension-specific configuration.
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub citadel_grpc_port: u32,
    pub citadel_grpc_addr: String,
    pub cass_grpc_port: u32,
    pub cass_grpc_addr: String,
    pub redis_socket: String,
    pub citadel_socket: String,
    #[serde(default = "default_admin_socket")]
    pub admin_socket: String,
}

fn default_admin_socket() -> String {
    DEFAULT_ADMIN_SOCK.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub db_name: String,
    pub drain_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsConfig {
    pub deadman_timeout_secs: u64,
    pub dark_timeout_secs: u64,
    pub purge_timeout_secs: u64,
    pub lease_duration_secs: u64,
    pub reaper_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub blacklist: Vec<String>,
    pub sanctuary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub version: String,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpConfig {
    pub level_curve_exponent: f32,
    pub base_xp_per_level: u32,
    pub grant_cap_per_turn: i32,
}

impl Default for XpConfig {
    fn default() -> Self {
        Self {
            level_curve_exponent: 1.5,
            base_xp_per_level: 100,
            grant_cap_per_turn: 50,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillRuntimeType {
    #[default]
    Builtin,
    Wasm,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBlueprint {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub description: String,
    pub xp_multiplier: f32,
    #[serde(default)]
    pub max_level: u32,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub runtime: SkillRuntimeType,
    #[serde(default)]
    pub entry_point: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Backward-compat alias — prefer `SkillBlueprint` in new code.
#[deprecated(since = "3.2.0", note = "Use SkillBlueprint instead")]
pub type SkillDefinition = SkillBlueprint;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillInstance {
    pub blueprint_id: String,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub current_xp: u32,
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IntegrationsConfig {
    pub github: Option<GithubConfig>,
    pub notion: Option<NotionConfig>,
    pub airtable: Option<AirtableConfig>,
    pub slack: Option<SlackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubConfig {
    pub default_owner: String,
    pub default_repo: String,
    pub default_project_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionConfig {
    pub enabled: bool,
    pub index: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirtableConfig {
    pub enabled: bool,
    pub base_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub enabled: bool,
    pub channel_id: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    /// Canonical root of the KoadOS workspace.
    pub workspace_root: Option<String>,
    /// List of paths protected from modification by agents.
    pub protected_paths: Vec<String>,
    /// Global whitelist of directories accessible via the Filesystem MCP Server.
    #[serde(default)]
    pub allowed_directories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentityConfig {
    pub name: String,
    pub role: String,
    pub rank: String,
    pub bio: String,
    pub vault: Option<String>,
    /// URI for vault resolution (e.g. file://~/.tyr/ or ghost://tyr).
    pub vault_uri: Option<String>,
    pub bootstrap: Option<String>,
    pub preferences: Option<AgentPreferences>,
    /// Required runtime body for this agent (e.g. "gemini", "claude", "codex").
    /// If set, boot will refuse to hydrate unless KOAD_RUNTIME matches.
    #[serde(default)]
    pub runtime: Option<String>,
    /// Agent tier level (1=Initiate, 2=Crew, 3=Officer, 4=Captain).
    #[serde(default = "default_agent_tier")]
    pub tier: u32,
    /// Persistent XP total for this agent.
    #[serde(default)]
    pub xp: u32,
    /// Skills this agent has equipped, keyed to blueprint IDs.
    #[serde(default)]
    pub skills: Vec<SkillInstance>,
}

fn default_agent_tier() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPreferences {
    /// List of access keys/tokens used by the agent.
    pub access_keys: Vec<String>,
    /// Agent-specific whitelist of directories accessible via the Filesystem MCP Server.
    #[serde(default)]
    pub allowed_directories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    pub driver: String,
    pub bootstrap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub path: PathBuf,
    pub level: Option<String>,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub default_project: Option<u32>,
    pub station: Option<String>,
    pub credential_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDirConfig {
    pub path: PathBuf,
}

impl KoadConfig {
    pub fn load() -> Result<Self> {
        let home = match env::var("KOADOS_HOME").or_else(|_| env::var("KOAD_HOME")) {
            Ok(val) => {
                if val.starts_with('~') {
                    let home_dir = dirs::home_dir()
                        .context("Could not determine home directory for tilde expansion.")?;
                    PathBuf::from(val.replacen('~', &home_dir.to_string_lossy(), 1))
                } else {
                    PathBuf::from(val)
                }
            }
            Err(_) => dirs::home_dir()
                .context("Could not determine home directory and KOADOS_HOME is not set")?
                .join(".koad-os"),
        };

        let kernel_path = home.join("config/kernel.toml");
        let status_path = home.join("config/citadel_status.toml");
        let mut builder = Config::builder()
            .set_default("home", home.to_string_lossy().to_string())?
            .add_source(File::from(kernel_path).required(false))
            .add_source(File::from(status_path).required(false));

        // Glob identities
        let identities_dir = home.join("config/identities");
        if identities_dir.exists() {
            for entry in std::fs::read_dir(identities_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    builder = builder.add_source(File::from(path).required(false));
                }
            }
        }

        // Glob projects
        let projects_dir = home.join("config/projects");
        if projects_dir.exists() {
            for entry in std::fs::read_dir(projects_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    builder = builder.add_source(File::from(path).required(false));
                }
            }
        }

        let s = builder
            .add_source(config::Environment::with_prefix("KOAD").separator("__"))
            .build()?;

        s.try_deserialize()
            .context("Failed to deserialize KoadConfig")
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to parse config JSON")
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize KoadConfig to JSON")
    }

    pub fn get_db_path(&self) -> PathBuf {
        self.home.join(&self.storage.db_name)
    }

    pub fn agent_dir(&self, name: &str) -> PathBuf {
        self.home.join("agents").join(name)
    }

    pub fn vault_path(&self, name: &str) -> PathBuf {
        self.home.join("agents").join(name)
    }

    /// Resolves the active agent name by checking environment variables
    /// and verifying against the live Citadel session if possible.
    pub async fn resolve_active_agent(&self) -> String {
        // 1. Check for active session ID
        if let Ok(session_id) = env::var("KOAD_SESSION_ID") {
            if !session_id.is_empty() {
                // Future: Add Redis lookup here if koad-core gains async redis dependency.
                // For now, we trust the env var if it's set after a boot.
            }
        }

        // 2. Fallback to name-based env var
        if let Ok(name) = env::var("KOAD_AGENT_NAME") {
            return name;
        }

        // 3. System Default
        "admin".to_string()
    }

    pub fn get_redis_socket(&self) -> PathBuf {
        self.home.join(&self.network.redis_socket)
    }

    pub fn get_citadel_socket(&self) -> PathBuf {
        self.home.join(&self.network.citadel_socket)
    }

    pub fn get_admin_socket(&self) -> PathBuf {
        self.home.join(&self.network.admin_socket)
    }

    pub fn get_agent_name(&self) -> String {
        if let Ok(agent) = env::var("KOAD_AGENT") {
            return agent;
        }
        "Admiral".to_string()
    }

    /// Resolves the vault URI for the current agent.
    pub fn resolve_vault_uri(&self, agent_name: &str) -> Option<String> {
        // 1. Check for explicit environment override
        if let Ok(uri) = env::var("KOAD_VAULT_URI") {
            if !uri.is_empty() {
                return Some(uri);
            }
        }

        // 2. Check agent identity config
        if let Some(id_config) = self.identities.get(&agent_name.to_lowercase()) {
            if let Some(uri) = &id_config.vault_uri {
                return Some(uri.clone());
            }
            // If only vault path is set, convert it to a file:// URI
            if let Some(path) = &id_config.vault {
                return Some(format!("file://{}", path));
            }
        }

        // 3. Fallback to standard local discovery (file://<home>/agents/<name>)
        let home = dirs::home_dir()?;
        let path = self.agent_dir(&agent_name.to_lowercase());
        if path.exists() {
            return Some(format!("file://{}", path.display()));
        }

        // 4. Final fallback: ~/.<name>
        let legacy_path = home.join(format!(".{}", agent_name.to_lowercase()));
        if legacy_path.exists() {
            return Some(format!("file://{}", legacy_path.display()));
        }

        None
    }

    /// Resolves a vault URI to a local PathBuf.
    /// Currently only supports file:// scheme.
    pub fn resolve_vault_path(&self, uri: &str) -> Result<PathBuf> {
        if let Some(path_str) = uri.strip_prefix("file://") {
            let mut p_str = path_str.to_string();
            if p_str.starts_with('~') {
                let home = dirs::home_dir()
                    .context("Could not determine home directory for tilde expansion.")?;
                p_str = p_str.replacen('~', &home.to_string_lossy(), 1);
            }
            let path = PathBuf::from(p_str);
            if path.exists() {
                Ok(path)
            } else {
                anyhow::bail!("Vault path does not exist: {}", path.display())
            }
        } else {
            anyhow::bail!(
                "Unsupported vault URI scheme: {}. Currently only 'file://' is supported.",
                uri
            )
        }
    }

    pub fn get_github_owner(&self, project: Option<&str>) -> String {
        if let Some(proj) = project {
            if let Some(p_config) = self.projects.get(proj) {
                if let Some(owner) = &p_config.github_owner {
                    // Check if owner is a reference (e.g. KOADOS_...)
                    if owner.starts_with("KOADOS_") {
                        return self.resolve_indirect_value(owner);
                    }
                    return owner.clone();
                }
            }
        }

        // 2. Check System Config
        if let Some(ref o) = self.system.github_owner {
            return o.clone();
        }

        // Hierarchical fallback for owner
        let owner = self.resolve_secret("GITHUB_USER", project);
        if !owner.is_empty() {
            return owner;
        }

        self.integrations
            .github
            .as_ref()
            .map(|g| g.default_owner.clone())
            .unwrap_or_else(|| DEFAULT_GITHUB_OWNER.to_string())
    }

    pub fn get_github_repo(&self, project: Option<&str>) -> String {
        if let Some(proj) = project {
            if let Some(p_config) = self.projects.get(proj) {
                if let Some(repo) = &p_config.github_repo {
                    return repo.clone();
                }
            }
        }

        // 2. Check System Config
        if let Some(ref r) = self.system.github_repo {
            return r.clone();
        }

        self.integrations
            .github
            .as_ref()
            .map(|g| g.default_repo.clone())
            .unwrap_or_else(|| DEFAULT_GITHUB_REPO.to_string())
    }

    /// Resolves a secret hierarchically: Outpost > Station > Main.
    /// Supports indirect resolution where the KOADOS_ variable points to another variable.
    pub fn resolve_secret(&self, key_id: &str, project: Option<&str>) -> String {
        let mut station_name = None;
        let mut outpost_name = None;

        // 1. Detect Outpost & Station (Search upward from current directory or project path)
        let mut current_dir = if let Some(proj) = project {
            self.projects
                .get(proj)
                .map(|p| p.path.clone())
                .unwrap_or_else(|| env::current_dir().unwrap_or_default())
        } else {
            env::current_dir().unwrap_or_default()
        };

        while current_dir.parent().is_some() {
            if outpost_name.is_none() {
                let outpost_marker = current_dir.join(".agent-outpost");
                if outpost_marker.exists() {
                    if let Ok(name) = std::fs::read_to_string(outpost_marker) {
                        outpost_name = Some(name.trim().to_uppercase());
                    }
                }
            }
            if station_name.is_none() {
                let station_marker = current_dir.join(".agent-station");
                if station_marker.exists() {
                    if let Ok(name) = std::fs::read_to_string(station_marker) {
                        station_name = Some(name.trim().to_uppercase());
                    }
                }
            }
            if outpost_name.is_some() && station_name.is_some() {
                break;
            }
            current_dir = current_dir.parent().unwrap().to_path_buf();
        }

        // 2. Try Outpost Override
        if let Some(outpost) = outpost_name {
            let outpost_key = format!("KOADOS_OUTPOST_{}_{}", outpost, key_id);
            if let Ok(val) = env::var(&outpost_key) {
                return self.resolve_indirect_value(&val);
            }
        }

        // 3. Try Station Override
        if let Some(station) = station_name {
            let station_key = format!("KOADOS_STATION_{}_{}", station, key_id);
            if let Ok(val) = env::var(&station_key) {
                return self.resolve_indirect_value(&val);
            }
        }

        // 4. Fallback to Main (Citadel)
        let main_key = format!("KOADOS_MAIN_{}", key_id);
        if let Ok(val) = env::var(&main_key) {
            return self.resolve_indirect_value(&val);
        }

        // 5. Legacy Fallback (direct environment variable)
        env::var(key_id).unwrap_or_default()
    }

    /// Resolves an indirect value (e.g. if KOADOS_... points to another env var)
    pub fn resolve_indirect_value(&self, val: &str) -> String {
        if let Ok(deref) = env::var(val) {
            deref
        } else {
            val.to_string()
        }
    }

    pub fn resolve_gh_token(&self, project: Option<&str>, _agent: Option<&str>) -> Result<String> {
        let key_id = if let Some(proj) = project {
            self.projects
                .get(proj)
                .and_then(|p| p.credential_key.as_ref())
                .map(|k| k.as_str())
                .unwrap_or("GITHUB_PAT")
        } else {
            "GITHUB_PAT"
        };

        // Try Hierarchical first
        let token = self.resolve_secret(key_id, project);
        if !token.is_empty() {
            return Ok(token);
        }

        // Fallback to direct GITHUB_PAT
        Ok(env::var("GITHUB_PAT").unwrap_or_default())
    }

    pub fn resolve_project_context(&self, path: &Path) -> Option<(String, String)> {
        for (name, config) in &self.projects {
            if path.starts_with(&config.path) {
                return Some((name.clone(), name.clone()));
            }
        }
        None
    }
}

pub fn default_network() -> NetworkConfig {
    NetworkConfig {
        citadel_grpc_port: DEFAULT_CITADEL_GRPC_PORT,
        citadel_grpc_addr: DEFAULT_CITADEL_GRPC_ADDR.to_string(),
        cass_grpc_port: DEFAULT_CASS_GRPC_PORT,
        cass_grpc_addr: DEFAULT_CASS_GRPC_ADDR.to_string(),
        redis_socket: DEFAULT_REDIS_SOCK.to_string(),
        citadel_socket: DEFAULT_CITADEL_SOCK.to_string(),
        admin_socket: DEFAULT_ADMIN_SOCK.to_string(),
    }
}

pub fn default_storage() -> StorageConfig {
    StorageConfig {
        db_name: "data/db/koad.db".to_string(),
        drain_interval_secs: 30,
    }
}

pub fn default_sessions() -> SessionsConfig {
    SessionsConfig {
        deadman_timeout_secs: DEFAULT_DEADMAN_TIMEOUT_SECS,
        dark_timeout_secs: DEFAULT_DARK_TIMEOUT_SECS,
        purge_timeout_secs: DEFAULT_PURGE_TIMEOUT_SECS,
        lease_duration_secs: DEFAULT_LEASE_DURATION_SECS,
        reaper_interval_secs: DEFAULT_REAPER_INTERVAL_SECS,
    }
}

fn default_sandbox() -> SandboxConfig {
    SandboxConfig {
        enabled: true,
        blacklist: vec![
            "sudo ".to_string(),
            "su ".to_string(),
            "rm -rf /".to_string(),
            "koad boot".to_string(),
        ],
        sanctuary: vec![
            ".koad-os".to_string(),
            "/etc".to_string(),
            "/var".to_string(),
            "/root".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_blueprint_serialization() {
        // Parse the hello-world fixture inline (no filesystem dependency)
        let toml_str = r#"
            id = "hello-world"
            name = "Hello World"
            description = "Test blueprint"
            xp_multiplier = 1.0
            max_level = 5
            version = "0.1.0"
            runtime = "builtin"
            entry_point = ""
            capabilities = ["fs_read"]
        "#;
        let bp: SkillBlueprint = toml::from_str(toml_str).expect("blueprint should parse");
        assert_eq!(bp.id, "hello-world");
        assert_eq!(bp.name, "Hello World");
        assert!(matches!(bp.runtime, SkillRuntimeType::Builtin));
        assert_eq!(bp.capabilities, vec!["fs_read"]);
    }

    #[test]
    fn test_skill_instance_equip_round_trip() {
        use std::collections::HashMap;
        // Build an AgentIdentityConfig with a SkillInstance and round-trip through TOML
        let instance = SkillInstance {
            blueprint_id: "hello-world".to_string(),
            level: 1,
            current_xp: 10,
            settings: HashMap::from([("debug".to_string(), "true".to_string())]),
        };
        let agent = AgentIdentityConfig {
            name: "test-agent".to_string(),
            role: "Tester".to_string(),
            rank: "Crew".to_string(),
            bio: "A test agent".to_string(),
            vault: None,
            vault_uri: None,
            bootstrap: None,
            preferences: None,
            runtime: None,
            tier: 3,
            xp: 0,
            skills: vec![instance],
        };
        let serialized = toml::to_string(&agent).expect("should serialize");
        let deserialized: AgentIdentityConfig =
            toml::from_str(&serialized).expect("should deserialize");
        assert_eq!(deserialized.skills.len(), 1);
        assert_eq!(deserialized.skills[0].blueprint_id, "hello-world");
        assert_eq!(deserialized.skills[0].level, 1);
        assert_eq!(
            deserialized.skills[0]
                .settings
                .get("debug")
                .map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn test_skill_scanner_discovery() {
        use crate::skills::SkillScanner;
        use std::fs;

        // Write two .skill.toml blueprints to a temp directory
        let tmp = tempfile::tempdir().expect("tempdir");
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let bp1 = "name = \"Alpha\"\ndescription = \"First\"\nxp_multiplier = 1.0\n";
        let bp2 = "name = \"Beta\"\ndescription = \"Second\"\nxp_multiplier = 2.0\n";
        fs::write(skills_dir.join("alpha.skill.toml"), bp1).unwrap();
        fs::write(skills_dir.join("beta.skill.toml"), bp2).unwrap();

        let scanner = SkillScanner::new(tmp.path());
        let found = scanner.scan().expect("scan should succeed");
        assert_eq!(found.len(), 2, "scanner should find exactly 2 blueprints");
    }
}
