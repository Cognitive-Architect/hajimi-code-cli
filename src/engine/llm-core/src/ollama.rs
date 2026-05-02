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
/// Ollama /api/chat request format.
#[derive(Serialize)] struct ChatReq { model: String, messages: Vec<crate::ChatMessage>, stream: bool }
#[derive(Deserialize)] struct ChatResp { message: Option<Msg>, done: bool }
#[derive(Deserialize)] struct Msg { content: String }

#[async_trait]
impl LlmClient for OllamaClient {
    async fn stream_chat(&self, prompt: String) -> Result<ChannelStream, EngineError> {
        self.stream_chat_with_context(
            vec![crate::ChatMessage { role: "user".into(), content: prompt, timestamp: None }],
            None,
        ).await
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
        let req = ChatReq { model: model.clone(), messages, stream: true };
        tokio::spawn(async move {
            match client.post(&url).json(&req).send().await {
                Ok(r) => {
                    let mut s = r.bytes_stream();
                    while let Some(Ok(d)) = s.next().await {
                        for l in String::from_utf8_lossy(&d).lines().filter(|l| !l.is_empty()) {
                            if let Ok(resp) = serde_json::from_str::<ChatResp>(l) {
                                if let Some(msg) = resp.message { tx.send(StreamChunk::Output(msg.content)).await.ok(); }
                                if resp.done { tx.send(StreamChunk::Done).await.ok(); }
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
