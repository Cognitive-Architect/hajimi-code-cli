//! Raw user intent carrier.
//!
//! This type is the single source of truth for the user's original request in the LLM-Native path.
//! **No local rule-based rewriting is ever allowed on the `text` field.**

use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Carries the user's raw, unmodified input into the LLM-Native turn execution.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RawUserIntent {
    /// The exact text the user typed (Chinese, English, or any language).
    /// This field MUST remain unchanged throughout the entire turn.
    pub text: String,

    /// Optional language hint (e.g. from UI or system locale).
    pub language_hint: Option<String>,

    /// Session identifier for correlation.
    pub session_id: String,

    /// When this intent was created.
    pub timestamp: DateTime<Utc>,

    /// Optional structured attachments (paths, images, etc.).
    /// These are metadata only; they do not trigger local rule rewriting of `text`.
    pub attachments: Vec<IntentAttachment>,
}

/// Structured attachment that can accompany a user intent.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum IntentAttachment {
    /// Local filesystem path (file or directory).
    LocalPath(PathBuf),

    /// Reference to an image (e.g. base64 or asset id).
    ImageRef(String),

    /// Future extension point.
    Other {
        kind: String,
        data: serde_json::Value,
    },
}

impl RawUserIntent {
    /// Create a minimal intent from plain text.
    pub fn from_text(text: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            language_hint: None,
            session_id: session_id.into(),
            timestamp: Utc::now(),
            attachments: Vec::new(),
        }
    }
}
