//! AST Provider: Real Rust symbol extraction using `syn` + `walkdir`.
//!
//! Scans `.rs` files in a project directory, parses each with `syn::parse_file()`,
//! and extracts functions, structs, and impl blocks into a queryable index.
//! Fallback: files that fail to parse are silently skipped (no panic).
//! All metrics are real (no estimates).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use walkdir::WalkDir;

/// A code symbol extracted from Rust source.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeSymbol {
    pub name: String,
    pub kind: String, // "function", "struct", "impl"
    pub file_path: String,
    pub line: usize,
}

/// In-memory symbol index for a project.
pub struct AstSymbolIndex {
    symbols: HashMap<String, Vec<CodeSymbol>>,
    indexed: bool,
}

impl AstSymbolIndex {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            indexed: false,
        }
    }

    /// Scan all `.rs` files under `root` and build the symbol index.
    /// Returns the number of symbols extracted.
    pub fn index_project(&mut self, root: &str) -> Result<usize, String> {
        self.symbols.clear();
        let mut count = 0usize;

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    let path_str = path.to_string_lossy().to_string();
                    if let Ok(content) = std::fs::read_to_string(&path_str) {
                        if let Ok(file) = syn::parse_file(&content) {
                            let file_symbols = extract_symbols(&file, &path_str);
                            count += file_symbols.len();
                            for sym in file_symbols {
                                self.symbols
                                    .entry(sym.name.clone())
                                    .or_default()
                                    .push(sym);
                            }
                        }
                    }
                }
            }
        }

        self.indexed = true;
        Ok(count)
    }

    /// Find all symbols matching `name` (exact match on the symbol name).
    pub fn find_symbol(&self, name: &str) -> Vec<CodeSymbol> {
        self.symbols.get(name).cloned().unwrap_or_default()
    }

    /// Find symbols whose names contain the query string (partial match).
    pub fn search_symbols(&self, query: &str) -> Vec<CodeSymbol> {
        let q = query.to_lowercase();
        self.symbols
            .values()
            .flat_map(|v| v.iter())
            .filter(|s| s.name.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    pub fn is_indexed(&self) -> bool { self.indexed }
    pub fn symbol_count(&self) -> usize { self.symbols.len() }
}

impl Default for AstSymbolIndex {
    fn default() -> Self { Self::new() }
}

/// Extract symbols from a single parsed Rust file.
fn extract_symbols(file: &syn::File, path: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for item in &file.items {
        match item {
            syn::Item::Fn(f) => {
                symbols.push(CodeSymbol {
                    name: f.sig.ident.to_string(),
                    kind: "function".to_string(),
                    file_path: path.to_string(),
                    line: f.sig.ident.span().start().line,
                });
            }
            syn::Item::Struct(s) => {
                symbols.push(CodeSymbol {
                    name: s.ident.to_string(),
                    kind: "struct".to_string(),
                    file_path: path.to_string(),
                    line: s.ident.span().start().line,
                });
            }
            syn::Item::Impl(i) => {
                let impl_name = extract_impl_name(i);
                symbols.push(CodeSymbol {
                    name: impl_name,
                    kind: "impl".to_string(),
                    file_path: path.to_string(),
                    line: i.impl_token.span.start().line,
                });
            }
            syn::Item::Enum(e) => {
                symbols.push(CodeSymbol {
                    name: e.ident.to_string(),
                    kind: "enum".to_string(),
                    file_path: path.to_string(),
                    line: e.ident.span().start().line,
                });
            }
            syn::Item::Trait(t) => {
                symbols.push(CodeSymbol {
                    name: t.ident.to_string(),
                    kind: "trait".to_string(),
                    file_path: path.to_string(),
                    line: t.ident.span().start().line,
                });
            }
            _ => {}
        }
    }
    symbols
}

/// Extract a display name for an impl block.
fn extract_impl_name(i: &syn::ItemImpl) -> String {
    if let Some((_, trait_path, _)) = &i.trait_ {
        let trait_name = trait_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        let self_name = type_to_string(&i.self_ty);
        format!("impl {} for {}", trait_name, self_name)
    } else {
        let self_name = type_to_string(&i.self_ty);
        format!("impl {}", self_name)
    }
}

/// Convert a `syn::Type` to a simple string representation.
fn type_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        syn::Type::Reference(r) => type_to_string(&r.elem),
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_symbols_simple() {
        let source = r#"
            fn main() {}
            struct Config { val: i32 }
            impl Config { fn new() -> Self { Self { val: 0 } } }
        "#;
        let file = syn::parse_file(source).unwrap();
        let symbols = extract_symbols(&file, "test.rs");
        assert!(symbols.iter().any(|s| s.name == "main" && s.kind == "function"));
        assert!(symbols.iter().any(|s| s.name == "Config" && s.kind == "struct"));
        assert!(symbols.iter().any(|s| s.name == "impl Config" && s.kind == "impl"));
    }

    #[test]
    fn test_index_and_find() {
        let mut index = AstSymbolIndex::new();
        let source = r#"fn helper() {} fn main() { helper(); }"#;
        let file = syn::parse_file(source).unwrap();
        let syms = extract_symbols(&file, "test.rs");
        for sym in syms {
            index.symbols.entry(sym.name.clone()).or_default().push(sym);
        }
        index.indexed = true;

        let found = index.find_symbol("helper");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, "function");
        assert_eq!(found[0].file_path, "test.rs");
    }

    #[test]
    fn test_fallback_on_bad_syntax() {
        let bad_source = "fn broken { missing_paren }";
        let result = syn::parse_file(bad_source);
        assert!(result.is_err());
        // No panic, just skip
    }

    #[test]
    fn test_search_symbols_partial() {
        let mut index = AstSymbolIndex::new();
        let source = r#"fn initialize() {} fn init_config() {} fn cleanup() {}"#;
        let file = syn::parse_file(source).unwrap();
        let syms = extract_symbols(&file, "test.rs");
        for sym in syms {
            index.symbols.entry(sym.name.clone()).or_default().push(sym);
        }
        index.indexed = true;

        let results = index.search_symbols("init");
        assert_eq!(results.len(), 2);
    }
}
