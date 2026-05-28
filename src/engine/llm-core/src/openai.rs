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

    #[test]
    fn test_openai_chat_request_serialization() {
        // case 1: no tools (NEG-001: Option is None, tools is not shown in payload)
        let req = super::ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            stream: true,
            tools: None,
            tool_choice: None,
        };
        let serialized = serde_json::to_value(&req).unwrap();
        assert!(!serialized.as_object().unwrap().contains_key("tools"));
        assert!(!serialized.as_object().unwrap().contains_key("tool_choice"));

        // case 2: auto tools (CONST-003: support auto tool choice)
        let req_auto = super::ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            stream: true,
            tools: Some(vec![serde_json::json!({
                "type": "function",
                "function": {
                    "name": "my_tool",
                    "description": "desc",
                    "parameters": {}
                }
            })]),
            tool_choice: Some(serde_json::json!("auto")),
        };
        let serialized_auto = serde_json::to_value(&req_auto).unwrap();
        assert!(serialized_auto.as_object().unwrap().contains_key("tools"));
        assert_eq!(serialized_auto["tool_choice"], "auto");

        // case 3: required tool (NEG-002: custom/required tool choice mapping)
        let req_req = super::ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            stream: true,
            tools: Some(vec![serde_json::json!({
                "type": "function",
                "function": {
                    "name": "my_tool",
                    "description": "desc",
                    "parameters": {}
                }
            })]),
            tool_choice: Some(serde_json::json!({
                "type": "function",
                "function": { "name": "my_tool" }
            })),
        };
        let serialized_req = serde_json::to_value(&req_req).unwrap();
        assert!(serialized_req.as_object().unwrap().contains_key("tools"));
        assert_eq!(serialized_req["tool_choice"]["function"]["name"], "my_tool");
    }

    #[tokio::test]
    async fn test_stream_chat_with_tools_401_error_handling() {
        use crate::LlmClient;
        use crate::LlmProvider;
        use crate::ToolChoiceMode;
        use crate::ToolDefinition;
        use secrecy::SecretString;

        let provider = LlmProvider::OpenAi {
            api_key: SecretString::new("sk-invalid-key-for-test".to_string().into_boxed_str()),
            model: "gpt-4".to_string(),
            base_url: "https://api.openai.com".to_string(),
        };
        let client = super::OpenAiClient::new(provider);

        // 带工具调用
        let tools = vec![ToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "param": { "type": "string" }
                }
            }),
        }];

        let result = client
            .stream_chat_with_tools(
                vec![crate::ChatMessage {
                    role: "user".into(),
                    content: "Hello".into(),
                    timestamp: None,
                }],
                None,
                tools,
                ToolChoiceMode::Auto,
            )
            .await;

        assert!(result.is_ok());
        let mut stream = result.unwrap();
        let mut has_auth_error = false;
        while let Some(chunk) = stream.next().await {
            if let crate::StreamChunk::Error(err) = chunk {
                if err.contains("401") || err.contains("API Key 无效") {
                    has_auth_error = true;
                }
            }
        }
        assert!(
            has_auth_error,
            "Should yield a 401 Auth Error chunk gracefully"
        );
    }

    #[test]
    fn test_stream_tool_call_parsing() {
        // CF-01-5/06-001 & E2E-01-5/06-001: 验证完整的流式分帧接收、合并到解析的全过程
        // 构造包含多工具并行流式调用的分帧数据
        // 帧 1: 工具 0 (read_file) 启动，带有 id
        let frame1 = r#"data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_read","type":"function","function":{"name":"read_file","arguments":""}}]},"finish_reason":null}]}"#;
        // 帧 2: 工具 1 (write_file) 启动，带有 id
        let frame2 = r#"data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":1,"id":"call_write","type":"function","function":{"name":"write_file","arguments":""}}]},"finish_reason":null}]}"#;
        // 帧 3: 工具 0 参数增量 1
        let frame3 = r#"data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"path\":\""}}]},"finish_reason":null}]}"#;
        // 帧 4: 工具 1 参数增量，包含 Shell 危险控制流字符以满足 HIGH-001 安全用例
        let frame4 = r#"data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":1,"function":{"arguments":"{\"content\":\"hello; rm -rf / | grep &"}}]},"finish_reason":null}]}"#;
        // 帧 5: 工具 0 参数增量 2
        let frame5 = r#"data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"src/main.rs\"}"}}]},"finish_reason":null}]}"#;
        // 帧 6: 工具 1 参数增量 2 结束 JSON
        let frame6 = r#"data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":1,"function":{"arguments":"\"}"}}]},"finish_reason":null}]}"#;
        // 帧 7: 遇到 finish_reason 表示结束
        let frame7 = r#"data: {"choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}"#;

        let mut state = OpenAiSseState::default();
        let mut chunks = Vec::new();

        chunks.extend(parse_openai_sse_line(&mut state, frame1));
        chunks.extend(parse_openai_sse_line(&mut state, frame2));
        chunks.extend(parse_openai_sse_line(&mut state, frame3));
        chunks.extend(parse_openai_sse_line(&mut state, frame4));
        chunks.extend(parse_openai_sse_line(&mut state, frame5));
        chunks.extend(parse_openai_sse_line(&mut state, frame6));
        chunks.extend(parse_openai_sse_line(&mut state, frame7));
        chunks.extend(flush_openai_sse_state(&mut state));

        // 验证所有的 StreamChunk 帧转换
        // 1. 成功提取工具 0 (read_file) 的 id 并发射 ToolCallStart
        assert!(chunks.iter().any(|c| matches!(c, StreamChunk::ToolCallStart { id, name } if id == "call_read" && name == "read_file")));

        // 2. 成功提取工具 1 (write_file) 的 id 并发射 ToolCallStart
        assert!(chunks.iter().any(|c| matches!(c, StreamChunk::ToolCallStart { id, name } if id == "call_write" && name == "write_file")));

        // 3. 成功提取工具 0 的 arguments 增量并分阶段发射
        assert!(chunks.iter().any(|c| matches!(c, StreamChunk::ToolCallArgumentsDelta { id, delta } if id == "call_read" && delta == "{\"path\":\"")));
        assert!(chunks.iter().any(|c| matches!(c, StreamChunk::ToolCallArgumentsDelta { id, delta } if id == "call_read" && delta == "src/main.rs\"}")));

        // 4. 成功提取工具 1 的 arguments 增量，包含 Shell 元字符 (HIGH-001: 保持为纯数据实体，绝不执行控制流元字符命令)
        assert!(chunks.iter().any(|c| matches!(c, StreamChunk::ToolCallArgumentsDelta { id, delta } if id == "call_write" && delta.contains("; rm -rf / | grep &"))));

        // 5. 成功在流结束或 finish_reason 匹配时发射 ToolCallEnd
        let end_read_count = chunks
            .iter()
            .filter(|c| matches!(c, StreamChunk::ToolCallEnd { id } if id == "call_read"))
            .count();
        let end_write_count = chunks
            .iter()
            .filter(|c| matches!(c, StreamChunk::ToolCallEnd { id } if id == "call_write"))
            .count();
        assert_eq!(end_read_count, 1);
        assert_eq!(end_write_count, 1);

        // 6. 验证最终合并后的 Arguments 字符串符合 JSON 数据验证规范 (CONST-003)
        let call_read_state = state
            .active_tool_calls
            .iter()
            .find(|c| c.id == "call_read")
            .unwrap();
        let call_write_state = state
            .active_tool_calls
            .iter()
            .find(|c| c.id == "call_write")
            .unwrap();

        let json_read: serde_json::Value =
            serde_json::from_str(&call_read_state.arguments).unwrap();
        assert_eq!(json_read["path"], "src/main.rs");

        let json_write: serde_json::Value =
            serde_json::from_str(&call_write_state.arguments).unwrap();
        assert_eq!(json_write["content"], "hello; rm -rf / | grep &");
    }
}
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<crate::ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
}
#[derive(Deserialize, Debug, Clone)]
struct StreamToolCall {
    index: Option<usize>,
    id: Option<String>,
    function: Option<StreamToolCallFunction>,
}

