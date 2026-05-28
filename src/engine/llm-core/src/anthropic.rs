//! Anthropic Claude API Client - SSE Streaming

use async_trait::async_trait;
use futures::TryStreamExt;
use reqwest::{header, Client};
use secrecy::ExposeSecret;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;

use crate::streaming::{ChannelStream, StreamChunk};
use crate::EngineError;
use crate::{LlmClient, LlmProvider, Usage};

pub struct AnthropicClient {
    provider: LlmProvider,
    client: Client,
    timeout_ms: u64,
    last_usage: Arc<Mutex<Option<Usage>>>,
}

impl AnthropicClient {
    pub fn new(provider: LlmProvider) -> Self {
        Self {
            provider,
            client: Client::new(),
            timeout_ms: 30_000,
            last_usage: Arc::new(Mutex::new(None)),
        }
    }
    pub fn from_env() -> Result<Self, EngineError> {
        Ok(Self::new(LlmProvider::anthropic_from_env()?))
    }
    pub fn with_timeout(mut self, t: u64) -> Self {
        self.timeout_ms = t;
        self
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
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
        system_prompt: Option<String>,
    ) -> Result<ChannelStream, EngineError> {
        self.stream_chat_with_tools(
            messages,
            system_prompt,
            Vec::new(),
            crate::ToolChoiceMode::None,
        )
        .await
    }

    async fn stream_chat_with_tools(
        &self,
        messages: Vec<crate::ChatMessage>,
        system_prompt: Option<String>,
        tools: Vec<crate::ToolDefinition>,
        tool_choice: crate::ToolChoiceMode,
    ) -> Result<ChannelStream, EngineError> {
        let (stream, tx) = ChannelStream::new(100);
        let (api_key_secret, model, url) = match &self.provider {
            LlmProvider::Anthropic {
                api_key,
                model,
                base_url,
            } => (api_key.clone(), model.clone(), base_url.clone()),
            _ => return Err(EngineError::InvalidParameters("bad provider".into())),
        };
        let msgs: Vec<serde_json::Value> = messages
            .into_iter()
            .map(|m| json!({"role": m.role, "content": m.content}))
            .collect();
        let mut body = json!({
            "model": model,
            "messages": msgs,
            "stream": true,
            "max_tokens": 4096,
        });
        if let Some(system) = system_prompt {
            body["system"] = json!(system);
        }

        // 注入 tools 与 tool_choice
        if !tools.is_empty() {
            let mut ant_tools = Vec::new();
            for t in tools {
                ant_tools.push(json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.parameters,
                }));
            }
            body["tools"] = json!(ant_tools);

            let ant_choice = match tool_choice {
                crate::ToolChoiceMode::Auto => json!({"type": "auto"}),
                crate::ToolChoiceMode::None => json!({"type": "auto"}),
                crate::ToolChoiceMode::Required(name) => json!({"type": "tool", "name": name}),
            };
            body["tool_choice"] = ant_choice;
        }

        // Outgoing trace 日志
        log::trace!("Anthropic outgoing messages body: {}", body);

        let client = self.client.clone();
        let timeout = std::time::Duration::from_millis(self.timeout_ms);
        let key = api_key_secret.expose_secret().to_string();
        let usage_ref = self.last_usage.clone();
        tokio::spawn(async move {
            match client
                .post(format!("{}/v1/messages", url))
                .header("x-api-key", &key)
                .header("anthropic-version", "2023-06-01")
                .header(header::CONTENT_TYPE, "application/json")
                .timeout(timeout)
                .body(body.to_string())
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => {
                    match r.bytes_stream().try_collect::<Vec<_>>().await {
                        Ok(chunks) => {
                            if let Some(usage) = parse_sse(&chunks.concat(), &tx).await {
                                *usage_ref.lock().unwrap() = Some(usage);
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(StreamChunk::Error(e.to_string())).await;
                        }
                    }
                }
                Ok(r) => {
                    let status = r.status();
                    let err_msg = match status.as_u16() {
                        400 => "请求格式错误 (400 - Bad Request)".to_string(),
                        401 => "API Key 无效或已过期，请检查配置 (401)".to_string(),
                        403 => "API Key 权限不足，请检查配置 (403)".to_string(),
                        429 => "请求过于频繁，请稍后再试 (429)".to_string(),
                        _ => format!("HTTP 错误: {}", status),
                    };
                    let _ = tx.send(StreamChunk::Error(err_msg)).await;
                }
                Err(e) => {
                    let _ = tx.send(StreamChunk::Error(e.to_string())).await;
                }
            }
            let _ = tx.send(StreamChunk::Done).await;
        });
        Ok(stream)
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

