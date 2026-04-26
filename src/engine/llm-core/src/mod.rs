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
pub use error::EngineError;
pub use streaming::channel_stream::ChannelStream;
pub use streaming::StreamChunk;
use async_trait::async_trait;
use secrecy::SecretString;
use std::env;

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
    Ollama {
        base_url: String,
        model: String,
    },
}

/// Manual Debug implementation to redact sensitive api_key fields
///
/// Security: api_key (now SecretString) is displayed as ***REDACTED*** to prevent accidental logging
impl std::fmt::Debug for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic { api_key: _, model, base_url } => f
                .debug_struct("Anthropic")
                .field("api_key", &"***REDACTED***")
                .field("model", model)
                .field("base_url", base_url)
                .finish(),
            Self::OpenAi { api_key: _, model, base_url } => f
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
            Self::Anthropic { api_key, model, base_url } => Self::Anthropic {
                api_key: api_key.clone(),
                model: model.clone(),
                base_url: base_url.clone(),
            },
            Self::OpenAi { api_key, model, base_url } => Self::OpenAi {
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
            .map_err(|_| EngineError::InvalidParameters(
                "ANTHROPIC_API_KEY not set".to_string()
            ))?;
        Ok(Self::Anthropic {
            api_key: SecretString::new(api_key.into_boxed_str()),
            model: "claude-3-sonnet-20240229".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
        })
    }

    /// Create OpenAI provider from environment
    pub fn openai_from_env() -> Result<Self, EngineError> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| EngineError::InvalidParameters(
                "OPENAI_API_KEY not set".to_string()
            ))?;
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

/// Unified LLM client trait
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Stream chat completion
    async fn stream_chat(&self, prompt: String) -> Result<ChannelStream, EngineError>;
    
    /// Get provider type
    fn provider(&self) -> &LlmProvider;
    
    /// Get timeout configuration (default 30s)
    fn timeout_ms(&self) -> u64 {
        30_000
    }
}

pub mod anthropic;
pub mod ollama;
pub mod openai;

pub use anthropic::AnthropicClient;
pub use ollama::OllamaClient;
pub use openai::OpenAiClient;
