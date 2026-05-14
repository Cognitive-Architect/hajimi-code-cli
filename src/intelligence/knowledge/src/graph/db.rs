//! SQLite数据库连接池与事务封装（零unsafe）
use super::{EntityType, GraphError, Node, Result};
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

/// 图数据库连接池
pub struct GraphDb {
    pub(crate) conn: Connection, // E-002修复：添加 pub(crate)
}

impl GraphDb {
    /// 打开数据库（自动初始化Schema）
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path).map_err(GraphError::Database)?;
        conn.execute_batch(include_str!("schema.sql"))
            .map_err(GraphError::Database)?;
        Ok(Self { conn })
    }

    /// 插入节点（事务包裹）
    pub fn insert_node(&mut self, node: &Node) -> Result<()> {
        let tx = self.conn.transaction().map_err(GraphError::Database)?;
        let embedding_blob = node
            .embedding
            .as_ref()
            .map(|e| bincode::serialize(e).map_err(|_| GraphError::Serialization))
            .transpose()?;
        tx.execute(
            "INSERT OR REPLACE INTO nodes (id, label, entity_type, properties, embedding, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![node.id, node.label, format!("{:?}", node.entity_type),
                node.properties.to_string(), embedding_blob, node.created_at.timestamp(), node.updated_at.timestamp()],
        ).map_err(GraphError::Database)?;
        tx.commit().map_err(GraphError::Database)
    }

    /// 获取节点
    pub fn get_node(&self, node_id: &str) -> Result<Node> {
        self.conn
            .query_row(
                "SELECT id, label, entity_type, properties, embedding FROM nodes WHERE id = ?1",
                [node_id],
                Self::row_to_node_with_embedding,
            )
            .map_err(GraphError::Database)
    }

    /// 获取节点嵌入
    pub fn get_node_embedding(&self, node_id: &str) -> Result<Option<Vec<f32>>> {
        let result: std::result::Result<Option<Vec<u8>>, rusqlite::Error> = self
            .conn
            .query_row(
                "SELECT embedding FROM nodes WHERE id = ?1",
                [node_id],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional();
        match result.map_err(GraphError::Database)? {
            Some(blob) => Ok(Some(
                bincode::deserialize(&blob).map_err(|_| GraphError::Serialization)?,
            )),
            None => Ok(None),
        }
    }

    /// 获取所有节点
    pub fn get_all_nodes(&self) -> Result<Vec<Node>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, label, entity_type, properties, embedding FROM nodes")
            .map_err(GraphError::Database)?;
        let rows = stmt
            .query_map([], Self::row_to_node_with_embedding)
            .map_err(GraphError::Database)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(GraphError::Database)
    }

    // E-001修复：提取公共辅助方法
    /// 将数据库行转换为Node（pub(crate)供其他模块使用）
    #[allow(dead_code)]
    pub(crate) fn row_to_node(row: &rusqlite::Row) -> rusqlite::Result<Node> {
        use chrono::Utc;
        let entity_type: String = row.get(2)?;
        Ok(Node {
            id: row.get(0)?,
            label: row.get(1)?,
            entity_type: match entity_type.as_str() {
                "ADR" => EntityType::ADR,
                "Function" => EntityType::Function,
                "Module" => EntityType::Module,
                _ => EntityType::Concept,
            },
            properties: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|_| {
                rusqlite::Error::InvalidParameterName("json parse failed".to_string())
            })?,
            embedding: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// 将数据库行转换为Node（含embedding）
    fn row_to_node_with_embedding(row: &rusqlite::Row) -> rusqlite::Result<Node> {
        use chrono::Utc;
        let entity_type: String = row.get(2)?;
        let embedding_blob: Option<Vec<u8>> = row.get(4).ok();
        let embedding = embedding_blob.and_then(|b| bincode::deserialize(&b).ok());
        Ok(Node {
            id: row.get(0)?,
            label: row.get(1)?,
            entity_type: match entity_type.as_str() {
                "ADR" => EntityType::ADR,
                "Function" => EntityType::Function,
                "Module" => EntityType::Module,
                _ => EntityType::Concept,
            },
            properties: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|_| {
                rusqlite::Error::InvalidParameterName("json parse failed".to_string())
            })?,
            embedding,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}
