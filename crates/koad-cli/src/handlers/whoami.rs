use anyhow::Result;
use koad_core::config::KoadConfig;
use koad_core::db::KoadDB;
use koad_proto::citadel::v5::citadel_session_client::CitadelSessionClient;
use koad_proto::citadel::v5::HeartbeatRequest;
use std::env;

/// `koad whoami`: identity from config, tether status from the Citadel.
///
/// Tether status comes from a Heartbeat RPC, the Citadel's own answer. This
/// used to parse the Redis lease as an `AgentSession`, whose shape does not
/// match the lease JSON, so it always reported NOT_TETHERED.
pub async fn handle_whoami(config: &KoadConfig, _db: &KoadDB) -> Result<()> {
    let session_id = env::var("KOAD_SESSION_ID").unwrap_or_default();
    let agent = env::var("KOAD_AGENT_NAME").unwrap_or_default();

    let tether = if session_id.is_empty() {
        "\x1b[33m[NOT_TETHERED]\x1b[0m No session loaded (run agent boot).".to_string()
    } else {
        match CitadelSessionClient::connect(config.network.citadel_grpc_addr.clone()).await {
            Err(_) => format!(
                "\x1b[33m[NOT_TETHERED]\x1b[0m Citadel unreachable at {}.",
                config.network.citadel_grpc_addr
            ),
            Ok(mut client) => {
                let req = HeartbeatRequest {
                    context: Some(crate::utils::get_trace_context(&agent, 1)),
                    session_id: session_id.clone(),
                    metrics: None,
                };
                match client
                    .heartbeat(crate::utils::authenticated_request(req))
                    .await
                {
                    Ok(_) => format!("\x1b[32m[TETHERED]\x1b[0m Session: {}", session_id),
                    Err(e) => format!(
                        "\x1b[33m[NOT_TETHERED]\x1b[0m Session {} rejected: {} (run agent boot).",
                        session_id,
                        e.message()
                    ),
                }
            }
        }
    };
    println!("{tether}");

    let identity = config
        .identities
        .get(&agent.to_lowercase())
        .or_else(|| config.identities.values().next());
    match identity {
        Some(id) => println!(
            "Identity: {} [{}]\nRank:     {}\nBio:      {}",
            id.name, id.role, id.rank, id.bio
        ),
        None => println!("No identities found in config."),
    }
    Ok(())
}
