use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum GraphError { NotFound(String), DepthLimitExceeded, InvalidEntityId }

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Entity not found: {}", id),
            Self::DepthLimitExceeded => write!(f, "Depth limit exceeded (max 2 hops)"),
            Self::InvalidEntityId => write!(f, "Invalid entity ID"),
        }
    }
}

impl std::error::Error for GraphError {}

#[derive(Clone, Debug)]
pub struct KnowledgeGraph { nodes: HashMap<String, Node>, edges: Vec<Edge> }

#[derive(Clone, Debug)]
pub struct Node { pub id: String, pub label: String }

#[derive(Clone, Debug)]
pub struct Edge { pub source_id: String, pub target_id: String, pub relation: String }

impl KnowledgeGraph {
    pub fn new() -> Self { Self { nodes: HashMap::new(), edges: Vec::new() } }
    pub fn add_node(&mut self, id: &str, label: &str) { self.nodes.insert(id.to_string(), Node { id: id.to_string(), label: label.to_string() }); }
    pub fn add_edge(&mut self, source: &str, target: &str, relation: &str) { self.edges.push(Edge { source_id: source.to_string(), target_id: target.to_string(), relation: relation.to_string() }); }
    pub fn get_neighbors(&self, node_id: &str) -> Vec<(&Edge, &Node)> { self.edges.iter().filter(|e| e.source_id == node_id).filter_map(|e| self.nodes.get(&e.target_id).map(|n| (e, n))).collect() }
    pub fn get_node(&self, id: &str) -> Option<&Node> { self.nodes.get(id) }
}

impl Default for KnowledgeGraph { fn default() -> Self { Self::new() } }

#[derive(Clone, Debug, PartialEq)]
pub struct Path { pub segments: Vec<PathSegment> }

#[derive(Clone, Debug, PartialEq)]
pub struct PathSegment { pub from: String, pub to: String, pub relation: String }

impl Path {
    pub fn new() -> Self { Self { segments: Vec::new() } }
    pub fn len(&self) -> usize { self.segments.len() }
    pub fn is_empty(&self) -> bool { self.segments.is_empty() }
    pub fn start(&self) -> Option<&str> { self.segments.first().map(|s| s.from.as_str()) }
    pub fn end(&self) -> Option<&str> { self.segments.last().map(|s| s.to.as_str()) }
}

impl Default for Path { fn default() -> Self { Self::new() } }

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.segments.is_empty() { return write!(f, "(empty path)"); }
        write!(f, "{} --[{}]--> {}", self.segments[0].from, self.segments[0].relation, self.segments[0].to)?;
        for seg in &self.segments[1..] { write!(f, " --[{}]--> {}", seg.relation, seg.to)?; }
        Ok(())
    }
}

pub fn bfs_traverse(graph: &KnowledgeGraph, start_id: &str, max_depth: u8) -> Result<Vec<String>, GraphError> {
    if max_depth > 2 { return Err(GraphError::DepthLimitExceeded); }
    if graph.get_node(start_id).is_none() { return Err(GraphError::NotFound(start_id.to_string())); }
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();
    queue.push_back((start_id.to_string(), 0u8));
    visited.insert(start_id.to_string());
    while let Some((node_id, depth)) = queue.pop_front() {
        result.push(node_id.clone());
        if depth >= max_depth { continue; }
        for (edge, _) in graph.get_neighbors(&node_id) { if visited.insert(edge.target_id.clone()) { queue.push_back((edge.target_id.clone(), depth + 1)); } }
    }
    Ok(result)
}

pub fn dfs_traverse(graph: &KnowledgeGraph, start_id: &str, max_depth: u8) -> Result<Vec<String>, GraphError> {
    if max_depth > 2 { return Err(GraphError::DepthLimitExceeded); }
    if graph.get_node(start_id).is_none() { return Err(GraphError::NotFound(start_id.to_string())); }
    let mut visited = HashSet::new();
    let mut stack = vec![(start_id.to_string(), 0u8)];
    let mut result = Vec::new();
    while let Some((node_id, depth)) = stack.pop() {
        if visited.contains(&node_id) { continue; }
        visited.insert(node_id.clone());
        result.push(node_id.clone());
        if depth >= max_depth { continue; }
        for (edge, _) in graph.get_neighbors(&node_id) { if !visited.contains(&edge.target_id) { stack.push((edge.target_id.clone(), depth + 1)); } }
    }
    Ok(result)
}

pub fn find_paths(graph: &KnowledgeGraph, start_id: &str, end_id: &str, max_depth: u8) -> Result<Vec<Path>, GraphError> {
    if max_depth > 2 { return Err(GraphError::DepthLimitExceeded); }
    if graph.get_node(start_id).is_none() { return Err(GraphError::NotFound(start_id.to_string())); }
    if graph.get_node(end_id).is_none() { return Err(GraphError::NotFound(end_id.to_string())); }
    let mut paths = Vec::new();
    let mut stack: Vec<(String, Path, HashSet<String>)> = vec![(start_id.to_string(), Path::new(), HashSet::new())];
    while let Some((node_id, path, visited)) = stack.pop() {
        if node_id == end_id && !path.is_empty() { paths.push(path); continue; }
        if path.len() >= max_depth as usize { continue; }
        for (edge, _) in graph.get_neighbors(&node_id) {
            if visited.contains(&edge.target_id) { continue; }
            let mut new_path = path.clone();
            let mut new_visited = visited.clone();
            new_path.segments.push(PathSegment { from: node_id.clone(), to: edge.target_id.clone(), relation: edge.relation.clone() });
            new_visited.insert(node_id.clone());
            stack.push((edge.target_id.clone(), new_path, new_visited));
        }
    }
    Ok(paths)
}

