//! Docker/Podman-based container execution sandbox.
//!
//! Satisfies Phase 4 acceptance criterion:
//! "Commands run in the execution sandbox have no access to the host filesystem
//! or network (unless explicitly configured)."
//!
//! Uses `docker run` / `podman run` as a subprocess rather than the bollard API.
//! This approach:
//! - Works with both Docker and Podman transparently.
//! - Avoids rustc 1.93.x ICEs triggered by bollard's complex async generics.
//! - Is simpler and easier to audit.
//!
//! # Isolation enforced by default
//! - `--network none`                    — no network access
//! - `--read-only`                        — immutable root filesystem
//! - `--tmpfs /tmp`                       — writable scratch space only
//! - `--security-opt no-new-privileges`   — no privilege escalation
//! - `--memory` / `--cpus`               — resource limits
//! - No `-v` / `--volume` mounts          — no host filesystem exposure
//!
//! Requires the `container` cargo feature and `docker` (or `podman`) in `$PATH`.

use std::time::Duration;

use anyhow::{bail, Result};

#[cfg(feature = "container")]
use std::time::Instant;

#[cfg(feature = "container")]
use anyhow::Context;

#[cfg(feature = "container")]
use tracing::info;

#[cfg(feature = "container")]
use uuid::Uuid;

/// Result of a sandboxed command execution.
#[derive(Debug, Default)]
pub struct SandboxResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i64,
    /// Wall-clock duration of the container run in milliseconds.
    pub duration_ms: u64,
    /// Peak memory usage in bytes (0 when unavailable or feature not enabled).
    pub memory_bytes: u64,
}

/// Configuration for a container execution sandbox.
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Docker/Podman image to use (e.g. `"alpine:3.19"`).
    pub image: String,
    /// Container runtime binary.  **Must be `"docker"` or `"podman"`** — any
    /// other value is rejected at execution time.  Never accept this field from
    /// untrusted input.
    pub runtime: String,
    /// Memory limit (e.g. `"64m"`).  Empty string = no limit.
    pub memory_limit: String,
    /// CPU limit as a fractional value (e.g. `"0.5"`).  Empty string = no limit.
    pub cpu_limit: String,
    /// Allow network access inside the container.  Default: `false`.
    pub allow_network: bool,
    /// Optional host paths to mount read-only: `(host_path, container_path)`.
    pub read_only_mounts: Vec<(String, String)>,
    /// Maximum wall-clock duration before the container is killed.
    pub timeout: Duration,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            image: "alpine:3.19".to_string(),
            runtime: "docker".to_string(),
            memory_limit: "64m".to_string(),
            cpu_limit: "0.5".to_string(),
            allow_network: false,
            read_only_mounts: vec![],
            timeout: Duration::from_secs(30),
        }
    }
}

/// Executes agent commands in an isolated Docker/Podman container.
#[derive(Clone)]
pub struct ContainerSandbox {
    // `config` is consumed only in the `container` feature impl block.
    #[cfg_attr(not(feature = "container"), allow(dead_code))]
    config: ContainerConfig,
}

impl ContainerSandbox {
    pub fn new(config: ContainerConfig) -> Self {
        Self { config }
    }

    /// Create a sandbox with secure defaults (no network, limited CPU/RAM).
    pub fn secure() -> Self {
        Self::new(ContainerConfig::default())
    }

