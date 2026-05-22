//! Module entry for Hajimi Agent Skills.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

pub mod errors;
pub mod eval;
pub mod golden_tests;
pub mod loader;
pub mod permissions;
pub mod registry;
pub mod router;
pub mod runtime;
pub mod scoring;
pub mod types;

pub use errors::SkillError;
pub use eval::{evaluate_output, parse_eval_fixture, SkillEvalResult};
pub use loader::SkillLoader;
pub use permissions::{map_skill_permissions_to_approval, SkillToolAction};
pub use registry::SkillRegistry;
pub use router::SkillRouter;
pub use runtime::{
    default_runtime_tool_names, filter_tool_by_constraints, is_agent_skill_runtime_enabled,
    normalize_runtime_tool_name, SkillRuntime,
};
pub use scoring::score_skill;
pub use types::{
    LoadedSkill, SkillAllowedToolsReport, SkillEvalCase, SkillEvalCriterion, SkillEvalFixture,
    SkillManifest, SkillMatch, SkillMatchScore, SkillPermissions, SkillRiskLevel,
    SkillRouteReceipt, SkillRouteResult, SkillRouterConfig, SkillToolConstraint,
    SkillToolConstraints, SkillToolPermissionLevel, BB_ACTIVE_SKILLS, BB_SKILL_EVAL_CRITERIA,
    BB_SKILL_INSTRUCTIONS, BB_SKILL_ROUTE_RECEIPT, BB_SKILL_TOOL_CONSTRAINTS,
    HAJIMI_AGENT_SKILL_RUNTIME_ENV,
};
