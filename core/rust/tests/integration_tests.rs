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
        .stdout(predicate::str::contains("Identity:"));
}

#[test]
fn test_remember_fact() {
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("remember").arg("fact").arg("Test fact").arg("--tags").arg("test");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Memory updated."));
    
    // Check if we can query it
    let mut cmd = Command::cargo_bin("koad").unwrap();
    cmd.arg("query").arg("Test fact");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test fact"));
}
