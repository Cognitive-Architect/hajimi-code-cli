//! Turn管理 - 轻量级移植自Codex
//! 参考: codex-twist/codex-rs/protocol/src/protocol.rs
//!
//! Turn = 单次"用户提问 -> AI回复"的完整交互

/// Turn ID类型
pub type TurnId = String;

/// Turn状态
#[derive(Clone, Debug, PartialEq, Default)]
pub enum TurnStatus {
    /// 待处理
    #[default]
    Pending,
    /// 流式响应中
    Streaming,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 出错
    Error(String),
}

/// 令牌使用量统计
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// 工具调用
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// 响应内容
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseContent {
    /// 纯文本
    Text(String),
    /// 工具调用
    ToolCall(ToolCall),
    /// 思考过程（如Claude的thinking）
    Thinking(String),
}

/// Turn结构 - 单次对话回合
///
/// 对应Codex的Turn概念，但移除了云端同步字段
#[derive(Default)]
pub struct Turn {
    /// Turn ID
    pub id: TurnId,
    /// 所属Thread ID
    pub thread_id: String,
    /// 用户输入
    pub prompt: String,
    /// 响应列表（支持多段响应）
    pub responses: Vec<ResponseContent>,
    /// 工具调用结果
    pub tool_results: Vec<ToolResult>,
    /// 状态
    pub status: TurnStatus,
    /// 创建时间戳
    pub timestamp: u64,
    /// 完成时间戳
    pub completed_at: Option<u64>,
    /// 令牌使用
    pub token_usage: TokenUsage,
}

/// 工具执行结果
#[derive(Clone, Debug, PartialEq)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output: String,
    pub is_error: bool,
}

impl Turn {
    /// 创建新Turn
    ///
    /// # Arguments
    /// * `thread_id` - 所属Thread
    /// * `prompt` - 用户输入
    pub fn new(thread_id: String, prompt: String) -> Self {
        Self {
            id: generate_turn_id(),
            thread_id,
            prompt,
            responses: Vec::new(),
            tool_results: Vec::new(),
            status: TurnStatus::Pending,
            timestamp: now(),
            completed_at: None,
            token_usage: TokenUsage::default(),
        }
    }

    /// 添加文本响应
    pub fn add_response(&mut self, text: String) {
        self.responses.push(ResponseContent::Text(text));
        self.update_status();
    }

    /// 添加工具调用
    pub fn add_tool_call(&mut self, call: ToolCall) {
        self.responses.push(ResponseContent::ToolCall(call));
        self.update_status();
    }

    /// 流式追加响应
    pub fn append_response(&mut self, chunk: String) {
        self.status = TurnStatus::Streaming;

        // 追加到最后一个文本响应，或创建新响应
        if let Some(ResponseContent::Text(ref mut text)) = self.responses.last_mut() {
            text.push_str(&chunk);
        } else {
            self.responses.push(ResponseContent::Text(chunk));
        }
    }

    /// 添加工具结果
    pub fn add_tool_result(&mut self, result: ToolResult) {
        self.tool_results.push(result);
    }

    /// 标记完成
    pub fn complete(&mut self, response: String) {
        if !matches!(self.status, TurnStatus::Cancelled) {
            self.responses.push(ResponseContent::Text(response));
            self.status = TurnStatus::Completed;
            self.completed_at = Some(now());
        }
    }

    /// 标记错误
    pub fn mark_error(&mut self, error: String) {
        self.status = TurnStatus::Error(error);
        self.completed_at = Some(now());
    }

    /// 取消Turn
    pub fn cancel(&mut self) {
        self.status = TurnStatus::Cancelled;
        self.completed_at = Some(now());
    }

    /// 获取完整响应文本
    pub fn response_content(&self) -> String {
        self.responses
            .iter()
            .filter_map(|r| match r {
                ResponseContent::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// 检查是否包含工具调用
    pub fn has_tool_calls(&self) -> bool {
        self.responses
            .iter()
            .any(|r| matches!(r, ResponseContent::ToolCall(_)))
    }

    /// 获取工具调用列表
    pub fn get_tool_calls(&self) -> Vec<&ToolCall> {
        self.responses
            .iter()
            .filter_map(|r| match r {
                ResponseContent::ToolCall(call) => Some(call),
                _ => None,
            })
            .collect()
    }

    /// 更新状态
    fn update_status(&mut self) {
        if matches!(self.status, TurnStatus::Pending) {
            self.status = TurnStatus::Completed;
            self.completed_at = Some(now());
        }
    }

    /// 计算响应长度
    pub fn response_length(&self) -> usize {
        self.response_content().len()
    }

    /// 是否为空响应
    pub fn is_empty_response(&self) -> bool {
        self.responses.is_empty()
    }
}

impl Clone for Turn {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            thread_id: self.thread_id.clone(),
            prompt: self.prompt.clone(),
            responses: self.responses.clone(),
            tool_results: self.tool_results.clone(),
            status: self.status.clone(),
            timestamp: self.timestamp,
            completed_at: self.completed_at,
            token_usage: self.token_usage.clone(),
        }
    }
}

/// 生成Turn ID
fn generate_turn_id() -> String {
    format!(
        "turn_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}

/// 获取当前时间戳
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_lifecycle() {
        let mut turn = Turn::new("thread-001".to_string(), "Hello".to_string());

        assert_eq!(turn.status, TurnStatus::Pending);

        turn.append_response("Hi".to_string());
        assert_eq!(turn.status, TurnStatus::Streaming);

        turn.complete("Hi there!".to_string());
        assert_eq!(turn.status, TurnStatus::Completed);
        assert_eq!(turn.response_content(), "HiHi there!");
    }

    #[test]
    fn test_tool_calls() {
        let mut turn = Turn::new("thread-002".to_string(), "Run ls".to_string());

        turn.add_tool_call(ToolCall {
            id: "call_1".to_string(),
            name: "shell".to_string(),
            arguments: r#"{"command": "ls -la"}"#.to_string(),
        });

        assert!(turn.has_tool_calls());
        assert_eq!(turn.get_tool_calls().len(), 1);
    }

    #[test]
    fn test_turn_cancel() {
        let mut turn = Turn::new("thread-003".to_string(), "Test".to_string());
        turn.cancel();

        assert_eq!(turn.status, TurnStatus::Cancelled);
        assert!(turn.completed_at.is_some());
    }
}
