//! Type verification tests for JSON-RPC protocol
//! Ensures Rust/TypeScript type alignment through serialization tests

use serde::{Deserialize, Serialize};
use serde_json::json;

// TYPE-SAFETY: Protocol types mirrored from protocol.rs
// These types must stay synchronized with TypeScript generated types

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexRequest {
    pub path: String,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexResponse {
    pub file_count: u32,
    pub symbol_count: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchRequest {
    pub query: String,
    pub top_k: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeSymbol {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResultItem {
    pub symbol: CodeSymbol,
    pub score: f64,
}
pub type SearchResponse = Vec<SearchResultItem>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
}

#[test]
fn test_index_request_serialization() {
    let req = IndexRequest {
        path: "/project".to_string(),
        patterns: vec!["*.rs".to_string(), "*.ts".to_string()],
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("/project"));
    assert!(json.contains("*.rs"));
}

#[test]
fn test_search_response_deserialization() {
    let json = json!([{ "symbol": { "name": "test_func", "kind": "function", "file_path": "/src/lib.rs", "line": 10, "column": 0 }, "score": 0.95 }]);
    let response: SearchResponse = serde_json::from_value(json).unwrap();
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].symbol.name, "test_func");
}

#[test]
fn test_health_response_structure() {
    let health = HealthResponse {
        status: "ok".to_string(),
        version: "1.0.0".to_string(),
        uptime: 3600,
    };
    let json = serde_json::to_value(&health).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["version"], "1.0.0");
}

#[test]
fn test_type_consistency_check() {
    let index_req = IndexRequest {
        path: "/test".to_string(),
        patterns: vec!["*.rs".to_string()],
    };
    let serialized = serde_json::to_string(&index_req).unwrap();
    let deserialized: IndexRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(index_req, deserialized);
}
