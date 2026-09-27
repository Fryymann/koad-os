//! The one way tools reach CASS, with bounded connect and request time.
//!
//! On WSL with mirrored networking a closed loopback port drops connection
//! attempts instead of refusing them, so an unbounded `connect()` hangs the
//! MCP client. A CASS that *accepts* the TCP connection but never answers
//! (overloaded, wedged) is a second failure mode `connect()` alone can't
//! catch: tonic's HTTP/2 handshake completes locally without waiting for the
//! peer's SETTINGS frame, so `connect()` returns `Ok` almost immediately even
//! though nothing is actually listening on the other end. The real liveness
//! check is the first RPC's round trip — [`call`] bounds that too, and is
//! the reason it exists alongside [`channel`]/[`memory`]/[`pulse`].

use anyhow::{anyhow, Context, Result};
use koad_proto::cass::v1::memory_service_client::MemoryServiceClient;
use koad_proto::cass::v1::pulse_service_client::PulseServiceClient;
use std::future::Future;
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use tonic::{Response, Status};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
// Calibration: cold `memory.search_semantic` measured ~1.8s on Jupiter
// (2026-09-27). 8s keeps ~4x margin over that while still failing fast when
// CASS is silent.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

/// Open a channel to CASS or fail within `CONNECT_TIMEOUT`.
pub async fn channel(url: &str) -> Result<Channel> {
    // Both `connect_timeout` (tonic's own bound on the TCP dial) and the
    // outer `tokio::time::timeout` below are kept deliberately: on WSL
    // mirrored networking a closed loopback port drops the SYN instead of
    // refusing it, so `connect_timeout` alone can still hang past its own
    // deadline waiting on the OS-level retransmission timer. The outer
    // `tokio::time::timeout` is what reliably fires in that case — don't
    // drop either one.
    let endpoint = Endpoint::from_shared(url.to_string())
        .with_context(|| format!("invalid CASS_URL {url}"))?
        .connect_timeout(CONNECT_TIMEOUT);
    tokio::time::timeout(CONNECT_TIMEOUT, endpoint.connect())
        .await
        .map_err(|_| {
            anyhow!(
                "CASS unreachable at {url} (no answer within {}s)",
                CONNECT_TIMEOUT.as_secs()
            )
        })?
        .with_context(|| format!("CASS unreachable at {url}"))
}

pub async fn memory(url: &str) -> Result<MemoryServiceClient<Channel>> {
    Ok(MemoryServiceClient::new(channel(url).await?))
}

pub async fn pulse(url: &str) -> Result<PulseServiceClient<Channel>> {
    Ok(PulseServiceClient::new(channel(url).await?))
}

/// Await an RPC call through a channel built by this module, or fail within
/// `REQUEST_TIMEOUT` with an explicit "CASS unreachable" error.
///
/// This is needed because `connect()` can succeed before CASS ever answers
/// (see module docs), so a silent peer is only caught when the first real
/// request stalls. A `Status` the RPC itself returned (a real, if unwelcome,
/// answer from CASS) is passed through unchanged — only our own timeout is
/// reported as unreachable.
///
/// A timed-out call (e.g. `commit_fact`) may still complete server-side after
/// we've already given up on it. For `commit_fact` specifically that's safe
/// to retry: fact ids are `sha256(partition:content)`, so a retried commit
/// after a timeout is idempotent rather than creating a duplicate.
pub async fn call<T, F>(fut: F) -> Result<Response<T>>
where
    F: Future<Output = std::result::Result<Response<T>, Status>>,
{
    let resp = tokio::time::timeout(REQUEST_TIMEOUT, fut)
        .await
        .map_err(|_| {
            anyhow!(
                "CASS unreachable (no answer within {}s)",
                REQUEST_TIMEOUT.as_secs()
            )
        })?;
    Ok(resp?)
}
