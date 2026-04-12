//! Anthropic Claude API Client - SSE Streaming

use async_trait::async_trait;
use futures::TryStreamExt;
use reqwest::{Client, header};
use serde_json::json;
use tokio::sync::mpsc::Sender;

use crate::error::EngineError;
use crate::llm::{LlmClient, LlmProvider};
use crate::streaming::{ChannelStream, StreamChunk};

pub struct AnthropicClient {
    provider: LlmProvider,
    client: Client,
    timeout_ms: u64,
}

impl AnthropicClient {
    pub fn new(provider: LlmProvider) -> Self {
        Self { provider, client: Client::new(), timeout_ms: 30_000 }
    }
    pub fn from_env() -> Result<Self, EngineError> {
        Ok(Self::new(LlmProvider::anthropic_from_env()?))
    }
    pub fn with_timeout(mut self, t: u64) -> Self { self.timeout_ms = t; self }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn stream_chat(&self, prompt: String) -> Result<ChannelStream, EngineError> {
        let (stream, tx) = ChannelStream::new(100);
        let (key, model, url) = match &self.provider {
            LlmProvider::Anthropic { api_key, model, base_url } => (api_key.clone(), model.clone(), base_url.clone()),
            _ => return Err(EngineError::InvalidParameters("bad provider".into())),
        };
        let body = json!({"model": model, "messages": [{"role": "user", "content": prompt}], "stream": true, "max_tokens": 4096});
        let client = self.client.clone();
        let timeout = std::time::Duration::from_millis(self.timeout_ms);
        tokio::spawn(async move {
            match client.post(format!("{}/v1/messages", url)).header("x-api-key", key).header("anthropic-version", "2023-06-01").header(header::CONTENT_TYPE, "application/json").timeout(timeout).body(body.to_string()).send().await {
                Ok(r) if r.status().is_success() => {
                    match r.bytes_stream().try_collect::<Vec<_>>().await {
                        Ok(chunks) => { let _ = parse_sse(&chunks.concat(), &tx).await; }
                        Err(e) => { let _ = tx.send(StreamChunk::Error(e.to_string())).await; }
                    }
                }
                Ok(r) => { let _ = tx.send(StreamChunk::Error(format!("HTTP {}", r.status()))).await; }
                Err(e) => { let _ = tx.send(StreamChunk::Error(e.to_string())).await; }
            }
            let _ = tx.send(StreamChunk::Done).await;
        });
        Ok(stream)
    }
    fn provider(&self) -> &LlmProvider { &self.provider }
    fn timeout_ms(&self) -> u64 { self.timeout_ms }
}

async fn parse_sse(data: &[u8], tx: &Sender<StreamChunk>) {
    for line in String::from_utf8_lossy(data).lines() {
        if let Some(json) = line.strip_prefix("data: ") {
            if json == "[DONE]" { break; }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
                if let Some(t) = v.get("delta").and_then(|d| d.get("text")).and_then(|t| t.as_str()) {
                    tx.send(StreamChunk::Output(t.to_string())).await.ok();
                }
            }
        }
    }
}
