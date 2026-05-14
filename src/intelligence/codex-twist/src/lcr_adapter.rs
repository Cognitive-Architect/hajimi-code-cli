//! LCR存储适配 - 用.hctx替换Codex的云端Thread存储
//! 将Thread/Turn序列化为HCTX JSON格式（v1.0），实现本地持久化

use crate::storage::ContextChunk;
use crate::thread::{Thread, ThreadConfig};
use crate::turn::{ResponseContent, TokenUsage, Turn, TurnStatus};
use serde::{Deserialize, Serialize};

/// HCTX文档根结构
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HctxDocument {
    pub version: String,
    pub metadata: HctxMetadata,
    pub config: ThreadConfigJson,
    pub turns: Vec<TurnRecord>,
}

/// HCTX元数据
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HctxMetadata {
    pub thread_id: String,
    pub thread_name: String,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
}

/// Thread配置JSON表示
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ThreadConfigJson {
    pub model: String,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    pub max_context_length: usize,
    pub approval_policy: String,
}

/// Turn记录JSON表示
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TurnRecord {
    pub turn_id: String,
    pub thread_id: String,
    pub prompt: String,
    pub responses: Vec<ResponseItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<Vec<ToolResultJson>>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
    pub token_usage: TokenUsageJson,
}

/// 响应项JSON表示
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseItem {
    pub r#type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_arguments: Option<String>,
}

