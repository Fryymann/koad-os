use koad_proto::skill::skill_service_server::{SkillService, SkillServiceServer};
use koad_proto::skill::{Empty, RunRequest, RunUpdate, StatusUpdate};
use std::pin::Pin;
use tokio_stream::Stream;
use tonic::{transport::Server, Request, Response, Status};

pub struct AirtableSkill {
    _client: reqwest::Client,
}

impl AirtableSkill {
    pub fn new() -> Self {
        Self {
            _client: reqwest::Client::new(),
        }
    }
}

#[tonic::async_trait]
impl SkillService for AirtableSkill {
    type RunStream = Pin<Box<dyn Stream<Item = Result<RunUpdate, Status>> + Send>>;

    async fn run(&self, request: Request<RunRequest>) -> Result<Response<Self::RunStream>, Status> {
        let req = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        println!("AirtableSkill: Running tool {}", req.skill_id);

        tokio::spawn(async move {
            // Placeholder: Implement actual Airtable API logic here
            let _ = tx
                .send(Ok(RunUpdate {
                    skill_id: req.skill_id,
                    output: "Airtable operation completed via Rust Skill Bridge.".to_string(),
                    progress: 1.0,
                    finished: true,
                }))
                .await;
        });

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::RunStream))
    }

    async fn report_status(
        &self,
        _request: Request<StatusUpdate>,
    ) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let uds_path = "/tmp/koad-airtable.sock";
    if std::fs::metadata(uds_path).is_ok() {
        std::fs::remove_file(uds_path)?;
    }

    let uds = tokio::net::UnixListener::bind(uds_path)?;
    let uds_stream = tokio_stream::wrappers::UnixListenerStream::new(uds);

    let skill = AirtableSkill::new();
    println!("Airtable Skill listening on {}...", uds_path);

    Server::builder()
        .add_service(SkillServiceServer::new(skill))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}
