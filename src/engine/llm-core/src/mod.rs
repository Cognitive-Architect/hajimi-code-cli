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

/// Build a chat completions URL for OpenAI-compatible providers.
///
/// Many compatible providers ask users to paste a versioned base URL such as
/// `https://api.example.com/v1` or `https://open.bigmodel.cn/api/paas/v4`.
/// Appending another `/v1` breaks those endpoints, so versioned base paths get
/// `/chat/completions` directly while host-only base URLs keep OpenAI's `/v1`.
pub fn openai_chat_completions_url(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        return base.to_string();
    }

    let last_segment = base.rsplit('/').next().unwrap_or_default();
    let versioned_base = last_segment.len() > 1
        && last_segment.starts_with('v')
        && last_segment[1..].chars().all(|c| c.is_ascii_digit());

    if versioned_base {
        format!("{}/chat/completions", base)
    } else {
        format!("{}/v1/chat/completions", base)
    }
}

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

/// Definition of a tool exposed to the LLM model.
///
/// Contains the schema information required by the model to perform
/// structured function calls (tool choice auto).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDefinition {
    /// The unique name of the tool.
    pub name: String,
    /// A description of what the tool does and when to use it.
    pub description: String,
    /// JSON Schema representing the tool's input parameters.
    pub parameters: serde_json::Value,
}

/// How the model is instructed/allowed to use tools in a request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    /// Model decides freely whether and which tools to call (default).
    Auto,
    /// Model must not call any tools.
    None,
    /// Model must call a specific tool.
    #[serde(untagged)]
    Required(String),
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

    /// Stream chat completion with support for tool definitions and choice.
    ///
    /// By default, falls back to `stream_chat_with_context` ignoring tools,
    /// allowing legacy or unadapted providers to compile and work without breaking.
    async fn stream_chat_with_tools(
        &self,
        messages: Vec<ChatMessage>,
        system_prompt: Option<String>,
        _tools: Vec<ToolDefinition>,
        _tool_choice: ToolChoiceMode,
    ) -> Result<ChannelStream, EngineError> {
        self.stream_chat_with_context(messages, system_prompt).await
    }

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

#[cfg(test)]
mod tests {
    use super::openai_chat_completions_url;

    #[test]
    fn openai_chat_url_appends_v1_for_host_only_base() {
        assert_eq!(
            openai_chat_completions_url("https://api.openai.com"),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn openai_chat_url_preserves_versioned_provider_base() {
        assert_eq!(
            openai_chat_completions_url("https://api.deepseek.com/v1"),
            "https://api.deepseek.com/v1/chat/completions"
        );
        assert_eq!(
            openai_chat_completions_url("https://open.bigmodel.cn/api/paas/v4"),
            "https://open.bigmodel.cn/api/paas/v4/chat/completions"
        );
    }

    #[test]
    fn openai_chat_url_keeps_explicit_endpoint() {
        assert_eq!(
            openai_chat_completions_url("https://api.example.test/v1/chat/completions"),
            "https://api.example.test/v1/chat/completions"
        );
    }

    #[test]
    fn test_stream_chunk_variants() {
        // FUNC-001: StreamChunk 能携带 ToolCallStart/ArgumentsDelta/ToolCallEnd 信息
        let start = crate::StreamChunk::ToolCallStart {
            id: "call_1".to_string(),
            name: "read_file".to_string(),
        };
        let delta = crate::StreamChunk::ToolCallArgumentsDelta {
            id: "call_1".to_string(),
            delta: "{\"path\":".to_string(),
        };
        let end = crate::StreamChunk::ToolCallEnd {
            id: "call_1".to_string(),
        };

        if let crate::StreamChunk::ToolCallStart { id, name } = start {
            assert_eq!(id, "call_1");
            assert_eq!(name, "read_file");
        } else {
            panic!("Expected ToolCallStart");
        }

        if let crate::StreamChunk::ToolCallArgumentsDelta { id, delta } = delta {
            assert_eq!(id, "call_1");
            assert_eq!(delta, "{\"path\":");
        } else {
            panic!("Expected ToolCallArgumentsDelta");
        }

        if let crate::StreamChunk::ToolCallEnd { id } = end {
            assert_eq!(id, "call_1");
        } else {
            panic!("Expected ToolCallEnd");
        }
    }

    #[test]
    fn test_tool_definition_serialization() {
        // FUNC-002: ToolDefinition 包含 name, description, parameters 属性
        // CONST-004: ToolDefinition 的 parameters 为 serde_json::Value 类型
        let def = crate::ToolDefinition {
            name: "test_tool".to_string(),
            description: "Test description".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "param": { "type": "string" }
                }
            }),
        };

        let serialized = serde_json::to_string(&def).unwrap();
        assert!(serialized.contains("test_tool"));
        assert!(serialized.contains("properties"));

        let deserialized: crate::ToolDefinition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.name, "test_tool");
        assert_eq!(
            deserialized.parameters["properties"]["param"]["type"],
            "string"
        );
    }

    #[test]
    fn test_tool_choice_mode_serialization() {
        // FUNC-003: ToolChoiceMode 适配并能序列化
        let auto = crate::ToolChoiceMode::Auto;
        let none = crate::ToolChoiceMode::None;
        let required = crate::ToolChoiceMode::Required("my_tool".to_string());

        assert_eq!(serde_json::to_string(&auto).unwrap(), "\"auto\"");
        assert_eq!(serde_json::to_string(&none).unwrap(), "\"none\"");
        assert_eq!(serde_json::to_string(&required).unwrap(), "\"my_tool\"");

        let des_auto: crate::ToolChoiceMode = serde_json::from_str("\"auto\"").unwrap();
        let des_none: crate::ToolChoiceMode = serde_json::from_str("\"none\"").unwrap();
        let des_required: crate::ToolChoiceMode = serde_json::from_str("\"my_tool\"").unwrap();

        assert_eq!(des_auto, auto);
        assert_eq!(des_none, none);
        assert_eq!(des_required, required);
    }
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
