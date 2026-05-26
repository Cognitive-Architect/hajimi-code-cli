//! OpenAI GPT API Client
use crate::streaming::{ChannelStream, StreamChunk};
use crate::EngineError;
use crate::{LlmClient, LlmProvider, Usage};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// OpenAI GPT client
pub struct OpenAiClient {
    provider: LlmProvider,
    timeout_ms: u64,
    last_usage: Arc<Mutex<Option<Usage>>>,
}
impl OpenAiClient {
    pub fn new(provider: LlmProvider) -> Self {
        Self {
            provider,
            timeout_ms: 30_000,
            last_usage: Arc::new(Mutex::new(None)),
        }
    }
    pub fn from_env() -> Result<Self, EngineError> {
        Ok(Self::new(LlmProvider::openai_from_env()?))
    }
    pub fn with_timeout(mut self, t: u64) -> Self {
        self.timeout_ms = t;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{
        append_openai_sse_bytes, flush_openai_sse_state, parse_openai_sse_line, OpenAiSseState,
    };
    use crate::StreamChunk;

    fn parse_chunks(chunks: &[&[u8]]) -> Vec<StreamChunk> {
        let mut state = OpenAiSseState::default();
        let mut out = Vec::new();
        for chunk in chunks {
            for line in append_openai_sse_bytes(&mut state, chunk) {
                out.extend(parse_openai_sse_line(&mut state, &line));
            }
        }
        out.extend(flush_openai_sse_state(&mut state));
        out
    }

    #[test]
    fn openai_sse_parser_keeps_split_json_lines() {
        let out = parse_chunks(&[
            br#"data: {"choices":[{"delta":{"con"#,
            r#"tent":"你好"}}]}"#.as_bytes(),
            b"\n\n",
            b"data: [DONE]\n\n",
        ]);

        assert_eq!(
            out,
            vec![StreamChunk::Output("你好".to_string()), StreamChunk::Done]
        );
    }

    #[test]
    fn openai_sse_parser_maps_reasoning_content_to_thinking_tags() {
        let out = parse_chunks(&[
            r#"data: {"choices":[{"delta":{"reasoning_content":"先想一下"}}]}"#.as_bytes(),
            b"\n\n",
            r#"data: {"choices":[{"delta":{"content":"我是 DeepSeek。"}}]}"#.as_bytes(),
            b"\n\n",
            b"data: [DONE]\n\n",
        ]);

        assert_eq!(
            out,
            vec![
                StreamChunk::Output("<thinking>".to_string()),
                StreamChunk::Output("先想一下".to_string()),
                StreamChunk::Output("</thinking>".to_string()),
                StreamChunk::Output("我是 DeepSeek。".to_string()),
                StreamChunk::Done
            ]
        );
    }

    #[test]
    fn openai_sse_parser_surfaces_reasoning_only_streams() {
        let out = parse_chunks(&[
            r#"data: {"choices":[{"delta":{"reasoning_content":"只有推理"}}]}"#.as_bytes(),
            b"\n\n",
            b"data: [DONE]\n\n",
        ]);

        assert_eq!(
            out,
            vec![
                StreamChunk::Output("<thinking>".to_string()),
                StreamChunk::Output("只有推理".to_string()),
                StreamChunk::Output("</thinking>".to_string()),
                StreamChunk::Output("\n\n模型仅返回了推理内容，未返回最终回答。".to_string()),
                StreamChunk::Done
            ]
        );
    }

    #[test]
    fn openai_sse_parser_surfaces_stream_error_payloads() {
        let out = parse_chunks(&[br#"data: {"error":{"message":"model not found"}}"#, b"\n\n"]);

        assert_eq!(
            out,
            vec![
                StreamChunk::Error("model not found".to_string()),
                StreamChunk::Done
            ]
        );
    }

    #[test]
    fn openai_parser_accepts_non_stream_chat_completion_json() {
        let out = parse_chunks(&[
            r#"{"choices":[{"message":{"reasoning_content":"先判断问题","content":"我是 DeepSeek。"}}]}"#
                .as_bytes(),
        ]);

        assert_eq!(
            out,
            vec![
                StreamChunk::Output("<thinking>".to_string()),
                StreamChunk::Output("先判断问题".to_string()),
                StreamChunk::Output("</thinking>".to_string()),
                StreamChunk::Output("我是 DeepSeek。".to_string()),
                StreamChunk::Done
            ]
        );
    }
}
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<crate::ChatMessage>,
    stream: bool,
}
#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
    reasoning_content: Option<String>,
    reasoning: Option<String>,
    thinking_content: Option<String>,
}
#[derive(Deserialize)]
struct Choice {
    delta: Delta,
}
#[derive(Deserialize)]
struct UsageData {
    prompt_tokens: u64,
    completion_tokens: u64,
}
#[derive(Deserialize)]
struct StreamResp {
    choices: Vec<Choice>,
    usage: Option<UsageData>,
}
#[derive(Deserialize)]
struct NonStreamChoice {
    message: Option<NonStreamMessage>,
}
#[derive(Deserialize)]
struct NonStreamMessage {
    content: Option<String>,
    reasoning_content: Option<String>,
    reasoning: Option<String>,
    thinking_content: Option<String>,
}
#[derive(Deserialize)]
struct NonStreamResp {
    choices: Vec<NonStreamChoice>,
    usage: Option<UsageData>,
}
#[derive(Deserialize)]
struct ApiErrorResp {
    error: Option<ApiErrorData>,
}
#[derive(Deserialize)]
struct ApiErrorData {
    message: String,
}

#[derive(Default)]
struct OpenAiSseState {
    pending_line: String,
    thinking_open: bool,
    saw_content: bool,
    done_sent: bool,
}

fn append_openai_sse_bytes(state: &mut OpenAiSseState, bytes: &[u8]) -> Vec<String> {
    state.pending_line.push_str(&String::from_utf8_lossy(bytes));
    let mut lines = Vec::new();

    while let Some(newline_idx) = state.pending_line.find('\n') {
        let mut line = state.pending_line[..newline_idx].to_string();
        if line.ends_with('\r') {
            line.pop();
        }
        lines.push(line);
        state.pending_line = state.pending_line[newline_idx + 1..].to_string();
    }

    lines
}

fn flush_openai_sse_state(state: &mut OpenAiSseState) -> Vec<StreamChunk> {
    let mut chunks = Vec::new();
    if !state.pending_line.trim().is_empty() {
        let mut line = std::mem::take(&mut state.pending_line);
        if line.ends_with('\r') {
            line.pop();
        }
        chunks.extend(parse_openai_sse_line(state, &line));
    }
    if state.thinking_open {
        chunks.push(StreamChunk::Output("</thinking>".to_string()));
        if !state.saw_content {
            chunks.push(StreamChunk::Output(
                "\n\n模型仅返回了推理内容，未返回最终回答。".to_string(),
            ));
            state.saw_content = true;
        }
        state.thinking_open = false;
    }
    if !state.done_sent {
        chunks.push(StreamChunk::Done);
        state.done_sent = true;
    }
    chunks
}

fn parse_openai_sse_line(state: &mut OpenAiSseState, line: &str) -> Vec<StreamChunk> {
    let mut chunks = Vec::new();
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return chunks;
    }

