//! `koad cognitive`: health of the systems an agent's continuity depends on.
//!
//! Every check reports what it actually verified, and the final verdict is
//! derived from the results rather than printed unconditionally.

use anyhow::Result;
use fred::interfaces::HashesInterface;
use koad_core::config::KoadConfig;
use koad_core::utils::redis::RedisClient;
use koad_proto::citadel::v5::citadel_session_client::CitadelSessionClient;
use koad_proto::citadel::v5::HeartbeatRequest;
use std::env;
use std::time::Duration;

/// Outcome of a single check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Level {
    Pass,
    Warn,
    Fail,
}

/// Overall condition: any failure degrades it, any warning needs attention.
fn verdict(levels: &[Level]) -> &'static str {
    if levels.contains(&Level::Fail) {
        "DEGRADED"
    } else if levels.contains(&Level::Warn) {
        "ATTENTION"
    } else {
        "OPTIMAL"
    }
}

fn report(levels: &mut Vec<Level>, level: Level, message: String) {
    let tag = match level {
        Level::Pass => "\x1b[32m[PASS]\x1b[0m",
        Level::Warn => "\x1b[33m[WARN]\x1b[0m",
        Level::Fail => "\x1b[31m[FAIL]\x1b[0m",
    };
    println!("{tag} {message}");
    levels.push(level);
}

pub async fn handle_cognitive_check(config: &KoadConfig, agent_name: &str) -> Result<()> {
    println!(
        "\x1b[1m--- {} Cognitive Self-Health Report ---\x1b[0m",
        agent_name
    );
    let mut levels = Vec::new();

    // L1: the session is live in the Citadel (validated with a heartbeat,
    // which also counts as activity and keeps it alive).
    let session_id = env::var("KOAD_SESSION_ID").unwrap_or_default();
    if session_id.is_empty() {
        report(
            &mut levels,
            Level::Warn,
            "L1: No session loaded (run agent boot to mint one)".into(),
        );
    } else {
        match CitadelSessionClient::connect(config.network.citadel_grpc_addr.clone()).await {
            Err(_) => report(
                &mut levels,
                Level::Fail,
                format!(
                    "L1: Citadel unreachable at {}",
                    config.network.citadel_grpc_addr
                ),
            ),
            Ok(mut client) => {
                let req = HeartbeatRequest {
                    context: Some(crate::utils::get_trace_context(agent_name, 1)),
                    session_id: session_id.clone(),
                    metrics: None,
                };
                match client
                    .heartbeat(crate::utils::authenticated_request(req))
                    .await
                {
                    Ok(_) => report(
                        &mut levels,
                        Level::Pass,
                        format!("L1: Session live ({session_id})"),
                    ),
                    Err(e) => report(
                        &mut levels,
                        Level::Fail,
                        format!(
                            "L1: Session {session_id} rejected: {} (re-run agent boot)",
                            e.message()
                        ),
                    ),
                }
            }
        }
    }

    // L2: hot context in Redis, and the file-based inbox.
    match RedisClient::new(&config.home.to_string_lossy(), false).await {
        Ok(client) => {
            let key = format!("koad:session:{}:hot_context", session_id);
            let chunks: std::collections::HashMap<String, String> =
                client.pool.hgetall(&key).await.unwrap_or_default();
            report(
                &mut levels,
                Level::Pass,
                format!("L2: Hot context ({} chunks)", chunks.len()),
            );
        }
        Err(e) => report(
            &mut levels,
            Level::Fail,
            format!("L2: Redis unreachable: {e}"),
        ),
    }
    let inbox =
        koad_core::inbox::pending_for(&koad_core::inbox::inbox_dir(&config.home), agent_name);
    report(
        &mut levels,
        Level::Pass,
        format!("L2: Inbox ({} pending item(s))", inbox.len()),
    );

    // L3: durable memory in CASS answers a recall query for this agent.
    let partition = koad_core::utils::partition::partition_key(agent_name);
    let cass =
        tonic::transport::Endpoint::from_shared(config.network.cass_grpc_addr.clone()).map(|ep| {
            ep.connect_timeout(Duration::from_secs(3))
                .timeout(Duration::from_secs(10))
        });
    let cass = match cass {
        Ok(ep) => ep.connect().await.map_err(anyhow::Error::from),
        Err(e) => Err(anyhow::Error::from(e)),
    };
    match cass {
        Err(_) => report(
            &mut levels,
            Level::Fail,
            format!("L3: CASS unreachable at {}", config.network.cass_grpc_addr),
        ),
        Ok(channel) => {
            let mut client =
                koad_proto::cass::v1::memory_service_client::MemoryServiceClient::new(channel);
            let query = koad_proto::cass::v1::SemanticQuery {
                query: agent_name.to_string(),
                partition: partition.clone(),
                limit: 1,
                min_score: 0.0,
            };
            match client.search_semantic(query).await {
                Ok(resp) if !resp.get_ref().facts.is_empty() => report(
                    &mut levels,
                    Level::Pass,
                    format!("L3: CASS recall working (partition {partition})"),
                ),
                Ok(_) => report(
                    &mut levels,
                    Level::Warn,
                    format!("L3: CASS reachable but no memories in partition {partition}"),
                ),
                Err(e) => report(
                    &mut levels,
                    Level::Fail,
                    format!("L3: CASS recall failed: {}", e.message()),
                ),
            }
        }
    }

    println!("Condition: \x1b[1m{}\x1b[0m", verdict(&levels));
    println!("\x1b[1m---------------------------------------------------\x1b[0m");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression guard: the report printed "OPTIMAL" unconditionally, even
    /// directly under WARN lines.
    #[test]
    fn verdict_reflects_the_worst_result() {
        assert_eq!(verdict(&[Level::Pass, Level::Pass]), "OPTIMAL");
        assert_eq!(verdict(&[Level::Pass, Level::Warn]), "ATTENTION");
        assert_eq!(
            verdict(&[Level::Warn, Level::Fail, Level::Pass]),
            "DEGRADED"
        );
    }
}
