//! Module entry for Hajimi Agent Skills.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

pub mod errors;
pub mod eval;
pub mod golden_tests;
pub mod loader;
pub mod registry;
pub mod router;
pub mod scoring;
pub mod types;

pub use errors::SkillError;
pub use eval::{evaluate_output, parse_eval_fixture, SkillEvalResult};
pub use loader::SkillLoader;
pub use registry::SkillRegistry;
pub use router::SkillRouter;
pub use scoring::score_skill;
pub use types::{
    LoadedSkill, SkillEvalCase, SkillEvalCriterion, SkillEvalFixture, SkillManifest, SkillMatch,
    SkillMatchScore, SkillPermissions, SkillRiskLevel, SkillRouteReceipt, SkillRouteResult,
    SkillRouterConfig, BB_ACTIVE_SKILLS, BB_SKILL_EVAL_CRITERIA, BB_SKILL_INSTRUCTIONS,
    BB_SKILL_ROUTE_RECEIPT,
};
