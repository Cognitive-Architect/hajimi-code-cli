//! 文档与重构工具 - B-W12/03: generate_docs + update_readme + refactor_code

use super::{
    edit_file, EditOperation, PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind,
    ToolOutput, ToolPermissions,
};
use serde_json::Value;
use std::path::PathBuf;
use syn::{parse_file, visit::Visit, ImplItemFn, ItemEnum, ItemFn, ItemImpl, ItemStruct};

pub struct GenerateDocsTool;
pub struct UpdateReadmeTool;
pub struct RefactorCodeTool;

impl Default for GenerateDocsTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerateDocsTool {
    pub fn new() -> Self {
        Self
    }
}
impl Default for UpdateReadmeTool {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateReadmeTool {
    pub fn new() -> Self {
        Self
    }
}
impl Default for RefactorCodeTool {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactorCodeTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for GenerateDocsTool {
    fn name(&self) -> &str {
        "generate_docs"
    }
    fn description(&self) -> &str {
        "Extract doc comments from Rust code"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("Missing path"))?;
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => ToolError {
                    message: format!("File not found: {}", path),
                    kind: ToolErrorKind::InvalidArgs,
                },
                _ => ToolError::new(format!("Read error: {}", e)),
            })?;
        let ast = parse_file(&content).map_err(|e| ToolError {
            message: format!("AST parse failed: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;
        let mut extractor = DocExtractor::default();
        extractor.visit_file(&ast);
        let docs = extractor.docs.join("\n\n---\n\n");
        Ok(ToolOutput::success(if docs.is_empty() {
            "No doc comments found".into()
        } else {
            docs
        }))
    }
}

#[async_trait::async_trait]
impl Tool for UpdateReadmeTool {
    fn name(&self) -> &str {
        "update_readme"
    }
    fn description(&self) -> &str {
        "Update README.md content"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: true,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("README.md");
        let content = args
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("Missing content"))?;
        let pos = args
            .get("position")
            .and_then(Value::as_str)
            .unwrap_or("end");
        let path_buf: PathBuf = path.into();
        if !path_buf.exists() {
            tokio::fs::write(&path_buf, "# README\n\n")
                .await
                .map_err(|e| ToolError::new(format!("Create failed: {}", e)))?;
        }
        let op = match pos {
            "start" => EditOperation::Insert {
                line: 3,
                content: content.to_string(),
                after: false,
            },
            _ => EditOperation::Replace {
                old: "".into(),
                new: content.to_string(),
            },
        };
        let edited = if matches!(op, EditOperation::Replace { .. }) {
            let old = tokio::fs::read_to_string(&path_buf)
                .await
                .map_err(|e| ToolError::new(format!("Read: {}", e)))?;
            format!("{}\n{}", old.trim_end(), content)
        } else {
            edit_file(&path_buf, &op, false).await?
        };
        edit_file(
            &path_buf,
            &EditOperation::Replace {
                old: tokio::fs::read_to_string(&path_buf)
                    .await
                    .unwrap_or_default(),
                new: edited,
            },
            false,
        )
        .await?;
        Ok(ToolOutput::success("README updated"))
    }
}

#[async_trait::async_trait]
impl Tool for RefactorCodeTool {
    fn name(&self) -> &str {
        "refactor_code"
    }
    fn description(&self) -> &str {
        "Analyze code and suggest refactorings"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("Missing path"))?;
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => ToolError {
                    message: format!("File not found: {}", path),
                    kind: ToolErrorKind::InvalidArgs,
                },
                _ => ToolError::new(format!("Read error: {}", e)),
            })?;
        let ast = parse_file(&content).map_err(|e| ToolError {
            message: format!("AST parse failed: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;
        let mut analyzer = RefactorAnalyzer::default();
        analyzer.visit_file(&ast);
        let suggestions = analyzer.suggestions.join("\n");
        Ok(ToolOutput::success(if suggestions.is_empty() {
            "No refactoring suggestions".into()
        } else {
            suggestions
        }))
    }
}

#[derive(Default)]
struct DocExtractor {
    docs: Vec<String>,
}
impl<'ast> Visit<'ast> for DocExtractor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if let Some(doc) = extract_doc(&node.attrs) {
            self.docs
                .push(format!("### fn {}\n{}", node.sig.ident, doc));
        }
        syn::visit::visit_item_fn(self, node);
    }
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if let Some(doc) = extract_doc(&node.attrs) {
            self.docs
                .push(format!("### struct {}\n{}", node.ident, doc));
        }
        syn::visit::visit_item_struct(self, node);
    }
    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        if let Some(doc) = extract_doc(&node.attrs) {
            self.docs.push(format!("### enum {}\n{}", node.ident, doc));
        }
        syn::visit::visit_item_enum(self, node);
    }
}

#[derive(Default)]
struct RefactorAnalyzer {
    suggestions: Vec<String>,
    fn_count: usize,
    impl_fn_count: usize,
}
impl<'ast> Visit<'ast> for RefactorAnalyzer {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.fn_count += 1;
        let lines = quote::quote!(#node).to_string().lines().count();
        if lines > 50 {
            self.suggestions.push(format!(
                "fn {}: {} lines, consider splitting",
                node.sig.ident, lines
            ));
        }
        if node.sig.inputs.len() > 5 {
            self.suggestions.push(format!(
                "fn {}: {} params, consider struct",
                node.sig.ident,
                node.sig.inputs.len()
            ));
        }
        if extract_doc(&node.attrs).is_none() {
            self.suggestions
                .push(format!("fn {}: missing docs", node.sig.ident));
        }
        syn::visit::visit_item_fn(self, node);
    }
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        for item in &node.items {
            if let syn::ImplItem::Fn(ImplItemFn { attrs, sig, .. }) = item {
                self.impl_fn_count += 1;
                if extract_doc(attrs).is_none() {
                    self.suggestions
                        .push(format!("impl fn {}: missing docs", sig.ident));
                }
            }
        }
        syn::visit::visit_item_impl(self, node);
    }
}

fn extract_doc(attrs: &[syn::Attribute]) -> Option<String> {
    let docs: Vec<String> = attrs
        .iter()
        .filter_map(|a| {
            if a.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &a.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        return Some(s.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect();
    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_generate_docs_name() {
        assert_eq!(GenerateDocsTool.name(), "generate_docs");
    }
    #[test]
    fn test_update_readme_name() {
        assert_eq!(UpdateReadmeTool.name(), "update_readme");
    }
    #[test]
    fn test_refactor_code_name() {
        assert_eq!(RefactorCodeTool.name(), "refactor_code");
    }
    #[test]
    fn test_extract_doc_found() -> Result<(), Box<dyn std::error::Error>> {
        let code = r#"/// Hello world
fn test() {}"#;
        let ast = syn::parse_file(code)?;
        let docs: Vec<_> = ast
            .items
            .iter()
            .filter_map(|item| {
                if let syn::Item::Fn(f) = item {
                    extract_doc(&f.attrs)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(docs, vec!["Hello world"]);
        Ok(())
    }
    #[test]
    fn test_extract_doc_none() -> Result<(), Box<dyn std::error::Error>> {
        let code = r#"fn test() {}"#;
        let ast = syn::parse_file(code)?;
        let docs: Vec<_> = ast
            .items
            .iter()
            .filter_map(|item| {
                if let syn::Item::Fn(f) = item {
                    extract_doc(&f.attrs)
                } else {
                    None
                }
            })
            .collect();
        assert!(docs.is_empty());
        Ok(())
    }
}
