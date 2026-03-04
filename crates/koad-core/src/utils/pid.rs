use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use sysinfo::{Pid, System};

pub struct PidGuard {
    path: PathBuf,
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
