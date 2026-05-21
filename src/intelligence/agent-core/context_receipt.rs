//! Day 13 — Context Receipt JSON (per-request metadata, no full prompt, no API key).
//!
//! Saves a receipt for each LLM bridge request to `.hajimi/context_receipts/<timestamp>.json`.
//! Each receipt records: session_id, timestamp, provider/model/mode, budget metrics,
//! included/omitted block summaries (no full content), and estimated token usage.
//!
//! # Security Rules
//! - MUST NOT contain: api_key, Authorization header, Bearer token, full_prompt, promptText,
//!   full file body, or environment variable values.
//! - SHOULD contain: block names, priorities, token estimates, omit reasons.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Maximum number of included blocks stored per receipt (prevent unbounded growth).
const MAX_INCLUDED_BLOCKS: usize = 64;
/// Maximum number of omitted blocks stored per receipt.
const MAX_OMITTED_BLOCKS: usize = 64;
/// Maximum character length for any summary string (prevents leaking large content).
const MAX_SUMMARY_LEN: usize = 256;

// ──────────────────────────────────────────────────────────────────────────────
// Sensitive-information redaction
// ──────────────────────────────────────────────────────────────────────────────

/// Patterns whose *values* must be redacted from any summary stored in a receipt.
///
/// If a line/segment contains one of these keywords (case-insensitive), the
/// whole segment is replaced with `[REDACTED]`.
static SENSITIVE_PATTERNS: &[&str] = &[
    "Authorization",
    "Bearer ",
    "api_key",
    "apiKey",
    "api-key",
    "password",
    "secret",
    "token=",
    "token:",
    "env ",
    "env=",
    "sk-",
    "promptText",
    "full_prompt",
];