async fn parse_sse(data: &[u8], tx: &Sender<StreamChunk>) -> Option<Usage> {
    let mut prompt_tokens = 0u64;
    let mut completion_tokens = 0u64;

    struct AnthropicActiveToolCall {
        index: usize,
        id: String,
        arguments: String,
    }

    let mut active_tool_calls: Vec<AnthropicActiveToolCall> = Vec::new();

    for line in String::from_utf8_lossy(data).lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(':') {
            continue;
        }

        // 兼容以 "data:" 或 "data: " 头的 SSE line 格式
        let data_str = trimmed
            .strip_prefix("data:")
            .map(str::trim)
            .unwrap_or(trimmed);

        if data_str == "[DONE]" {
            break;
        }

        if let Ok(v) = serde_json::from_str::<serde_json::Value>(data_str) {
            let msg_type = v.get("type").and_then(|t| t.as_str());
            match msg_type {
                Some("message_start") => {
                    if let Some(input) = v
                        .get("message")
                        .and_then(|m| m.get("usage"))
                        .and_then(|u| u.get("input_tokens"))
                        .and_then(|n| n.as_u64())
                    {
                        prompt_tokens = input;
                    }
                }
                Some("message_delta") => {
                    if let Some(output) = v
                        .get("usage")
                        .and_then(|u| u.get("output_tokens"))
                        .and_then(|n| n.as_u64())
                    {
                        completion_tokens = output;
                    }
                }
                Some("content_block_start") => {
                    if let Some(block) = v.get("content_block") {
                        let block_type = block.get("type").and_then(|t| t.as_str());
                        if block_type == Some("tool_use") {
                            let id = block
                                .get("id")
                                .and_then(|i| i.as_str())
                                .unwrap_or("")
                                .to_string();
                            let name = block
                                .get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("")
                                .to_string();
                            if let Some(index_val) = v.get("index").and_then(|i| i.as_u64()) {
                                let idx = index_val as usize;
                                active_tool_calls.push(AnthropicActiveToolCall {
                                    index: idx,
                                    id: id.clone(),
                                    arguments: String::new(),
                                });
                                tx.send(StreamChunk::ToolCallStart {
                                    id: id.clone(),
                                    name: name.clone(),
                                })
                                .await
                                .ok();
                                log::trace!("Anthropic tool_use started: id={}, name={}", id, name);
                            }
                        }
                    }
                }
                Some("content_block_delta") => {
                    if let Some(delta_obj) = v.get("delta") {
                        let delta_type = delta_obj.get("type").and_then(|t| t.as_str());
                        match delta_type {
                            Some("text_delta") => {
                                if let Some(text) = delta_obj.get("text").and_then(|t| t.as_str()) {
                                    tx.send(StreamChunk::Output(text.to_string())).await.ok();
                                }
                            }
                            Some("input_json_delta") => {
                                if let Some(partial) =
                                    delta_obj.get("partial_json").and_then(|t| t.as_str())
                                {
                                    if let Some(index_val) = v.get("index").and_then(|i| i.as_u64())
                                    {
                                        let idx = index_val as usize;
                                        if let Some(call) =
                                            active_tool_calls.iter_mut().find(|c| c.index == idx)
                                        {
                                            call.arguments.push_str(partial);
                                            tx.send(StreamChunk::ToolCallArgumentsDelta {
                                                id: call.id.clone(),
                                                delta: partial.to_string(),
                                            })
                                            .await
                                            .ok();
                                            log::trace!(
                                                "Anthropic tool_use delta: id={}, delta={}",
                                                call.id,
                                                partial
                                            );
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Some("content_block_stop") => {
                    if let Some(index_val) = v.get("index").and_then(|i| i.as_u64()) {
                        let idx = index_val as usize;
                        if let Some(call) = active_tool_calls.iter().find(|c| c.index == idx) {
                            tx.send(StreamChunk::ToolCallEnd {
                                id: call.id.clone(),
                            })
                            .await
                            .ok();
                            log::trace!("Anthropic tool_use ended: id={}", call.id);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if prompt_tokens > 0 || completion_tokens > 0 {
        Some(Usage {
            prompt_tokens,
            completion_tokens,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_anthropic_stream_parsing() {
        // E2E-01-5/07-001: 完整覆盖 start/delta/stop 帧转换全流程的 Mock SSE 单元集成测试
        let mock_sse_data = "\
event: message_start
data: {\"type\": \"message_start\", \"message\": {\"id\": \"msg_123\", \"type\": \"message\", \"role\": \"assistant\", \"content\": [], \"model\": \"claude-3-opus-20240229\", \"usage\": {\"input_tokens\": 15}}}

event: content_block_start
data: {\"type\": \"content_block_start\", \"index\": 0, \"content_block\": {\"type\": \"tool_use\", \"id\": \"toolu_01A\", \"name\": \"get_weather\"}}

event: content_block_delta
data: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"input_json_delta\", \"partial_json\": \"{\\\"location\\\": \\\"San\"}}

event: content_block_delta
data: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"input_json_delta\", \"partial_json\": \" Francisco\\\"}\"}}

event: content_block_stop
data: {\"type\": \"content_block_stop\", \"index\": 0}

event: message_delta
data: {\"type\": \"message_delta\", \"usage\": {\"output_tokens\": 25}}
";

        let (tx, mut rx) = mpsc::channel(10);
        let usage = parse_sse(mock_sse_data.as_bytes(), &tx).await;

        // 验证 Usage 解析成功
        assert!(usage.is_some());
        let u = usage.unwrap();
        assert_eq!(u.prompt_tokens, 15);
        assert_eq!(u.completion_tokens, 25);

        // 验证 StreamChunk 的 ToolCall 系列事件转换
        let mut chunks = Vec::new();
        while let Ok(chunk) = rx.try_recv() {
            chunks.push(chunk);
        }

        // 1. ToolCallStart
        assert!(chunks.iter().any(|c| matches!(c, StreamChunk::ToolCallStart { id, name } if id == "toolu_01A" && name == "get_weather")));

        // 2. ToolCallArgumentsDelta 增量拼接
        assert!(chunks.iter().any(|c| matches!(c, StreamChunk::ToolCallArgumentsDelta { id, delta } if id == "toolu_01A" && delta == "{\"location\": \"San")));
        assert!(chunks.iter().any(|c| matches!(c, StreamChunk::ToolCallArgumentsDelta { id, delta } if id == "toolu_01A" && delta == " Francisco\"}")));

        // 3. ToolCallEnd
        assert!(chunks
            .iter()
            .any(|c| matches!(c, StreamChunk::ToolCallEnd { id } if id == "toolu_01A")));
    }
}
