//! OpenAI GPT API Client
use crate::EngineError;
use crate::{LlmClient, LlmProvider};
use crate::streaming::{ChannelStream, StreamChunk};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

/// OpenAI GPT client
pub struct OpenAiClient { provider: LlmProvider, timeout_ms: u64 }
impl OpenAiClient {
    pub fn new(provider: LlmProvider) -> Self { Self { provider, timeout_ms: 30_000 } }
    pub fn from_env() -> Result<Self, EngineError> { Ok(Self::new(LlmProvider::openai_from_env()?)) }
    pub fn with_timeout(mut self, t: u64) -> Self { self.timeout_ms = t; self }
}
#[derive(Serialize)] struct ChatRequest { model: String, messages: Vec<crate::ChatMessage>, stream: bool }
#[derive(Deserialize)] struct Delta { content: Option<String> }
#[derive(Deserialize)] struct Choice { delta: Delta }
#[derive(Deserialize)] struct StreamResp { choices: Vec<Choice> }

#[async_trait]
impl LlmClient for OpenAiClient {
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
        let (api_key_secret, model, base_url) = match &self.provider {
            LlmProvider::OpenAi { api_key, model, base_url } => (api_key.clone(), model.clone(), base_url.clone()),
            _ => return Err(EngineError::InvalidParameters("Invalid provider type".into())),
        };
        let mut msgs = messages;
        if let Some(system) = system_prompt {
            msgs.insert(0, crate::ChatMessage { role: "system".into(), content: system, timestamp: None });
        }
        let client = Client::new();
        let url = format!("{}/v1/chat/completions", base_url);
        let req = ChatRequest { model: model.clone(), messages: msgs, stream: true };
        let key = api_key_secret.expose_secret().to_string();
        tokio::spawn(async move {
            match client.post(&url).header("Authorization", format!("Bearer {}", key))
                .json(&req).send().await {
                Ok(r) => {
                    let status = r.status();
                    if status == 401 || status == 403 || status == 429 {
                        let err_msg = match status.as_u16() {
                            401 => "API Key 无效或已过期，请检查配置 (401)".to_string(),
                            403 => "API Key 权限不足，请检查配置 (403)".to_string(),
                            429 => "请求过于频繁，请稍后再试 (429)".to_string(),
                            _ => format!("HTTP 错误: {}", status),
                        };
                        let _ = tx.send(StreamChunk::Error(err_msg)).await;
                    } else if !status.is_success() {
                        let _ = tx.send(StreamChunk::Error(format!("HTTP 错误: {}", status))).await;
                    } else {
                        let mut s = r.bytes_stream();
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
                }
                Err(e) => { tx.send(StreamChunk::Error(e.to_string())).await.ok(); }
            }
        });
        Ok(stream)
    }

    fn provider(&self) -> &LlmProvider { &self.provider }
    fn timeout_ms(&self) -> u64 { self.timeout_ms }
}
