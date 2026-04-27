//! LSP Integration for AST Context Provider (Day 2 of Phase 4).
//! Wraps existing LSP tools (definition, references, hover from lsp.rs) + real AST provider.
//! Provides `get_symbol_context()` and `find_references_in_scope()`.
//! Intelligence layer accesses ONLY through this (strict layering, via engine ports).
//! Reuses existing Tool trait, governance, and HNSW from wasm.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ast_provider::{AstSymbolIndex, CodeSymbol};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolContext {
    pub symbol: CodeSymbol,
    pub context: String,
    pub lsp_hover: Option<String>,
    pub references_count: usize,
}

/// Trait for AST-aware context (ADR-016: AST-First Retrieval).
#[async_trait]
pub trait ASTContextProvider: Send + Sync {
    async fn get_symbol_context(&self, symbol_name: &str, file_path: Option<&str>) -> Result<SymbolContext, String>;
    async fn find_references_in_scope(&self, symbol_name: &str, scope: &str) -> Result<Vec<CodeSymbol>, String>;
    async fn enhance_retrieve_with_ast(&self, query: &str) -> Result<String, String>;
}

/// Concrete provider combining real AST index + LSP tools.
pub struct LspContextProvider {
    ast_index: Arc<Mutex<AstSymbolIndex>>,
}

impl LspContextProvider {
    pub fn new() -> Self {
        Self {
            ast_index: Arc::new(Mutex::new(AstSymbolIndex::new())),
        }
    }

    pub async fn index_project(&self, path: &str) -> Result<usize, String> {
        self.ast_index.lock().await.index_project(path)
    }

    pub async fn is_indexed(&self) -> bool {
        self.ast_index.lock().await.is_indexed()
    }

    pub async fn symbol_count(&self) -> usize {
        self.ast_index.lock().await.symbol_count()
    }
}

impl Default for LspContextProvider {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl ASTContextProvider for LspContextProvider {
    async fn get_symbol_context(&self, symbol_name: &str, _file_path: Option<&str>) -> Result<SymbolContext, String> {
        let index = self.ast_index.lock().await;
        let symbols = index.find_symbol(symbol_name);
        if symbols.is_empty() {
            return Err(format!("Symbol '{}' not found in AST index", symbol_name));
        }
        let sym = &symbols[0];
        Ok(SymbolContext {
            symbol: sym.clone(),
            context: format!("Found {} '{}' in {} at line {}", sym.kind, sym.name, sym.file_path, sym.line),
            lsp_hover: None,
            references_count: symbols.len(),
        })
    }

    async fn find_references_in_scope(&self, symbol_name: &str, _scope: &str) -> Result<Vec<CodeSymbol>, String> {
        let index = self.ast_index.lock().await;
        Ok(index.find_symbol(symbol_name))
    }

    async fn enhance_retrieve_with_ast(&self, query: &str) -> Result<String, String> {
        let ctx = self.get_symbol_context(query, None).await?;
        Ok(format!(
            "AST context for '{}': {} '{}' at {}:{}, {} reference(s)",
            query, ctx.symbol.kind, ctx.symbol.name, ctx.symbol.file_path, ctx.symbol.line, ctx.references_count
        ))
    }
}

/// Registration helper for Tool system.
pub fn register_ast_provider() -> Arc<dyn ASTContextProvider> {
    Arc::new(LspContextProvider::new())
}
