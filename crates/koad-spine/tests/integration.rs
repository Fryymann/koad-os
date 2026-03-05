use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile;

#[test]
fn test_cli_kernel_handshake() {
    let koad_home =
        std::env::var("KOAD_HOME").unwrap_or_else(|_| "/home/ideans/.koad-os".to_string());
    let manifest_path = format!("{}/Cargo.toml", koad_home);
    let temp_dir = tempfile::tempdir().unwrap();
    let koad_home_str = temp_dir.path().to_str().unwrap();

    // 1. Build everything
    let build_status = Command::new("cargo")
        .args(["build", "--manifest-path", manifest_path])
        .status()
        .unwrap();
    assert!(build_status.success());

    // 2. Start Kernel in background
    let mut kernel = Command::new("cargo")
        .env("KOAD_HOME", koad_home_str)
        .args(["run", "-p", "koad-spine", "--manifest-path", manifest_path])
        .spawn()
        .unwrap();

    // 3. Wait for socket
    thread::sleep(Duration::from_secs(10));

    // 4. Run CLI command
    let output = Command::new("cargo")
        .env("KOAD_HOME", koad_home_str)
        .args([
            "run",
            "-p",
            "koad-cli",
            "--manifest-path",
            manifest_path,
            "--",
            "status",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    // 5. Cleanup
    kernel.kill().expect("Failed to kill kernel");
    let _ = Command::new("pkill").arg("redis-server").status(); // redis might already be dead, don't fail here

    assert!(stdout.contains("Neural Link") || stderr.contains("Spine Backbone"));
}
