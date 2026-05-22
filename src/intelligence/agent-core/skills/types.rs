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

/// Validates that a string is a valid lowercase kebab-case format.
/// Must be non-empty, and consist of lowercase ASCII alphanumeric characters and hyphens only.
fn validate_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Validates that a path is relative, non-empty, and safe (no leading slashes, path traversal, or drive letters).
fn is_safe_relative_skill_path(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.starts_with('/') || s.starts_with('\\') {
        return false;
    }
    if Path::new(s).is_absolute() {
        return false;
    }
    if s.contains("..") {
        return false;
    }
    if s.contains(':') {
        return false;
    }
    true
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

        if !validate_kebab_case(&self.name) {
            return Err(SkillError::InvalidManifest(format!(
                "Invalid skill name '{}'. Must be lowercase kebab-case.",
                self.name
            )));
        }

        if let Some(ref group) = self.exclusive_group {
            if !validate_kebab_case(group) {
                return Err(SkillError::InvalidManifest(format!(
                    "Invalid exclusive_group '{}'. Must be lowercase kebab-case and non-empty.",
                    group
                )));
            }
        }

        if !is_safe_relative_skill_path(&self.entry) {
            return Err(SkillError::InvalidPath(format!(
                "Invalid entry path '{}'. Must be a safe relative path without '..', leading slashes, or drive letters.",
                self.entry
            )));
        }

        if let Some(ref eval) = self.eval_entry {
            if eval.is_empty() {
                return Err(SkillError::InvalidManifest(
                    "eval_entry cannot be empty when specified".to_string(),
                ));
            }
            if !is_safe_relative_skill_path(eval) {
                return Err(SkillError::InvalidPath(format!(
                    "Invalid eval_entry path '{}'. Must be a safe relative path without '..', leading slashes, or drive letters.",
                    eval
                )));
            }
        }

        Ok(())
    }
}

pub const BB_ACTIVE_SKILLS: &str = "__hajimi_active_skills";
pub const BB_SKILL_ROUTE_RECEIPT: &str = "__hajimi_skill_route_receipt";
pub const BB_SKILL_INSTRUCTIONS: &str = "__hajimi_skill_instructions";
pub const BB_SKILL_EVAL_CRITERIA: &str = "__hajimi_skill_eval_criteria";
pub const BB_SKILL_TOOL_CONSTRAINTS: &str = "__hajimi_skill_tool_constraints";
pub const HAJIMI_AGENT_SKILL_RUNTIME_ENV: &str = "HAJIMI_AGENT_SKILL_RUNTIME";

/// Runtime-level tool permission after intersecting a Skill's allowed_tools and permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillToolPermissionLevel {
    Deny,
    Ask,
    Allow,
}

/// Single tool decision produced by SkillRuntime. It is declarative only and never executes tools.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillToolConstraint {
    pub tool_name: String,
    pub permission: SkillToolPermissionLevel,
    pub approval_level: crate::governance::ApprovalLevel,
    pub reason: String,
}

/// Full constrained runtime report for one Skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillToolConstraints {
    pub skill_name: String,
    pub allowed: Vec<SkillToolConstraint>,
    pub denied: Vec<SkillToolConstraint>,
    pub warnings: Vec<String>,
}

/// Result of validating manifest allowed_tools against known tool names.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillAllowedToolsReport {
    pub valid_tools: Vec<String>,
    pub unknown_tools: Vec<String>,
    pub warnings: Vec<String>,
}

/// Represents the result of matching a Skill against an input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillMatch {
    pub name: String,
    pub score: f32,
    pub reason: String,
    pub matched_terms: Vec<String>,
    pub risk_level: SkillRiskLevel,
    pub category: Option<String>,
    pub exclusive_group: Option<String>,
}

/// Detailed breakdown of a Skill score computation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillMatchScore {
    pub score: f32,
    pub matched_triggers: Vec<String>,
    pub matched_name_or_title: Option<String>,
    pub matched_description_terms: Vec<String>,
    pub reason: String,
}

/// The receipt containing routing details and selected/rejected skills breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillRouteReceipt {
    pub input_hash: String,
    pub router_version: String,
    pub selected: Vec<SkillMatch>,
    pub rejected: Vec<SkillMatch>,
    pub timestamp: String,
}

/// The output result of a SkillRouter operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillRouteResult {
    pub selected: Vec<SkillMatch>,
    pub rejected: Vec<SkillMatch>,
    pub receipt: SkillRouteReceipt,
}

/// Configuration for the SkillRouter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillRouterConfig {
    pub threshold: f32,
    pub max_active_skills: usize,
}

impl Default for SkillRouterConfig {
    fn default() -> Self {
        Self {
            threshold: 0.55,
            max_active_skills: 3,
        }
    }
}

/// A fully loaded skill containing the manifest, resolved instruction content, and token estimates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedSkill {
    pub manifest: SkillManifest,
    pub instructions: String,
    pub token_estimate: usize,
}

/// Lightweight output acceptance criteria derived from a Skill eval fixture.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillEvalCriterion {
    pub skill_name: String,
    pub must_include: Vec<String>,
    pub must_not_include: Vec<String>,
    pub expected_structure: Option<Vec<String>>,
    pub failure_reason: Option<String>,
}

/// Deterministic output evaluation case used by golden tests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillEvalCase {
    pub name: String,
    pub output: String,
    pub expected_pass: bool,
    pub expected_failure_reason: Option<String>,
}

/// On-disk V0 eval fixture format referenced by `SkillManifest::eval_entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillEvalFixture {
    pub schema_version: String,
    pub skill_name: String,
    pub criteria: SkillEvalCriterion,
    pub cases: Vec<SkillEvalCase>,
}

/// Detailed evaluation report entry for a single Skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillEvalReportEntry {
    pub skill_name: String,
    pub passed: bool,
    pub failure_reason: Option<String>,
    pub matched_must_include: Vec<String>,
    pub missing_must_include: Vec<String>,
    pub matched_must_not_include: Vec<String>,
}

/// Detailed evaluation report for all active Skills.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillEvalReport {
    pub passed: bool,
    pub skill_reports: Vec<SkillEvalReportEntry>,
    pub failure_reason: Option<String>,
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
    fn test_real_fixture_deserialization() {
        let json = include_str!("../../../../tests/fixtures/skills/auto-save/skill.json");
        let manifest: SkillManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "auto-save");
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
    fn test_invalid_exclusive_group() {
        let json = r#"{
            "schema_version": "hajimi.skill.v0",
            "version": "0.1.0",
            "name": "auto-save",
            "title": "自动存档",
            "description": "描述",
            "enabled": true,
            "exclusive_group": "Handoff_Output",
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

        let manifest: SkillManifest = serde_json::from_str(json).unwrap();
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

    #[test]
    fn test_path_with_drive_letter_denied() {
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
            entry: "C:\\SKILL.md".to_string(),
            eval_entry: None,
            context_budget_tokens: None,
            allowed_tools: vec![],
            permissions: SkillPermissions::default(),
        };

        assert!(manifest.validate().is_err());
    }
}
