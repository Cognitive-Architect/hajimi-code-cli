//! Constrained Skill runtime skeleton.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::skills::permissions::{map_tool_action_to_permission, SkillToolAction};
use crate::skills::types::{
    SkillAllowedToolsReport, SkillManifest, SkillToolConstraint, SkillToolConstraints,
    SkillToolPermissionLevel, HAJIMI_AGENT_SKILL_RUNTIME_ENV,
};
use std::collections::BTreeSet;

/// Default-off runtime gate for V0b constrained Skill runtime.
pub fn is_agent_skill_runtime_enabled() -> bool {
    std::env::var(HAJIMI_AGENT_SKILL_RUNTIME_ENV)
        .map(|value| value == "true")
        .unwrap_or(false)
}

/// Runtime builds tool constraints only. It never calls or executes tools.
#[derive(Debug, Clone)]
pub struct SkillRuntime {
    available_tools: BTreeSet<String>,
}

impl SkillRuntime {
    pub fn new<I, S>(available_tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            available_tools: available_tools
                .into_iter()
                .map(|tool| normalize_runtime_tool_name(&tool.into()))
                .collect(),
        }
    }

    /// Filters manifest allowed_tools against known tool names and returns warnings for gaps.
    pub fn validate_allowed_tools(&self, manifest: &SkillManifest) -> SkillAllowedToolsReport {
        let mut valid_tools = BTreeSet::new();
        let mut unknown_tools = BTreeSet::new();
        let mut warnings = Vec::new();

        for raw_tool in &manifest.allowed_tools {
            let tool_name = normalize_runtime_tool_name(raw_tool);
            if self.available_tools.contains(&tool_name) {
                valid_tools.insert(tool_name);
            } else {
                unknown_tools.insert(tool_name.clone());
                warnings.push(format!(
                    "Skill '{}' requested unknown tool '{}' and it was filtered",
                    manifest.name, tool_name
                ));
            }
        }

        SkillAllowedToolsReport {
            valid_tools: valid_tools.into_iter().collect(),
            unknown_tools: unknown_tools.into_iter().collect(),
            warnings,
        }
    }

    /// Builds allowed and denied tool constraints by intersecting allowed_tools with permissions.
    pub fn build_tool_constraints(&self, manifest: &SkillManifest) -> SkillToolConstraints {
        let report = self.validate_allowed_tools(manifest);
        let mut allowed = Vec::new();
        let mut denied = Vec::new();

        for tool_name in &report.valid_tools {
            let action = classify_tool_action(tool_name);
            let (permission, approval_level, reason) =
                map_tool_action_to_permission(action, &manifest.permissions);
            let constraint = SkillToolConstraint {
                tool_name: tool_name.clone(),
                permission,
                approval_level,
                reason: reason.to_string(),
            };

            if permission == SkillToolPermissionLevel::Deny {
                denied.push(constraint);
            } else {
                allowed.push(constraint);
            }
        }

        SkillToolConstraints {
            skill_name: manifest.name.clone(),
            allowed,
            denied,
            warnings: report.warnings,
        }
    }
}

/// Tool names known to the V0b constrained runtime without constructing a ToolRegistry.
pub fn default_runtime_tool_names() -> &'static [&'static str] {
    &[
        "analyze_complexity",
        "api_request",
        "apply_patch",
        "benchmark",
        "cargo_build",
        "cmake",
        "coverage_report",
        "delete_file",
        "dependency_graph",
        "edit_file",
        "fetch_url",
        "find",
        "generate_docs",
        "generate_pr_description",
        "git_commit",
        "git_diff",
        "git_log",
        "git_status",
        "glob",
        "grep",
        "js_bundle_analyzer",
        "list_directory",
        "lsp_definition",
        "lsp_hover",
        "lsp_init",
        "lsp_references",
        "ls",
        "make",
        "mcp_init",
        "mcp_invoke",
        "multi_edit",
        "npm_run",
        "planning",
        "powershell",
        "read_file",
        "refactor_code",
        "reflection",
        "run_tests",
        "rust_doc_generator",
        "security_audit",
        "shell",
        "smart_commit",
        "update_readme",
        "view_image",
        "web_search",
        "write_file",
    ]
}

/// Normalize runtime and ToolRegistry names to a single comparison form.
pub fn normalize_runtime_tool_name(tool_name: &str) -> String {
    tool_name.trim().to_ascii_lowercase().replace('_', "-")
}

/// Applies active Skill constraints to a candidate tool name.
pub fn filter_tool_by_constraints(
    tool_name: &str,
    constraints: &[SkillToolConstraints],
) -> Result<Option<SkillToolConstraint>, String> {
    if constraints.is_empty() {
        return Ok(None);
    }

    let normalized_tool_name = normalize_runtime_tool_name(tool_name);
    for group in constraints {
        if let Some(denied) = group.denied.iter().find(|constraint| {
            normalize_runtime_tool_name(&constraint.tool_name) == normalized_tool_name
        }) {
            return Err(format!(
                "Skill runtime denied tool '{}': {}",
                tool_name, denied.reason
            ));
        }
    }

    let mut selected: Option<SkillToolConstraint> = None;
    for allowed in constraints.iter().flat_map(|group| &group.allowed) {
        if normalize_runtime_tool_name(&allowed.tool_name) != normalized_tool_name {
            continue;
        }
        selected = Some(match selected {
            Some(current) if is_constraint_more_restrictive(&current, allowed) => current,
            _ => allowed.clone(),
        });
    }

    selected.map(Some).ok_or_else(|| {
        format!(
            "Skill runtime denied tool '{}': tool is not present in active Skill allowed_tools",
            tool_name
        )
    })
}

