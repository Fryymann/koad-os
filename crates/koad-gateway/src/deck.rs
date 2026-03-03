use std::env;
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub struct DeckManager {
    path: String,
    _process: Option<Child>,
}

impl DeckManager {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            _process: None,
        }
    }

    pub async fn start(mut self) -> anyhow::Result<Self> {
        let deck_path = Path::new(&self.path);
        if !deck_path.exists() {
            return Err(anyhow::anyhow!(
                "Vite Deck directory not found at {}",
                self.path
            ));
        }

        if env::var("KOAD_DEV_MODE").unwrap_or_default() == "1" {
            println!("DeckManager: Launching Vite Dashboard (DEV MODE) in background...");

            // Start npm run dev in the deck directory
            let child = Command::new("npm")
                .arg("run")
                .arg("dev")
                .current_dir(deck_path)
                .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/home/ideans/.cargo/bin:/home/ideans/.nvm/versions/node/v22.21.1/bin")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;

            self._process = Some(child);
        } else {
            println!(
                "DeckManager: Running in Production mode. Serving static assets via WebGateway."
            );
        }

        Ok(self)
    }
}

impl Drop for DeckManager {
    fn drop(&mut self) {
        if let Some(mut process) = self._process.take() {
            println!("DeckManager: Stopping Vite Dev Server...");
            let _ = process.kill();
        }
    }
}
