//! Tests for the Windows-bridge launchers in scripts/.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_script(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts")
        .join(name)
}

/// A fake KOAD_HOME with both launchers in bin/ and a stand-in koad-os-mcp
/// that prints the environment it was given.
fn fake_home() -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(home.path().join("config/identities")).unwrap();
    for s in ["koad-wsl-env", "koad-mcp-stdio"] {
        fs::copy(repo_script(s), bin.join(s)).unwrap();
    }
    let fake = bin.join("koad-os-mcp");
    fs::write(&fake, "#!/usr/bin/env bash\nenv | sort\n").unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    home
}

/// Mimic `wsl.exe -e`: minimal environment, no KOAD_HOME, no KoadOS PATH.
/// `extra_env` simulates whatever stale/foreign values the inherited
/// environment might already carry.
fn run_bare(home: &Path, args: &[&str], extra_env: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new("bash");
    cmd.arg(home.join("bin/koad-wsl-env"))
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin");
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd.output().unwrap()
}

#[test]
fn wsl_env_sets_koad_home_path_and_user() {
    let home = fake_home();
    let out = run_bare(home.path(), &["env"], &[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env = String::from_utf8(out.stdout).unwrap();
    let h = home.path().display();
    assert!(env.lines().any(|l| l == format!("KOAD_HOME={h}")), "{env}");
    assert!(
        env.lines().any(|l| l == format!("KOADOS_HOME={h}")),
        "{env}"
    );
    assert!(
        env.lines()
            .any(|l| l.starts_with(&format!("PATH={h}/bin:"))),
        "{env}"
    );
    assert!(
        env.lines().any(|l| l.starts_with("USER=") && l.len() > 5),
        "{env}"
    );
}

#[test]
fn mcp_stdio_starts_the_server_for_a_known_agent() {
    let home = fake_home();
    fs::write(
        home.path().join("config/identities/clyde.toml"),
        "[identities.clyde]\n",
    )
    .unwrap();
    // Simulate an inherited environment carrying stale/foreign values: a
    // leftover AGENT_PARTITION, a CASS_URL that should pass through
    // unchanged, and a wrong KOAD_HOME that the script's own location must
    // win over.
    let out = run_bare(
        home.path(),
        &["koad-mcp-stdio", "Clyde"],
        &[
            ("AGENT_PARTITION", "stale"),
            ("CASS_URL", "http://x:1"),
            ("KOAD_HOME", "/wrong"),
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env = String::from_utf8(out.stdout).unwrap();
    let h = home.path().display();
    for want in [
        "MCP_TRANSPORT=stdio",
        "AGENT_NAME=clyde",
        "MCP_MODE=read_write",
        "CASS_URL=http://x:1",
    ] {
        assert!(env.lines().any(|l| l == want), "missing {want}: {env}");
    }
    assert!(
        !env.contains("AGENT_PARTITION="),
        "partition must be derived by koad-os-mcp"
    );
    assert!(
        env.lines().any(|l| l == format!("KOAD_HOME={h}")),
        "script location must win over inherited KOAD_HOME: {env}"
    );
}

#[test]
fn mcp_stdio_refuses_an_unknown_agent_without_touching_stdout() {
    let home = fake_home();
    let out = run_bare(home.path(), &["koad-mcp-stdio", "nobody"], &[]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty(), "stdout must stay clean for MCP");
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown agent"));
}

#[test]
fn mcp_stdio_refuses_a_path_traversal_agent_name() {
    let home = fake_home();
    fs::write(
        home.path().join("config/identities/clyde.toml"),
        "[identities.clyde]\n",
    )
    .unwrap();
    let out = run_bare(home.path(), &["koad-mcp-stdio", "../clyde"], &[]);
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(64), "{:?}", out);
    assert!(out.stdout.is_empty(), "stdout must stay clean for MCP");
}
