//! `koad-agent anchor`: print an identity anchor for another body.
//!
//! The Windows body bridge runs this from a Claude Code for Windows
//! SessionStart hook through `wsl.exe`; its stdout becomes session context.
//! It mints no Citadel session: memory goes to CASS directly over MCP. The
//! only file it touches is `$KOAD_HOME/logs/anchor.log`, when hydration fails.

use super::self_anchor::{
    fetch_cass_packet, latest_journal, log_cass_miss, read_self, render_self_section, CassMiss,
    CASS_CONNECT_TIMEOUT, CASS_HYDRATE_TIMEOUT,
};
use anyhow::{Context, Result};
use koad_core::config::KoadConfig;

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
    let win_path = linux_path.replace('/', r"\");
    if linux_path.starts_with('/') {
        format!(r"\\wsl.localhost\{distro}{win_path}")
    } else {
        format!(r"\\wsl.localhost\{distro}\{win_path}")
    }
}

/// Render the Windows-body anchor. `self_section` comes from
/// `render_self_section` and follows the bio.
pub fn render_windows_anchor(
    id: &AnchorIdentity,
    timestamp: &str,
    koad_home: &str,
    distro: &str,
    vault_unc: &str,
    self_section: &str,
    cass: Result<&str, &CassMiss>,
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
    s.push_str(self_section);
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
    match cass {
        Err(miss) => {
            s.push('\n');
            s.push_str(&miss.anchor_line());
            s.push('\n');
        }
        Ok(p) if !p.is_empty() => {
            s.push_str("\n## 🧠 Temporal Context Packet (CASS)\n");
            s.push_str(p);
        }
        Ok(_) => {}
    }
    s
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
    let cass_addr = &config.network.cass_grpc_addr;
    let packet = fetch_cass_packet(
        cass_addr,
        &key,
        &koad_home,
        CASS_CONNECT_TIMEOUT,
        CASS_HYDRATE_TIMEOUT,
    )
    .await;
    // Stdout becomes session context, so the reason goes to stderr and, since
    // the hook keeps no stderr, to the anchor log.
    if let Err(miss) = &packet {
        let line = miss.log_line(cass_addr);
        eprintln!("koad-agent anchor: {line}");
        log_cass_miss(&config.home, &key, "windows", &line);
    }
    let journal = latest_journal(&vault_path).map(|p| wsl_unc(&distro, &p.to_string_lossy()));
    let self_section = render_self_section(read_self(&vault_path).as_deref(), journal.as_deref());
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
            &self_section,
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
            "",
            Ok("## Ⅰ. Episodes\n- x\n"),
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
        let a = render_windows_anchor(&clyde(), "T", "/k", "Ubuntu", VAULT, "", Ok("p"));
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
        let a = render_windows_anchor(&clyde(), "T", "/k", "Ubuntu-24.04", VAULT, "", Ok("p"));
        assert!(
            a.contains("wsl.exe -d Ubuntu-24.04 -e //k/bin/koad-wsl-env koad"),
            "{a}"
        );
        assert!(!a.contains("///"), "{a}");
    }

    #[test]
    fn offline_cass_is_stated_not_hidden() {
        let a = render_windows_anchor(
            &clyde(),
            "T",
            "/k",
            "Ubuntu",
            VAULT,
            "",
            Err(&CassMiss::Unreachable("refused".into())),
        );
        assert!(a.contains("Memory: offline (CASS unreachable)"), "{a}");
        assert!(!a.contains("Temporal Context Packet"));
    }

    /// `Ok("")` means CASS answered with an empty packet — distinct from a
    /// miss. Neither a memory warning nor a packet section belongs in the
    /// anchor for this case.
    #[test]
    fn empty_cass_packet_prints_neither_offline_nor_packet_section() {
        let a = render_windows_anchor(&clyde(), "T", "/k", "Ubuntu", VAULT, "", Ok(""));
        assert!(!a.contains("Memory: offline"), "{a}");
        assert!(!a.contains("Temporal Context Packet"), "{a}");
    }

    /// A slow CASS is not reported as offline: that false report is what
    /// this anchor used to give whenever hydrate took longer than 3s.
    #[test]
    fn slow_cass_is_not_called_offline() {
        let miss = CassMiss::Slow(std::time::Duration::from_secs(15));
        let a = render_windows_anchor(&clyde(), "T", "/k", "Ubuntu", VAULT, "", Err(&miss));
        assert!(
            a.contains("CASS is up but hydration timed out after 15s"),
            "{a}"
        );
        assert!(!a.contains("offline"), "{a}");
    }

    #[test]
    fn self_section_sits_between_bio_and_working_environment() {
        let section = render_self_section(Some("# SELF — Clyde\nI'm Clyde."), Some(r"\\v\j.md"));
        let a = render_windows_anchor(&clyde(), "T", "/k", "Ubuntu", VAULT, &section, Ok("p"));
        let bio = a.find("## Bio").expect("bio");
        let me = a.find("# SELF — Clyde").expect("self");
        let env = a.find("## Working Environment").expect("env");
        assert!(bio < me && me < env, "{a}");
        assert!(a.contains(r"Read it before starting: `\\v\j.md`"), "{a}");
    }
}
