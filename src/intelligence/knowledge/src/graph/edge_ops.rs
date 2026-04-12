//! Edges表操作（插入/查询/删除）
use super::{Edge, GraphError, Node, Result};
use rusqlite::params;

impl super::GraphDb {
    pub fn insert_edge(&mut self, edge: &Edge) -> Result<()> {
        let tx = self.conn.transaction().map_err(GraphError::Database)?;
        tx.execute(
            "INSERT INTO edges (from_id, to_id, rel_type, weight, source, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![edge.from_id, edge.to_id, edge.rel_type, edge.weight, "", 0],
        ).map_err(GraphError::Database)?;
        tx.commit().map_err(GraphError::Database)
    }

    pub fn get_neighbors(&self, node_id: &str, rel_type: Option<&str>) -> Result<Vec<Node>> {
        let sql = if rel_type.is_some() {
            "SELECT n.* FROM nodes n JOIN edges e ON n.id = e.to_id WHERE e.from_id = ?1 AND e.rel_type = ?2 ORDER BY e.weight DESC"
        } else {
            "SELECT n.* FROM nodes n JOIN edges e ON n.id = e.to_id WHERE e.from_id = ?1 ORDER BY e.weight DESC"
        };
        let mut stmt = self.conn.prepare(sql).map_err(GraphError::Database)?;
        let rows = if let Some(rt) = rel_type {
            stmt.query_map(params![node_id, rt], super::GraphDb::row_to_node)
        } else {
            stmt.query_map(params![node_id], super::GraphDb::row_to_node)
        }.map_err(GraphError::Database)?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(GraphError::Database)
    }

    pub fn insert_edges_batch(&mut self, edges: &[Edge]) -> Result<()> {
        let tx = self.conn.transaction().map_err(GraphError::Database)?;
        for edge in edges {
            tx.execute(
                "INSERT INTO edges (from_id, to_id, rel_type, weight, source, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![edge.from_id, edge.to_id, edge.rel_type, edge.weight, "", 0],
            ).map_err(GraphError::Database)?;
        }
        tx.commit().map_err(GraphError::Database)
    }
}
