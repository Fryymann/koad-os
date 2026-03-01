use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use walkdir::WalkDir;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub endpoint: String, // e.g. unix:///tmp/koad-notion.sock
    pub tools: Vec<String>,
}

pub struct SkillRegistry {
    pub skills: HashMap<String, SkillManifest>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn scan_directory(&mut self, dir_path: &str) -> anyhow::Result<()> {
        println!("Scanning for skills in: {}", dir_path);
        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name() == "manifest.yaml" {
                let content = std::fs::read_to_string(entry.path())?;
                let manifest: SkillManifest = serde_yaml::from_str(&content)?;
                println!("Discovered Skill: {} ({})", manifest.name, manifest.id);
                self.skills.insert(manifest.id.clone(), manifest);
            }
        }
        Ok(())
    }
}
