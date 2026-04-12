//! Code Index WASM Interface - Week 19 Local IntelliSense
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

/// Code symbol (function, variable, struct, etc.)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeSymbol {
    pub name: String,
    pub kind: String, // "function", "variable", "class", "module"
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub documentation: Option<String>,
}

/// Search result from code index
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub symbol: CodeSymbol,
    pub score: f32,
}

/// Index result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexResult {
    pub file_count: usize,
    pub symbol_count: usize,
    pub errors: Vec<String>,
}

/// Code Index for local project
#[wasm_bindgen]
pub struct CodeIndex {
    symbols: Vec<CodeSymbol>,
    indexed: bool,
}

#[wasm_bindgen]
impl CodeIndex {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { symbols: Vec::new(), indexed: false }
    }

    /// Index project files (mock implementation for Week 19)
    #[wasm_bindgen(js_name = indexProject)]
    pub fn index_project(&mut self, _path: String, _patterns: Vec<String>) -> Result<JsValue, JsValue> {
        // Mock: generate sample symbols for testing
        self.symbols = vec![
            CodeSymbol {
                name: "search_code".to_string(),
                kind: "function".to_string(),
                file_path: "src/lib.rs".to_string(),
                line: 1,
                column: 0,
                documentation: Some("Search code in the indexed project".to_string()),
            },
            CodeSymbol {
                name: "CodeIndex".to_string(),
                kind: "class".to_string(),
                file_path: "src/index.rs".to_string(),
                line: 10,
                column: 0,
                documentation: Some("Main code index structure".to_string()),
            },
            CodeSymbol {
                name: "index_project".to_string(),
                kind: "function".to_string(),
                file_path: "src/index.rs".to_string(),
                line: 25,
                column: 0,
                documentation: Some("Index a project directory".to_string()),
            },
        ];
        self.indexed = true;
        
        Ok(serde_wasm_bindgen::to_value(&IndexResult {
            file_count: 3,
            symbol_count: self.symbols.len(),
            errors: vec![],
        })?)
    }

    /// Search code symbols (mock implementation)
    #[wasm_bindgen(js_name = searchCode)]
    pub fn search_code(&self, query: String, top_k: usize) -> Result<JsValue, JsValue> {
        if !self.indexed {
            return Err(JsValue::from_str("Project not indexed. Call indexProject first."));
        }
        
        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchResult> = self.symbols
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&query_lower))
            .enumerate()
            .map(|(i, s)| SearchResult {
                symbol: s.clone(),
                score: 1.0 - (i as f32 * 0.1),
            })
            .take(top_k)
            .collect();
        
        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    /// Check if project is indexed
    #[wasm_bindgen(js_name = isIndexed)]
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    /// Get symbol count
    #[wasm_bindgen(js_name = symbolCount)]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Clear index (memory cleanup)
    #[wasm_bindgen(js_name = clearIndex)]
    pub fn clear_index(&mut self) {
        self.symbols.clear();
        self.indexed = false;
    }
}

impl Default for CodeIndex {
    fn default() -> Self { Self::new() }
}