fn is_constraint_more_restrictive(
    current: &SkillToolConstraint,
    candidate: &SkillToolConstraint,
) -> bool {
    let current_rank = approval_rank(current.approval_level) + permission_rank(current.permission);
    let candidate_rank =
        approval_rank(candidate.approval_level) + permission_rank(candidate.permission);
    current_rank >= candidate_rank
}

fn approval_rank(level: crate::governance::ApprovalLevel) -> u8 {
    match level {
        crate::governance::ApprovalLevel::Auto => 0,
        crate::governance::ApprovalLevel::Advisory => 1,
        crate::governance::ApprovalLevel::Required => 2,
        crate::governance::ApprovalLevel::Critical => 3,
        crate::governance::ApprovalLevel::Override => 4,
    }
}

fn permission_rank(level: SkillToolPermissionLevel) -> u8 {
    match level {
        SkillToolPermissionLevel::Allow => 0,
        SkillToolPermissionLevel::Ask => 1,
        SkillToolPermissionLevel::Deny => 4,
    }
}

fn classify_tool_action(tool_name: &str) -> SkillToolAction {
    if is_delete_tool(tool_name) {
        SkillToolAction::Delete
    } else if is_shell_tool(tool_name) {
        SkillToolAction::RunShell
    } else if is_network_tool(tool_name) {
        SkillToolAction::Network
    } else if is_write_tool(tool_name) {
        SkillToolAction::WriteWorkspace
    } else {
        SkillToolAction::ReadWorkspace
    }
}

fn is_shell_tool(tool_name: &str) -> bool {
    tool_name == "shell"
        || tool_name == "terminal-shell"
        || tool_name == "pwsh"
        || tool_name == concat!("ba", "sh")
        || tool_name == concat!("power", "shell")
}

fn is_network_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "api-request" | "download" | "fetch-url" | "web-search"
    )
}

fn is_delete_tool(tool_name: &str) -> bool {
    matches!(tool_name, "delete-file" | "delete" | "remove-file")
}

