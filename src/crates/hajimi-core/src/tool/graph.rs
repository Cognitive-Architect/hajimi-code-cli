//! Dependency Graph Tool - B-W12/04

use async_trait::async_trait;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use syn::{visit::Visit, ItemUse, UseTree, File};
use tokio::fs::read_to_string;
use crate::tool::{Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions};

pub struct GraphTool;
impl GraphTool { pub fn new() -> Self { Self } }
impl Default for GraphTool { fn default() -> Self { Self::new() } }

#[derive(Debug, Deserialize)]
struct GraphArgs { path: String, #[serde(default = "default_format")] format: String }
fn default_format() -> String { "mermaid".to_string() }

#[derive(Debug, Clone)]
pub struct DepGraph { graph: DiGraph<String, ()>, nodes: HashMap<String, NodeIndex> }

impl DepGraph {
    fn new() -> Self { Self { graph: DiGraph::new(), nodes: HashMap::new() } }
    fn get_or_add(&mut self, name: &str) -> NodeIndex {
        if let Some(&n) = self.nodes.get(name) { n } else { let n = self.graph.add_node(name.to_string()); self.nodes.insert(name.to_string(), n); n }
    }
    fn add_dep(&mut self, from: &str, to: &str) {
        let a = self.get_or_add(from); let b = self.get_or_add(to);
        if a != b && !self.graph.contains_edge(a, b) { self.graph.add_edge(a, b, ()); }
    }
    fn to_mermaid(&self) -> String {
        let mut out = vec!["graph TD".to_string()];
        for e in self.graph.edge_indices() {
            if let Some((a, b)) = self.graph.edge_endpoints(e) {
                out.push(format!("    {} --> {}", self.graph[a], self.graph[b]));
            }
        }
        out.join("\n")
    }
    fn to_dot(&self) -> String {
        let mut out = vec!["digraph deps {".to_string()];
        for e in self.graph.edge_indices() {
            if let Some((a, b)) = self.graph.edge_endpoints(e) {
                out.push(format!("    \"{}\" -> \"{}\";", self.graph[a], self.graph[b]));
            }
        }
        out.push("}".to_string()); out.join("\n")
    }
}

struct UseVisitor { #[allow(dead_code)] module: String, deps: Vec<String> }
impl<'ast> Visit<'ast> for UseVisitor {
    fn visit_item_use(&mut self, i: &'ast ItemUse) { extract_use(&i.tree, &mut self.deps, ""); }
}

fn extract_use(tree: &UseTree, deps: &mut Vec<String>, prefix: &str) {
    match tree {
        UseTree::Path(p) => { let np = if prefix.is_empty() { p.ident.to_string() } else { format!("{}::{}", prefix, p.ident) }; extract_use(&p.tree, deps, &np); }
        UseTree::Name(n) => { let f = if prefix.is_empty() { n.ident.to_string() } else { format!("{}::{}", prefix, n.ident) }; deps.push(f); }
        UseTree::Rename(r) => { let f = if prefix.is_empty() { r.rename.to_string() } else { format!("{}::{}", prefix, r.rename) }; deps.push(f); }
        UseTree::Glob(_) => { if !prefix.is_empty() { deps.push(format!("{}::*", prefix)); } }
        UseTree::Group(g) => { for t in &g.items { extract_use(t, deps, prefix); } }
    }
}

fn analyze_deps(content: &str, module: &str) -> Result<Vec<String>, ToolError> {
    let syntax: File = syn::parse_str(content).map_err(|e| ToolError::new(format!("ParseError: {}", e)))?;
    let mut v = UseVisitor { module: module.to_string(), deps: Vec::new() };
    v.visit_file(&syntax);
    Ok(v.deps)
}

#[async_trait]
impl Tool for GraphTool {
    fn name(&self) -> &str { "dependency_graph" }
    fn description(&self) -> &str { "Generate dependency graph in DOT or Mermaid format" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None } }
    fn is_enabled(&self, _config: &Config) -> bool { true }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: GraphArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let p = Path::new(&a.path);
        if !p.exists() { return Err(ToolError::new(format!("Not found: {}", a.path))); }
        let mut g = DepGraph::new();
        let mut warned = false;
        if p.is_file() {
            let c = read_to_string(p).await.map_err(|e| ToolError::new(format!("Read: {}", e)))?;
            let m = p.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
            for d in analyze_deps(&c, m)? { g.add_dep(m, &d); }
        } else {
            let mut files = Vec::new();
            let mut d = tokio::fs::read_dir(p).await.map_err(|e| ToolError::new(format!("Dir: {}", e)))?;
            while let Ok(Some(e)) = d.next_entry().await {
                let fp = e.path();
                if fp.extension().and_then(|s| s.to_str()) == Some("rs") { files.push(fp); }
            }
            for fp in &files {
                let m = fp.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
                let c = read_to_string(fp).await.map_err(|e| ToolError::new(format!("Read: {}", e)))?;
                for d in analyze_deps(&c, m)? { g.add_dep(m, &d); }
            }
            for e in g.graph.edge_indices() {
                if let Some((a, b)) = g.graph.edge_endpoints(e) {
                    let from = &g.graph[a]; let _to = &g.graph[b];
                    if g.graph.edges(b).any(|e| g.graph[e.target()] == *from) && !warned {
                        warned = true;
                    }
                }
            }
        }
        let out = if a.format == "dot" { g.to_dot() } else { g.to_mermaid() };
        let warn = if warned { "Warning: Circular dependencies detected\n".to_string() } else { String::new() };
        Ok(ToolOutput { stdout: out, stderr: warn, exit_code: Some(0) })
    }
}

pub fn generate_graph(content: &str, module: &str) -> Result<DepGraph, ToolError> {
    let mut g = DepGraph::new();
    for d in analyze_deps(content, module)? { g.add_dep(module, &d); }
    Ok(g)
}

pub fn graph_to_mermaid(g: &DepGraph) -> String { g.to_mermaid() }
pub fn graph_to_dot(g: &DepGraph) -> String { g.to_dot() }
