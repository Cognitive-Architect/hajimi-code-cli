//! SkillLoader — Reads resolved instruction content and estimates token sizes.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::context_window_manager::estimate_tokens;
use crate::skills::errors::SkillError;
use crate::skills::skills_registry::SkillRegistry;
use crate::skills::types::LoadedSkill;
use std::fs;

/// Handles loading individual Skill pack contents on-demand.
pub struct SkillLoader {
    pub registry: SkillRegistry,
}

impl SkillLoader {
    /// Creates a new SkillLoader associated with a SkillRegistry.
    pub fn new(registry: SkillRegistry) -> Self {
        Self { registry }
    }

    /// Loads the resolved instructions for a Skill by its name, verifying path constraints.
    pub fn load(&self, name: &str) -> Result<LoadedSkill, SkillError> {
        let manifest = self
            .registry
            .get(name)
            .ok_or_else(|| SkillError::NotFound(name.to_string()))?
            .clone();

        let skill_dir = self.registry.root.join(&manifest.name);
        let entry_path = skill_dir.join(&manifest.entry);

        if !entry_path.exists() {
            return Err(SkillError::MissingEntry(format!(
                "Entry file '{}' not found in skill directory '{}'",
                manifest.entry,
                skill_dir.display()
            )));
        }

        // Enforce physical path traversal boundaries
        let canonical_skill_dir = fs::canonicalize(&skill_dir)?;
        let canonical_entry_path = fs::canonicalize(&entry_path)?;
        if !canonical_entry_path.starts_with(&canonical_skill_dir) {
            return Err(SkillError::InvalidPath(format!(
                "Path traversal detected: entry file '{}' resolves outside skill directory '{}'",
                manifest.entry,
                skill_dir.display()
            )));
        }

        let instructions = fs::read_to_string(&entry_path)?;
        let token_estimate = estimate_tokens(&instructions);

        Ok(LoadedSkill {
            manifest,
            instructions,
            token_estimate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::SkillManifest;
    use crate::skills::types::SkillPermissions;
    use crate::skills::types::SkillRiskLevel;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn build_test_manifest(name: &str, entry: &str) -> SkillManifest {
        SkillManifest {
            schema_version: "hajimi.skill.v0".to_string(),
            version: "0.1.0".to_string(),
            name: name.to_string(),
            title: "Test Title".to_string(),
            description: "Test Desc".to_string(),
            enabled: true,
            category: None,
            exclusive_group: None,
            triggers: vec![],
            risk_level: SkillRiskLevel::Low,
            entry: entry.to_string(),
            eval_entry: None,
            context_budget_tokens: None,
            allowed_tools: vec![],
            permissions: SkillPermissions::default(),
        }
    }

    #[test]
    fn test_skills_loader_loads_successfully() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("test-skill");
        fs::create_dir(&skill_dir).unwrap();

        let manifest = build_test_manifest("test-skill", "SKILL.md");
        let manifest_json = serde_json::to_string(&manifest).unwrap();

        let mut f = File::create(skill_dir.join("skill.json")).unwrap();
        f.write_all(manifest_json.as_bytes()).unwrap();

        let mut f2 = File::create(skill_dir.join("SKILL.md")).unwrap();
        f2.write_all(b"Hello from Skill instructions!").unwrap();

        let registry = SkillRegistry::scan(dir.path()).unwrap();
        let loader = SkillLoader::new(registry);
        let loaded = loader.load("test-skill").unwrap();

        assert_eq!(loaded.instructions, "Hello from Skill instructions!");
        assert!(loaded.token_estimate > 0);
    }

    #[test]
    fn test_skills_loader_missing_entry_fails() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("test-skill");
        fs::create_dir(&skill_dir).unwrap();

        let manifest = build_test_manifest("test-skill", "SKILL.md");
        let manifest_json = serde_json::to_string(&manifest).unwrap();

        let mut f = File::create(skill_dir.join("skill.json")).unwrap();
        f.write_all(manifest_json.as_bytes()).unwrap();

        let registry = SkillRegistry::scan(dir.path()).unwrap();
        let loader = SkillLoader::new(registry);
        let res = loader.load("test-skill");

        assert!(matches!(res, Err(SkillError::MissingEntry(_))));
    }
}
