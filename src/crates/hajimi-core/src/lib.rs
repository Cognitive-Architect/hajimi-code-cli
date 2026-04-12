//! HAJIMI Core - Query Engine Foundation
//!
//! Week 1 Deliverables: QueryEngine base architecture
//!
//! # Debt Status - 2026-04-03
//!
//! ## Cleared Debts
//! - DEBT-W01-001: [CLEARED 2026-04-03] parallel.rs:60 unwrap已修复为match/map_err处理
//! - DEBT-W01-002: [CLEARED 2026-04-03] retry.rs:29 已改为expect带BUG说明
//! - DEBT-W01-003: [CLEARED 2026-04-03] streaming模块已实现StreamingExecutor
//! - DEBT-W02-001: [CLEARED 2026-04-03] 同W01-001，parallel.rs已修复
//! - DEBT-W03-001: [CLEARED 2026-04-03] LlmProvider手动实现Debug，api_key已REDACTED
//! - DEBT-LINES-W04-03: [ACCEPTED 2026-04-04] 238/180/+58行，技术必要性认可
//! - DEBT-LINES-W05-02: [CLEARED 2026-04-04] 221→118行(-103行)，合并loader+env+宏优化
//!
//! ## Active Debts
//! - DEBT-LINES-W12-02: [ACCEPTED 2026-04-03] 211/160/+51行，download.rs(77)+parse.rs(134)
//!   B-W12/02 要求4工具130±10行，但流式解析器复杂性超过熔断线，技术必要性认可
//!
//! ## Next Audit
//! - NEXT-AUDIT: Week 13矫正后
//! - LAST-CLEARED: 2026-04-04

pub mod config;
pub mod core;
pub mod error;
pub mod executor;
pub mod knowledge;
pub mod llm;
pub mod query;
pub mod retry;
pub mod streaming;
pub mod tool;
pub mod ui;

pub use error::EngineError;
pub use executor::{Executor, ParallelExecutor, SerialExecutor};
pub use llm::{AnthropicClient, LlmClient, LlmProvider, OllamaClient, OpenAiClient};
pub use query::{Query, QueryResult};
pub use retry::with_retry;
pub use streaming::{
    BatchConfig, BatchedStream, ChannelStream, StreamChunk, StreamingExecutor, StreamConfig,
};
pub use tool::{
    BashTool, EditFileTool, FindTool, GlobTool, GrepTool, ListDirectoryTool, LsTool, ReadFileTool, Tool, ToolRegistry, WriteFileTool,
    WebSearchTool, FetchUrlTool, ApiRequestTool,
};

// Config exports - B-W05-03: ConfigManager::new, enable_hot_reload
pub use config::{Config, ConfigManager, ConfigLoader, HotReloadHandle};
pub use config::{CliArgs, ConfigError, FeaturePreset};

pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;
pub const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Create default tool registry with all 5 tools registered
pub fn create_default_registry() -> ToolRegistry {
    use std::sync::Arc;

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(ReadFileTool::new()));
    registry.register(Arc::new(WriteFileTool::new()));
    registry.register(Arc::new(BashTool::new()));
    registry.register(Arc::new(GrepTool::new()));
    registry.register(Arc::new(LsTool::new()));
    registry
}
