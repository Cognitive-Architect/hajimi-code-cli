//! Data types for the Hajimi Agent Skills system.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::skills::errors::SkillError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The risk levels defined for a Skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Permissions declared by a Skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SkillPermissions {
    pub read_workspace: bool,
    pub write_workspace: bool,
    pub run_shell: bool,
    pub network: bool,
    pub delete: bool,
}

/// Manifest metadata for a Skill, loaded from `skill.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillManifest {
    pub schema_version: String,
    pub version: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub enabled: bool,
    pub category: Option<String>,
    pub exclusive_group: Option<String>,
    pub triggers: Vec<String>,
    pub risk_level: SkillRiskLevel,
    pub entry: String,
    pub eval_entry: Option<String>,
    pub context_budget_tokens: Option<usize>,
    pub allowed_tools: Vec<String>,
    pub permissions: SkillPermissions,
}

impl SkillManifest {
    /// Validates the structure and constraints of the loaded manifest.
    pub fn validate(&self) -> Result<(), SkillError> {
        if self.schema_version != "hajimi.skill.v0" {
            return Err(SkillError::InvalidManifest(format!(
                "Invalid schema version '{}'. Expected 'hajimi.skill.v0'.",
                self.schema_version
            )));
        }

        if self.name.is_empty() {
            return Err(SkillError::InvalidManifest(
                "Name cannot be empty".to_string(),
            ));
        }

        // Validate kebab-case name: lowercase ASCII alphanumeric and hyphens only
        for c in self.name.chars() {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
                return Err(SkillError::InvalidManifest(format!(
                    "Invalid character '{}' in skill name '{}'. Must be lowercase kebab-case.",
                    c, self.name
                )));
            }
        }

        if self.entry.is_empty() {
            return Err(SkillError::MissingEntry(
                "Entry cannot be empty".to_string(),
            ));
        }

        let entry_path = Path::new(&self.entry);
        if entry_path.is_absolute()
            || self.entry.starts_with('/')
            || self.entry.starts_with('\\')
            || self.entry.contains("..")
        {
            return Err(SkillError::InvalidPath(format!(
                "Invalid entry path '{}'. Must be a relative path without '..'.",
                self.entry
            )));
        }

        if let Some(ref eval) = self.eval_entry {
            if eval.is_empty() {
                return Err(SkillError::InvalidManifest(
                    "eval_entry cannot be empty when specified".to_string(),
                ));
            }
            let eval_path = Path::new(eval);
            if eval_path.is_absolute()
                || eval.starts_with('/')
                || eval.starts_with('\\')
                || eval.contains("..")
            {
                return Err(SkillError::InvalidPath(format!(
                    "Invalid eval_entry path '{}'. Must be a relative path without '..'.",
                    eval
                )));
            }
        }

        Ok(())
    }
}

/// A fully loaded skill containing the manifest, resolved instruction content, and token estimates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedSkill {
    pub manifest: SkillManifest,
    pub instructions: String,
    pub token_estimate: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_manifest_deserialization_and_validation() {
        let json = r#"{
            "schema_version": "hajimi.skill.v0",
            "version": "0.1.0",
            "name": "auto-save",
            "title": "自动存档",
            "description": "当任务推进、状态变化时，生成存档块。",
            "enabled": true,
            "category": "handoff",
            "exclusive_group": "handoff-output",
            "triggers": ["存档", "handoff"],
            "risk_level": "low",
            "entry": "SKILL.md",
            "eval_entry": "evals/output_cases.json",
            "context_budget_tokens": 1200,
            "allowed_tools": [],
            "permissions": {
                "read_workspace": true,
                "write_workspace": false,
                "run_shell": false,
                "network": false,
                "delete": false
            }
        }"#;

        let manifest: SkillManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "auto-save");
        assert_eq!(manifest.risk_level, SkillRiskLevel::Low);
        assert!(manifest.permissions.read_workspace);
        assert!(!manifest.permissions.delete);
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_invalid_risk_level() {
        let json = r#"{
            "schema_version": "hajimi.skill.v0",
            "version": "0.1.0",
            "name": "auto-save",
            "title": "自动存档",
            "description": "当任务推进、状态变化时，生成存档块。",
            "enabled": true,
            "triggers": [],
            "risk_level": "extreme",
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

        let res: Result<SkillManifest, _> = serde_json::from_str(json);
        assert!(res.is_err());
    }

    #[test]
    fn test_missing_name() {
        // Name is missing from json
        let json = r#"{
            "schema_version": "hajimi.skill.v0",
            "version": "0.1.0",
            "title": "自动存档",
            "description": "当任务推进时生成存档。",
            "enabled": true,
            "triggers": [],
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

        let res: Result<SkillManifest, _> = serde_json::from_str(json);
        assert!(res.is_err());
    }

    #[test]
    fn test_invalid_name_format() {
        let manifest = SkillManifest {
            schema_version: "hajimi.skill.v0".to_string(),
            version: "0.1.0".to_string(),
            name: "Auto_Save".to_string(), // Invalid uppercase and underscore
            title: "自动存档".to_string(),
            description: "描述".to_string(),
            enabled: true,
            category: None,
            exclusive_group: None,
            triggers: vec![],
            risk_level: SkillRiskLevel::Low,
            entry: "SKILL.md".to_string(),
            eval_entry: None,
            context_budget_tokens: None,
            allowed_tools: vec![],
            permissions: SkillPermissions::default(),
        };

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_absolute_entry_path_denied() {
        let manifest = SkillManifest {
            schema_version: "hajimi.skill.v0".to_string(),
            version: "0.1.0".to_string(),
            name: "auto-save".to_string(),
            title: "自动存档".to_string(),
            description: "描述".to_string(),
            enabled: true,
            category: None,
            exclusive_group: None,
            triggers: vec![],
            risk_level: SkillRiskLevel::Low,
            entry: "/absolute/path/SKILL.md".to_string(),
            eval_entry: None,
            context_budget_tokens: None,
            allowed_tools: vec![],
            permissions: SkillPermissions::default(),
        };

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_path_traversal_entry_denied() {
        let manifest = SkillManifest {
            schema_version: "hajimi.skill.v0".to_string(),
            version: "0.1.0".to_string(),
            name: "auto-save".to_string(),
            title: "自动存档".to_string(),
            description: "描述".to_string(),
            enabled: true,
            category: None,
            exclusive_group: None,
            triggers: vec![],
            risk_level: SkillRiskLevel::Low,
            entry: "../SKILL.md".to_string(),
            eval_entry: None,
            context_budget_tokens: None,
            allowed_tools: vec![],
            permissions: SkillPermissions::default(),
        };

        assert!(manifest.validate().is_err());
    }
}
