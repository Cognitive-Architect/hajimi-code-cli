//! Complexity Analysis Tool - B-W12/04

use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use syn::{visit::Visit, ExprBinary, ExprIf, ExprWhile, ExprForLoop, ExprMatch, ExprLoop, BinOp, ItemFn, File};
use tokio::fs::read_to_string;
use crate::{Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions};

pub struct AnalyzeTool;
impl AnalyzeTool { pub fn new() -> Self { Self } }
impl Default for AnalyzeTool { fn default() -> Self { Self::new() } }

#[derive(Debug, Deserialize)]
struct AnalyzeArgs { path: String }

#[derive(Debug, Clone)]
pub struct Complexity { pub cyclomatic: u32, pub cognitive: u32, pub function: String }

pub struct ComplexityVisitor { cyclomatic: u32, cognitive: u32, nesting: u32, #[allow(dead_code)] function: String }
impl ComplexityVisitor {
    fn new(name: &str) -> Self { Self { cyclomatic: 1, cognitive: 0, nesting: 0, function: name.to_string() } }
    fn decision_point(&mut self) { self.cyclomatic += 1; self.cognitive += 1 + self.nesting; }
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr_if(&mut self, i: &'ast ExprIf) { self.decision_point(); self.nesting += 1; syn::visit::visit_expr_if(self, i); self.nesting -= 1; }
    fn visit_expr_while(&mut self, w: &'ast ExprWhile) { self.decision_point(); self.nesting += 1; syn::visit::visit_expr_while(self, w); self.nesting -= 1; }
    fn visit_expr_for_loop(&mut self, f: &'ast ExprForLoop) { self.decision_point(); self.nesting += 1; syn::visit::visit_expr_for_loop(self, f); self.nesting -= 1; }
    fn visit_expr_match(&mut self, m: &'ast ExprMatch) { self.decision_point(); self.nesting += 1; syn::visit::visit_expr_match(self, m); self.nesting -= 1; }
    fn visit_expr_loop(&mut self, l: &'ast ExprLoop) { self.decision_point(); self.nesting += 1; syn::visit::visit_expr_loop(self, l); self.nesting -= 1; }
    fn visit_expr_binary(&mut self, b: &'ast ExprBinary) {
        if matches!(b.op, BinOp::And(_) | BinOp::Or(_)) { self.cyclomatic += 1; self.cognitive += self.nesting + 1; }
        syn::visit::visit_expr_binary(self, b);
    }
    fn visit_item_fn(&mut self, f: &'ast ItemFn) { self.nesting += 1; syn::visit::visit_item_fn(self, f); self.nesting -= 1; }
}

fn analyze_file(content: &str) -> Result<Vec<Complexity>, ToolError> {
    let syntax: File = syn::parse_str(content).map_err(|e| ToolError::new(format!("ParseError: {}", e)))?;
    let mut results = Vec::new();
    for item in &syntax.items {
        if let syn::Item::Fn(f) = item {
            let name = f.sig.ident.to_string();
            let mut v = ComplexityVisitor::new(&name);
            v.visit_item_fn(f);
            results.push(Complexity { cyclomatic: v.cyclomatic, cognitive: v.cognitive, function: name });
        }
    }
    Ok(results)
}

#[async_trait]
impl Tool for AnalyzeTool {
    fn name(&self) -> &str { "analyze_complexity" }
    fn description(&self) -> &str { "Analyze cyclomatic and cognitive complexity of Rust code" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None } }
    fn is_enabled(&self, _config: &Config) -> bool { true }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: AnalyzeArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let p = Path::new(&a.path);
        if !p.exists() { return Err(ToolError::new(format!("Not found: {}", a.path))); }
        let mut results = Vec::new();
        if p.is_file() {
            let c = read_to_string(p).await.map_err(|e| ToolError::new(format!("Read: {}", e)))?;
            results.extend(analyze_file(&c)?);
        } else {
            let mut d = tokio::fs::read_dir(p).await.map_err(|e| ToolError::new(format!("Dir: {}", e)))?;
            while let Ok(Some(e)) = d.next_entry().await {
                let fp = e.path();
                if fp.extension().and_then(|s| s.to_str()) == Some("rs") {
                    if let Ok(c) = read_to_string(&fp).await { if let Ok(r) = analyze_file(&c) { results.extend(r); } }
                }
            }
        }
        let out = results.iter().map(|c| format!("{}: cyclo={}, cog={}", c.function, c.cyclomatic, c.cognitive)).collect::<Vec<_>>().join("\n");
        Ok(ToolOutput { stdout: out, stderr: String::new(), exit_code: Some(0) })
    }
}

pub fn analyze_complexity(content: &str) -> Result<Vec<Complexity>, ToolError> { analyze_file(content) }
pub use Complexity as ComplexityResult;
