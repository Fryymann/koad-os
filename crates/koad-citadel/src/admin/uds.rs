//! Admin Unix Domain Socket (UDS) Listener
//!
//! Provides a privileged, local-only gRPC interface for direct Citadel administration.
//! Access is restricted by filesystem permissions on the socket.

use anyhow::Context;
use hyper::http;
use std::path::Path;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::server::NamedService;
use tonic::transport::Server;
use tower::Service;
use tracing::info;

/// Start the Admin Override UDS listener (Issue #162).
pub async fn start_admin_uds_listener<S>(socket_path: &Path, service: S) -> anyhow::Result<()>
where
    S: Service<
            http::Request<tonic::body::BoxBody>,
            Response = http::Response<tonic::body::BoxBody>,
            Error = std::convert::Infallible,
        > + NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    // Clean up stale socket
    if socket_path.exists() {
        std::fs::remove_file(socket_path)
            .with_context(|| format!("Failed to remove stale admin socket at {:?}", socket_path))?;
    }

    let uds = UnixListener::bind(socket_path)
        .with_context(|| format!("Failed to bind admin UDS at {:?}", socket_path))?;

    // Set socket permissions to owner-only (0o600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(socket_path, perms)
            .with_context(|| "Failed to set admin UDS permissions")?;
    }

    info!(
        "Admin UDS listener started at {:?} (privileged access only)",
        socket_path
    );

    let incoming = UnixListenerStream::new(uds);

    // Serve
    Server::builder()
        .add_service(service)
        .serve_with_incoming(incoming)
        .await
        .context("Admin UDS server failed")?;

    Ok(())
}
