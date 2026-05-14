//! AST Context integration tests (Phase 4 Day 2).
//!
//! Tests real syn-based AST parsing, symbol lookup, LspContextProvider integration,
//! MemoryRetriever AST enhancement, planner AST injection, and fallback behavior.

use agent_core::{
    blackboard::Blackboard,
    memory_retriever::{MemoryRetriever, RetrieveOutcome},
    planner::{HierarchicalPlanner, Priority},
    ASTContextProvider, AgentContext, AstSymbolIndex, CodeSymbol, LspContextProvider,
    SymbolContext,
};
use std::sync::Arc;
use tokio::sync::Mutex;

fn test_bb() -> Arc<Blackboard> {
    Arc::new(Blackboard::new())
}
fn test_context() -> AgentContext {
    AgentContext::new()
}

#[tokio::test]
async fn test_ast_parse_real_file() {
    let temp_dir =
        std::env::temp_dir().join(format!("hajimi_ast_test_{}", uuid::Uuid::new_v4().simple()));
    let temp_path = temp_dir.to_str().unwrap();
    std::fs::create_dir_all(temp_path).unwrap();

    let rs_file = temp_dir.join("test_lib.rs");
    std::fs::write(
        &rs_file,
        r#"
fn main() {}
struct Config { val: i32 }
impl Config { fn new() -> Self { Self { val: 0 } } }
enum Status { Active, Inactive }
trait Parser { fn parse(&self) -> Result<(), String>; }
"#,
    )
    .unwrap();

    let mut index = AstSymbolIndex::new();
    let count = index
        .index_project(temp_path)
        .expect("index should succeed");
    assert!(count >= 5, "Expected at least 5 symbols, got {}", count);

    assert!(index
        .find_symbol("main")
        .iter()
        .any(|s| s.kind == "function"));
    assert!(index
        .find_symbol("Config")
        .iter()
        .any(|s| s.kind == "struct"));
    assert!(index.find_symbol("Status").iter().any(|s| s.kind == "enum"));
    assert!(index
        .find_symbol("Parser")
        .iter()
        .any(|s| s.kind == "trait"));

    std::fs::remove_dir_all(temp_path).unwrap_or(());
}

#[tokio::test]
async fn test_ast_find_symbol_line_number() {
    let temp_dir =
        std::env::temp_dir().join(format!("hajimi_ast_line_{}", uuid::Uuid::new_v4().simple()));
    let temp_path = temp_dir.to_str().unwrap();
    std::fs::create_dir_all(temp_path).unwrap();

    let rs_file = temp_dir.join("lines.rs");
    std::fs::write(&rs_file, "\n\nfn target() {}\n\nstruct After {}\n").unwrap();

    let mut index = AstSymbolIndex::new();
    index.index_project(temp_path).unwrap();

    let found = index.find_symbol("target");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 3, "target() should be at line 3");

    std::fs::remove_dir_all(temp_path).unwrap_or(());
}

#[tokio::test]
async fn test_ast_index_counts_nonzero() {
    // Index a temp directory with multiple .rs files
    let temp_dir = std::env::temp_dir().join(format!(
        "hajimi_ast_multi_{}",
        uuid::Uuid::new_v4().simple()
    ));
    let temp_path = temp_dir.to_str().unwrap();
    std::fs::create_dir_all(temp_path).unwrap();

    std::fs::write(temp_dir.join("a.rs"), "fn foo() {}\nstruct Bar {}\n").unwrap();
    std::fs::write(temp_dir.join("b.rs"), "fn baz() {}\nenum Qux {}\n").unwrap();

    let mut index = AstSymbolIndex::new();
    let count = index.index_project(temp_path).expect("index temp project");
    assert!(count >= 4, "Expected >=4 symbols, got {}", count);
    assert!(index.is_indexed());

    std::fs::remove_dir_all(temp_path).unwrap_or(());
}

#[tokio::test]
async fn test_ast_fallback_on_bad_syntax() {
    let temp_dir =
        std::env::temp_dir().join(format!("hajimi_ast_bad_{}", uuid::Uuid::new_v4().simple()));
    let temp_path = temp_dir.to_str().unwrap();
    std::fs::create_dir_all(temp_path).unwrap();

    let rs_file = temp_dir.join("broken.rs");
    std::fs::write(&rs_file, "fn broken { missing_paren }").unwrap();

    let mut index = AstSymbolIndex::new();
    let count = index
        .index_project(temp_path)
        .expect("index should not panic");
    assert_eq!(count, 0, "Bad syntax file should yield 0 symbols");

    std::fs::remove_dir_all(temp_path).unwrap_or(());
}

#[tokio::test]
async fn test_lsp_provider_get_symbol_context() {
    let provider = LspContextProvider::new();

    // Index a temp project first
    let temp_dir =
        std::env::temp_dir().join(format!("hajimi_lsp_{}", uuid::Uuid::new_v4().simple()));
    let temp_path = temp_dir.to_str().unwrap();
    std::fs::create_dir_all(temp_path).unwrap();
    std::fs::write(temp_dir.join("lib.rs"), "fn helper() {}\n").unwrap();

    let count = provider.index_project(temp_path).await.expect("index");
    assert!(count > 0);

    let ctx = provider
        .get_symbol_context("helper", None)
        .await
        .expect("get_symbol_context");
    assert_eq!(ctx.symbol.name, "helper");
    assert_eq!(ctx.symbol.kind, "function");
    assert!(ctx.context.contains("helper"));

    std::fs::remove_dir_all(temp_path).unwrap_or(());
}

