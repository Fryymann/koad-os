#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_cli_kernel_handshake() {
        // This test requires redis-server to be available but kspine handles it.
        // We run the bins directly from the target directory.
        let manifest_path = "/home/ideans/.koad-os/Cargo.toml";
        
        // 1. Build everything
        let build_status = Command::new("cargo")
            .args(["build", "--manifest-path", manifest_path])
            .status()
            .unwrap();
        assert!(build_status.success());

        // 2. Start Kernel in background
        let mut kernel = Command::new("cargo")
            .args(["run", "-p", "koad-spine", "--manifest-path", manifest_path])
            .spawn()
            .unwrap();

        // 3. Wait for socket
        thread::sleep(Duration::from_secs(5));

        // 4. Run CLI command
        let output = Command::new("cargo")
            .args(["run", "-p", "koad-cli", "--manifest-path", manifest_path, "--", "run", "ping"])
            .output()
            .unwrap();

        // 5. Cleanup
        let _ = kernel.kill();
        let _ = Command::new("pkill").arg("redis-server").status();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("executed successfully"));
    }
}
