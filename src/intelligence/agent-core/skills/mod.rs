//! Module entry for Hajimi Agent Skills.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

pub mod errors;
pub mod loader;
pub mod registry;
pub mod router;
pub mod scoring;
pub mod types;

pub use errors::SkillError;
pub use loader::SkillLoader;
pub use registry::SkillRegistry;
pub use router::{SkillRouteResult, SkillRouter};
pub use scoring::score_skill;
pub use types::{
    LoadedSkill, SkillManifest, SkillMatch, SkillMatchScore, SkillPermissions, SkillRiskLevel,
    SkillRouteReceipt, SkillRouterConfig,
};