pub struct MemoryGateway { graph: KnowledgeGraph }

impl MemoryGateway {
    pub fn new(graph: KnowledgeGraph) -> Self { Self { graph } }
    pub fn query_graph(&self, entity_id: Uuid, depth: u8) -> Result<Vec<Path>, GraphError> {
        if depth > 2 { return Err(GraphError::DepthLimitExceeded); }
        let id_str = entity_id.to_string();
        if self.graph.get_node(&id_str).is_none() { return Err(GraphError::InvalidEntityId); }
        let mut paths = Vec::new();
        for (_, node) in self.graph.get_neighbors(&id_str) {
            let mut path = Path::new();
            path.segments.push(PathSegment { from: id_str.clone(), to: node.id.clone(), relation: "relates_to".to_string() });
            paths.push(path);
        }
        if depth == 2 {
            let neighbors: Vec<String> = self.graph.get_neighbors(&id_str).into_iter().map(|(_, n)| n.id.clone()).collect();
            for n1_id in neighbors {
                for (_, n2) in self.graph.get_neighbors(&n1_id) {
                    let mut path = Path::new();
                    path.segments.push(PathSegment { from: id_str.clone(), to: n1_id.clone(), relation: "relates_to".to_string() });
                    path.segments.push(PathSegment { from: n1_id.clone(), to: n2.id.clone(), relation: "relates_to".to_string() });
                    paths.push(path);
                }
            }
        }
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn create_test_graph() -> KnowledgeGraph {
        let mut g = KnowledgeGraph::new();
        g.add_node("A", "Node A");
        g.add_node("B", "Node B");
        g.add_node("C", "Node C");
        g.add_node("D", "Node D");
        g.add_edge("A", "B", "knows");
        g.add_edge("B", "C", "works_with");
        g.add_edge("A", "D", "owns");
        g
    }

    #[test]
    fn test_bfs_basic_two_hop() {
        let g = create_test_graph();
        let result = bfs_traverse(&g, "A", 2).unwrap();
        assert!(result.contains(&"A".to_string()) && result.contains(&"B".to_string()));
        assert!(result.contains(&"C".to_string()) && result.contains(&"D".to_string()));
    }

    #[test]
    fn test_dfs_deep_search() {
        let g = create_test_graph();
        let result = dfs_traverse(&g, "A", 2).unwrap();
        assert_eq!(result[0], "A");
        assert!(result.contains(&"B".to_string()) && result.contains(&"C".to_string()));
    }

    #[test]
    fn test_no_path_returns_empty() {
        let g = create_test_graph();
        assert!(find_paths(&g, "C", "A", 2).unwrap().is_empty());
    }

    #[test]
    fn test_cycle_detection_terminates() {
        let mut g = KnowledgeGraph::new();
        g.add_node("A", "Node A");
        g.add_node("B", "Node B");
        g.add_edge("A", "B", "rel");
        g.add_edge("B", "A", "rel_back");
        assert_eq!(bfs_traverse(&g, "A", 2).unwrap().len(), 2);
    }

    #[test]
    fn test_depth_limit_enforced() {
        let g = create_test_graph();
        assert!(bfs_traverse(&g, "A", 3).is_err() && dfs_traverse(&g, "A", 3).is_err());
        assert!(find_paths(&g, "A", "C", 3).is_err());
    }

    #[test]
    fn test_invalid_entity_id_handled() {
        let g = KnowledgeGraph::new();
        assert!(bfs_traverse(&g, "nonexistent", 1).is_err());
        assert!(dfs_traverse(&g, "nonexistent", 1).is_err());
    }

    #[test]
    fn test_path_format_display() {
        let mut path = Path::new();
        path.segments.push(PathSegment { from: "A".to_string(), to: "B".to_string(), relation: "knows".to_string() });
        let s = path.to_string();
        assert!(s.contains("A") && s.contains("B") && s.contains("knows"));
    }

    #[test]
    fn test_gateway_integration_query() {
        let mut g = KnowledgeGraph::new();
        let id_a = Uuid::new_v4();
        g.add_node(&id_a.to_string(), "Entity A");
        g.add_node(&Uuid::new_v4().to_string(), "Entity B");
        let gateway = MemoryGateway::new(g);
        assert!(gateway.query_graph(id_a, 1).is_ok());
    }

    #[test]
    fn test_1000_nodes_query_latency() {
        let mut g = KnowledgeGraph::new();
        for i in 0..1000 { g.add_node(&format!("node{}", i), &format!("Node {}", i)); }
        for i in 0..999 { g.add_edge(&format!("node{}", i), &format!("node{}", i + 1), "next"); }
        let start = Instant::now();
        let _ = bfs_traverse(&g, "node0", 2);
        assert!(start.elapsed().as_millis() < 100);
    }

    #[test]
    fn test_concurrent_query_isolation() {
        use std::thread;
        let g = create_test_graph();
        let handles: Vec<_> = (0..10).map(|i| {
            let gc = g.clone();
            thread::spawn(move || bfs_traverse(&gc, "A", 2).map(|r| (i, r)))
        }).collect();
        assert_eq!(handles.into_iter().map(|h| h.join().unwrap()).count(), 10);
    }

    #[test]
    fn test_query_result_deterministic() {
        let g = create_test_graph();
        assert_eq!(bfs_traverse(&g, "A", 2).unwrap(), bfs_traverse(&g, "A", 2).unwrap());
    }

    #[test]
    fn test_find_paths_returns_valid_paths() {
        let g = create_test_graph();
        let paths = find_paths(&g, "A", "C", 2).unwrap();
        assert!(!paths.is_empty() && paths[0].segments.len() == 2);
        assert_eq!(paths[0].start(), Some("A"));
        assert_eq!(paths[0].end(), Some("C"));
    }
}
