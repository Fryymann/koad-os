use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_diagnostic_basic() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("diagnostic");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--- KoadOS Diagnostic ---"))
        .stdout(predicate::str::contains("Status: OPERATIONAL"));
}

#[test]
fn test_diagnostic_full() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("diagnostic").arg("--full");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--- Full Integration Check ---"))
        .stdout(predicate::str::contains("Bridge skill gemini/remember.py found"))
        .stdout(predicate::str::contains("--- Tool Availability ---"))
        .stdout(predicate::str::contains("[PASS] git is available"));
}

#[test]
fn test_diagnostic_degraded() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.env("KOAD_HOME", home);
    cmd.arg("diagnostic");
    
    // It should fail for koad.json but still run
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[FAIL] koad.json is missing"))
        .stdout(predicate::str::contains("Status: DEGRADED"));
}

#[test]
fn test_whoami() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("whoami");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Persona:"));
}

#[test]
fn test_spec_cycle() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("spec").arg("set").arg("Testing Spec").arg("--status").arg("In-Test");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("spec").arg("read");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Testing Spec"))
        .stdout(predicate::str::contains("In-Test"));
}

#[test]
fn test_query_limits() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("query").arg("").arg("--limit").arg("2");
    cmd.assert().success();
}

#[test]
fn test_boot_basic() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("boot");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<koad_boot>"))
        .stdout(predicate::str::contains("Identity:"));
}

#[test]
fn test_remember_cycle() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("remember").arg("fact").arg("Integrations are fun").arg("--tags").arg("test-tag");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Memory updated."));

    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("query").arg("Integrations are fun");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fact"))
        .stdout(predicate::str::contains("Integrations are fun"));
}

#[test]
fn test_guide_list() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("guide");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--- KoadOS Developer & Onboarding Guides ---"));
}

#[test]
fn test_guide_topic() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("guide").arg("onboarding");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("# KoadOS: Your First 15 Minutes"));
}

#[test]
fn test_serve_lifecycle() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    
    // Create necessary koad.json
    let config = r#"{
        "version": "2.4",
        "identity": {"name": "Test", "role": "Admin", "bio": "Test"},
        "preferences": {"languages": [], "booster_enabled": true, "style": "test", "principles": []},
        "drivers": {},
        "notion": {"mcp": false, "index": {}}
    }"#;
    std::fs::write(home.join("koad.json"), config).unwrap();
    std::fs::create_dir_all(home.join("bin")).unwrap();
    
    // Mock the daemon binary - just a sleep script
    let daemon_mock = home.join("bin/koad-daemon");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(&daemon_mock, "#!/bin/sh\nsleep 1000").unwrap();
        let mut perms = std::fs::metadata(&daemon_mock).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&daemon_mock, perms).unwrap();
    }

    // 1. Start Serve
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.env("KOAD_HOME", home);
    cmd.arg("serve");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[PASS] Daemon launched"));

    // Give it a moment to spawn
    std::thread::sleep(std::time::Duration::from_millis(500));

    assert!(home.join("daemon.pid").exists());

    // 2. Stop Serve
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.env("KOAD_HOME", home);
    cmd.arg("serve").arg("--stop");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[PASS] Daemon stopped"));

    assert!(!home.join("daemon.pid").exists());
}

#[test]
fn test_spine_connectivity() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    
    // 1. Setup minimal koad.json
    let config = r#"{
        "version": "2.4",
        "identity": {"name": "SpineTest", "role": "Admin", "bio": "Test"},
        "preferences": {"languages": [], "booster_enabled": true, "style": "test", "principles": []},
        "drivers": {},
        "notion": {"mcp": false, "index": {}}
    }"#;
    std::fs::write(home.join("koad.json"), config).unwrap();
    std::fs::create_dir_all(home.join("bin")).unwrap();
    
    // 2. Link actual binaries to sandbox bin/
    let koad_exe = assert_cmd::cargo::cargo_bin("koad");
    let daemon_exe = assert_cmd::cargo::cargo_bin("koad-daemon");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&daemon_exe, home.join("bin/koad-daemon")).unwrap();
    }

    // 3. Start Hub
    let mut hub_cmd = Command::cargo_bin("koad").unwrap();
    hub_cmd.env("KOAD_HOME", home);
    hub_cmd.arg("host").arg("--port").arg("8081"); // Use non-standard port for testing
    hub_cmd.assert().success();

    // 4. Start Daemon
    let mut serve_cmd = Command::cargo_bin("koad").unwrap();
    serve_cmd.env("KOAD_HOME", home);
    serve_cmd.env("KOAD_HUB_URL", "ws://localhost:8081/ws");
    serve_cmd.arg("serve");
    serve_cmd.assert().success();

    // 5. Verify basic HTTP availability first
    std::thread::sleep(std::time::Duration::from_secs(3));
    let mut check_cmd = Command::cargo_bin("koad").unwrap();
    check_cmd.arg("diagnostic");
    check_cmd.env("KOAD_HOME", home);
    check_cmd.assert().success();

    // Verify port 8081 is actually listening via curl
    let output = std::process::Command::new("curl").arg("-I").arg("http://localhost:8081").output().unwrap();
    println!("HTTP Check Status: {}", output.status);

    // 6. Verify WebSocket Flow (Topic: metrics)
    use tungstenite::{connect, Message};
    let url = "ws://localhost:8081/ws/metrics";
    
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        // Use a shorter timeout for the connection attempt
        let (mut socket, _) = connect(url)?;
        
        // Set a read timeout so we don't hang forever
        if let tungstenite::stream::MaybeTlsStream::Plain(s) = socket.get_mut() {
            let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(5)));
        }

        // Wait for metrics message
        let msg = socket.read()?;
        if let Message::Text(text) = msg {
            println!("Received Metrics: {}", text);
            if text.contains("cpu") && text.contains("mem") {
                return Ok(());
            }
        }
        Err("Invalid message format".into())
    })();

    // Cleanup before assertion
    let _ = Command::cargo_bin("koad").unwrap().env("KOAD_HOME", home).arg("host").arg("--stop").ok();
    let _ = Command::cargo_bin("koad").unwrap().env("KOAD_HOME", home).arg("serve").arg("--stop").ok();

    if result.is_err() {
        println!("--- Hub Log ---");
        if let Ok(l) = std::fs::read_to_string(home.join("hub.log")) { println!("{}", l); }
        println!("--- Daemon Log ---");
        if let Ok(l) = std::fs::read_to_string(home.join("daemon.log")) { println!("{}", l); }
    }

    assert!(result.is_ok(), "Spine failed to broadcast metrics over WebSocket: {:?}", result.err());
}
