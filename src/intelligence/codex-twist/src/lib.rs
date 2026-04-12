//! Codex-Twist 轻量级Thread/Turn架构移植
//! 
//! 从OpenAI Codex核心概念提取：
//! - Thread: 对话会话容器
//! - Turn: 单次用户输入到AI回复的完整交互
//! - Approval: 安全审批系统
//! 
//! 关键改造：
//! - 云端JSON存储 → LCR本地.hctx存储
//! - OAuth认证 → 用户自填API Key
//! - 轻量级实现：<600行，最小侵入

pub mod approval;
pub mod lcr_adapter;
pub mod memory;
pub mod storage;
pub mod thread;
pub mod tiered;
pub mod turn;

// FFI绑定层 (napi-rs)
#[cfg(feature = "napi")]
pub mod ffi;

// 重新导出常用类型
pub use approval::{ApprovalPolicy, ApprovalRequest, ApprovalResult, RiskLevel};
pub use lcr_adapter::ParseError;
pub use storage::{ContextChunk, HctxStorage, StorageError};
pub use thread::{Thread, ThreadConfig, ThreadId, ThreadStats};
pub use turn::{ResponseContent, TokenUsage, ToolCall, ToolResult, Turn, TurnStatus};

/// 模块版本
pub const VERSION: &str = "0.1.0";

/// 创建新Thread的便捷函数
pub fn create_thread(storage_path: std::path::PathBuf) -> Result<Thread, StorageError> {
    Thread::new_with_storage(storage_path)
}

/// 审批策略字符串解析
pub fn parse_policy(s: &str) -> ApprovalPolicy {
    match s.to_lowercase().as_str() {
        "ask" | "ask-before-exec" => ApprovalPolicy::AskBeforeExec,
        "dangerous" | "ask-for-dangerous" => ApprovalPolicy::AskForDangerous,
        "once" | "ask-once-then-auto" => ApprovalPolicy::AskOnceThenAuto,
        "auto" | "full-auto" => ApprovalPolicy::FullAuto,
        "deny" | "full-deny" => ApprovalPolicy::FullDeny,
        _ => ApprovalPolicy::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_policy() {
        assert!(matches!(parse_policy("ask"), ApprovalPolicy::AskBeforeExec));
        assert!(matches!(parse_policy("auto"), ApprovalPolicy::FullAuto));
        assert!(matches!(parse_policy("deny"), ApprovalPolicy::FullDeny));
    }

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }
}
