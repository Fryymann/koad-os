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

/// `\\wsl.localhost\<distro>\…` form of a Linux path.
pub fn wsl_unc(distro: &str, linux_path: &str) -> String {
    format!(
        r"\\wsl.localhost\{}{}",
        distro,
        linux_path.replace('/', r"\")
    )
}

/// Render the Windows-body anchor. `cass_packet` is `None` when CASS was
/// unreachable.
pub fn render_windows_anchor(
    id: &AnchorIdentity,
    timestamp: &str,
    koad_home: &str,
    vault_unc: &str,
    cass_packet: Option<&str>,
) -> String {
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
         - **KoadOS CLI:** runs only in WSL. From PowerShell, when truly needed: \
         `wsl.exe -e {koad_home}/bin/koad-wsl-env koad <command>`.\n\
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

/// Ask CASS for the agent's hydration packet. `None` when CASS is unreachable.
pub async fn fetch_cass_packet(
    cass_addr: &str,
    agent: &str,
    project_root: &str,
    timeout: Duration,
) -> Option<String> {
    // Bounded: on WSL mirrored networking a down CASS drops packets, and an
    // unbounded connect would stall the SessionStart hook.
    let endpoint = Endpoint::from_shared(cass_addr.to_string())
        .ok()?
        .connect_timeout(timeout)
        .timeout(timeout);
    let channel = tokio::time::timeout(timeout, endpoint.connect())
        .await
        .ok()?
        .ok()?;
    let mut client = HydrationServiceClient::new(channel);
    let req = tonic::Request::new(HydrationRequest {
        agent_name: agent.to_string(),
        project_root: project_root.to_string(),
        level: WorkspaceLevel::LevelUnspecified as i32,
        token_budget: 4000,
        task_id: String::new(),
    });
    client
        .hydrate(req)
        .await
        .ok()
        .map(|r| r.into_inner().markdown_packet)
}

fn expand_home(path: &str) -> String {
    match path.strip_prefix("~/") {
        Some(rest) => format!(
            "{}/{}",
            dirs::home_dir().unwrap_or_default().display(),
            rest
        ),
        None => path.to_string(),
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
    let vault = identity
        .vault
        .clone()
        .unwrap_or_else(|| format!("{koad_home}/agents/KAPVs/{key}"));
    let distro = std::env::var("WSL_DISTRO_NAME").unwrap_or_else(|_| "Ubuntu".to_string());
    let vault_unc = wsl_unc(&distro, &expand_home(&vault));
    let packet = fetch_cass_packet(
        &config.network.cass_grpc_addr,
        &identity.name,
        &koad_home,
        Duration::from_secs(5),
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
    fn windows_anchor_has_identity_body_and_memory_tools() {
        let a = render_windows_anchor(
            &clyde(),
            "T",
            "/home/ideans/.citadel-jupiter",
            VAULT,
            Some("## Ⅰ. Episodes\n- x\n"),
        );
        assert!(a.starts_with("# KoadOS Agent Identity Anchor\n"), "{a}");
        assert!(a.contains("Name: Clyde"));
        assert!(a.contains("Body: windows"));
        assert!(a.contains("memory.search_semantic"));
        assert!(a.contains("memory.commit"));
        assert!(a.contains(VAULT));
        assert!(a.contains("wsl.exe -e /home/ideans/.citadel-jupiter/bin/koad-wsl-env koad"));
        assert!(a.contains("## 🧠 Temporal Context Packet (CASS)\n## Ⅰ. Episodes"));
    }

    /// The WSL-body session instructions do not apply on Windows.
    #[test]
    fn windows_anchor_has_no_wsl_session_instructions() {
        let a = render_windows_anchor(&clyde(), "T", "/k", VAULT, Some("p"));
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
    fn offline_cass_is_stated_not_hidden() {
        let a = render_windows_anchor(&clyde(), "T", "/k", VAULT, None);
        assert!(a.contains("Memory: offline (CASS unreachable)"), "{a}");
        assert!(!a.contains("Temporal Context Packet"));
    }
}