/// Token使用量JSON表示
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TokenUsageJson {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// 工具结果JSON表示
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolResultJson {
    pub tool_call_id: String,
    pub output: String,
    pub is_error: bool,
}

impl From<&TokenUsage> for TokenUsageJson {
    fn from(u: &TokenUsage) -> Self {
        Self {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }
    }
}

impl From<TokenUsageJson> for TokenUsage {
    fn from(u: TokenUsageJson) -> Self {
        Self {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }
    }
}

/// Thread序列化为HCTX JSON格式
pub fn thread_to_hctx(thread: &Thread) -> Vec<ContextChunk> {
    let doc = thread_to_document(thread);
    let json = serde_json::to_string_pretty(&doc).unwrap_or_default();
    vec![ContextChunk::system(json)]
}

/// Thread转换为HCTX文档
fn thread_to_document(thread: &Thread) -> HctxDocument {
    HctxDocument {
        version: "1.0".to_string(),
        metadata: HctxMetadata {
            thread_id: thread.id.clone(),
            thread_name: thread.name.clone(),
            created_at: thread.created_at,
            updated_at: thread.updated_at,
            generator: Some(format!("codex-twist {}", crate::VERSION)),
        },
        config: ThreadConfigJson {
            model: thread.config.model.clone(),
            base_url: thread.config.base_url.clone(),
            api_key_present: Some(thread.config.api_key.is_some()),
            system_prompt: thread.config.system_prompt.clone(),
            max_context_length: thread.config.max_context_length,
            approval_policy: format!("{:?}", thread.config.approval_policy).to_lowercase(),
        },
        turns: thread.turns.iter().map(turn_to_record).collect(),
    }
}

/// Turn转换为记录
fn turn_to_record(turn: &Turn) -> TurnRecord {
    TurnRecord {
        turn_id: turn.id.clone(),
        thread_id: turn.thread_id.clone(),
        prompt: turn.prompt.clone(),
        responses: turn
            .responses
            .iter()
            .map(|r| match r {
                ResponseContent::Text(t) => ResponseItem {
                    r#type: "text".to_string(),
                    content: t.clone(),
                    tool_call_id: None,
                    tool_name: None,
                    tool_arguments: None,
                },
                ResponseContent::ToolCall(tc) => ResponseItem {
                    r#type: "tool_call".to_string(),
                    content: tc.arguments.clone(),
                    tool_call_id: Some(tc.id.clone()),
                    tool_name: Some(tc.name.clone()),
                    tool_arguments: Some(tc.arguments.clone()),
                },
                ResponseContent::Thinking(t) => ResponseItem {
                    r#type: "thinking".to_string(),
                    content: t.clone(),
                    tool_call_id: None,
                    tool_name: None,
                    tool_arguments: None,
                },
            })
            .collect(),
        tool_results: if turn.tool_results.is_empty() {
            None
        } else {
            Some(
                turn.tool_results
                    .iter()
                    .map(|tr| ToolResultJson {
                        tool_call_id: tr.tool_call_id.clone(),
                        output: tr.output.clone(),
                        is_error: tr.is_error,
                    })
                    .collect(),
            )
        },
        status: match turn.status {
            TurnStatus::Pending => "pending",
            TurnStatus::Streaming => "streaming",
            TurnStatus::Completed => "completed",
            TurnStatus::Cancelled => "cancelled",
            TurnStatus::Error(_) => "error",
        }
        .to_string(),
        error_message: match &turn.status {
            TurnStatus::Error(e) => Some(e.clone()),
            _ => None,
        },
        timestamp: turn.timestamp,
        completed_at: turn.completed_at,
        token_usage: TokenUsageJson::from(&turn.token_usage),
    }
}

/// 从LCR恢复Thread
pub fn hctx_to_thread(
    chunks: &[ContextChunk],
    storage_path: std::path::PathBuf,
) -> Result<Thread, ParseError> {
    if chunks.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    let json_text = match &chunks[0] {
        ContextChunk::System { content, .. } => content,
        _ => return Err(ParseError::MissingMetadata),
    };
    let doc: HctxDocument =
        serde_json::from_str(json_text).map_err(|e| ParseError::InvalidMetadata(e.to_string()))?;
    document_to_thread(doc, storage_path)
}

/// HCTX文档转换为Thread
fn document_to_thread(
    doc: HctxDocument,
    storage_path: std::path::PathBuf,
) -> Result<Thread, ParseError> {
    let mut thread = Thread::new(doc.metadata.thread_id, storage_path);
    thread.name = doc.metadata.thread_name;
    thread.created_at = doc.metadata.created_at;
    thread.updated_at = doc.metadata.updated_at;
    thread.config.model = doc.config.model;
    thread.config.base_url = doc.config.base_url;
    thread.config.max_context_length = doc.config.max_context_length;
    thread.config.system_prompt = doc.config.system_prompt;
    if let Some(true) = doc.config.api_key_present {
        thread.config.api_key = Some("[PRESENT]".to_string());
    }
    thread.turns = doc
        .turns
        .into_iter()
        .map(record_to_turn)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(thread)
}

/// 记录转换为Turn
fn record_to_turn(record: TurnRecord) -> Result<Turn, ParseError> {
    let mut turn = Turn::new(record.thread_id, record.prompt);
    turn.id = record.turn_id;
    turn.timestamp = record.timestamp;
    turn.completed_at = record.completed_at;
    turn.token_usage = TokenUsage::from(record.token_usage);
    turn.status = parse_status(&record.status, record.error_message)?;
    turn.responses = record
        .responses
        .into_iter()
        .map(|r| match r.r#type.as_str() {
            "text" => Ok(ResponseContent::Text(r.content)),
            "thinking" => Ok(ResponseContent::Thinking(r.content)),
            "tool_call" => Ok(ResponseContent::ToolCall(crate::turn::ToolCall {
                id: r.tool_call_id.unwrap_or_default(),
                name: r.tool_name.unwrap_or_default(),
                arguments: r.tool_arguments.unwrap_or(r.content),
            })),
            _ => Err(ParseError::InvalidMetadata(format!(
                "未知响应类型: {}",
                r.r#type
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(turn)
}

/// 解析状态
fn parse_status(s: &str, error_msg: Option<String>) -> Result<TurnStatus, ParseError> {
    match s {
        "pending" => Ok(TurnStatus::Pending),
        "streaming" => Ok(TurnStatus::Streaming),
        "completed" => Ok(TurnStatus::Completed),
        "cancelled" => Ok(TurnStatus::Cancelled),
        "error" => Ok(TurnStatus::Error(error_msg.unwrap_or_default())),
        _ => Err(ParseError::InvalidMetadata(format!("无效状态: {}", s))),
    }
}

/// 解析错误类型
#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    EmptyInput,
    MissingMetadata,
    InvalidMetadata(String),
    MissingField(String),
    InvalidTimestamp(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "输入为空"),
            ParseError::MissingMetadata => write!(f, "缺少元数据块"),
            ParseError::InvalidMetadata(s) => write!(f, "无效的元数据: {}", s),
            ParseError::MissingField(s) => write!(f, "缺少字段: {}", s),
            ParseError::InvalidTimestamp(s) => write!(f, "无效的时间戳: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

/// Thread配置序列化为JSON
pub fn config_to_hctx(config: &ThreadConfig) -> ContextChunk {
    let json = serde_json::json!({ "model": config.model, "base_url": config.base_url, "api_key_present": config.api_key.is_some(), "system_prompt": config.system_prompt, "max_context_length": config.max_context_length });
    ContextChunk::system(json.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    #[test]
    fn test_thread_roundtrip() {
        let mut thread = Thread::new("test-001".to_string(), PathBuf::from(".test"));
        thread.name = "测试对话".to_string();
        let _turn = thread.create_turn("Hello".to_string());
        thread.complete_turn("Hi there!".to_string());
        let chunks = thread_to_hctx(&thread);
        assert!(!chunks.is_empty());
        let restored = hctx_to_thread(&chunks, PathBuf::from(".test")).unwrap();
        assert_eq!(restored.id, thread.id);
        assert_eq!(restored.name, thread.name);
        assert_eq!(restored.turns.len(), thread.turns.len());
    }
    #[test]
    fn test_json_structure() {
        let mut thread = Thread::new("test-002".to_string(), PathBuf::from(".test"));
        thread.name = "JSON测试".to_string();
        let _turn = thread.create_turn("Test".to_string());
        thread.complete_turn("Response".to_string());
        let doc = thread_to_document(&thread);
        assert_eq!(doc.version, "1.0");
        assert_eq!(doc.metadata.thread_id, "test-002");
        assert_eq!(doc.metadata.thread_name, "JSON测试");
        assert_eq!(doc.turns.len(), 1);
        assert_eq!(doc.turns[0].prompt, "Test");
        assert_eq!(doc.turns[0].responses[0].r#type, "text");
    }
    #[test]
    fn test_serde_json_usage() {
        let doc = HctxDocument {
            version: "1.0".to_string(),
            metadata: HctxMetadata {
                thread_id: "test-003".to_string(),
                thread_name: "Serde Test".to_string(),
                created_at: 1234567890,
                updated_at: 1234567890,
                generator: None,
            },
            config: ThreadConfigJson {
                model: "gpt-4".to_string(),
                base_url: "http://test".to_string(),
                api_key_present: None,
                system_prompt: None,
                max_context_length: 8192,
                approval_policy: "ask".to_string(),
            },
            turns: vec![],
        };
        let json = serde_json::to_string_pretty(&doc).expect("serde_json序列化失败");
        assert!(json.contains("\"version\": \"1.0\""));
        assert!(json.contains("thread_id"));
        let restored: HctxDocument = serde_json::from_str(&json).expect("serde_json反序列化失败");
        assert_eq!(restored.version, "1.0");
        assert_eq!(restored.metadata.thread_id, "test-003");
    }
}
