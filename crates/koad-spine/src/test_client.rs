use chrono::Utc;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::{HotContextChunk, HydrationRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SpineServiceClient::connect("http://127.0.0.1:50051").await?;

    let request = HydrationRequest {
        session_id: "test-session".to_string(),
        chunk: Some(HotContextChunk {
            chunk_id: "test-chunk".to_string(),
            content: "Test context hydration".to_string(),
            file_path: "".to_string(),
            ttl_seconds: 0,
            created_at: Some(prost_types::Timestamp {
                seconds: Utc::now().timestamp(),
                nanos: Utc::now().timestamp_subsec_nanos() as i32,
            }),
        }),
    };

    let response = client.hydrate_context(request).await;
    println!("Hydrate Response: {:?}", response);

    Ok(())
}
