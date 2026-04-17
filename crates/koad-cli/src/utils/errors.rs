use std::fmt;

/// Wraps raw tonic transport and status errors into human-readable KoadOS prompts.
#[derive(Debug)]
pub enum KoadGrpcError {
    /// Could not connect to the service (connection refused, timeout, etc.)
    ConnectionFailed { service: String, addr: String },
    /// RPC call returned a non-OK status
    RpcFailed {
        service: String,
        code: tonic::Code,
        message: String,
    },
}

impl fmt::Display for KoadGrpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KoadGrpcError::ConnectionFailed { service, addr } => {
                write!(
                    f,
                    "\x1b[31m[OFFLINE]\x1b[0m {} is not reachable at {}. Run '\x1b[1mkoad system start\x1b[0m' to ignite the kernel.",
                    service, addr
                )
            }
            KoadGrpcError::RpcFailed {
                service,
                code,
                message,
            } => {
                let hint = match code {
                    tonic::Code::PermissionDenied => {
                        "Check your agent's rank or role assignment.".to_string()
                    }
                    tonic::Code::NotFound => {
                        "The session or entity is missing or expired. Try re-booting your agent."
                            .to_string()
                    }
                    tonic::Code::Unavailable => {
                        format!(
                            "{} may have crashed. Run 'koad system status' to diagnose.",
                            service
                        )
                    }
                    tonic::Code::Unauthenticated => {
                        "Session token is invalid or missing. Re-run 'koad-agent boot <name>'."
                            .to_string()
                    }
                    _ => format!("RPC error ({:?}): {}", code, message),
                };
                write!(
                    f,
                    "\x1b[31m[RPC ERROR]\x1b[0m {} returned an error.\n  \x1b[33m→\x1b[0m {}",
                    service, hint
                )
            }
        }
    }
}

impl std::error::Error for KoadGrpcError {}

impl From<tonic::Status> for KoadGrpcError {
    fn from(s: tonic::Status) -> Self {
        KoadGrpcError::RpcFailed {
            service: "KoadOS Service".to_string(),
            code: s.code(),
            message: s.message().to_string(),
        }
    }
}

/// Map a tonic transport connection error to a KoadGrpcError.
pub fn map_connect_err(service: &str, addr: &str, e: tonic::transport::Error) -> KoadGrpcError {
    let _ = e; // transport::Error doesn't expose useful structured data
    KoadGrpcError::ConnectionFailed {
        service: service.to_string(),
        addr: addr.to_string(),
    }
}

/// Map a tonic::Status RPC error to a KoadGrpcError with service name.
pub fn map_status_err(service: &str, s: tonic::Status) -> KoadGrpcError {
    KoadGrpcError::RpcFailed {
        service: service.to_string(),
        code: s.code(),
        message: s.message().to_string(),
    }
}
