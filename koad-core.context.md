# Context Packet: koad-core
Generated: 2026-04-06

## Purpose
Koad-Core: The Hull of the Spaceship
Shared traits, types, and constants for the KoadOS workspace.

## Public API

File: /home/ideans/.koad-os/crates/koad-core/src/config.rs
  - func: default_admin_socket
  - func: default
  - func: default_agent_tier
  - func: load
  - func: from_json
  - func: to_json
  - func: get_db_path
  - func: resolve_active_agent
  - func: get_redis_socket
  - func: get_citadel_socket
  - func: get_admin_socket
  - func: get_agent_name
  - func: resolve_vault_uri
  - func: resolve_vault_path
  - func: get_github_owner
  - func: get_github_repo
  - func: resolve_secret
  - func: resolve_indirect_value
  - func: resolve_gh_token
  - func: resolve_project_context
  - func: default_network
  - func: default_storage
  - func: default_sessions
  - func: default_watchdog
  - func: default_sandbox
  - func: test_config_loading
  - struct: CitadelSubsystem
  - struct: MotdConfig
  - struct: StatusBoardConfig
  - struct: CitadelStatusRegistry
  - struct: KoadConfig
  - struct: NetworkConfig
  - struct: StorageConfig
  - struct: SessionsConfig
  - struct: WatchdogConfig
  - struct: SandboxConfig
  - struct: SystemConfig
  - struct: XpConfig
  - struct: SkillDefinition
  - struct: IntegrationsConfig
  - struct: GithubConfig
  - struct: NotionConfig
  - struct: AirtableConfig
  - struct: SlackConfig
  - struct: FilesystemConfig
  - struct: AgentIdentityConfig
  - struct: AgentPreferences
  - struct: InterfaceConfig
  - struct: ProjectConfig
  - struct: ProjectDirConfig

File: /home/ideans/.koad-os/crates/koad-core/src/health.rs
  - func: new
  - func: add
  - func: check_subsystems
  - func: run_probe
  - struct: SystemStatus
  - struct: HealthCheck
  - struct: HealthRegistry

File: /home/ideans/.koad-os/crates/koad-core/src/hierarchy.rs
  - func: new
  - func: resolve_level
  - func: validate_access
  - struct: HierarchyManager

File: /home/ideans/.koad-os/crates/koad-core/src/identity.rs
  - func: test_identity_serialization
  - struct: Identity

File: /home/ideans/.koad-os/crates/koad-core/src/intelligence.rs
  - func: new
  - struct: FactCard
  - struct: ContextSummary

File: /home/ideans/.koad-os/crates/koad-core/src/intent.rs
  - func: test_intent_execute_serialization
  - func: test_intent_legacy_compatibility
  - struct: ExecuteIntent
  - struct: SkillIntent
  - struct: SessionIntent
  - struct: SystemIntent
  - struct: GovernanceIntent

File: /home/ideans/.koad-os/crates/koad-core/src/lib.rs
  - trait: Component

File: /home/ideans/.koad-os/crates/koad-core/src/logging.rs
  - func: init_logging

File: /home/ideans/.koad-os/crates/koad-core/src/session.rs
  - func: new
  - func: is_active
  - func: make_session
  - func: new_initializes_with_active_status
  - func: is_active_returns_true_for_fresh_session
  - func: is_active_returns_false_when_heartbeat_exceeds_timeout
  - func: is_active_boundary_just_inside_timeout_is_active
  - func: is_active_boundary_at_exact_timeout_is_inactive
  - func: agent_path_uses_agents_folder
  - struct: AgentSession
  - struct: ProjectContext

File: /home/ideans/.koad-os/crates/koad-core/src/signal.rs
  - func: new
  - func: stream_key
  - func: consumer_group
  - func: broadcast
  - func: ensure_consumer_groups
  - func: read_messages
  - func: ack
  - func: make_redis
  - func: test_broadcast_appends_to_stream
  - func: test_consumer_group_is_idempotent
  - func: test_two_agents_can_exchange_signal
  - struct: SignalCorps

File: /home/ideans/.koad-os/crates/koad-core/src/storage.rs
  - struct: StateMetadata
  - trait: StorageBridge

File: /home/ideans/.koad-os/crates/koad-core/src/types.rs
  - func: test_log_entry_serialization
  - struct: Ticket
  - struct: LogEntry
  - struct: HotContextChunk

File: /home/ideans/.koad-os/crates/koad-core/src/utils/lock.rs
  - func: try_acquire
  - func: release
  - func: drop
  - func: lock
  - func: unlock
  - func: try_acquire_returns_some_when_lock_is_available
  - func: try_acquire_returns_none_when_lock_is_held
  - func: release_returns_ok_when_ownership_is_valid
  - func: with_sector_lock_macro_executes_body_and_returns_value
  - func: with_sector_lock_macro_returns_lock_denied_on_contention
  - struct: SectorLockGuard
  - struct: MockLock
  - trait: DistributedLock

File: /home/ideans/.koad-os/crates/koad-core/src/utils/pid.rs
  - func: new
  - func: drop
  - func: pid_file_is_live
  - func: find_ghosts
  - func: new_creates_pid_file_with_current_pid
  - func: drop_removes_pid_file
  - func: new_fails_when_referenced_process_is_alive
  - func: new_overwrites_stale_pid_file_and_succeeds
  - func: new_creates_file_when_no_existing_pid_file
  - func: find_ghosts_returns_empty_for_clean_directory
  - func: find_ghosts_detects_stale_kcitadel_pid
  - func: find_ghosts_ignores_live_process
  - struct: PidGuard

File: /home/ideans/.koad-os/crates/koad-core/src/utils/redis.rs
  - func: new
  - func: drop
  - struct: RedisClient

File: /home/ideans/.koad-os/crates/koad-core/src/utils/tokens.rs
  - func: count_tokens

## Recent Git Activity
```
1835555 prep for stable release
47bc605 update 3-31-2026
0c1be0a updates
5a285b8 major cleanup
9ab5a22 more jupiter updates
af87531 rename agent folders without the '.' prefix.
786478a feat(citadel): Jupiter migration session 4 — agent runtime model, system init, build fixes
7e067c6 fix(config): align env var reads with KOADOS_ namespace for Jupiter migration
d1e3eb2 feat: hierarchical context resolution and identity decoupling (v3)
a0bba5a migration update
```

## Key Dependencies
```toml
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
uuid.workspace = true
anyhow.workspace = true
rusqlite.workspace = true
async-trait.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
tracing-appender.workspace = true
sysinfo.workspace = true
fred = { workspace = true, features = ["unix-sockets", "subscriber-client", "i-scripts", "i-pubsub"] }
tokio.workspace = true
config = { workspace = true, features = ["toml"] }
toml.workspace = true
```
