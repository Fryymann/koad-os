use anyhow::Result;

pub async fn handle_info(agent: String) -> Result<()> {
    println!("Agent Identity: {}", agent);
    // ... add more info here if needed
    Ok(())
}
