//! Error types for the Hajimi Agent Skills system.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use thiserror::Error;

/// Errors returned by the Skill System operations.
#[derive(Debug, Error)]
pub enum SkillError {
    /// I/O errors during scanning or loading.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors during manifest validation or deserialization.
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    /// Path traversal or directory boundaries violations.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// The specified entry file (e.g. SKILL.md) is missing.
    #[error("Missing entry: {0}")]
    MissingEntry(String),

    /// Permission boundaries violation.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Token budget allocation exceeded.
    #[error("Token budget exceeded: {0}")]
    TokenBudgetExceeded(String),

    /// Serde JSON errors.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
