use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile;

#[test]
fn test_cli_kernel_handshake() {
    let manifest_path = "/home/ideans/.koad-os/Cargo.toml";
    let temp_dir = tempfile::tempdir().unwrap();
    let koad_home = temp_dir.path().to_str().unwrap();

    // 1. Build everything
    let build_status = Command::new("cargo")
        .args(["build", "--manifest-path", manifest_path])
        .status()
        .unwrap();
    assert!(build_status.success());

    // 2. Start Kernel in background
    let mut kernel = Command::new("cargo")
        .env("KOAD_HOME", koad_home)
        .args(["run", "-p", "koad-spine", "--manifest-path", manifest_path])
        .spawn()
        .unwrap();

    // 3. Wait for socket
    thread::sleep(Duration::from_secs(10));

    // 4. Run CLI command
    let output = Command::new("cargo")
        .env("KOAD_HOME", koad_home)
        .args([
            "run",
            "-p",
            "koad-cli",
            "--manifest-path",
            manifest_path,
            "--",
            "run",
            "ping",
        ])
        .output()
        .unwrap();

    // 5. Cleanup
    let _ = kernel.kill();
    let _ = Command::new("pkill").arg("redis-server").status();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);
    assert!(stdout.contains("executed successfully"));
}
