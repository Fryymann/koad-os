#[tokio::main]
async fn main() -> anyhow::Result<()> {
    koad_agent::run().await
}
