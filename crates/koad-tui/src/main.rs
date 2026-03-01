pub mod app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    crate::app::run_tui().await
}
