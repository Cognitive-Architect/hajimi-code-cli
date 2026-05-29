//! Ollama Local LLM Client
use crate::streaming::{ChannelStream, StreamChunk};
use crate::EngineError;
use crate::{LlmClient, LlmProvider, Usage};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Ollama local LLM client
pub struct OllamaClient {
    provider: LlmProvider,
    timeout_ms: u64,
    last_usage: Arc<Mutex<Option<Usage>>>,
}
impl OllamaClient {
    pub fn new(provider: LlmProvider) -> Self {
        Self {
            provider,
            timeout_ms: 60_000,
            last_usage: Arc::new(Mutex::new(None)),
        }
    }
    pub fn default_local() -> Self {
        Self::new(LlmProvider::ollama_default())
    }
    pub fn with_timeout(mut self, t: u64) -> Self {
        self.timeout_ms = t;
        self
    }
}
/// Ollama /api/chat request format.
#[derive(Serialize)]
struct ChatReq {
    model: String,
    messages: Vec<crate::ChatMessage>,
    stream: bool,
}
#[derive(Deserialize)]
struct ChatResp {
    message: Option<Msg>,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u64>,
    #[serde(default)]
    eval_count: Option<u64>,
}
#[derive(Deserialize)]
struct Msg {
    content: String,
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn stream_chat(&self, prompt: String) -> Result<ChannelStream, EngineError> {
        self.stream_chat_with_context(
            vec![crate::ChatMessage {
                role: "user".into(),
                content: prompt,
                timestamp: None,
            }],
            None,
        )
        .await
    }

    async fn stream_chat_with_context(
        &self,
        messages: Vec<crate::ChatMessage>,
        _system_prompt: Option<String>,
    ) -> Result<ChannelStream, EngineError> {
        let (stream, tx) = ChannelStream::new(100);
        let LlmProvider::Ollama { base_url, model } = &self.provider else {
            return Err(EngineError::InvalidParameters("Invalid".into()));
        };
        let client = Client::new();
        let url = format!("{}/api/chat", base_url);
        let req = ChatReq {
            model: model.clone(),
            messages,
            stream: true,
        };
        let usage_ref = self.last_usage.clone();
        tokio::spawn(async move {
            match client.post(&url).json(&req).send().await {
                Ok(r) => {
                    let mut s = r.bytes_stream();
                    while let Some(Ok(d)) = s.next().await {
                        for l in String::from_utf8_lossy(&d)
                            .lines()
                            .filter(|l| !l.is_empty())
                        {
                            if let Ok(resp) = serde_json::from_str::<ChatResp>(l) {
                                if let Some(msg) = resp.message {
                                    tx.send(StreamChunk::Output(msg.content)).await.ok();
                                }
                                if resp.done {
                                    if let (Some(p), Some(c)) =
                                        (resp.prompt_eval_count, resp.eval_count)
                                    {
                                        *usage_ref.lock().unwrap() = Some(Usage {
                                            prompt_tokens: p,
                                            completion_tokens: c,
                                        });
                                    }
                                    tx.send(StreamChunk::Done).await.ok();
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tx.send(StreamChunk::Error(e.to_string())).await.ok();
                }
            }
        });
        Ok(stream)
    }

    async fn stream_chat_with_tools(
        &self,
        messages: Vec<crate::ChatMessage>,
        system_prompt: Option<String>,
        tools: Vec<crate::ToolDefinition>,
        _tool_choice: crate::ToolChoiceMode,
    ) -> Result<ChannelStream, EngineError> {
        if !tools.is_empty() {
            log::warn!("OllamaClient does not natively support tool calling. Falling back to plain text dialogue.");
        }
        self.stream_chat_with_context(messages, system_prompt).await
    }

    fn provider(&self) -> &LlmProvider {
        &self.provider
    }
    fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    fn last_usage(&self) -> Option<Usage> {
        *self.last_usage.lock().unwrap()
    }

    fn count_tokens(
        &self,
        messages: Vec<crate::ChatMessage>,
        model: &str,
    ) -> Result<usize, crate::EngineError> {
        #[cfg(feature = "exact-tokens")]
        {
            let normalized = crate::normalize_model_for_tiktoken(model);
            let tiktoken_msgs = crate::to_tiktoken_messages(&messages);
            tiktoken_rs::num_tokens_from_messages(&normalized, &tiktoken_msgs).map_err(|e| {
                crate::EngineError::InvalidParameters(format!("Token count failed: {}", e))
            })
        }
        #[cfg(not(feature = "exact-tokens"))]
        {
            let _ = model;
            Ok(crate::heuristic_token_count(&messages))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_fallback_graceful() {
        // CF-01-5/08-001: 验证 Ollama 客户端在携带 tools 时能优雅降级回普通纯文本
        let provider = LlmProvider::ollama_default();
        let client = OllamaClient::new(provider);

        let tools = vec![crate::ToolDefinition {
            name: "test_tool".to_string(),
            description: "desc".to_string(),
            parameters: serde_json::json!({}),
        }];

        let res = client
            .stream_chat_with_tools(
                vec![crate::ChatMessage {
                    role: "user".to_string(),
                    content: "hello".to_string(),
                    timestamp: None,
                }],
                None,
                tools,
                crate::ToolChoiceMode::Auto,
            )
            .await;

        assert!(
            res.is_ok(),
            "Should gracefully return stream even with tools fallback"
        );
    }
}
