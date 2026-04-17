//! Admin Unix Domain Socket (UDS) Listener
//!
//! Provides a privileged, local-only gRPC interface for direct Citadel administration.

use anyhow::Context;
use std::path::Path;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::server::Router;
use tracing::info;

/// Start the Admin Override UDS listener (Issue #162).
pub async fn start_admin_uds_listener(socket_path: &Path, router: Router) -> anyhow::Result<()> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path).context("Failed to remove stale socket")?;
    }

    let uds = UnixListener::bind(socket_path).context("Failed to bind UDS")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(socket_path, perms).context("Failed to set UDS permissions")?;
    }

    info!(
        "Admin UDS listener started at {:?} (privileged access only)",
        socket_path
    );
    let incoming = UnixListenerStream::new(uds);

    router
        .serve_with_incoming(incoming)
        .await
        .context("Admin UDS server failed")?;

    Ok(())
}
