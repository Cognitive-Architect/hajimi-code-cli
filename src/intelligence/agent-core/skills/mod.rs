//! Module entry for Hajimi Agent Skills.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

pub mod errors;
pub mod types;

pub use errors::SkillError;
pub use types::{LoadedSkill, SkillManifest, SkillPermissions, SkillRiskLevel};
