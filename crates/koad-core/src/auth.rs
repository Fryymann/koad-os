use tonic::{Request, Status, service::Interceptor};
use crate::config::KoadConfig;

#[derive(Clone)]
pub struct AdminInterceptor {
    token: String,
}

impl AdminInterceptor {
    pub fn new(config: &KoadConfig) -> Self {
        Self {
            token: config.network.admin_token.clone().unwrap_or_else(|| "koados-default-admin-token".to_string()),
        }
    }
}

impl Interceptor for AdminInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        match request.metadata().get("x-admin-token") {
            Some(t) if t == self.token.as_str() => Ok(request),
            _ => Err(Status::unauthenticated("Invalid or missing x-admin-token")),
        }
    }
}