fn is_write_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "apply-patch"
            | "edit-file"
            | "generate-docs"
            | "git-commit"
            | "multi-edit"
            | "refactor-code"
            | "smart-commit"
            | "update-readme"
            | "write-file"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::ApprovalLevel;
    use crate::skills::types::{SkillPermissions, SkillRiskLevel};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn manifest_with(allowed_tools: Vec<&str>, permissions: SkillPermissions) -> SkillManifest {
        SkillManifest {
            schema_version: "hajimi.skill.v0".to_string(),
            version: "0.1.0".to_string(),
            name: "runtime-test".to_string(),
            title: "Runtime Test".to_string(),
            description: "Runtime constraint test".to_string(),
            enabled: true,
            category: None,
            exclusive_group: None,
            triggers: vec![],
            risk_level: SkillRiskLevel::Low,
            entry: "SKILL.md".to_string(),
            eval_entry: None,
            context_budget_tokens: None,
            allowed_tools: allowed_tools.into_iter().map(str::to_string).collect(),
            permissions,
        }
    }

    #[test]
    fn skill_runtime_gate_defaults_false() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var(HAJIMI_AGENT_SKILL_RUNTIME_ENV);
        assert!(!is_agent_skill_runtime_enabled());

        std::env::set_var(HAJIMI_AGENT_SKILL_RUNTIME_ENV, "TRUE");
        assert!(!is_agent_skill_runtime_enabled());

        std::env::set_var(HAJIMI_AGENT_SKILL_RUNTIME_ENV, "true");
        assert!(is_agent_skill_runtime_enabled());
        std::env::remove_var(HAJIMI_AGENT_SKILL_RUNTIME_ENV);
    }

    #[test]
    fn skill_runtime_validate_allowed_tools_filters_unknown_with_warning() {
        let runtime = SkillRuntime::new(["read-file", "write-file"]);
        let mut permissions = SkillPermissions::default();
        permissions.read_workspace = true;
        let manifest = manifest_with(vec!["read_file", "missing-tool"], permissions);

        let report = runtime.validate_allowed_tools(&manifest);

        assert_eq!(report.valid_tools, vec!["read-file"]);
        assert_eq!(report.unknown_tools, vec!["missing-tool"]);
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("unknown tool")));
    }

    #[test]
    fn skill_runtime_build_tool_constraints_intersects_allowed_tools_and_permissions() {
        let runtime = SkillRuntime::new(["read-file", "write-file", "web-search"]);
        let mut permissions = SkillPermissions::default();
        permissions.read_workspace = true;
        let manifest = manifest_with(vec!["read-file", "write-file", "web-search"], permissions);

        let constraints = runtime.build_tool_constraints(&manifest);

        assert_eq!(constraints.allowed.len(), 1);
        assert_eq!(constraints.allowed[0].tool_name, "read-file");
        assert_eq!(
            constraints.allowed[0].permission,
            SkillToolPermissionLevel::Allow
        );
        assert_eq!(constraints.denied.len(), 2);
        assert!(constraints
            .denied
            .iter()
            .any(|tool| tool.tool_name == "write-file"));
        assert!(constraints
            .denied
            .iter()
            .any(|tool| tool.tool_name == "web-search"));
    }

    #[test]
    fn skill_runtime_run_shell_false_denies_shell_tool() {
        let runtime = SkillRuntime::new(["shell"]);
        let mut permissions = SkillPermissions::default();
        permissions.read_workspace = true;
        permissions.run_shell = false;
        let manifest = manifest_with(vec!["shell"], permissions);

        let constraints = runtime.build_tool_constraints(&manifest);

        assert!(constraints.allowed.is_empty());
        assert_eq!(constraints.denied.len(), 1);
        assert_eq!(constraints.denied[0].tool_name, "shell");
        assert_eq!(
            constraints.denied[0].permission,
            SkillToolPermissionLevel::Deny
        );
        assert_eq!(
            constraints.denied[0].approval_level,
            ApprovalLevel::Critical
        );
        assert!(constraints.denied[0].reason.contains("run_shell=false"));
    }

    #[test]
    fn skill_runtime_run_shell_true_still_requires_approval() {
        let runtime = SkillRuntime::new(["shell"]);
        let mut permissions = SkillPermissions::default();
        permissions.run_shell = true;
        let manifest = manifest_with(vec!["shell"], permissions);

        let constraints = runtime.build_tool_constraints(&manifest);

        assert_eq!(constraints.allowed.len(), 1);
        assert_eq!(
            constraints.allowed[0].permission,
            SkillToolPermissionLevel::Ask
        );
        assert_eq!(
            constraints.allowed[0].approval_level,
            ApprovalLevel::Required
        );
    }

    #[test]
    fn skill_runtime_delete_permission_stays_denied() {
        let runtime = SkillRuntime::new(["delete-file"]);
        let mut permissions = SkillPermissions::default();
        permissions.delete = true;
        let manifest = manifest_with(vec!["delete-file"], permissions);

        let constraints = runtime.build_tool_constraints(&manifest);

        assert!(constraints.allowed.is_empty());
        assert_eq!(constraints.denied.len(), 1);
        assert_eq!(constraints.denied[0].tool_name, "delete-file");
        assert_eq!(
            constraints.denied[0].permission,
            SkillToolPermissionLevel::Deny
        );
        assert_eq!(
            constraints.denied[0].approval_level,
            ApprovalLevel::Critical
        );
        assert!(constraints.denied[0].reason.contains("delete permission"));
    }

    #[test]
    fn skill_runtime_unknown_tool_does_not_enter_constraints() {
        let runtime = SkillRuntime::new(["read-file"]);
        let mut permissions = SkillPermissions::default();
        permissions.read_workspace = true;
        let manifest = manifest_with(vec!["read-file", "unknown-tool"], permissions);

        let constraints = runtime.build_tool_constraints(&manifest);

        assert_eq!(constraints.allowed.len(), 1);
        assert!(constraints.denied.is_empty());
        assert!(constraints
            .warnings
            .iter()
            .any(|warning| warning.contains("unknown-tool")));
    }

    #[test]
    fn skill_runtime_filter_allows_normalized_allowed_tool() {
        let runtime = SkillRuntime::new(default_runtime_tool_names().iter().copied());
        let mut permissions = SkillPermissions::default();
        permissions.write_workspace = true;
        let manifest = manifest_with(vec!["write_file"], permissions);
        let constraints = vec![runtime.build_tool_constraints(&manifest)];

        let allowed = filter_tool_by_constraints("write_file", &constraints)
            .expect("write_file should pass constraints")
            .expect("matching constraint should be returned");

        assert_eq!(allowed.tool_name, "write-file");
        assert_eq!(allowed.permission, SkillToolPermissionLevel::Ask);
        assert_eq!(allowed.approval_level, ApprovalLevel::Required);
    }

    #[test]
    fn skill_runtime_filter_denies_tool_outside_allowed_tools() {
        let runtime = SkillRuntime::new(default_runtime_tool_names().iter().copied());
        let mut permissions = SkillPermissions::default();
        permissions.read_workspace = true;
        let manifest = manifest_with(vec!["read_file"], permissions);
        let constraints = vec![runtime.build_tool_constraints(&manifest)];

        let denied = filter_tool_by_constraints("write_file", &constraints)
            .expect_err("write_file should be denied outside allowed_tools");

        assert!(denied.contains("not present in active Skill allowed_tools"));
    }
}
