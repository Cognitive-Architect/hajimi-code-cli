//! Ollama Local LLM Client
use crate::EngineError;
use crate::{LlmClient, LlmProvider};
use crate::streaming::{ChannelStream, StreamChunk};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Ollama local LLM client
pub struct OllamaClient { provider: LlmProvider, timeout_ms: u64 }
impl OllamaClient {
    pub fn new(provider: LlmProvider) -> Self { Self { provider, timeout_ms: 60_000 } }
    pub fn default_local() -> Self { Self::new(LlmProvider::ollama_default()) }
    pub fn with_timeout(mut self, t: u64) -> Self { self.timeout_ms = t; self }
}
#[derive(Serialize)] struct GenReq { model: String, prompt: String, stream: bool }
#[derive(Deserialize)] struct GenResp { response: Option<String>, done: bool }

#[async_trait]
impl LlmClient for OllamaClient {
    async fn stream_chat(&self, prompt: String) -> Result<ChannelStream, EngineError> {
        let (stream, tx) = ChannelStream::new(100);
        let LlmProvider::Ollama { base_url, model } = &self.provider else {
            return Err(EngineError::InvalidParameters("Invalid".into()));
        };
        let client = Client::new();
        let url = format!("{}/api/generate", base_url);
        let req = GenReq { model: model.clone(), prompt, stream: true };
        tokio::spawn(async move {
            match client.post(&url).json(&req).send().await {
                Ok(r) => { let mut s = r.bytes_stream();
                    while let Some(Ok(d)) = s.next().await {
                        for l in String::from_utf8_lossy(&d).lines().filter(|l| !l.is_empty()) {
                            if let Ok(r) = serde_json::from_str::<GenResp>(l) {
                                if let Some(t) = r.response { tx.send(StreamChunk::Output(t)).await.ok(); }
                                if r.done { tx.send(StreamChunk::Done).await.ok(); }
                            }
                        }
                    }
                }
                Err(e) => { tx.send(StreamChunk::Error(e.to_string())).await.ok(); }
            }
        });
        Ok(stream)
    }
    fn provider(&self) -> &LlmProvider { &self.provider }
    fn timeout_ms(&self) -> u64 { self.timeout_ms }
}