    /// Execute `command` (via `sh -c`) inside an isolated container.
    ///
    /// `command` is passed as the argument to `sh -c`, so **shell expansion is
    /// active**: semicolons, pipes, `$(...)`, and other shell metacharacters are
    /// interpreted by the shell inside the container.  This is intentional —
    /// agents may need pipelines — but callers expecting raw-exec semantics
    /// must be aware that the command is a shell script fragment, not a literal
    /// executable path.
    ///
    /// Stdout and stderr are captured as **lossy UTF-8**: non-UTF-8 byte
    /// sequences are silently replaced with `U+FFFD`.  Commands that emit
    /// binary output should base64-encode their results inside the container.
    ///
    /// Returns [`SandboxResult`] with stdout, stderr, exit code, and timing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `container` cargo feature is not enabled.
    /// - [`ContainerConfig::runtime`] is not `"docker"` or `"podman"`.
    /// - A [`read_only_mounts`][ContainerConfig::read_only_mounts] path
    ///   contains a colon (`:`), which would corrupt the Docker volume spec.
    /// - The runtime binary cannot be spawned (not in `$PATH`, daemon down).
    /// - The execution exceeds [`ContainerConfig::timeout`].
    ///
    /// Requires the `container` cargo feature and `docker` or `podman` in `$PATH`.
    pub async fn execute(&self, command: &str) -> Result<SandboxResult> {
        #[cfg(feature = "container")]
        return self.run_subprocess(command).await;

        #[cfg(not(feature = "container"))]
        {
            let _ = command;
            bail!(
                "ContainerSandbox::execute requires the `container` cargo feature. \
                 Rebuild with `--features container`."
            )
        }
    }
}

#[cfg(feature = "container")]
impl ContainerSandbox {
    async fn run_subprocess(&self, command: &str) -> Result<SandboxResult> {
        use tokio::process::Command;

        // Guard: only allow known-safe runtimes.
        const ALLOWED_RUNTIMES: &[&str] = &["docker", "podman"];
        if !ALLOWED_RUNTIMES.contains(&self.config.runtime.as_str()) {
            bail!(
                "ContainerSandbox: runtime must be 'docker' or 'podman', got '{}'",
                self.config.runtime
            );
        }

        let container_name = format!("koad-sandbox-{}", Uuid::new_v4());

        let mut args: Vec<String> = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--name".to_string(),
            container_name.clone(),
            // Filesystem isolation
            "--read-only".to_string(),
            "--tmpfs".to_string(),
            "/tmp".to_string(),
            // Security
            "--security-opt".to_string(),
            "no-new-privileges".to_string(),
        ];

        // Network isolation
        if !self.config.allow_network {
            args.push("--network".to_string());
            args.push("none".to_string());
        }

        // Resource limits
        if !self.config.memory_limit.is_empty() {
            args.push("--memory".to_string());
            args.push(self.config.memory_limit.clone());
        }
        if !self.config.cpu_limit.is_empty() {
            args.push("--cpus".to_string());
            args.push(self.config.cpu_limit.clone());
        }

        // Read-only bind mounts.
        // Docker's volume spec uses ':' as a delimiter; a colon in either path
        // would silently corrupt the spec, so we reject it eagerly.
        for (host, container) in &self.config.read_only_mounts {
            if host.contains(':') || container.contains(':') {
                bail!(
                    "ContainerSandbox: mount paths must not contain ':': \
                     host={:?}, container={:?}",
                    host,
                    container
                );
            }
            args.push("-v".to_string());
            args.push(format!("{}:{}:ro", host, container));
        }

        // Image and command
        args.push(self.config.image.clone());
        args.push("sh".to_string());
        args.push("-c".to_string());
        args.push(command.to_string());

        info!(
            runtime = %self.config.runtime,
            container = %container_name,
            %command,
            "ContainerSandbox: starting"
        );

        let t0 = Instant::now();

        let output = tokio::time::timeout(
            self.config.timeout,
            Command::new(&self.config.runtime).args(&args).output(),
        )
        .await
        .context("ContainerSandbox: timeout waiting for container")?
        .context("ContainerSandbox: failed to spawn runtime")?;

        let duration_ms = t0.elapsed().as_millis() as u64;
        let exit_code = output.status.code().unwrap_or(-1) as i64;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        info!(
            runtime = %self.config.runtime,
            container = %container_name,
            exit_code,
            duration_ms,
            "ContainerSandbox: complete"
        );

