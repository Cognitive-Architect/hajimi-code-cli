//! SkillLoader — Reads resolved instruction content and estimates token sizes.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::context_window_manager::estimate_tokens;
use crate::skills::errors::SkillError;
use crate::skills::registry::SkillRegistry;
use crate::skills::types::{LoadedSkill, SkillEvalCriterion, SkillEvalFixture};
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

        // Safety verification of the entry relative path
        if manifest.entry.is_empty()
            || manifest.entry.starts_with('/')
            || manifest.entry.starts_with('\\')
            || manifest.entry.contains("..")
            || manifest.entry.contains(':')
        {
            return Err(SkillError::InvalidPath(format!(
                "Unsafe entry relative path '{}' detected in manifest '{}'",
                manifest.entry, manifest.name
            )));
        }

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

    /// Loads lightweight output evaluation criteria for a selected Skill.
    pub fn load_eval_criteria(&self, name: &str) -> Result<Option<SkillEvalCriterion>, SkillError> {
        let manifest = self
            .registry
            .get(name)
            .ok_or_else(|| SkillError::NotFound(name.to_string()))?
            .clone();

        let Some(eval_entry) = manifest.eval_entry.as_ref() else {
            return Ok(None);
        };

        if eval_entry.is_empty()
            || eval_entry.starts_with('/')
            || eval_entry.starts_with('\\')
            || eval_entry.contains("..")
            || eval_entry.contains(':')
        {
            return Err(SkillError::InvalidPath(format!(
                "Unsafe eval_entry relative path '{}' detected in manifest '{}'",
                eval_entry, manifest.name
            )));
        }

        let skill_dir = self.registry.root.join(&manifest.name);
        let eval_path = skill_dir.join(eval_entry);

        if !eval_path.exists() {
            return Err(SkillError::MissingEntry(format!(
                "Eval entry '{}' not found in skill directory '{}'",
                eval_entry,
                skill_dir.display()
            )));
        }

        let canonical_skill_dir = fs::canonicalize(&skill_dir)?;
        let canonical_eval_path = fs::canonicalize(&eval_path)?;
        if !canonical_eval_path.starts_with(&canonical_skill_dir) {
            return Err(SkillError::InvalidPath(format!(
                "Path traversal detected: eval entry '{}' resolves outside skill directory '{}'",
                eval_entry,
                skill_dir.display()
            )));
        }

        let fixture_raw = fs::read_to_string(&eval_path)?;
        let mut fixture: SkillEvalFixture = serde_json::from_str(&fixture_raw).map_err(|e| {
            SkillError::InvalidManifest(format!(
                "Failed to parse eval fixture '{}': {}",
                eval_path.display(),
                e
            ))
        })?;

        if fixture.schema_version != "hajimi.skill.eval.v0" {
            return Err(SkillError::InvalidManifest(format!(
                "Invalid eval schema version '{}'. Expected 'hajimi.skill.eval.v0'.",
                fixture.schema_version
            )));
        }
        if fixture.skill_name != manifest.name {
            return Err(SkillError::InvalidManifest(format!(
                "Eval skill_name '{}' does not match manifest '{}'.",
                fixture.skill_name, manifest.name
            )));
        }

        fixture.criteria.skill_name = manifest.name;
        Ok(Some(fixture.criteria))
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
    use std::path::Path;
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

    #[test]
    fn test_skills_loader_real_fixture_e2e() {
        // Construct deterministic workspace absolute path using CARGO_MANIFEST_DIR
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/fixtures/skills");

        let registry = SkillRegistry::scan(&root).unwrap();
        assert!(registry.get("auto-save").is_some());

        let loader = SkillLoader::new(registry);
        let loaded = loader.load("auto-save").unwrap();

        // Verify loaded instructions match specifications
        assert!(
            loaded.instructions.contains("AUTO SAVE")
                || loaded.instructions.contains("做了什么")
                || loaded.instructions.contains("当前状态")
        );
        assert!(loaded.token_estimate > 0);
    }

    #[test]
    fn test_skills_loader_loads_eval_criteria_from_output_cases() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/fixtures/skills");

        let registry = SkillRegistry::scan(&root).unwrap();
        let loader = SkillLoader::new(registry);
        let criteria = loader
            .load_eval_criteria("auto-save")
            .unwrap()
            .expect("auto-save should have output criteria");

        assert_eq!(criteria.skill_name, "auto-save");
        assert!(criteria.must_include.iter().any(|v| v == "=== AUTO SAVE"));
        assert!(criteria.must_not_include.iter().any(|v| v == "TODO"));
    }

    #[test]
    fn test_skills_loader_traversal_escape_denied() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("test-skill");
        fs::create_dir(&skill_dir).unwrap();

        // 1. entry = "../SKILL.md"
        let manifest = build_test_manifest("test-skill", "../SKILL.md");
        let mut registry = SkillRegistry {
            root: dir.path().to_path_buf(),
            manifests: std::collections::HashMap::new(),
        };
        registry.manifests.insert(manifest.name.clone(), manifest);

        let loader = SkillLoader::new(registry);
        let res = loader.load("test-skill");
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), SkillError::InvalidPath(_)));
    }

    #[test]
    fn test_skills_loader_absolute_paths_denied() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("test-skill");
        fs::create_dir(&skill_dir).unwrap();

        // 1. starts_with('/')
        let manifest1 = build_test_manifest("test-skill", "/tmp/SKILL.md");
        let mut registry1 = SkillRegistry {
            root: dir.path().to_path_buf(),
            manifests: std::collections::HashMap::new(),
        };
        registry1
            .manifests
            .insert(manifest1.name.clone(), manifest1);
        let loader1 = SkillLoader::new(registry1);
        let res1 = loader1.load("test-skill");
        assert!(res1.is_err());
        assert!(matches!(res1.unwrap_err(), SkillError::InvalidPath(_)));

        // 2. contains(':')
        let manifest2 = build_test_manifest("test-skill", "C:\\SKILL.md");
        let mut registry2 = SkillRegistry {
            root: dir.path().to_path_buf(),
            manifests: std::collections::HashMap::new(),
        };
        registry2
            .manifests
            .insert(manifest2.name.clone(), manifest2);
        let loader2 = SkillLoader::new(registry2);
        let res2 = loader2.load("test-skill");
        assert!(res2.is_err());
        assert!(matches!(res2.unwrap_err(), SkillError::InvalidPath(_)));
    }
}
