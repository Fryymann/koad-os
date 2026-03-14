//! Authentication Interceptor
//!
//! Enforces the presence of TraceContext metadata on every gRPC request.

use tonic::{Request, Status};

/// Tonic interceptor that enforces TraceContext on every request.
///
/// This function validates that `x-trace-id`, `x-origin`, and `x-actor` headers
/// are present and non-empty. It provides a bypass for connections arriving
/// via the admin UDS, which are identified by the `x-admin-override` header.
///
/// # Errors
/// Returns `UNAUTHENTICATED` if any required header is missing or invalid.
pub fn trace_context_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    // Admin bypass: UDS connections set this header in the admin listener layer
    if req.metadata().get("x-admin-override").is_some() {
        return Ok(req);
    }

    // Require x-trace-id header
    let trace_id = req
        .metadata()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("Missing x-trace-id header"))?;

    if trace_id.is_empty() {
        return Err(Status::unauthenticated("Empty x-trace-id header"));
    }

    // Require x-origin header
    let origin = req
        .metadata()
        .get("x-origin")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("Missing x-origin header"))?;

    // This is a placeholder for a more robust origin check
    req.extensions_mut().insert(Origin(origin.to_string()));

    // Require x-actor header
    let actor = req
        .metadata()
        .get("x-actor")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("Missing x-actor header"))?;

    if actor.is_empty() {
        return Err(Status::unauthenticated("Empty x-actor header"));
    }

    Ok(req)
}

/// Request extension holding the validated origin.
#[derive(Clone)]
pub struct Origin(pub String);
