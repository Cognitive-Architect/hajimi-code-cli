//! LLM Client Module - Week 3 QueryEngine v1.0
//!
//! Provides unified interface for multiple LLM providers:
//! - Anthropic Claude (cloud)
//! - OpenAI GPT (cloud)
//! - Ollama (local)
//!
//! DEBT-W03-001: [CLEARED 2026-04-03] Manual Debug impl to redact api_key

mod error;
mod streaming;
use async_trait::async_trait;
pub use error::EngineError;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::env;
pub use streaming::channel_stream::ChannelStream;
pub use streaming::StreamChunk;

/// LLM provider enumeration
///
/// Note: Debug is manually implemented to prevent api_key leakage
/// Clone is manually implemented to preserve field-level cloning
pub enum LlmProvider {
    /// Anthropic Claude API
    Anthropic {
        api_key: SecretString,
        model: String,
        base_url: String,
    },
    /// OpenAI GPT API
    OpenAi {
        api_key: SecretString,
        model: String,
        base_url: String,
    },
    /// Ollama local LLM
    Ollama { base_url: String, model: String },
}

/// Manual Debug implementation to redact sensitive api_key fields
///
/// Security: api_key (now SecretString) is displayed as ***REDACTED*** to prevent accidental logging
impl std::fmt::Debug for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic {
                api_key: _,
                model,
                base_url,
            } => f
                .debug_struct("Anthropic")
                .field("api_key", &"***REDACTED***")
                .field("model", model)
                .field("base_url", base_url)
                .finish(),
            Self::OpenAi {
                api_key: _,
                model,
                base_url,
            } => f
                .debug_struct("OpenAi")
                .field("api_key", &"***REDACTED***")
                .field("model", model)
                .field("base_url", base_url)
                .finish(),
            Self::Ollama { base_url, model } => f
                .debug_struct("Ollama")
                .field("base_url", base_url)
                .field("model", model)
                .finish(),
        }
    }
}

/// Manual Clone implementation to preserve field-level cloning behavior.
/// Now uses SecretString for api_key (which implements Clone safely).
impl Clone for LlmProvider {
    fn clone(&self) -> Self {
        match self {
            Self::Anthropic {
                api_key,
                model,
                base_url,
            } => Self::Anthropic {
                api_key: api_key.clone(),
                model: model.clone(),
                base_url: base_url.clone(),
            },
            Self::OpenAi {
                api_key,
                model,
                base_url,
            } => Self::OpenAi {
                api_key: api_key.clone(),
                model: model.clone(),
                base_url: base_url.clone(),
            },
            Self::Ollama { base_url, model } => Self::Ollama {
                base_url: base_url.clone(),
                model: model.clone(),
            },
        }
    }
}

impl LlmProvider {
    /// Create Anthropic provider from environment
    pub fn anthropic_from_env() -> Result<Self, EngineError> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| EngineError::InvalidParameters("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self::Anthropic {
            api_key: SecretString::new(api_key.into_boxed_str()),
            model: "claude-3-sonnet-20240229".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
        })
    }

    /// Create OpenAI provider from environment
    pub fn openai_from_env() -> Result<Self, EngineError> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| EngineError::InvalidParameters("OPENAI_API_KEY not set".to_string()))?;
        Ok(Self::OpenAi {
            api_key: SecretString::new(api_key.into_boxed_str()),
            model: "gpt-4".to_string(),
            base_url: "https://api.openai.com".to_string(),
        })
    }

    /// Create Ollama provider (local default)
    pub fn ollama_default() -> Self {
        Self::Ollama {
            base_url: "http://localhost:11434".to_string(),
            model: "llama3".to_string(),
        }
    }
}

/// Chat message for multi-turn conversations.
/// Compatible with OpenAI/Anthropic/Ollama chat format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<u64>,
}

/// Token usage reported by the LLM provider.
///
/// Populated after a streaming call completes by parsing the provider-specific
/// `usage` payload (OpenAI `usage`, Anthropic `message_start`/`message_delta`,
/// Ollama `prompt_eval_count`/`eval_count`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

/// Token-level event for real-time streaming consumers (B-09/12).
/// Each chunk from the LLM provider maps to one TokenEvent.
/// Front-end consumers can subscribe to these events for incremental
/// parsing of thinking tags without waiting for the full response.
#[derive(Debug, Clone)]
pub struct TokenEvent {
    pub text: String,
    pub timestamp_ms: u64,
}

