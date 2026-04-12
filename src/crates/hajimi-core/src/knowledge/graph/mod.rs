//! 知识图谱模块（SQLite三表Schema）
#![deny(unsafe_code)]

use thiserror::Error;

pub mod adapters;
pub mod attention;
pub mod db;
pub mod edge_ops;
pub mod embedder;
pub mod extractor;
pub mod gnn_impl;
pub mod models;
pub mod relations;
pub mod traversal;

pub use db::GraphDb;
pub use embedder::Embedder;
pub use extractor::EntityExtractor;
pub use gnn_impl::GnnEngine;
pub use models::{Edge, EntityType, Node, Relation};
pub use relations::{RelationExtractor, insert_relations_batch};

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
    #[error("无效的模式: {0}")]
    InvalidPattern(String),
}

/// 知识图谱Result类型别名
pub type Result<T> = std::result::Result<T, GraphError>;
