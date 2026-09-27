//! Binary-level tests for the stdio transport.

use std::io::Write;
use std::process::{Command, Stdio};

/// Regression guard: in stdio mode tracing wrote to stdout, interleaving log
/// lines with JSON-RPC. Every stdout line must be a JSON-RPC response, and the
/// partition must be derived from AGENT_NAME when AGENT_PARTITION is unset.
#[test]
fn stdio_stdout_carries_only_json_rpc() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_koad-os-mcp"))
        .env("MCP_TRANSPORT", "stdio")
        .env("AGENT_NAME", "clyde")
        .env_remove("AGENT_PARTITION")
        .env("CASS_URL", "http://127.0.0.1:9")
        .env("RUST_LOG", "debug")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn koad-os-mcp");
    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-11-25"}}}}"#).unwrap();
        writeln!(stdin).unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{{}}}}"#
        )
        .unwrap();
    }
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    let ids: Vec<i64> = stdout
        .lines()
        .map(|l| {
            serde_json::from_str::<serde_json::Value>(l)
                .unwrap_or_else(|e| panic!("non-JSON stdout line {l:?}: {e}"))["id"]
                .as_i64()
                .unwrap()
        })
        .collect();
    assert_eq!(ids, vec![1, 2]);

    let stderr = String::from_utf8_lossy(&out.stderr);
    let expected = koad_core::utils::partition::partition_key("clyde");
    assert!(
        stderr.contains(&expected),
        "partition not derived; stderr: {stderr}"
    );
}

/// Regression guard: with CASS silent (TCP accepted, never answered) a tool
/// call hung until the client gave up. It must fail fast with `isError`.
#[test]
fn tool_call_fails_fast_when_cass_does_not_answer() {
    let silent = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", silent.local_addr().unwrap());
    let started = std::time::Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_koad-os-mcp"))
        .env("MCP_TRANSPORT", "stdio")
        .env("AGENT_NAME", "clyde")
        .env_remove("AGENT_PARTITION")
        .env("CASS_URL", &url)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn koad-os-mcp");
    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-11-25"}}}}"#).unwrap();
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"memory.search_semantic","arguments":{{"query":"x","limit":1}}}}}}"#).unwrap();
    }
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();
    assert!(
        started.elapsed() < std::time::Duration::from_secs(15),
        "took {:?}",
        started.elapsed()
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    let reply: serde_json::Value = stdout
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .find(|v| v["id"] == 2)
        .expect("no response to the tool call");
    assert_eq!(reply["result"]["isError"], true, "{reply}");
    assert!(
        reply["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("CASS unreachable"),
        "{reply}"
    );
    drop(silent);
}
