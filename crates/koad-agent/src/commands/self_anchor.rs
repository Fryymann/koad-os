//! Shared by `koad-agent boot` (WSL body) and `koad-agent anchor` (Windows
//! body): the agent's own first-person identity file, its newest journal
//! entry, and a CASS hydration fetch that tells "slow" apart from "down".
//!
//! SELF.md is read straight from the vault so an agent wakes up knowing who
//! it is even when CASS is unavailable.

use koad_proto::cass::v1::hydration_service_client::HydrationServiceClient;
use koad_proto::cass::v1::HydrationRequest;
use koad_proto::citadel::v5::WorkspaceLevel;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tonic::transport::Endpoint;

/// Bounded so a down CASS (dropped packets on WSL mirrored networking) does
/// not stall a SessionStart hook.
pub const CASS_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// Hydrate takes ~2.5s on a quiet machine and more while a session starts
/// several processes at once; a 3s budget reported a healthy CASS as
/// offline. The Windows hook allows 30s overall.
pub const CASS_HYDRATE_TIMEOUT: Duration = Duration::from_secs(15);

/// Why no hydration packet came back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CassMiss {
    /// Could not connect at all.
    Unreachable(String),
    /// Connected, but hydrate did not answer within the budget.
    Slow(Duration),
    /// Connected, and hydrate returned an error.
    Errored(String),
}

impl CassMiss {
    /// One line for the anchor itself.
    pub fn anchor_line(&self) -> String {
        match self {
            CassMiss::Unreachable(_) => "Memory: offline (CASS unreachable). Memory tools will \
                return errors until CASS is back."
                .to_string(),
            CassMiss::Slow(t) => format!(
                "Memory: CASS is up but hydration timed out after {}s, so there is no context \
                 packet this boot. Memory tools should still work; check with `status.citadel`.",
                t.as_secs()
            ),
            CassMiss::Errored(_) => "Memory: CASS answered but hydration failed, so there is \
                no context packet this boot. Memory tools may still work; check with \
                `status.citadel`."
                .to_string(),
        }
    }

    /// One reasoned line for stderr and the anchor log.
    pub fn log_line(&self, cass_addr: &str) -> String {
        match self {
            CassMiss::Unreachable(e) => format!("CASS unreachable at {cass_addr}: {e}"),
            CassMiss::Slow(t) => {
                format!("CASS at {cass_addr} connected, hydrate timed out after {t:?}")
            }
            CassMiss::Errored(e) => format!("CASS at {cass_addr} connected, hydrate failed: {e}"),
        }
    }
}

/// Ask CASS for the agent's hydration packet.
pub async fn fetch_cass_packet(
    cass_addr: &str,
    agent: &str,
    project_root: &str,
    connect_timeout: Duration,
    hydrate_timeout: Duration,
) -> Result<String, CassMiss> {
    let endpoint = Endpoint::from_shared(cass_addr.to_string())
        .map_err(|e| CassMiss::Unreachable(format!("invalid endpoint: {e}")))?
        .connect_timeout(connect_timeout);
    let channel = match tokio::time::timeout(connect_timeout, endpoint.connect()).await {
        Ok(Ok(channel)) => channel,
        Ok(Err(e)) => return Err(CassMiss::Unreachable(e.to_string())),
        Err(_) => {
            return Err(CassMiss::Unreachable(format!(
                "connect timed out after {connect_timeout:?}"
            )))
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
    match tokio::time::timeout(hydrate_timeout, client.hydrate(req)).await {
        Ok(Ok(resp)) => Ok(resp.into_inner().markdown_packet),
        Ok(Err(status)) => Err(CassMiss::Errored(status.to_string())),
        Err(_) => Err(CassMiss::Slow(hydrate_timeout)),
    }
}

/// Append one timestamped line to `$KOAD_HOME/logs/anchor.log`. The Windows
/// SessionStart hook's stderr is not kept anywhere, so without this a failed
/// boot leaves no evidence. Best effort: logging never fails a boot.
pub fn log_cass_miss(koad_home: &Path, agent: &str, body: &str, line: &str) {
    let dir = koad_home.join("logs");
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("anchor.log"))
    {
        // One write per line: `writeln!` with arguments can issue several, and
        // concurrent anchors then interleave mid-line. A single small write to
        // an O_APPEND file lands whole.
        let entry = format!(
            "{} {agent} ({body}): {line}\n",
            chrono::Utc::now().to_rfc3339()
        );
        let _ = f.write_all(entry.as_bytes());
    }
}

/// The agent's first-person identity file, `identity/SELF.md`. `None` when it
/// is missing or blank.
pub fn read_self(vault: &Path) -> Option<String> {
    let text = std::fs::read_to_string(vault.join("identity").join("SELF.md")).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Newest `journal/*.md` entry. Entries are named `YYYY-MM-DD-slug.md`, so
/// the newest sorts last.
pub fn latest_journal(vault: &Path) -> Option<PathBuf> {
    std::fs::read_dir(vault.join("journal"))
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "md"))
        .max_by(|a, b| a.file_name().cmp(&b.file_name()))
}

