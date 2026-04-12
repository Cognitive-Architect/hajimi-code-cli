//! Thread管理 - 轻量级移植自Codex
//! 参考: codex-twist/codex-rs/core/src/codex_thread.rs
//! 
//! Thread是Codex的核心概念：一个有状态的对话会话容器
//! 本实现将云端存储替换为LCR本地.hctx存储

use std::path::{Path, PathBuf};
use crate::storage::{HctxStorage, StorageError};
use crate::turn::{Turn, TurnStatus};
use crate::approval::{ApprovalPolicy, ApprovalRequest};

/// Thread ID类型
pub type ThreadId = String;

/// Thread配置 - 用户可自定义
pub struct ThreadConfig {
    /// 模型名称
    pub model: String,
    /// API基础URL（支持Ollama等本地模型）
    pub base_url: String,
    /// API Key（可选，支持本地无Key模型）
    pub api_key: Option<String>,
    /// 审批策略
    pub approval_policy: ApprovalPolicy,
    /// 系统提示词
    pub system_prompt: Option<String>,
    /// 最大上下文长度
    pub max_context_length: usize,
}

impl Default for ThreadConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
            approval_policy: ApprovalPolicy::default(),
            system_prompt: None,
            max_context_length: 8192,
        }
    }
}

/// Thread结构 - 对话会话容器
/// 
/// 移植自Codex的CodexThread，但改为本地优先架构
pub struct Thread {
    /// Thread ID
    pub id: ThreadId,
    /// Thread名称
    pub name: String,
    /// 存储路径
    pub storage_path: PathBuf,
    /// 所有Turn
    pub turns: Vec<Turn>,
    /// 当前进行中的Turn
    pub current_turn: Option<Turn>,
    /// 配置
    pub config: ThreadConfig,
    /// 创建时间戳
    pub created_at: u64,
    /// 更新时间戳
    pub updated_at: u64,
}

