//! `koad-agent anchor`: print an identity anchor for another body.
//!
//! The Windows body bridge runs this from a Claude Code for Windows
//! SessionStart hook through `wsl.exe`; its stdout becomes session context.
//! It writes no files and mints no Citadel session: memory goes to CASS
//! directly over MCP.

use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_proto::cass::v1::hydration_service_client::HydrationServiceClient;
use koad_proto::cass::v1::HydrationRequest;
use koad_proto::citadel::v5::WorkspaceLevel;
use std::time::Duration;
use tonic::transport::Endpoint;

/// Where the anchored session runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum AnchorBody {
    /// Claude Code (or similar) on native Windows, bridged to WSL.
    Windows,
}

/// Identity fields shown in the anchor.
pub struct AnchorIdentity<'a> {
    pub name: &'a str,
    pub role: &'a str,
    pub rank: &'a str,
    pub bio: &'a str,
}

/// Timeout for anchor's CASS connect. Matches boot's `BOOT_SERVICE_TIMEOUT`
/// (crates/koad-agent/src/commands/boot.rs): bounded so a down CASS prints
/// the offline line instead of stalling the Windows SessionStart hook.
const ANCHOR_CASS_TIMEOUT: Duration = Duration::from_secs(3);

/// `\\wsl.localhost\<distro>\…` form of a Linux path.
pub fn wsl_unc(distro: &str, linux_path: &str) -> String {
    let win_path = linux_path.replace('/', r"\");
    if linux_path.starts_with('/') {
        format!(r"\\wsl.localhost\{distro}{win_path}")
    } else {
        format!(r"\\wsl.localhost\{distro}\{win_path}")
    }
}

/// Render the Windows-body anchor. `cass_packet` is `None` when CASS was
/// unreachable.
pub fn render_windows_anchor(
    id: &AnchorIdentity,
    timestamp: &str,
    koad_home: &str,
    distro: &str,
    vault_unc: &str,
    cass_packet: Option<&str>,
) -> String {
    // Leading `//`: Git Bash leaves it alone instead of rewriting a POSIX path
    // into `C:/Program Files/Git/...`; PowerShell, cmd and WSL accept it too.
    let env_wrapper = format!("//{}/bin/koad-wsl-env", koad_home.trim_start_matches('/'));
    let mut s = format!(
        "# KoadOS Agent Identity Anchor\n\
         Generated At: {timestamp}\n\
         Body: windows (Claude Code for Windows, bridged to Citadel Jupiter in WSL)\n\n\
         ## Identity\nName: {}\nRole: {}\nRank: {}\n\n## Bio\n{}\n",
        id.name, id.role, id.rank, id.bio
    );
    s.push_str(&format!(
        "\n## Working Environment (Windows body)\n\
         - **Memory:** use the `citadel-memory` MCP tools. Recall with \
         `memory.search_semantic` / `memory.recall` before rebuilding knowledge; store durable \
         lessons with `memory.commit` and verify by recall. Also `memory.list_topics`, \
         `intel.get`, `status.citadel`.\n\
         - **KoadOS CLI:** runs only in WSL. From PowerShell, cmd or Git Bash, when truly needed: \
         `wsl.exe -d {distro} -e {env_wrapper} koad <command>` (the leading `//` is deliberate: \
         it stops Git Bash path conversion; from Git Bash, prefix `MSYS_NO_PATHCONV=1` when \
         passing Linux paths as arguments).\n\
         - **Vault:** `{vault_unc}`\n\
         - **Handoffs:** inbox files in the Citadel home in WSL (see the `koad-inbox` skill).\n"
    ));
    match cass_packet {
        None => s.push_str(
            "\nMemory: offline (CASS unreachable). Memory tools will return errors until CASS is back.\n",
        ),
        Some(p) if !p.is_empty() => {
            s.push_str("\n## 🧠 Temporal Context Packet (CASS)\n");
            s.push_str(p);
        }
        Some(_) => {}
    }
    s
}

/// Ask CASS for the agent's hydration packet. `None` when CASS is
/// unreachable; each failure writes one reasoned line to stderr so the
/// Windows SessionStart hook (whose stdout becomes session context) stays
/// diagnosable without polluting the anchor itself.
pub async fn fetch_cass_packet(
    cass_addr: &str,
    agent: &str,
    project_root: &str,
    timeout: Duration,
) -> Option<String> {
    // Bounded: on WSL mirrored networking a down CASS drops packets, and an
    // unbounded connect would stall the SessionStart hook.
    let endpoint = match Endpoint::from_shared(cass_addr.to_string()) {
        Ok(e) => e.connect_timeout(timeout).timeout(timeout),
        Err(e) => {
            eprintln!("koad-agent anchor: CASS unreachable at {cass_addr}: invalid endpoint: {e}");
            return None;
        }
    };
    let channel = match tokio::time::timeout(timeout, endpoint.connect()).await {
        Ok(Ok(channel)) => channel,
        Ok(Err(e)) => {
            eprintln!("koad-agent anchor: CASS unreachable at {cass_addr}: {e}");
            return None;
        }
        Err(_) => {
            eprintln!(
                "koad-agent anchor: CASS unreachable at {cass_addr}: connect timed out after {timeout:?}"
            );
            return None;
        }
    };
    let mut client = HydrationServiceClient::new(channel);
    let req = tonic::Request::new(HydrationRequest {
        agent_name: agent.to_string(),
        project_root: project_root.to_string(),
        level: WorkspaceLevel::LevelUnspecified as i32,
        token_budget: 4000,
        task_id: String::new(),
    });
    match client.hydrate(req).await {
        Ok(resp) => Some(resp.into_inner().markdown_packet),
        Err(e) => {
            eprintln!("koad-agent anchor: CASS unreachable at {cass_addr}: hydrate failed: {e}");
            None
        }
    }
}

/// Print the identity anchor for `agent` running in `body`.
pub async fn handle_anchor(config: &KoadConfig, agent: &str, body: AnchorBody) -> Result<()> {
    let AnchorBody::Windows = body;
    let key = agent.to_lowercase();
    let identity = config
        .identities
        .get(&key)
        .with_context(|| format!("Unknown agent '{agent}'"))?;
    let koad_home = config.home.to_string_lossy().to_string();

    // Resolve the vault exactly as boot does (crates/koad-core/src/config.rs
    // resolve_vault_uri: KOAD_VAULT_URI env, then identity vault_uri, then
    // identity vault, then agent_dir()), but without requiring the path to
    // exist — this only needs to render a UNC path, not open the vault.
    let vault_uri = config
        .resolve_vault_uri(agent)
        .with_context(|| format!("Could not resolve vault URI for agent '{agent}'"))?;
    let vault_path = config.resolve_vault_path_unchecked(&vault_uri)?;

    let distro = std::env::var("WSL_DISTRO_NAME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Ubuntu".to_string());
    let vault_unc = wsl_unc(&distro, &vault_path.to_string_lossy());
    let packet = fetch_cass_packet(
        &config.network.cass_grpc_addr,
        &key,
        &koad_home,
        ANCHOR_CASS_TIMEOUT,
    )
    .await;
    let id = AnchorIdentity {
        name: &identity.name,
        role: &identity.role,
        rank: &identity.rank,
        bio: &identity.bio,
    };
    print!(
        "{}",
        render_windows_anchor(
            &id,
            &chrono::Utc::now().to_rfc3339(),
            &koad_home,
            &distro,
            &vault_unc,
            packet.as_deref()
        )
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clyde() -> AnchorIdentity<'static> {
        AnchorIdentity {
            name: "Clyde",
            role: "Citadel Officer and Implementation Engineer",
            rank: "Officer",
            bio: "Sovereign KoadOS Agent.",
        }
    }

    const VAULT: &str = r"\\wsl.localhost\Ubuntu\home\ideans\.citadel-jupiter\agents\KAPVs\clyde";

    #[test]
    fn unc_path_for_a_linux_path() {
        assert_eq!(
            wsl_unc("Ubuntu", "/home/ideans/.citadel-jupiter/agents/KAPVs/clyde"),
            VAULT
        );
    }

    #[test]
    fn unc_path_keeps_a_trailing_slash() {
        assert_eq!(
            wsl_unc("Ubuntu", "/home/ideans/"),
            r"\\wsl.localhost\Ubuntu\home\ideans\"
        );
    }

    /// A relative path has no leading `/` to become the separator after the
    /// distro name, so `wsl_unc` must insert one itself.
    #[test]
    fn unc_path_for_a_relative_path_gets_a_separator() {
        assert_eq!(
            wsl_unc("Ubuntu", "relative/path"),
            r"\\wsl.localhost\Ubuntu\relative\path"
        );
    }

    #[test]
    fn windows_anchor_has_identity_body_and_memory_tools() {
        let a = render_windows_anchor(
            &clyde(),
            "T",
            "/home/ideans/.citadel-jupiter",
            "Ubuntu",
            VAULT,
            Some("## Ⅰ. Episodes\n- x\n"),
        );
        assert!(a.starts_with("# KoadOS Agent Identity Anchor\n"), "{a}");
        assert!(a.contains("Name: Clyde"));
        assert!(a.contains("Body: windows"));
        assert!(a.contains("memory.search_semantic"));
        assert!(a.contains("memory.commit"));
        assert!(a.contains(VAULT));
        assert!(a.contains(
            "`wsl.exe -d Ubuntu -e //home/ideans/.citadel-jupiter/bin/koad-wsl-env koad <command>`"
        ));
        assert!(a.contains("leading `//` is deliberate"));
        assert!(a.contains("## 🧠 Temporal Context Packet (CASS)\n## Ⅰ. Episodes"));
    }

    /// The WSL-body session instructions do not apply on Windows.
    #[test]
    fn windows_anchor_has_no_wsl_session_instructions() {
        let a = render_windows_anchor(&clyde(), "T", "/k", "Ubuntu", VAULT, Some("p"));
        for wsl_only in [
            "agent-boot",
            "current.env",
            "koad system heartbeat",
            "koad-functions.sh",
        ] {
            assert!(!a.contains(wsl_only), "unexpected {wsl_only:?} in {a}");
        }
    }

    #[test]
    fn cli_line_uses_the_given_distro_and_a_double_slash_path() {
        let a = render_windows_anchor(&clyde(), "T", "/k", "Ubuntu-24.04", VAULT, Some("p"));
        assert!(
            a.contains("wsl.exe -d Ubuntu-24.04 -e //k/bin/koad-wsl-env koad"),
            "{a}"
        );
        assert!(!a.contains("///"), "{a}");
    }

    #[test]
    fn offline_cass_is_stated_not_hidden() {
        let a = render_windows_anchor(&clyde(), "T", "/k", "Ubuntu", VAULT, None);
        assert!(a.contains("Memory: offline (CASS unreachable)"), "{a}");
        assert!(!a.contains("Temporal Context Packet"));
    }

    /// `Some("")` means CASS answered with an empty packet — distinct from
    /// `None` (unreachable). Neither the offline line nor a packet section
    /// belongs in the anchor for this case.
    #[test]
    fn empty_cass_packet_prints_neither_offline_nor_packet_section() {
        let a = render_windows_anchor(&clyde(), "T", "/k", "Ubuntu", VAULT, Some(""));
        assert!(!a.contains("Memory: offline"), "{a}");
        assert!(!a.contains("Temporal Context Packet"), "{a}");
    }

    /// An unparseable CASS address must fail before any network I/O, so this
    /// returns well within the timeout rather than waiting it out.
    #[tokio::test]
    async fn fetch_cass_packet_with_invalid_uri_returns_none_quickly() {
        let start = std::time::Instant::now();
        let result =
            fetch_cass_packet("not a uri", "agent", "/root", Duration::from_millis(500)).await;
        assert!(result.is_none());
        assert!(
            start.elapsed() < Duration::from_millis(500),
            "invalid URI should fail fast, took {:?}",
            start.elapsed()
        );
    }
}