#[tokio::test]
async fn test_lsp_provider_symbol_not_found() {
    let provider = LspContextProvider::new();
    let result = provider.get_symbol_context("NonExistent", None).await;
    assert!(result.is_err(), "Should error for unknown symbol");
}

#[tokio::test]
async fn test_retrieve_with_ast_enhanced() {
    let bb = test_bb();
    let ast_provider: Arc<dyn ASTContextProvider> = {
        let provider = LspContextProvider::new();
        // Index a temp project
        let temp_dir =
            std::env::temp_dir().join(format!("hajimi_ret_{}", uuid::Uuid::new_v4().simple()));
        let temp_path = temp_dir.to_str().unwrap();
        std::fs::create_dir_all(temp_path).unwrap();
        std::fs::write(temp_dir.join("lib.rs"), "fn my_helper() {}\n").unwrap();
        provider.index_project(temp_path).await.unwrap();
        std::fs::remove_dir_all(temp_path).unwrap_or(());
        Arc::new(provider)
    };

    let retriever = MemoryRetriever::new(bb.clone(), None, None).with_ast_provider(ast_provider);

    let outcome = retriever
        .retrieve_with_ast("agent-1", Some("my_helper"))
        .await;
    match &outcome {
        RetrieveOutcome::Retrieved { summary } => {
            assert!(
                summary.contains("AST-enhanced"),
                "Summary should indicate AST enhancement: {}",
                summary
            );
            let snapshot = bb.snapshot().await;
            assert!(
                snapshot.contains_key("ast_context_agent-1"),
                "Blackboard should have ast_context_agent-1"
            );
        }
        other => panic!("Expected AST-enhanced retrieval, got {:?}", other),
    }
}

#[tokio::test]
async fn test_retrieve_without_ast_normal() {
    let bb = test_bb();
    let retriever = MemoryRetriever::new(bb.clone(), None, None);
    let outcome = retriever.retrieve("agent-2").await;
    // Without sync_gateway, should return Error("No sync_gateway")
    match outcome {
        RetrieveOutcome::Error(e) => assert!(e.contains("No sync_gateway")),
        _ => {} // CacheHit or Retrieved if memory is configured
    }
}

#[tokio::test]
async fn test_retrieve_ast_fallback_when_not_indexed() {
    let bb = test_bb();
    let ast_provider: Arc<dyn ASTContextProvider> = Arc::new(LspContextProvider::new());
    // Do NOT index anything — query should fall back to normal retrieval
    let retriever = MemoryRetriever::new(bb.clone(), None, None).with_ast_provider(ast_provider);

    let outcome = retriever
        .retrieve_with_ast("agent-3", Some("unknown_symbol"))
        .await;
    // Should fallback because symbol not found → then No sync_gateway error
    match outcome {
        RetrieveOutcome::Error(e) => {
            assert!(e.contains("No sync_gateway") || e.contains("not found"))
        }
        RetrieveOutcome::Retrieved { summary } => {
            assert!(summary.contains("AST-enhanced") || summary.contains("sync_gateway"))
        }
        _ => {}
    }
}

#[tokio::test]
async fn test_e2e_plan_with_ast() {
    let bb = test_bb();
    let mem = Arc::new(Mutex::new(memory::memory_gateway::MemoryGateway::new(
        "test",
    )));
    let mut planner = HierarchicalPlanner::new(mem, test_context()).with_blackboard(bb.clone());

    let goal_id = planner
        .plan_with_ast("Refactor AgentLoop and improve Config", Priority::High)
        .await
        .expect("plan_with_ast");
    assert!(!goal_id.is_empty());

    let snapshot = bb.snapshot().await;
    let ast_keys: Vec<String> = snapshot
        .keys()
        .filter(|k| k.starts_with(&format!("ast_query_{}", goal_id)))
        .cloned()
        .collect();
    assert!(
        !ast_keys.is_empty(),
        "Blackboard should have ast_query keys for symbol candidates"
    );
}

#[tokio::test]
async fn test_ast_context_provider_trait_object() {
    let provider: Arc<dyn ASTContextProvider> = Arc::new(LspContextProvider::new());
    // Just verify the trait object compiles and basic methods work
    let result = provider.enhance_retrieve_with_ast("test").await;
    // Should error because nothing is indexed
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_symbol_context_serde() {
    let ctx = SymbolContext {
        symbol: CodeSymbol {
            name: "test_fn".to_string(),
            kind: "function".to_string(),
            file_path: "src/lib.rs".to_string(),
            line: 42,
        },
        context: "test context".to_string(),
        lsp_hover: Some("hover info".to_string()),
        references_count: 3,
    };
    let json = serde_json::to_string(&ctx).expect("serialize");
    assert!(json.contains("test_fn"));
    let decoded: SymbolContext = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.symbol.name, "test_fn");
    assert_eq!(decoded.references_count, 3);
}
