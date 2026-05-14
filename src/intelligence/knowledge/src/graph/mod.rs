//! 知识图谱模块（SQLite三表Schema）
#![deny(unsafe_code)]

use thiserror::Error;

pub mod attention;
pub mod core_adapters;
pub mod core_models;
pub mod db;
#[cfg(feature = "onnx")]
pub mod embedder;
pub mod extractor;
pub mod gnn;

pub use core_models::{Edge, EntityType, Node};
pub use db::GraphDb;
#[cfg(feature = "onnx")]
pub use embedder::Embedder;
pub use extractor::EntityExtractor;

/// 知识图谱错误类型
#[derive(Debug, Error)]
pub enum GraphError {
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("序列化错误")]
    Serialization,
    #[error("重复ID: {0}")]
    DuplicateId(String),
    #[error("模型加载失败: {0}")]
    ModelLoad(String),
    #[error("维度不匹配: 预期{expected}, 实际{actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("推理执行失败: {0}")]
    Inference(String),
}

/// 知识图谱Result类型别名
pub type Result<T> = std::result::Result<T, GraphError>;
