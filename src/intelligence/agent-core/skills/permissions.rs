//! Permission mapping for the constrained Skill runtime.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::governance::ApprovalLevel;
use crate::skills::types::{SkillPermissions, SkillToolPermissionLevel};

/// Conservative action family used to intersect `allowed_tools` with manifest permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillToolAction {
    ReadWorkspace,
    WriteWorkspace,
    RunShell,
    Network,
    Delete,
}

/// Maps a Skill's broad permission request into the minimum Governance level it would require.
/// Delete stays Critical because B-09 only builds constraints and does not grant destructive access.
pub fn map_skill_permissions_to_approval(permissions: &SkillPermissions) -> ApprovalLevel {
    if permissions.delete {
        return ApprovalLevel::Critical;
    }
    if permissions.run_shell || permissions.network {
        return ApprovalLevel::Required;
    }
    if permissions.write_workspace {
        return ApprovalLevel::Required;
    }
    ApprovalLevel::Auto
}

/// Maps a single tool action to a constrained permission and approval level.
pub fn map_tool_action_to_permission(
    action: SkillToolAction,
    permissions: &SkillPermissions,
) -> (SkillToolPermissionLevel, ApprovalLevel, &'static str) {
    match action {
        SkillToolAction::ReadWorkspace if permissions.read_workspace => (
            SkillToolPermissionLevel::Allow,
            ApprovalLevel::Auto,
            "read_workspace=true allows read-only workspace tools",
        ),
        SkillToolAction::WriteWorkspace if permissions.write_workspace => (
            SkillToolPermissionLevel::Ask,
            ApprovalLevel::Required,
            "write_workspace=true requires approval for write tools",
        ),
        SkillToolAction::RunShell if permissions.run_shell => (
            SkillToolPermissionLevel::Ask,
            ApprovalLevel::Required,
            "run_shell=true requires required approval for shell tools",
        ),
        SkillToolAction::Network if permissions.network => (
            SkillToolPermissionLevel::Ask,
            ApprovalLevel::Required,
            "network=true requires required approval for network tools",
        ),
        SkillToolAction::Delete => (
            SkillToolPermissionLevel::Deny,
            ApprovalLevel::Critical,
            "delete permission is deny by default in B-09 constrained runtime",
        ),
        SkillToolAction::ReadWorkspace => (
            SkillToolPermissionLevel::Deny,
            ApprovalLevel::Required,
            "read_workspace=false denies read-only workspace tools",
        ),
        SkillToolAction::WriteWorkspace => (
            SkillToolPermissionLevel::Deny,
            ApprovalLevel::Required,
            "write_workspace=false denies write tools",
        ),
        SkillToolAction::RunShell => (
            SkillToolPermissionLevel::Deny,
            ApprovalLevel::Critical,
            "run_shell=false denies shell tools",
        ),
        SkillToolAction::Network => (
            SkillToolPermissionLevel::Deny,
            ApprovalLevel::Critical,
            "network=false denies network tools",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn permissions() -> SkillPermissions {
        SkillPermissions::default()
    }

    #[test]
    fn skill_runtime_permissions_map_read_only_to_auto() {
        let mut p = permissions();
        p.read_workspace = true;
        assert_eq!(map_skill_permissions_to_approval(&p), ApprovalLevel::Auto);
    }

    #[test]
    fn skill_runtime_permissions_map_write_to_required() {
        let mut p = permissions();
        p.write_workspace = true;
        assert_eq!(
            map_skill_permissions_to_approval(&p),
            ApprovalLevel::Required
        );
    }

    #[test]
    fn skill_runtime_permissions_map_shell_network_to_required() {
        let mut p = permissions();
        p.run_shell = true;
        assert_eq!(
            map_skill_permissions_to_approval(&p),
            ApprovalLevel::Required
        );

        p.run_shell = false;
        p.network = true;
        assert_eq!(
            map_skill_permissions_to_approval(&p),
            ApprovalLevel::Required
        );
    }

    #[test]
    fn skill_runtime_permissions_map_delete_to_critical() {
        let mut p = permissions();
        p.delete = true;
        assert_eq!(
            map_skill_permissions_to_approval(&p),
            ApprovalLevel::Critical
        );
    }
}