/// The self section of an anchor: SELF.md verbatim, then a pointer to the
/// newest journal entry. `journal` is already rendered for the body that will
/// read it (a UNC path on Windows). A missing SELF.md is stated, not hidden.
pub fn render_self_section(self_md: Option<&str>, journal: Option<&str>) -> String {
    let mut s = String::from("\n## Self (identity/SELF.md, in my own words)\n");
    match self_md {
        Some(text) => {
            s.push_str(text);
            s.push('\n');
        }
        None => s.push_str("No identity/SELF.md found in the vault.\n"),
    }
    if let Some(j) = journal {
        s.push_str(&format!(
            "\n## Latest journal entry\nRead it before starting: `{j}`\n"
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use koad_proto::cass::v1::hydration_service_server::{
        HydrationService, HydrationServiceServer,
    };
    use koad_proto::cass::v1::HydrationResponse;

    fn vault() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn read_self_returns_the_file_trimmed() {
        let v = vault();
        std::fs::create_dir_all(v.path().join("identity")).unwrap();
        std::fs::write(
            v.path().join("identity/SELF.md"),
            "\n# SELF\nI'm Clyde.\n\n",
        )
        .unwrap();
        assert_eq!(read_self(v.path()).as_deref(), Some("# SELF\nI'm Clyde."));
    }

    #[test]
    fn read_self_is_none_when_missing_or_blank() {
        let v = vault();
        assert_eq!(read_self(v.path()), None);
        std::fs::create_dir_all(v.path().join("identity")).unwrap();
        std::fs::write(v.path().join("identity/SELF.md"), "  \n").unwrap();
        assert_eq!(read_self(v.path()), None);
    }

    #[test]
    fn latest_journal_picks_the_newest_markdown_entry() {
        let v = vault();
        let j = v.path().join("journal");
        std::fs::create_dir_all(j.join("2026-12-31-a-dir.md")).unwrap();
        for name in [
            "2026-09-26-a.md",
            "2026-10-06-b.md",
            "2026-09-27-c.md",
            "zz-notes.txt",
        ] {
            std::fs::write(j.join(name), "x").unwrap();
        }
        assert_eq!(latest_journal(v.path()), Some(j.join("2026-10-06-b.md")));
    }

    #[test]
    fn latest_journal_is_none_without_entries() {
        let v = vault();
        assert_eq!(latest_journal(v.path()), None);
        std::fs::create_dir_all(v.path().join("journal")).unwrap();
        assert_eq!(latest_journal(v.path()), None);
    }

    #[test]
    fn self_section_has_self_verbatim_and_the_journal_pointer() {
        let s = render_self_section(Some("# SELF — Clyde\nI'm Clyde."), Some("/v/journal/j.md"));
        assert!(
            s.contains("## Self (identity/SELF.md, in my own words)\n# SELF — Clyde\nI'm Clyde.\n"),
            "{s}"
        );
        assert!(
            s.contains("Read it before starting: `/v/journal/j.md`"),
            "{s}"
        );
    }

    #[test]
    fn self_section_states_a_missing_self_and_omits_absent_journal() {
        let s = render_self_section(None, None);
        assert!(s.contains("No identity/SELF.md found"), "{s}");
        assert!(!s.contains("Latest journal"), "{s}");
    }

    #[test]
    fn slow_and_unreachable_read_differently() {
        let slow = CassMiss::Slow(Duration::from_secs(15)).anchor_line();
        let down = CassMiss::Unreachable("refused".into()).anchor_line();
        assert!(
            slow.contains("CASS is up but hydration timed out after 15s"),
            "{slow}"
        );
        assert!(!slow.contains("offline"), "{slow}");
        assert!(
            down.contains("Memory: offline (CASS unreachable)"),
            "{down}"
        );
    }

    #[test]
    fn log_appends_timestamped_lines() {
        let home = vault();
        log_cass_miss(home.path(), "clyde", "windows", "first");
        log_cass_miss(home.path(), "clyde", "windows", "second");
        let log = std::fs::read_to_string(home.path().join("logs/anchor.log")).unwrap();
        let lines: Vec<_> = log.lines().collect();
        assert_eq!(lines.len(), 2, "{log}");
        assert!(lines[1].ends_with("clyde (windows): second"), "{log}");
    }

    /// Several bodies can miss CASS at the same moment; each must still get
    /// a whole line of its own.
    #[test]
    fn concurrent_log_lines_do_not_interleave() {
        let home = vault();
        let threads: Vec<_> = (0..16)
            .map(|i| {
                let p = home.path().to_path_buf();
                std::thread::spawn(move || {
                    for _ in 0..50 {
                        log_cass_miss(&p, "clyde", "windows", &format!("miss {i}"));
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        let log = std::fs::read_to_string(home.path().join("logs/anchor.log")).unwrap();
        let lines: Vec<_> = log.lines().collect();
        assert_eq!(lines.len(), 16 * 50);
        for l in lines {
            assert!(
                l.matches("clyde (windows): miss ").count() == 1,
                "interleaved: {l}"
            );
        }
    }

    /// An unparseable CASS address must fail before any network I/O.
    #[tokio::test]
    async fn invalid_uri_is_unreachable_quickly() {
        let start = std::time::Instant::now();
        let r = fetch_cass_packet(
            "not a uri",
            "a",
            "/r",
            Duration::from_millis(500),
            Duration::from_millis(500),
        )
        .await;
        assert!(matches!(r, Err(CassMiss::Unreachable(_))), "{r:?}");
        assert!(start.elapsed() < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn closed_port_is_unreachable() {
        let port = {
            let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().port()
        };
        let r = fetch_cass_packet(
            &format!("http://127.0.0.1:{port}"),
            "a",
            "/r",
            Duration::from_millis(500),
            Duration::from_millis(500),
        )
        .await;
        assert!(matches!(r, Err(CassMiss::Unreachable(_))), "{r:?}");
    }

    /// A CASS that answers after `delay`.
    struct SlowCass {
        delay: Duration,
    }

    #[tonic::async_trait]
    impl HydrationService for SlowCass {
        async fn hydrate(
            &self,
            _req: tonic::Request<HydrationRequest>,
        ) -> Result<tonic::Response<HydrationResponse>, tonic::Status> {
            tokio::time::sleep(self.delay).await;
            Ok(tonic::Response::new(HydrationResponse {
                markdown_packet: "PACKET".into(),
                ..Default::default()
            }))
        }
    }

    async fn serve_slow_cass(delay: Duration) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(
            tonic::transport::Server::builder()
                .add_service(HydrationServiceServer::new(SlowCass { delay }))
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener)),
        );
        format!("http://{addr}")
    }

    /// The bug this module fixes: a hydrate slower than the connect budget
    /// must still succeed when it fits the hydrate budget.
    #[tokio::test]
    async fn hydrate_slower_than_connect_budget_still_succeeds() {
        let addr = serve_slow_cass(Duration::from_millis(600)).await;
        let r = fetch_cass_packet(
            &addr,
            "a",
            "/r",
            Duration::from_millis(300),
            Duration::from_secs(5),
        )
        .await;
        assert_eq!(r, Ok("PACKET".to_string()));
    }

    #[tokio::test]
    async fn hydrate_past_its_budget_is_slow_not_unreachable() {
        let addr = serve_slow_cass(Duration::from_secs(5)).await;
        let r = fetch_cass_packet(
            &addr,
            "a",
            "/r",
            Duration::from_secs(2),
            Duration::from_millis(300),
        )
        .await;
        assert_eq!(r, Err(CassMiss::Slow(Duration::from_millis(300))));
    }
}