impl Thread {
    /// 创建新Thread
    pub fn new(id: ThreadId, storage_path: PathBuf) -> Self {
        let now = now();
        Self {
            id,
            name: "New Conversation".to_string(),
            storage_path,
            turns: Vec::new(),
            current_turn: None,
            config: ThreadConfig::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 创建新Thread并自动分配存储路径
    pub fn new_with_storage(storage_dir: PathBuf) -> Result<Self, StorageError> {
        let id = generate_thread_id();
        let storage_path = storage_dir.join(".hctx");
        
        Ok(Self::new(id, storage_path))
    }

    /// 从存储加载Thread
    pub fn load(storage_path: PathBuf) -> Result<Self, StorageError> {
        // 简化实现：实际应调用lcr_adapter::hctx_to_thread
        let id = extract_thread_id_from_path(&storage_path);
        Ok(Self::new(id, storage_path))
    }

    /// 创建新Turn
    /// 
    /// # Arguments
    /// * `prompt` - 用户输入
    pub fn create_turn(&mut self, prompt: String) -> &mut Turn {
        let turn = Turn::new(self.id.clone(), prompt);
        self.current_turn = Some(turn);
        // SAFETY: current_turn was just set to Some above
        self.current_turn.as_mut().expect("just set current_turn")
    }

    /// 完成当前Turn
    pub fn complete_turn(&mut self, response: String) {
        if let Some(mut turn) = self.current_turn.take() {
            turn.complete(response);
            self.turns.push(turn);
            self.updated_at = now();
        }
    }

    /// 流式添加响应内容
    pub fn append_response(&mut self, chunk: String) {
        if let Some(ref mut turn) = self.current_turn {
            turn.append_response(chunk);
        }
    }

    /// 取消当前Turn
    pub fn cancel_turn(&mut self) {
        if let Some(mut turn) = self.current_turn.take() {
            turn.cancel();
            self.turns.push(turn);
            self.updated_at = now();
        }
    }

    /// 检查是否需要审批
    pub fn needs_approval(&self, command: &str) -> bool {
        self.config.approval_policy.needs_approval(command)
    }

    /// 创建审批请求
    pub fn create_approval_request(&self, command: String) -> ApprovalRequest {
        ApprovalRequest::new(command)
    }

    /// 获取完整对话历史（用于LLM上下文）
    pub fn get_history(&self) -> Vec<(String, String)> {
        self.turns
            .iter()
            .filter_map(|t| {
                if t.status == TurnStatus::Completed {
                    Some((t.prompt.clone(), t.response_content()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// 获取系统消息
    pub fn system_message(&self) -> String {
        self.config.system_prompt.clone()
            .unwrap_or_else(|| "You are a helpful AI assistant.".to_string())
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ThreadConfig) {
        self.config = config;
        self.updated_at = now();
    }

    /// 重命名Thread
    pub fn rename(&mut self, name: String) {
        self.name = name;
        self.updated_at = now();
    }

    /// 获取最后一条消息
    pub fn last_message(&self) -> Option<&Turn> {
        self.turns.last()
    }

    /// 统计信息
    pub fn stats(&self) -> ThreadStats {
        ThreadStats {
            total_turns: self.turns.len(),
            total_tokens: self.turns.iter().map(|t| t.token_usage.total_tokens).sum(),
            duration_secs: self.updated_at.saturating_sub(self.created_at),
        }
    }

    /// 保存到LCR存储
    /// 
    /// 将Thread序列化为.hctx格式并保存
    pub fn save_to_lcr(&self) -> Result<(), StorageError> {
        // 确保路径以.hctx结尾，如果是目录则在目录下创建.hctx文件
        let hctx_path = if self.storage_path.extension().is_some() {
            self.storage_path.clone()
        } else {
            self.storage_path.join(".hctx")
        };
        let storage = HctxStorage::new(hctx_path)?;
        let chunks = crate::lcr_adapter::thread_to_hctx(self);
        storage.save_context(&chunks)
    }

    /// 从LCR存储加载
    pub fn load_from_lcr(&mut self) -> Result<(), StorageError> {
        let storage = HctxStorage::new(self.storage_path.clone())?;
        let chunks = storage.load_context()?;
        
        // 使用lcr_adapter解析
        let loaded = crate::lcr_adapter::hctx_to_thread(&chunks, self.storage_path.clone())
            .map_err(|e| StorageError::ParseError(e.to_string()))?;
        
        *self = loaded;
        Ok(())
    }

    /// 导出为纯文本
    pub fn export_text(&self) -> String {
        let mut output = format!("# {}\n\n", self.name);
        
        for (i, turn) in self.turns.iter().enumerate() {
            output.push_str(&format!("## Turn {}\n\n", i + 1));
            output.push_str(&format!("**User:** {}\n\n", turn.prompt));
            output.push_str(&format!("**Assistant:** {}\n\n", turn.response_content()));
        }
        
        output
    }
}

/// Thread统计信息
pub struct ThreadStats {
    pub total_turns: usize,
    pub total_tokens: usize,
    pub duration_secs: u64,
}

/// 生成Thread ID
fn generate_thread_id() -> String {
    format!(
        "thread_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    )
}

/// 从路径提取Thread ID
fn extract_thread_id_from_path(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(generate_thread_id)
}

/// 获取当前时间戳
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_lifecycle() {
        let mut thread = Thread::new(
            "test-001".to_string(),
            std::env::temp_dir().join("test-codex-001")
        );

        // 创建Turn
        thread.create_turn("Hello, world!".to_string());
        assert!(thread.current_turn.is_some());

        // 完成Turn
        thread.complete_turn("Hi there!".to_string());
        assert!(thread.current_turn.is_none());
        assert_eq!(thread.turns.len(), 1);

        // 验证统计
        let stats = thread.stats();
        assert_eq!(stats.total_turns, 1);
    }

    #[test]
    fn test_thread_history() {
        let mut thread = Thread::new("test-002".to_string(), std::env::temp_dir().join("test-codex-002"));
        
        thread.create_turn("Q1".to_string());
        thread.complete_turn("A1".to_string());
        
        thread.create_turn("Q2".to_string());
        thread.complete_turn("A2".to_string());

        let history = thread.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].0, "Q1");
        assert_eq!(history[0].1, "A1");
    }

    #[test]
    fn test_thread_export() {
        let mut thread = Thread::new("test-003".to_string(), std::env::temp_dir().join("test-codex-003"));
        thread.name = "Test Thread".to_string();
        
        thread.create_turn("Question".to_string());
        thread.complete_turn("Answer".to_string());

        let text = thread.export_text();
        assert!(text.contains("# Test Thread"));
        assert!(text.contains("Question"));
        assert!(text.contains("Answer"));
    }
}