/// Unified LLM client trait
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Stream chat completion with a single prompt (backward compatible).
    async fn stream_chat(&self, prompt: String) -> Result<ChannelStream, EngineError>;

    /// Stream chat completion with multi-turn message context.
    /// `messages` should include the full conversation history.
    /// `system_prompt` is optional and will be sent as a system message
    /// (prepended to messages for OpenAI, top-level system param for Anthropic).
    async fn stream_chat_with_context(
        &self,
        messages: Vec<ChatMessage>,
        system_prompt: Option<String>,
    ) -> Result<ChannelStream, EngineError>;

    /// Get provider type
    fn provider(&self) -> &LlmProvider;

    /// Get timeout configuration (default 30s)
    fn timeout_ms(&self) -> u64 {
        30_000
    }

    /// Count the exact number of tokens in the given messages.
    ///
    /// When the `exact-tokens` feature is enabled, uses `tiktoken-rs` with the
    /// appropriate tokenizer for the given model. Falls back to a heuristic
    /// estimator (Chinese ≈ 1 token/char, English ≈ 1.3 tokens/word) when the
    /// feature is disabled.
    ///
    /// # Arguments
    /// * `messages` — Conversation history as a vector of `ChatMessage`
    /// * `model` — Model identifier (e.g. "gpt-4", "claude-3-sonnet", "llama3")
    ///
    /// # Returns
    /// `Result<usize, EngineError>` — Token count or an error if the model is unsupported
    fn count_tokens(&self, messages: Vec<ChatMessage>, model: &str) -> Result<usize, EngineError>;

    /// Retrieve the token usage from the most recent streaming call.
    ///
    /// Returns `None` if the provider did not include usage data in the
    /// response (e.g. older API versions or unsupported local models).
    fn last_usage(&self) -> Option<Usage>;
}

/// Convert internal `ChatMessage` to tiktoken-rs format for exact counting.
#[cfg(feature = "exact-tokens")]
fn to_tiktoken_messages(
    messages: &[ChatMessage],
) -> Vec<tiktoken_rs::ChatCompletionRequestMessage> {
    messages
        .iter()
        .map(|m| tiktoken_rs::ChatCompletionRequestMessage {
            role: m.role.clone(),
            content: Some(m.content.clone()),
            name: None,
            function_call: None,
            tool_calls: vec![],
            refusal: None,
        })
        .collect()
}

/// Normalize a model name to one recognized by tiktoken-rs.
/// Maps Claude and most open-source models to "gpt-4" (cl100k_base),
/// passes through OpenAI model names as-is.
pub fn normalize_model_for_tiktoken(model: &str) -> String {
    let lower = model.to_lowercase();
    if lower.contains("claude") {
        "gpt-4".to_string()
    } else if lower.contains("gpt-4") || lower.contains("gpt-3.5") || lower.contains("gpt-oss") {
        model.to_string()
    } else {
        "gpt-4".to_string()
    }
}

/// Heuristic token estimation (fallback when `exact-tokens` is disabled).
/// Based on the frontend `estimateTokens()` algorithm with empirical overhead
/// adjustments to approximate tiktoken-rs behavior:
/// - Chinese characters (\u4e00-\u9fff): ~0.9 tokens each
/// - English words: ~1.0 token each (short) / ~1.3 tokens each (long)
/// - Per-message overhead: 3 tokens (role framing)
/// - Reply priming: 3 tokens (when messages non-empty)
pub fn heuristic_token_count(messages: &[ChatMessage]) -> usize {
    let text: String = messages
        .iter()
        .map(|m| format!("{}: {}\n", m.role, m.content))
        .collect();
    let chinese = text
        .chars()
        .filter(|&c| ('\u{4e00}'..='\u{9fff}').contains(&c))
        .count();
    let english = text.split_whitespace().count();
    let coefficient = if english <= 5 { 1.0 } else { 1.3 };
    let base =
        (chinese as f64 * 0.9).ceil() as usize + (english as f64 * coefficient).ceil() as usize;
    let overhead = messages.len() * 3 + 3;
    base + overhead
}

pub mod anthropic;
pub mod ollama;
pub mod openai;

pub use anthropic::AnthropicClient;
pub use ollama::OllamaClient;
pub use openai::OpenAiClient;

// Re-export reqwest::Client for downstream consumers (e.g. desktop validate_provider)
/// # Safety: reqwest::Client is safe to clone and share across tasks
pub use reqwest::Client;

#[cfg(test)]
mod token_tests;