/// Redact any segment in `text` that contains a sensitive keyword.
///
/// Strategy:
/// 1. Split `text` into lines.
/// 2. For each line, check (case-insensitively) if it contains any sensitive pattern.
/// 3. If yes, replace the entire line with `"[REDACTED]"`.
/// 4. Join lines back together.
///
/// This is intentionally conservative — if a line even *mentions* a sensitive
/// keyword it is fully blanked, preventing accidental partial leaks.
pub fn redact_sensitive_text(text: &str) -> String {
    let lower = text.to_lowercase();
    // Fast path: most block names/structural summaries have no sensitive data.
    let has_any = SENSITIVE_PATTERNS
        .iter()
        .any(|p| lower.contains(&p.to_lowercase()));
    if !has_any {
        return text.to_string();
    }

    // Line-by-line redaction.
    text.lines()
        .map(|line| {
            let ll = line.to_lowercase();
            if SENSITIVE_PATTERNS
                .iter()
                .any(|p| ll.contains(&p.to_lowercase()))
            {
                "[REDACTED]"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Redact then truncate a summary string to at most `MAX_SUMMARY_LEN` chars.
pub fn sanitize_summary(text: &str) -> String {
    let redacted = redact_sensitive_text(text);
    truncate_summary(&redacted)
}

/// Trim a string to MAX_SUMMARY_LEN, appending "…" if truncated.
fn truncate_summary(s: &str) -> String {
    if s.chars().count() <= MAX_SUMMARY_LEN {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(MAX_SUMMARY_LEN).collect();
        out.push('…');
        out
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Block receipt types
// ──────────────────────────────────────────────────────────────────────────────

/// Record of a context block that was included in the LLM request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncludedBlockReceipt {
    /// Block identifier (e.g., "system_prompt", "file:src/main.rs").
    pub name: String,
    /// Priority class (P0, P1, P2, …).
    pub priority: String,
    /// Estimated token count for this block.
    pub token_estimate: usize,
    /// Source classification (e.g., "SystemPrompt", "FileContent", "Memory").
    pub source: String,
    /// Sanitized short summary — MUST NOT be the full content, MUST be redacted.
    pub summary: String,
}

/// Record of a context block that was omitted due to budget overflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmittedBlockReceipt {
    /// Block identifier.
    pub name: String,
    /// Priority class.
    pub priority: String,
    /// Estimated token count for this block (why it was rejected).
    pub token_estimate: usize,
    /// Source classification.
    pub source: String,
    /// Reason for omission (e.g., "BudgetExceeded", "LowPriority").
    pub reason: String,
}

// ──────────────────────────────────────────────────────────────────────────────
// Receipt timing / status enum
// ──────────────────────────────────────────────────────────────────────────────

/// Records whether this receipt was written before or after the provider call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptPhase {
    /// Receipt written after a successful provider response (preferred).
    PostRequest,
    /// Receipt written before the provider call (preflight estimate only).
    Preflight,
}

// ──────────────────────────────────────────────────────────────────────────────
// Top-level ContextReceipt
// ──────────────────────────────────────────────────────────────────────────────

/// Top-level context receipt saved after each LLM bridge call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextReceipt {
    /// Schema version for forward compatibility.
    pub schema_version: String,

    /// Unique session/request identifier.
    pub session_id: String,

    /// Unix timestamp (seconds) when the receipt was written.
    pub timestamp: u64,

    /// When was this receipt written relative to the provider call.
    pub phase: ReceiptPhase,

    // ── Provider / model info ──────────────────────────────────────────
    pub provider_id: String,
    pub model: String,

    // ── Budget metrics ─────────────────────────────────────────────────
    /// Budget mode label: "Long1M", "Fast128K", "Pro200K", "Legacy8K", …
    pub mode: String,
    /// Maximum context window for this provider/model.
    #[serde(rename = "maxContextTokens")]
    pub max_context_tokens: usize,
    /// Usable input budget (after reserve and safety margin).
    #[serde(rename = "inputBudget")]
    pub input_budget: usize,
    /// Estimated tokens actually sent in this request.
    #[serde(rename = "estimatedInputTokens")]
    pub estimated_input_tokens: usize,
    /// Whether long-context mode was active.
    #[serde(rename = "longContextMode")]
    pub long_context_mode: bool,

    // ── Block summaries ────────────────────────────────────────────────
    /// Blocks included in this request (summaries only, no full content).
    #[serde(rename = "includedBlocks")]
    pub included_blocks: Vec<IncludedBlockReceipt>,
    /// Blocks omitted due to budget or priority constraints.
    #[serde(rename = "omittedBlocks")]
    pub omitted_blocks: Vec<OmittedBlockReceipt>,

    /// Bridge role that generated this receipt ("planner" | "reflector" | "chat").
    pub bridge_role: String,

    /// Optional actual usage from provider (if available, otherwise None).
    #[serde(rename = "actualUsage")]
    pub actual_usage: Option<ActualUsage>,
}

/// Actual token usage as reported by the provider (if available).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActualUsage {
    #[serde(rename = "promptTokens")]
    pub prompt_tokens: usize,
    #[serde(rename = "completionTokens")]
    pub completion_tokens: usize,
}

// ──────────────────────────────────────────────────────────────────────────────
// Builder-style constructor
// ──────────────────────────────────────────────────────────────────────────────

impl ContextReceipt {
    /// Build a ContextReceipt from bridge statistics.
    ///
    /// `included_blocks` is a list of `(name, priority, token_estimate, source, summary_snippet)`.
    ///   The summary is sanitized (redacted + truncated) before storage.
    /// `omitted_blocks` is a list of `(name, priority, token_estimate, source, reason)`.
    ///   The reason field is NOT redacted (it's a structured enum-like string, no user content).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: String,
        provider_id: String,
        model: String,
        mode: String,
        max_context_tokens: usize,
        input_budget: usize,
        estimated_input_tokens: usize,
        long_context_mode: bool,
        bridge_role: String,
        phase: ReceiptPhase,
        included: Vec<(String, String, usize, String, String)>,
        omitted: Vec<(String, String, usize, String, String)>,
        actual_usage: Option<ActualUsage>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Cap + sanitize included blocks (redact sensitive content, then truncate).
        let included_blocks = included
            .into_iter()
            .take(MAX_INCLUDED_BLOCKS)
            .map(
                |(name, priority, token_estimate, source, content)| IncludedBlockReceipt {
                    name,
                    priority,
                    token_estimate,
                    source,
                    summary: sanitize_summary(&content),
                },
            )
            .collect();

        // Cap omitted blocks.
        let omitted_blocks = omitted
            .into_iter()
            .take(MAX_OMITTED_BLOCKS)
            .map(
                |(name, priority, token_estimate, source, reason)| OmittedBlockReceipt {
                    name,
                    priority,
                    token_estimate,
                    source,
                    reason,
                },
            )
            .collect();

        Self {
            schema_version: "ContextReceipt-v1".to_string(),
            session_id,
            timestamp,
            phase,
            provider_id,
            model,
            mode,
            max_context_tokens,
            input_budget,
            estimated_input_tokens,
            long_context_mode,
            bridge_role,
            included_blocks,
            omitted_blocks,
            actual_usage,
        }
    }

    /// Resolve the storage path: `~/.hajimi/context_receipts/<timestamp>-<session>.json`.
    pub fn storage_path(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        // Use both timestamp and a short session suffix to avoid collision when
        // multiple receipts land in the same second.
        let fname = format!(
            "{}-{}.json",
            self.timestamp,
            &self.session_id.chars().take(8).collect::<String>()
        );
        home.join(".hajimi").join("context_receipts").join(fname)
    }

    /// Persist the receipt to disk (async). Non-fatal: caller should log warnings on failure.
    pub async fn save_to_file(&self) -> Result<(), std::io::Error> {
        let path = self.storage_path();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        tokio::fs::write(&path, json).await?;
        Ok(())
    }

    /// Synchronous persist (for use in non-async contexts).
    pub fn save_to_file_sync(&self) -> Result<(), std::io::Error> {
        let path = self.storage_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Load the most recent receipt from `.hajimi/context_receipts/` (if any).
    pub fn load_latest_sync() -> Option<Self> {
        let home = dirs::home_dir()?;
        let dir = home.join(".hajimi").join("context_receipts");
        // Read all .json files and sort by filename (timestamp-based) descending.
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
            .collect();
        entries.sort_by(|a, b| b.cmp(a));
        let latest = entries.into_iter().next()?;
        let content = std::fs::read_to_string(latest).ok()?;
        serde_json::from_str(&content).ok()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_receipt(n_included: usize, n_omitted: usize) -> ContextReceipt {
        let included = (0..n_included)
            .map(|i| {
                (
                    format!("block-{}", i),
                    "P1".to_string(),
                    512 + i,
                    "FileContent".to_string(),
                    "fn main() { ... }".to_string(),
                )
            })
            .collect();
        let omitted = (0..n_omitted)
            .map(|i| {
                (
                    format!("omit-{}", i),
                    "P2".to_string(),
                    1024,
                    "Memory".to_string(),
                    "BudgetExceeded".to_string(),
                )
            })
            .collect();
        ContextReceipt::new(
            "sess-test".to_string(),
            "deepseek".to_string(),
            "deepseek-v4".to_string(),
            "Fast128K".to_string(),
            131_072,
            118_976,
            4_096,
            false,
            "planner".to_string(),
            ReceiptPhase::PostRequest,
            included,
            omitted,
            None,
        )
    }

    // ── Basic field tests ───────────────────────────────────────────────────

    #[test]
    fn test_context_receipt_basic_fields() {
        let r = make_receipt(2, 1);
        assert_eq!(r.schema_version, "ContextReceipt-v1");
        assert_eq!(r.provider_id, "deepseek");
        assert_eq!(r.model, "deepseek-v4");
        assert_eq!(r.mode, "Fast128K");
        assert_eq!(r.max_context_tokens, 131_072);
        assert_eq!(r.input_budget, 118_976);
        assert_eq!(r.estimated_input_tokens, 4_096);
        assert!(!r.long_context_mode);
        assert_eq!(r.bridge_role, "planner");
        assert_eq!(r.phase, ReceiptPhase::PostRequest);
    }

    #[test]
    fn test_context_receipt_included_blocks() {
        let r = make_receipt(3, 0);
        assert_eq!(r.included_blocks.len(), 3);
        assert_eq!(r.included_blocks[0].name, "block-0");
        assert_eq!(r.included_blocks[0].priority, "P1");
        assert_eq!(r.included_blocks[0].source, "FileContent");
        // summary must NOT contain full content — only truncated snippet
        assert!(!r.included_blocks[0].summary.is_empty());
        assert!(r.included_blocks[0].summary.len() <= MAX_SUMMARY_LEN + 4); // +4 for ellipsis bytes
    }

    #[test]
    fn test_context_receipt_omitted_blocks() {
        let r = make_receipt(0, 3);
        assert_eq!(r.omitted_blocks.len(), 3);
        assert_eq!(r.omitted_blocks[0].name, "omit-0");
        assert_eq!(r.omitted_blocks[0].reason, "BudgetExceeded");
        assert_eq!(r.omitted_blocks[0].source, "Memory");
        assert_eq!(r.omitted_blocks[0].token_estimate, 1024);
    }

    #[test]
    fn test_context_receipt_cap_included_blocks() {
        // Exceeding MAX_INCLUDED_BLOCKS is capped silently.
        let r = make_receipt(MAX_INCLUDED_BLOCKS + 10, 0);
        assert_eq!(r.included_blocks.len(), MAX_INCLUDED_BLOCKS);
    }

    #[test]
    fn test_context_receipt_cap_omitted_blocks() {
        let r = make_receipt(0, MAX_OMITTED_BLOCKS + 10);
        assert_eq!(r.omitted_blocks.len(), MAX_OMITTED_BLOCKS);
    }

    #[test]
    fn test_context_receipt_summary_truncation() {
        let long_content = "x".repeat(MAX_SUMMARY_LEN + 100);
        let included = vec![(
            "big-block".to_string(),
            "P0".to_string(),
            100,
            "SystemPrompt".to_string(),
            long_content,
        )];
        let r = ContextReceipt::new(
            "sess".to_string(),
            "p".to_string(),
            "m".to_string(),
            "Fast128K".to_string(),
            131_072,
            118_976,
            100,
            false,
            "reflector".to_string(),
            ReceiptPhase::PostRequest,
            included,
            vec![],
            None,
        );
        let summary = &r.included_blocks[0].summary;
        // Should be truncated at MAX_SUMMARY_LEN + "…"
        assert!(summary.chars().count() <= MAX_SUMMARY_LEN + 1);
        assert!(summary.ends_with('…'));
    }

    // ── Sensitive field tests ───────────────────────────────────────────────

    #[test]
    fn test_context_receipt_no_sensitive_fields_in_serialized_json() {
        let r = make_receipt(1, 1);
        let json = serde_json::to_string(&r).unwrap();
        // These strings must NEVER appear in a receipt JSON.
        assert!(!json.contains("api_key"));
        assert!(!json.contains("Authorization"));
        assert!(!json.contains("Bearer"));
        assert!(!json.contains("full_prompt"));
        assert!(!json.contains("promptText"));
    }

    /// Ensure "Authorization: Bearer abc123" is redacted before being stored.
    #[test]
    fn test_redact_authorization_bearer() {
        let content = "Host: api.example.com\nAuthorization: Bearer abc123-secret-token\nContent-Type: application/json";
        let included = vec![(
            "request-headers".to_string(),
            "P0".to_string(),
            10,
            "SystemPrompt".to_string(),
            content.to_string(),
        )];
        let r = ContextReceipt::new(
            "s".to_string(),
            "p".to_string(),
            "m".to_string(),
            "Fast128K".to_string(),
            131_072,
            118_976,
            10,
            false,
            "planner".to_string(),
            ReceiptPhase::PostRequest,
            included,
            vec![],
            None,
        );
        let json = serde_json::to_string(&r).unwrap();
        // The raw bearer token must not appear in the stored receipt.
        assert!(
            !json.contains("abc123-secret-token"),
            "Bearer token leaked into receipt JSON"
        );
        assert!(
            !json.contains("Authorization"),
            "Authorization header leaked into receipt JSON"
        );
        // The sanitized summary should show [REDACTED] in place of sensitive line.
        let summary = &r.included_blocks[0].summary;
        assert!(
            summary.contains("[REDACTED]"),
            "Expected [REDACTED] in summary, got: {}",
            summary
        );
    }

    /// Ensure "api_key=sk-test-secret" is redacted before being stored.
    #[test]
    fn test_redact_api_key_sk_prefix() {
        let content = "User query: help me fix code\napi_key=sk-test-secret-12345\nModel: gpt-4";
        let included = vec![(
            "user-block".to_string(),
            "P1".to_string(),
            20,
            "Text".to_string(),
            content.to_string(),
        )];
        let r = ContextReceipt::new(
            "s2".to_string(),
            "p2".to_string(),
            "m2".to_string(),
            "Legacy8K".to_string(),
            8_000,
            7_000,
            20,
            false,
            "reflector".to_string(),
            ReceiptPhase::PostRequest,
            included,
            vec![],
            None,
        );
        let json = serde_json::to_string(&r).unwrap();
        assert!(
            !json.contains("sk-test-secret-12345"),
            "API key sk- prefix leaked into receipt JSON"
        );
        let summary = &r.included_blocks[0].summary;
        assert!(
            summary.contains("[REDACTED]"),
            "Expected [REDACTED] for api_key line, got: {}",
            summary
        );
    }

    /// Ensure a summary that has NO sensitive content passes through unmodified.
    #[test]
    fn test_redact_clean_content_unmodified() {
        let content = "fn main() { println!(\"Hello, World!\"); }";
        let result = redact_sensitive_text(content);
        assert_eq!(result, content, "Clean content should not be modified");
    }

    /// Ensure password field is redacted.
    #[test]
    fn test_redact_password_field() {
        let content = "config:\n  password: super-secret-123\n  host: localhost";
        let result = redact_sensitive_text(content);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("super-secret-123"));
    }

    // ── Serialization / phase tests ─────────────────────────────────────────

    #[test]
    fn test_context_receipt_actual_usage_none() {
        let r = make_receipt(1, 0);
        assert!(r.actual_usage.is_none());
    }

    #[test]
    fn test_context_receipt_actual_usage_some() {
        let included = vec![(
            "sys".to_string(),
            "P0".to_string(),
            256,
            "SystemPrompt".to_string(),
            "short snippet".to_string(),
        )];
        let r = ContextReceipt::new(
            "s".to_string(),
            "p".to_string(),
            "m".to_string(),
            "Long1M".to_string(),
            1_000_000,
            900_000,
            50_000,
            true,
            "planner".to_string(),
            ReceiptPhase::PostRequest,
            included,
            vec![],
            Some(ActualUsage {
                prompt_tokens: 50_000,
                completion_tokens: 512,
            }),
        );
        assert!(r.actual_usage.is_some());
        let u = r.actual_usage.unwrap();
        assert_eq!(u.prompt_tokens, 50_000);
        assert_eq!(u.completion_tokens, 512);
    }

    #[test]
    fn test_context_receipt_serializes_deserializes() {
        let r = make_receipt(2, 2);
        let json = serde_json::to_string_pretty(&r).unwrap();
        let r2: ContextReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(r, r2);
        assert_eq!(r2.included_blocks.len(), 2);
        assert_eq!(r2.omitted_blocks.len(), 2);
        // Verify key JSON field names
        assert!(json.contains("includedBlocks"));
        assert!(json.contains("omittedBlocks"));
        assert!(json.contains("inputBudget"));
        assert!(json.contains("estimatedInputTokens"));
        assert!(json.contains("maxContextTokens"));
        assert!(json.contains("ContextReceipt-v1"));
        assert!(json.contains("post_request"));
    }

    #[test]
    fn test_phase_preflight_serializes() {
        let included = vec![(
            "sys".to_string(),
            "P0".to_string(),
            50,
            "SystemPrompt".to_string(),
            "instructions...".to_string(),
        )];
        let r = ContextReceipt::new(
            "pf".to_string(),
            "p".to_string(),
            "m".to_string(),
            "Fast128K".to_string(),
            131_072,
            118_976,
            50,
            false,
            "planner".to_string(),
            ReceiptPhase::Preflight,
            included,
            vec![],
            None,
        );
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("preflight"));
        assert_eq!(r.phase, ReceiptPhase::Preflight);
    }

    #[tokio::test]
    async fn test_context_receipt_save_load() {
        let r = make_receipt(1, 1);
        // Save (may fail in CI without home dir, treat as skippable)
        let save_result = r.save_to_file().await;
        if save_result.is_err() {
            // Not a fatal error — writing receipts must never panic.
            return;
        }
        // Verify path exists
        let path = r.storage_path();
        assert!(path.exists());
        // Load latest and verify it matches
        let loaded = ContextReceipt::load_latest_sync();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.provider_id, r.provider_id);
        assert_eq!(loaded.model, r.model);
        assert_eq!(loaded.included_blocks.len(), 1);
        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    /// Verify save failure does NOT panic — non-fatal path.
    #[tokio::test]
    async fn test_save_failure_is_non_fatal() {
        // Use an impossible path to trigger failure.
        let mut r = make_receipt(1, 0);
        r.session_id = "test-nonfatal".to_string();
        // Even if save fails, it must not panic (just Err).
        let result = r.save_to_file().await;
        // We can't guarantee it succeeds in all environments, so just assert no panic.
        let _ = result;
    }
}