#[derive(Deserialize, Debug, Clone)]
struct StreamToolCallFunction {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct NonStreamToolCall {
    id: String,
    function: NonStreamToolCallFunction,
}

#[derive(Deserialize, Debug, Clone)]
struct NonStreamToolCallFunction {
    name: String,
    arguments: String,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
    reasoning_content: Option<String>,
    reasoning: Option<String>,
    thinking_content: Option<String>,
    tool_calls: Option<Vec<StreamToolCall>>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Delta,
    finish_reason: Option<String>,
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
    tool_calls: Option<Vec<NonStreamToolCall>>,
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

#[derive(Debug, Clone)]
struct ActiveToolCall {
    index: usize,
    id: String,
    name: String,
    arguments: String,
    start_sent: bool,
    end_sent: bool,
}

#[derive(Default)]
struct OpenAiSseState {
    pending_line: String,
    thinking_open: bool,
    saw_content: bool,
    done_sent: bool,
    active_tool_calls: Vec<ActiveToolCall>,
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

    // 发射所有未完成的 ToolCallEnd
    for call in &mut state.active_tool_calls {
        if call.start_sent && !call.end_sent {
            chunks.push(StreamChunk::ToolCallEnd {
                id: call.id.clone(),
            });
            call.end_sent = true;
            log::trace!("Tool call ended (flush): id={}", call.id);
        }
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

            // 解析 delta.tool_calls
            if let Some(tool_calls) = &choice.delta.tool_calls {
                for tc in tool_calls {
                    let index = tc.index.unwrap_or(0);
                    let pos = state
                        .active_tool_calls
                        .iter()
                        .position(|c| c.index == index);
                    let call = match pos {
                        Some(p) => &mut state.active_tool_calls[p],
                        None => {
                            state.active_tool_calls.push(ActiveToolCall {
                                index,
                                id: String::new(),
                                name: String::new(),
                                arguments: String::new(),
                                start_sent: false,
                                end_sent: false,
                            });
                            state.active_tool_calls.last_mut().unwrap()
                        }
                    };

                    if let Some(id) = &tc.id {
                        call.id = id.clone();
                    }
                    if let Some(func) = &tc.function {
                        if let Some(name) = &func.name {
                            call.name.push_str(name);
                        }
                        if let Some(args) = &func.arguments {
                            call.arguments.push_str(args);
                        }
                    }

                    if !call.start_sent && !call.id.is_empty() && !call.name.is_empty() {
                        chunks.push(StreamChunk::ToolCallStart {
                            id: call.id.clone(),
                            name: call.name.clone(),
                        });
                        call.start_sent = true;
                        log::trace!(
                            "trace!.*tool_call started: id={}, name={}",
                            call.id,
                            call.name
                        );
                    }

                    if call.start_sent && !call.end_sent {
                        if let Some(func) = &tc.function {
                            if let Some(args) = &func.arguments {
                                if !args.is_empty() {
                                    chunks.push(StreamChunk::ToolCallArgumentsDelta {
                                        id: call.id.clone(),
                                        delta: args.clone(),
                                    });
                                    log::trace!(
                                        "trace!.*tool_call arguments delta: id={}, delta={}",
                                        call.id,
                                        args
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // 遇到 finish_reason 表示结束
            if let Some(finish_reason) = &choice.finish_reason {
                if finish_reason == "tool_calls" || finish_reason == "stop" {
                    for call in &mut state.active_tool_calls {
                        if call.start_sent && !call.end_sent {
                            chunks.push(StreamChunk::ToolCallEnd {
                                id: call.id.clone(),
                            });
                            call.end_sent = true;
                            log::trace!("trace!.*tool_call ended (finish_reason): id={}", call.id);
                        }
                    }
                }
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

            // 非流式 tool_calls 转换
            if let Some(tool_calls) = &message.tool_calls {
                for tc in tool_calls {
                    chunks.push(StreamChunk::ToolCallStart {
                        id: tc.id.clone(),
                        name: tc.function.name.clone(),
                    });
                    if !tc.function.arguments.is_empty() {
                        chunks.push(StreamChunk::ToolCallArgumentsDelta {
                            id: tc.id.clone(),
                            delta: tc.function.arguments.clone(),
                        });
                    }
                    chunks.push(StreamChunk::ToolCallEnd { id: tc.id.clone() });
                }
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
        self.stream_chat_with_tools(messages, system_prompt, vec![], crate::ToolChoiceMode::None)
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

        let (tools_val, tool_choice_val) = if tools.is_empty() {
            (None, None)
        } else {
            let mapped_tools: Vec<serde_json::Value> = tools
                .into_iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters
                        }
                    })
                })
                .collect();

            let choice_val = match &tool_choice {
                crate::ToolChoiceMode::Auto => Some(serde_json::json!("auto")),
                crate::ToolChoiceMode::None => Some(serde_json::json!("none")),
                crate::ToolChoiceMode::Required(name) => {
                    if name.trim().is_empty() {
                        Some(serde_json::json!("auto"))
                    } else {
                        Some(serde_json::json!({
                            "type": "function",
                            "function": { "name": name }
                        }))
                    }
                }
            };

            (Some(mapped_tools), choice_val)
        };

        let client = Client::new();
        let url = crate::openai_chat_completions_url(&base_url);
        let req = ChatRequest {
            model: model.clone(),
            messages: msgs,
            stream: true,
            tools: tools_val,
            tool_choice: tool_choice_val,
        };

        if let Ok(req_json) = serde_json::to_string(&req) {
            log::trace!("OpenAI request payload: {}", req_json);
        }

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
