//! Partition key canon: `{agent}_{host}_{user}` (e.g. `clyde_Jupiter_ideans`).
//! Matches the `AGENT_PARTITION` convention used by koad-os-mcp.

fn host() -> String {
    if let Ok(h) = std::env::var("KOAD_HOST") {
        if !h.is_empty() {
            return h;
        }
    }
    if let Ok(h) = std::fs::read_to_string("/etc/hostname") {
        let h = h.trim();
        if !h.is_empty() {
            return h.to_string();
        }
    }
    std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown-host".into())
}

fn user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown-user".into())
}

/// Canonical partition key for an agent on this instance.
pub fn partition_key(agent: &str) -> String {
    format!("{}_{}_{}", agent, host(), user())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_key_has_three_segments_prefixed_by_agent() {
        let key = partition_key("clyde");
        assert!(key.starts_with("clyde_"));
        assert!(key.split('_').count() >= 3);
    }
}
