use anyhow::Result;
use koad_core::config::KoadConfig;

pub fn run(_config: &KoadConfig, _force: bool) -> Result<()> {
    println!("[KoadOS] System Init - stub");
    Ok(())
}