    let data = trimmed
        .strip_prefix("data:")
        .map(str::trim)
        .unwrap_or(trimmed);
    if data.is_empty() {
        return chunks;
    }
    if data == "[DONE]" {
        chunks.extend(flush_openai_sse_state(state));
        return chunks;
    }

    if let Ok(err) = serde_json::from_str::<ApiErrorResp>(data) {
        if let Some(err) = err.error {
            chunks.push(StreamChunk::Error(err.message));
            return chunks;
        }
    }

    if let Ok(resp) = serde_json::from_str::<StreamResp>(data) {
        if let Some(choice) = resp.choices.first() {
            let reasoning = choice
                .delta
                .reasoning_content
                .as_deref()
                .or(choice.delta.reasoning.as_deref())
                .or(choice.delta.thinking_content.as_deref());
            if let Some(reasoning) = reasoning.filter(|text| !text.is_empty()) {
                if !state.thinking_open && !state.saw_content {
                    chunks.push(StreamChunk::Output("<thinking>".to_string()));
                    state.thinking_open = true;
                }
                if state.thinking_open {
                    chunks.push(StreamChunk::Output(reasoning.to_string()));
                }
            }

            if let Some(content) = choice
                .delta
                .content
                .as_deref()
                .filter(|text| !text.is_empty())
            {
                if state.thinking_open {
                    chunks.push(StreamChunk::Output("</thinking>".to_string()));
                    state.thinking_open = false;
                }
                state.saw_content = true;
                chunks.push(StreamChunk::Output(content.to_string()));
            }
        }
    }