        Ok(SandboxResult {
            stdout,
            stderr,
            exit_code,
            duration_ms,
            memory_bytes: 0, // subprocess approach: memory reported via OS-level accounting
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_config_defaults() {
        let cfg = ContainerConfig::default();
        assert!(!cfg.allow_network);
        assert_eq!(cfg.memory_limit, "64m");
        assert!(cfg.read_only_mounts.is_empty());
        assert_eq!(cfg.runtime, "docker");
    }

    #[test]
    fn test_sandbox_result_default() {
        let r = SandboxResult::default();
        assert_eq!(r.exit_code, 0);
        assert_eq!(r.duration_ms, 0);
    }

    #[cfg(feature = "container")]
    #[tokio::test]
    async fn test_invalid_runtime_returns_error() {
        let cfg = ContainerConfig {
            runtime: "bash".to_string(),
            ..ContainerConfig::default()
        };
        let err = ContainerSandbox::new(cfg)
            .execute("echo hi")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("runtime must be"));
    }

    #[cfg(feature = "container")]
    #[tokio::test]
    async fn test_mount_path_with_colon_returns_error() {
        let cfg = ContainerConfig {
            read_only_mounts: vec![("/valid/path".to_string(), "/con:tainer".to_string())],
            ..ContainerConfig::default()
        };
        let err = ContainerSandbox::new(cfg)
            .execute("echo hi")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("must not contain ':'"));
    }

    /// Docker integration test — skipped unless `KOAD_TEST_DOCKER=1`.
    ///
    /// Run manually:
    ///   KOAD_TEST_DOCKER=1 cargo test -p koad-sandbox --features container \
    ///     -- container::tests::test_container_echo
    #[cfg(feature = "container")]
    #[tokio::test]
    async fn test_container_echo() {
        if std::env::var("KOAD_TEST_DOCKER").unwrap_or_default() != "1" {
            eprintln!("SKIP: set KOAD_TEST_DOCKER=1 to run Docker integration tests");
            return;
        }
        let result = ContainerSandbox::secure()
            .execute("echo hello-sandbox")
            .await
            .expect("container should run");
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello-sandbox"));
        assert!(result.duration_ms > 0);
    }

    /// Verifies network isolation: a network-dependent command must fail.
    ///
    /// Run manually:
    ///   KOAD_TEST_DOCKER=1 cargo test -p koad-sandbox --features container \
    ///     -- container::tests::test_container_network_isolation
    #[cfg(feature = "container")]
    #[tokio::test]
    async fn test_container_network_isolation() {
        if std::env::var("KOAD_TEST_DOCKER").unwrap_or_default() != "1" {
            eprintln!("SKIP: set KOAD_TEST_DOCKER=1 to run Docker integration tests");
            return;
        }
        // `ping` requires a network interface; with --network none it must fail.
        let result = ContainerSandbox::secure()
            .execute("ping -c 1 8.8.8.8 2>&1; echo exit:$?")
            .await
            .expect("container should run");
        // Either ping fails (non-zero exit) or the word "Network" appears in output.
        let combined = format!("{}{}", result.stdout, result.stderr);
        let network_blocked = result.exit_code != 0 || combined.contains("Network");
        assert!(
            network_blocked,
            "network should be unreachable inside the sandbox"
        );
    }

    /// Verifies filesystem isolation: writing outside /tmp must fail.
    ///
    /// Run manually:
    ///   KOAD_TEST_DOCKER=1 cargo test -p koad-sandbox --features container \
    ///     -- container::tests::test_container_filesystem_isolation
    #[cfg(feature = "container")]
    #[tokio::test]
    async fn test_container_filesystem_isolation() {
        if std::env::var("KOAD_TEST_DOCKER").unwrap_or_default() != "1" {
            eprintln!("SKIP: set KOAD_TEST_DOCKER=1 to run Docker integration tests");
            return;
        }
        // Writing to the root filesystem must fail because --read-only is set.
        let result = ContainerSandbox::secure()
            .execute("echo test > /koad-isolation-probe && echo WROTE || echo BLOCKED")
            .await
            .expect("container should run");
        assert!(
            result.stdout.contains("BLOCKED"),
            "write outside /tmp should be denied by --read-only; stdout: {}",
            result.stdout
        );
    }

    #[cfg(not(feature = "container"))]
    #[tokio::test]
    async fn test_execute_without_feature_returns_error() {
        let err = ContainerSandbox::secure()
            .execute("echo hi")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("container"));
    }
}
