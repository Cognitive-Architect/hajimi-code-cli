//! OpenAI GPT API Client
use crate::EngineError;
use crate::{LlmClient, LlmProvider};
use crate::streaming::{ChannelStream, StreamChunk};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// OpenAI GPT client
pub struct OpenAiClient { provider: LlmProvider, timeout_ms: u64 }
impl OpenAiClient {
    pub fn new(provider: LlmProvider) -> Self { Self { provider, timeout_ms: 30_000 } }
    pub fn from_env() -> Result<Self, EngineError> { Ok(Self::new(LlmProvider::openai_from_env()?)) }
    pub fn with_timeout(mut self, t: u64) -> Self { self.timeout_ms = t; self }
}
#[derive(Serialize)] struct ChatMessage { role: String, content: String }
#[derive(Serialize)] struct ChatRequest { model: String, messages: Vec<ChatMessage>, stream: bool }
#[derive(Deserialize)] struct Delta { content: Option<String> }
#[derive(Deserialize)] struct Choice { delta: Delta }
#[derive(Deserialize)] struct StreamResp { choices: Vec<Choice> }

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn stream_chat(&self, prompt: String) -> Result<ChannelStream, EngineError> {
        let (stream, tx) = ChannelStream::new(100);
        let LlmProvider::OpenAi { api_key, model, base_url } = &self.provider else {
            return Err(EngineError::InvalidParameters("Invalid".into()));
        };
        let client = Client::new();
        let url = format!("{}/v1/chat/completions", base_url);
        let req = ChatRequest { model: model.clone(), messages: vec![
            ChatMessage { role: "user".into(), content: prompt }
        ], stream: true };
        let key = api_key.clone();
        tokio::spawn(async move {
            match client.post(&url).header("Authorization", format!("Bearer {}", key))
                .json(&req).send().await {
                Ok(r) => { let mut s = r.bytes_stream();
                    while let Some(Ok(d)) = s.next().await {
                        for l in String::from_utf8_lossy(&d).lines() {
                            if let Some(j) = l.strip_prefix("data: ") {
                                if j == "[DONE]" { tx.send(StreamChunk::Done).await.ok(); }
                                else if let Ok(r) = serde_json::from_str::<StreamResp>(j) {
                                    if let Some(c) = r.choices.first().and_then(|c| c.delta.content.clone()) {
                                        tx.send(StreamChunk::Output(c)).await.ok();
                                    }
                                }
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