    if let Ok(resp) = serde_json::from_str::<NonStreamResp>(data) {
        if let Some(message) = resp
            .choices
            .first()
            .and_then(|choice| choice.message.as_ref())
        {
            let reasoning = message
                .reasoning_content
                .as_deref()
                .or(message.reasoning.as_deref())
                .or(message.thinking_content.as_deref());
            if let Some(reasoning) = reasoning.filter(|text| !text.is_empty()) {
                chunks.push(StreamChunk::Output("<thinking>".to_string()));
                chunks.push(StreamChunk::Output(reasoning.to_string()));
                chunks.push(StreamChunk::Output("</thinking>".to_string()));
            }

            if let Some(content) = message.content.as_deref().filter(|text| !text.is_empty()) {
                state.saw_content = true;
                chunks.push(StreamChunk::Output(content.to_string()));
            }
        }
    }

    chunks
}

fn parse_openai_usage(data: &str) -> Option<Usage> {
    if let Ok(resp) = serde_json::from_str::<StreamResp>(data) {
        if let Some(u) = resp.usage {
            return Some(Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
            });
        }
    }
    if let Ok(resp) = serde_json::from_str::<NonStreamResp>(data) {
        if let Some(u) = resp.usage {
            return Some(Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
            });
        }
    }
    None
}

#[async_trait]
impl LlmClient for OpenAiClient {
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
        let (stream, tx) = ChannelStream::new(100);
        let (api_key_secret, model, base_url) = match &self.provider {
            LlmProvider::OpenAi {
                api_key,
                model,
                base_url,
            } => (api_key.clone(), model.clone(), base_url.clone()),
            _ => {
                return Err(EngineError::InvalidParameters(
                    "Invalid provider type".into(),
                ))
            }
        };
        let mut msgs = messages;
        if let Some(system) = system_prompt {
            msgs.insert(
                0,
                crate::ChatMessage {
                    role: "system".into(),
                    content: system,
                    timestamp: None,
                },
            );
        }
        let client = Client::new();
        let url = crate::openai_chat_completions_url(&base_url);
        let req = ChatRequest {
            model: model.clone(),
            messages: msgs,
            stream: true,
        };
        let key = api_key_secret.expose_secret().to_string();
        let usage_ref = self.last_usage.clone();
        tokio::spawn(async move {
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", key))
                .json(&req)
                .send()
                .await
            {
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
                        let _ = tx
                            .send(StreamChunk::Error(format!("HTTP 错误: {}", status)))
                            .await;
                    } else {
                        let mut s = r.bytes_stream();
                        let mut sse_state = OpenAiSseState::default();
                        while let Some(next) = s.next().await {
                            let d = match next {
                                Ok(d) => d,
                                Err(e) => {
                                    tx.send(StreamChunk::Error(e.to_string())).await.ok();
                                    break;
                                }
                            };
                            for line in append_openai_sse_bytes(&mut sse_state, &d) {
                                for chunk in parse_openai_sse_line(&mut sse_state, &line) {
                                    if let StreamChunk::Done = chunk {
                                        sse_state.done_sent = true;
                                    }
                                    tx.send(chunk).await.ok();
                                }

                                let data = line
                                    .trim()
                                    .strip_prefix("data:")
                                    .map(str::trim)
                                    .unwrap_or_else(|| line.trim());
                                if let Some(usage) = parse_openai_usage(data) {
                                    *usage_ref.lock().unwrap() = Some(usage);
                                }
                            }
                        }
                        for chunk in flush_openai_sse_state(&mut sse_state) {
                            tx.send(chunk).await.ok();
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
