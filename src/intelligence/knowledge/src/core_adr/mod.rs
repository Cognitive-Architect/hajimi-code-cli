//! ADR (Architecture Decision Records) 知识管理模块
//! Week 33: Frontmatter解析 + 目录监听基础

pub mod cli;
pub mod generator;
pub mod models;
pub mod parser;
pub mod watcher;

pub use cli::AdrCli;
pub use generator::AdrGenerator;
pub use models::{AdrEntry, AdrIndex, AdrStatus, GraphEntity};
pub use parser::{generate_frontmatter, parse_adr};
pub use watcher::AdrWatcher;

/// ADR模块错误类型
#[derive(Debug, thiserror::Error)]
pub enum AdrError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("Frontmatter解析错误: {0}")]
    Parse(String),
    #[error("缺少必填字段: {0}")]
    MissingField(String),
    #[error("重复ADR ID: {0}")]
    DuplicateId(String),
    #[error("锁错误: {0}")]
    Lock(String),
    #[error("无效状态: {0}")]
    InvalidStatus(String),
}

pub type Result<T> = std::result::Result<T, AdrError>;
