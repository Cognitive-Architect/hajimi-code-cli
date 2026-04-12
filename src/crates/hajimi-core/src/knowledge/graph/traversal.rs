//! 图遍历（BFS/DFS，显式栈，零递归）
use super::{GraphError, Node, Result};
use std::collections::{HashSet, VecDeque};

impl super::GraphDb {
    pub fn bfs_traversal(&self, start_id: &str, max_depth: usize) -> Result<Vec<Node>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        queue.push_back((start_id.to_string(), 0));
        visited.insert(start_id.to_string());
        while let Some((node_id, depth)) = queue.pop_front() {
            if depth > max_depth { continue; }
            result.push(self.get_node(&node_id)?);
        
            for neighbor_id in self.get_neighbor_ids(&node_id)? {
                if visited.insert(neighbor_id.clone()) { queue.push_back((neighbor_id, depth + 1)); }
            }
        }
        Ok(result)
    }

    pub fn dfs_traversal(&self, start_id: &str, max_depth: usize) -> Result<Vec<Node>> {
        let mut visited = HashSet::new();
        let mut stack = vec![(start_id.to_string(), 0)];
        let mut result = Vec::new();
        while let Some((node_id, depth)) = stack.pop() {
            if depth > max_depth || visited.contains(&node_id) { continue; }
            visited.insert(node_id.clone());
            result.push(self.get_node(&node_id)?);
            for neighbor_id in self.get_neighbor_ids(&node_id)? {
                if !visited.contains(&neighbor_id) { stack.push((neighbor_id, depth + 1)); }
            }
        }
        Ok(result)
    }

    fn get_neighbor_ids(&self, node_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT to_id FROM edges WHERE from_id = ?1 ORDER BY weight DESC").map_err(GraphError::Database)?;
        let rows = stmt.query_map([node_id], |row| row.get(0)).map_err(GraphError::Database)?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(GraphError::Database)
    }
}
