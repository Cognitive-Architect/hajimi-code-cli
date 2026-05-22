//! Module entry for Hajimi Agent Skills.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

pub mod errors;
pub mod skills_loader;
pub mod skills_registry;
pub mod types;

pub use errors::SkillError;
pub use skills_loader::SkillLoader;
pub use skills_registry::SkillRegistry;
pub use types::{LoadedSkill, SkillManifest, SkillPermissions, SkillRiskLevel};
