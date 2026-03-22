use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::{Pid, System};

pub struct PidGuard {
    pub path: PathBuf,
}

impl PidGuard {
    /// Create a new PidGuard. If a process is already running, returns an error.
    pub fn new(path: PathBuf) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read existing PID file at {:?}", path))?;

            if let Ok(old_pid_val) = content.trim().parse::<u32>() {
                let mut sys = System::new_all();
                sys.refresh_all();

                if sys.process(Pid::from(old_pid_val as usize)).is_some() {
                    anyhow::bail!(
                        "Process already running with PID {}. If this is a ghost, delete {:?}",
                        old_pid_val,
                        path
                    );
                } else {
                    // Stale PID file, safe to overwrite
                    let _ = fs::remove_file(&path);
                }
            }
        }

        let current_pid = std::process::id();
        fs::write(&path, current_pid.to_string())
            .with_context(|| format!("Failed to write PID file to {:?}", path))?;

        Ok(Self { path })
    }
}

impl Drop for PidGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Returns true if the PID written in `path` belongs to a currently running process.
/// Used internally for single-instance enforcement.
fn pid_file_is_live(path: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(pid) = content.trim().parse::<u32>() {
            let mut sys = System::new_all();
            sys.refresh_all();
            return sys.process(Pid::from(pid as usize)).is_some();
        }
    }
    false
}

pub fn find_ghosts(home: &Path) -> Vec<(u32, String)> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut ghosts = Vec::new();

    // Check common PID files
    let pid_files = vec!["redis.pid", "kcitadel.pid", "kgateway.pid", "koad-asm.pid"];
    for pf in pid_files {
        let pid_file = home.join(pf);
        if pid_file.exists() {
            if let Ok(pid_str) = fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if sys.process(Pid::from(pid as usize)).is_none() {
                        ghosts.push((pid, format!("Stale {} file", pf)));
                    }
                }
            }
        }
    }
    ghosts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn new_creates_pid_file_with_current_pid() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pid");

        let _guard = PidGuard::new(path.clone()).expect("PidGuard::new should succeed");

        assert!(path.exists(), "PID file should be created on disk");
        let content = fs::read_to_string(&path).unwrap();
        let written_pid: u32 = content.trim().parse().expect("PID file should contain a valid u32");
        assert_eq!(written_pid, std::process::id(), "PID file should contain the current process PID");
    }

    #[test]
    fn drop_removes_pid_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("drop_test.pid");

        let guard = PidGuard::new(path.clone()).unwrap();
        assert!(path.exists(), "PID file should exist before drop");

        drop(guard);
        assert!(!path.exists(), "PID file should be removed when guard is dropped");
    }

    #[test]
    fn new_fails_when_referenced_process_is_alive() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("live.pid");

        // Write the current test process's own PID — guaranteed to be running
        fs::write(&path, std::process::id().to_string()).unwrap();

        let result = PidGuard::new(path.clone());
        assert!(result.is_err(), "Should fail when PID file refers to a running process");
        assert!(
            result.unwrap_err().to_string().contains("Process already running"),
            "Error message should describe the conflict"
        );
        // The original PID file must not be overwritten on failure
        assert!(path.exists(), "PID file should be untouched after a failed guard creation");
    }

    #[test]
    fn new_overwrites_stale_pid_file_and_succeeds() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("stale.pid");

        // This PID value is far beyond Linux's max_pid (4194304) and will never be live
        fs::write(&path, "999999999").unwrap();

        let guard = PidGuard::new(path.clone());
        assert!(guard.is_ok(), "Should succeed when PID file contains a stale (dead) PID");

        let content = fs::read_to_string(&path).unwrap();
        let written_pid: u32 = content.trim().parse().unwrap();
        assert_eq!(written_pid, std::process::id(), "File should now contain the current process PID");
    }

    #[test]
    fn new_creates_file_when_no_existing_pid_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("fresh.pid");
        assert!(!path.exists(), "Precondition: no PID file should exist");

        let guard = PidGuard::new(path.clone());
        assert!(guard.is_ok(), "Should succeed when no PID file exists");
        assert!(path.exists());
    }

    #[test]
    fn find_ghosts_returns_empty_for_clean_directory() {
        let dir = tempdir().unwrap();
        let ghosts = find_ghosts(dir.path());
        assert!(ghosts.is_empty(), "No ghosts expected in a directory with no PID files");
    }

    #[test]
    fn find_ghosts_detects_stale_kcitadel_pid() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("kcitadel.pid"), "999999999").unwrap();

        let ghosts = find_ghosts(dir.path());
        assert_eq!(ghosts.len(), 1, "Expected exactly one ghost");
        assert_eq!(ghosts[0].0, 999999999u32, "Ghost PID should match the file contents");
        assert!(
            ghosts[0].1.contains("kcitadel.pid"),
            "Ghost description should name the PID file"
        );
    }

    #[test]
    fn find_ghosts_ignores_live_process() {
        let dir = tempdir().unwrap();
        // Write the current process's PID into a monitored file — should NOT be a ghost
        fs::write(
            dir.path().join("redis.pid"),
            std::process::id().to_string(),
        )
        .unwrap();

        let ghosts = find_ghosts(dir.path());
        assert!(ghosts.is_empty(), "A running process should not be reported as a ghost");
    }
}
