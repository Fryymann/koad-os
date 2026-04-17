//! PulseService gRPC handler — short-lived broadcast signals between agents.

use crate::storage::PulseTier;
use koad_proto::cass::v1::pulse_service_server::PulseService;
use koad_proto::cass::v1::{AddPulseRequest, GetPulsesRequest, GetPulsesResponse, Pulse};
use koad_proto::citadel::v5::StatusResponse;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct CassPulseService {
    store: Arc<dyn PulseTier>,
}

impl CassPulseService {
    pub fn new(store: Arc<dyn PulseTier>) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl PulseService for CassPulseService {
    async fn add_pulse(
        &self,
        request: Request<AddPulseRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let pulse = Pulse {
            id: Uuid::new_v4().to_string(),
            author: req.author,
            role: if req.role.is_empty() {
                "global".to_string()
            } else {
                req.role
            },
            message: req.message,
            ttl_seconds: if req.ttl_seconds == 0 {
                3600
            } else {
                req.ttl_seconds
            },
            created_at: None,
        };
        self.store
            .add_pulse(pulse)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(StatusResponse {
            success: true,
            message: "Pulse broadcast.".to_string(),
            ..Default::default()
        }))
    }

    async fn get_pulses(
        &self,
        request: Request<GetPulsesRequest>,
    ) -> Result<Response<GetPulsesResponse>, Status> {
        let req = request.into_inner();
        let role = if req.role.is_empty() {
            "global".to_string()
        } else {
            req.role
        };
        let pulses = self
            .store
            .get_active_pulses(&role)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetPulsesResponse { pulses }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::mock::MockPulseStore;

    #[tokio::test]
    async fn test_add_pulse_defaults_empty_role_to_global() -> anyhow::Result<()> {
        let mock_store = Arc::new(MockPulseStore::new());
        let service = CassPulseService::new(mock_store.clone());

        let request = Request::new(AddPulseRequest {
            context: None,
            author: "test-agent".to_string(),
            role: "".to_string(),
            message: "Hello from test".to_string(),
            ttl_seconds: 0,
        });

        let response = service.add_pulse(request).await?;
        assert!(response.into_inner().success);

        let stored = mock_store.all().await;
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].role, "global");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_pulses_returns_stored_pulses() -> anyhow::Result<()> {
        let mock_store = Arc::new(MockPulseStore::new());
        mock_store
            .seed(Pulse {
                id: "pulse-001".to_string(),
                author: "clyde".to_string(),
                role: "officer".to_string(),
                message: "Officer signal active".to_string(),
                ttl_seconds: 3600,
                created_at: None,
            })
            .await;

        let service = CassPulseService::new(mock_store);

        let request = Request::new(GetPulsesRequest {
            context: None,
            role: "officer".to_string(),
        });

        let response = service.get_pulses(request).await?;
        let pulses = response.into_inner().pulses;

        assert_eq!(pulses.len(), 1);
        assert_eq!(pulses[0].message, "Officer signal active");
        Ok(())
    }
}
