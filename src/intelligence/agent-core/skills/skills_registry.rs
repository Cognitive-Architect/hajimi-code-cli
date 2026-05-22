//! SkillRegistry — Scans and catalogs local skill packs in the workspace.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::skills::errors::SkillError;
use crate::skills::types::SkillManifest;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Catalogs skill pack metadata by scanning a specified directory.
#[derive(Debug, Clone)]
pub struct SkillRegistry {
    pub root: PathBuf,
    pub manifests: HashMap<String, SkillManifest>,
}

impl SkillRegistry {
    /// Scans the specified root directory for skill packs.
    /// Each subfolder under `root` that contains a valid `skill.json` represents a skill pack.
    pub fn scan(root: &Path) -> Result<Self, SkillError> {
        let mut manifests = HashMap::new();

        if !root.exists() {
            return Err(SkillError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Skill root directory '{}' not found", root.display()),
            )));
        }
        if !root.is_dir() {
            return Err(SkillError::InvalidPath(format!(
                "Skill root '{}' is not a directory",
                root.display()
            )));
        }

        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let skill_json_path = path.join("skill.json");
                if skill_json_path.exists() && skill_json_path.is_file() {
                    let content = fs::read_to_string(&skill_json_path)?;
                    let manifest: SkillManifest = serde_json::from_str(&content)?;
                    manifest.validate()?;
                    manifests.insert(manifest.name.clone(), manifest);
                }
            }
        }

        Ok(Self {
            root: root.to_path_buf(),
            manifests,
        })
    }

    /// Returns a list of references to all loaded skill manifests, sorted by name.
    pub fn list(&self) -> Vec<&SkillManifest> {
        let mut list: Vec<&SkillManifest> = self.manifests.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    /// Retrieves a specific skill manifest by its kebab-case name.
    pub fn get(&self, name: &str) -> Option<&SkillManifest> {
        self.manifests.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_skills_registry_scan_valid_fixtures() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("auto-save");
        fs::create_dir(&skill_dir).unwrap();

        let json = r#"{
            "schema_version": "hajimi.skill.v0",
            "version": "0.1.0",
            "name": "auto-save",
            "title": "自动存档",
            "description": "当任务推进时，生成存档。",
            "enabled": true,
            "exclusive_group": "handoff",
            "triggers": ["存档"],
            "risk_level": "low",
            "entry": "SKILL.md",
            "permissions": {
                "read_workspace": true,
                "write_workspace": false,
                "run_shell": false,
                "network": false,
                "delete": false
            },
            "allowed_tools": []
        }"#;

        let mut file = File::create(skill_dir.join("skill.json")).unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let registry = SkillRegistry::scan(dir.path()).unwrap();
        assert_eq!(registry.list().len(), 1);
        let manifest = registry.get("auto-save").unwrap();
        assert_eq!(manifest.name, "auto-save");
    }

    #[test]
    fn test_skills_registry_scan_malformed_json_fails() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("auto-save");
        fs::create_dir(&skill_dir).unwrap();

        let mut file = File::create(skill_dir.join("skill.json")).unwrap();
        file.write_all(b"invalid json").unwrap();

        let res = SkillRegistry::scan(dir.path());
        assert!(res.is_err());
    }

    #[test]
    fn test_skills_registry_scan_invalid_manifest_fails() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("auto-save");
        fs::create_dir(&skill_dir).unwrap();

        let json = r#"{
            "schema_version": "hajimi.skill.v0",
            "version": "0.1.0",
            "name": "auto-save",
            "title": "自动存档",
            "description": "当任务推进时，生成存档。",
            "enabled": true,
            "triggers": [],
            "risk_level": "low",
            "entry": "",
            "permissions": {},
            "allowed_tools": []
        }"#;

        let mut file = File::create(skill_dir.join("skill.json")).unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let res = SkillRegistry::scan(dir.path());
        assert!(res.is_err());
    }
}
