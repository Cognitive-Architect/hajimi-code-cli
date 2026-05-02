//! Anthropic Claude API Client - SSE Streaming

use async_trait::async_trait;
use futures::TryStreamExt;
use reqwest::{Client, header};
use secrecy::ExposeSecret;
use serde_json::json;
use tokio::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::EngineError;
use crate::{LlmClient, LlmProvider, Usage};
use crate::streaming::{ChannelStream, StreamChunk};

pub struct AnthropicClient {
    provider: LlmProvider,
    client: Client,
    timeout_ms: u64,
    last_usage: Arc<Mutex<Option<Usage>>>,
}

impl AnthropicClient {
    pub fn new(provider: LlmProvider) -> Self {
        Self { provider, client: Client::new(), timeout_ms: 30_000, last_usage: Arc::new(Mutex::new(None)) }
    }
    pub fn from_env() -> Result<Self, EngineError> {
        Ok(Self::new(LlmProvider::anthropic_from_env()?))
    }
    pub fn with_timeout(mut self, t: u64) -> Self { self.timeout_ms = t; self }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn stream_chat(&self, prompt: String) -> Result<ChannelStream, EngineError> {
        self.stream_chat_with_context(
            vec![crate::ChatMessage { role: "user".into(), content: prompt, timestamp: None }],
            None,
        ).await
    }

    async fn stream_chat_with_context(
        &self,
        messages: Vec<crate::ChatMessage>,
        system_prompt: Option<String>,
    ) -> Result<ChannelStream, EngineError> {
        let (stream, tx) = ChannelStream::new(100);
        let (api_key_secret, model, url) = match &self.provider {
            LlmProvider::Anthropic { api_key, model, base_url } => (api_key.clone(), model.clone(), base_url.clone()),
            _ => return Err(EngineError::InvalidParameters("bad provider".into())),
        };
        let msgs: Vec<serde_json::Value> = messages.into_iter()
            .map(|m| json!({"role": m.role, "content": m.content}))
            .collect();
        let mut body = json!({"model": model, "messages": msgs, "stream": true, "max_tokens": 4096});
        if let Some(system) = system_prompt {
            body["system"] = json!(system);
        }
        let client = self.client.clone();
        let timeout = std::time::Duration::from_millis(self.timeout_ms);
        let key = api_key_secret.expose_secret().to_string();
        let usage_ref = self.last_usage.clone();
        tokio::spawn(async move {
            match client.post(format!("{}/v1/messages", url)).header("x-api-key", &key).header("anthropic-version", "2023-06-01").header(header::CONTENT_TYPE, "application/json").timeout(timeout).body(body.to_string()).send().await {
                Ok(r) if r.status().is_success() => {
                    match r.bytes_stream().try_collect::<Vec<_>>().await {
                        Ok(chunks) => {
                            if let Some(usage) = parse_sse(&chunks.concat(), &tx).await {
                                *usage_ref.lock().unwrap() = Some(usage);
                            }
                        }
                        Err(e) => { let _ = tx.send(StreamChunk::Error(e.to_string())).await; }
                    }
                }
                Ok(r) => {
                    let status = r.status();
                    let err_msg = match status.as_u16() {
                        401 => "API Key 无效或已过期，请检查配置 (401)".to_string(),
                        403 => "API Key 权限不足，请检查配置 (403)".to_string(),
                        429 => "请求过于频繁，请稍后再试 (429)".to_string(),
                        _ => format!("HTTP 错误: {}", status),
                    };
                    let _ = tx.send(StreamChunk::Error(err_msg)).await;
                }
                Err(e) => { let _ = tx.send(StreamChunk::Error(e.to_string())).await; }
            }
            let _ = tx.send(StreamChunk::Done).await;
        });
        Ok(stream)
    }

    fn provider(&self) -> &LlmProvider { &self.provider }
    fn timeout_ms(&self) -> u64 { self.timeout_ms }

    fn last_usage(&self) -> Option<Usage> {
        *self.last_usage.lock().unwrap()
    }

    fn count_tokens(&self, messages: Vec<crate::ChatMessage>, model: &str) -> Result<usize, crate::EngineError> {
        #[cfg(feature = "exact-tokens")]
        {
            let normalized = crate::normalize_model_for_tiktoken(model);
            let tiktoken_msgs = crate::to_tiktoken_messages(&messages);
            tiktoken_rs::num_tokens_from_messages(&normalized, &tiktoken_msgs)
                .map_err(|e| crate::EngineError::InvalidParameters(format!("Token count failed: {}", e)))
        }
        #[cfg(not(feature = "exact-tokens"))]
        {
            let _ = model;
            Ok(crate::heuristic_token_count(&messages))
        }
    }
}

async fn parse_sse(data: &[u8], tx: &Sender<StreamChunk>) -> Option<Usage> {
    let mut prompt_tokens = 0u64;
    let mut completion_tokens = 0u64;
    for line in String::from_utf8_lossy(data).lines() {
        if let Some(json) = line.strip_prefix("data: ") {
            if json == "[DONE]" { break; }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
                let msg_type = v.get("type").and_then(|t| t.as_str());
                match msg_type {
                    Some("message_start") => {
                        if let Some(input) = v.get("message")
                            .and_then(|m| m.get("usage"))
                            .and_then(|u| u.get("input_tokens"))
                            .and_then(|n| n.as_u64()) {
                            prompt_tokens = input;
                        }
                    }
                    Some("message_delta") => {
                        if let Some(output) = v.get("usage")
                            .and_then(|u| u.get("output_tokens"))
                            .and_then(|n| n.as_u64()) {
                            completion_tokens = output;
                        }
                    }
                    _ => {
                        if let Some(t) = v.get("delta").and_then(|d| d.get("text")).and_then(|t| t.as_str()) {
                            tx.send(StreamChunk::Output(t.to_string())).await.ok();
                        }
                    }
                }
            }
        }
    }
    if prompt_tokens > 0 || completion_tokens > 0 {
        Some(Usage { prompt_tokens, completion_tokens })
    } else {
        None
    }
}
