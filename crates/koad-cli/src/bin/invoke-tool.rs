use anyhow::Result;
use koad_proto::cass::v1::tool_registry_service_client::ToolRegistryServiceClient;
use koad_proto::cass::v1::InvokeToolRequest;
use koad_proto::citadel::v5::TraceContext;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = ToolRegistryServiceClient::connect("http://127.0.0.1:50052").await?;

    let plugin_name = "hello-plugin";
    let topic = "test.verify";
    let payload = r#"{"test_id": 123, "command": "echo hello"}"#;

    println!("Invoking tool '{}' with topic '{}'...", plugin_name, topic);

    let request = tonic::Request::new(InvokeToolRequest {
        context: Some(TraceContext {
            trace_id: "VERIFY-TOOL-INVOKE".to_string(),
            origin: "Admin".to_string(),
            actor: "Tyr".to_string(),
            timestamp: None,
            level: 3,
        }),
        name: plugin_name.to_string(),
        topic: topic.to_string(),
        payload: payload.to_string(),
    });

    let response = client.invoke_tool(request).await?;
    let res = response.into_inner();

    println!("\x1b[32m[RESPONSE]\x1b[0m");
    println!("Output:   {}", res.output);
    println!("Duration: {}ms", res.duration_ms);

    Ok(())
}
