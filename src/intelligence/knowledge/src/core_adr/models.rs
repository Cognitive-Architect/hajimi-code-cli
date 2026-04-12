//! ADR领域模型

use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// ADR状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Rejected,
}

/// ADR条目结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdrEntry {
    pub id: String,           // ADR-NNNN
    pub title: String,
    pub status: AdrStatus,
    pub date: DateTime<Utc>,
    pub tags: Vec<String>,
    pub content: String,      // Markdown内容（不含Frontmatter）
}

/// ADR索引（内存缓存）
#[derive(Debug, Default)]
pub struct AdrIndex {
    pub entries: Vec<AdrEntry>,
}

/// 知识图谱实体结构（Week 34预留）
#[derive(Debug, Clone)]
pub struct GraphEntity {
    pub id: String,
    pub label: String,
    pub entity_type: String,
    pub properties: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
}

/// Graph实体转换接口（供Week 34知识图谱调用）
impl AdrEntry {
    pub fn to_entity(&self) -> GraphEntity {
        GraphEntity {
            id: self.id.clone(),
            label: self.title.clone(),
            entity_type: "ADR".to_string(),
            properties: serde_json::json!({
                "title": self.title,
                "status": format!("{:?}", self.status),
                "date": self.date.to_rfc3339(),
                "tags": self.tags,
            }),
            embedding: None,
        }
    }
}
