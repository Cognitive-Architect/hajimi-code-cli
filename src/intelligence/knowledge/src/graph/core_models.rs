//! 知识图谱领域模型
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EntityType {
    ADR,
    Function,
    Module,
    Concept,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub entity_type: EntityType,
    pub properties: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from_id: String,
    pub to_id: String,
    pub rel_type: String,
    pub weight: f64,
}

impl Node {
    pub fn from_graph_entity(entity: crate::core_adr::GraphEntity) -> Self {
        Self {
            id: entity.id,
            label: entity.label,
            entity_type: match entity.entity_type.as_str() {
                "ADR" => EntityType::ADR,
                _ => EntityType::Concept,
            },
            properties: entity.properties,
            embedding: entity.embedding,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// 元关系结构（Week 35关系抽取核心）
#[derive(Debug, Clone)]
pub struct Relation {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
    pub extracted_from: Option<String>,
    pub created_at: i64,
}
