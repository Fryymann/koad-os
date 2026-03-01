use std::process::{Command, Stdio};
use std::path::Path;

pub struct DeckManager {
    path: String,
}

impl DeckManager {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let deck_path = Path::new(&self.path);
        if !deck_path.exists() {
            return Err(anyhow::anyhow!("Vite Deck directory not found at {}", self.path));
        }

        println!("DeckManager: Launching Vite Dashboard in background...");

        // Start npm run dev in the deck directory
        // Setting a robust PATH to ensure npm and node are found
        let _child = Command::new("npm")
            .arg("run")
            .arg("dev")
            .current_dir(deck_path)
            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/home/ideans/.cargo/bin:/home/ideans/.nvm/versions/node/v22.21.1/bin")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // Note: In a production environment, we would handle child cleanup
        // For now, we let it run as a daemonized child of the Kernel.
        
        Ok(())
    }
}
