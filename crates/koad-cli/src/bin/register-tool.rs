use koad_proto::cass::v1::tool_registry_service_client::ToolRegistryServiceClient;
use koad_proto::cass::v1::RegisterToolRequest;
use koad_proto::citadel::v5::TraceContext;
use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = ToolRegistryServiceClient::connect("http://127.0.0.1:50052").await?;

    let plugin_name = "hello-plugin";
    let koad_home = std::env::var("KOADOS_HOME")
        .or_else(|_| std::env::var("KOAD_HOME"))
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".koad-os")
                .to_string_lossy()
                .into_owned()
        });
    let component_path = PathBuf::from(&koad_home)
        .join("crates/koad-plugins/wit/hello-plugin.component.wasm");

    if !component_path.exists() {
        anyhow::bail!("Component not found at {}", component_path.display());
    }

    println!("Registering tool '{}' from {}...", plugin_name, component_path.display());

    let request = tonic::Request::new(RegisterToolRequest {
        context: Some(TraceContext {
            trace_id: "BOOT-TOOL-VERIFY".to_string(),
            origin: "Admin".to_string(),
            actor: "Tyr".to_string(),
            timestamp: None,
            level: 3, // Citadel scope
        }),
        name: plugin_name.to_string(),
        component_path: component_path.to_string_lossy().into_owned(),
        container_image: String::new(),
    });

    let response = client.register_tool(request).await?;
    let res = response.into_inner();

    if res.success {
        println!("\x1b[32m[SUCCESS]\x1b[0m {}", res.message);
    } else {
        println!("\x1b[31m[FAILED]\x1b[0m {}", res.message);
    }

    Ok(())
}
