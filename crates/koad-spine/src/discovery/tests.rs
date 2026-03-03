#[cfg(test)]
mod tests {
    use crate::discovery::SkillRegistry;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_skill_discovery() {
        let dir = tempdir().unwrap();
        let skill_path = dir.path().join("test_skill");
        std::fs::create_dir(&skill_path).unwrap();

        let manifest_path = skill_path.join("manifest.yaml");
        let mut file = File::create(manifest_path).unwrap();
        writeln!(
            file,
            "id: test
name: Test Skill
version: 1.0.0
endpoint: unix:///tmp/test.sock
tools: [tool1]"
        )
        .unwrap();

        let mut registry = SkillRegistry::new();
        registry
            .scan_directory(dir.path().to_str().unwrap())
            .unwrap();

        assert!(registry.skills.contains_key("test"));
        let manifest = registry.skills.get("test").unwrap();
        assert_eq!(manifest.name, "Test Skill");
        assert_eq!(manifest.tools.len(), 1);
    }
}
