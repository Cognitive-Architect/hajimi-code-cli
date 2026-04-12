//! E2E Test: Glob Performance - CORR-W09-04
//! Tests: File matching performance < 500ms

use hajimi_core::{GlobTool, Tool};
use serde_json::json;
use std::time::Instant;

const PERFORMANCE_THRESHOLD_MS: u128 = 500;
const FILE_COUNT: usize = 500;

/// Test: 500 file matching performance
#[tokio::test]
async fn test_glob_performance_500_files() {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("test_glob_perf_500");
    
    // Clean up and create directory
    let _ = tokio::fs::remove_dir_all(&test_dir).await;
    tokio::fs::create_dir_all(&test_dir).await
        .expect("Failed to create test directory");
    
    // Create 500 files with different extensions
    for i in 0..FILE_COUNT {
        let ext = if i % 2 == 0 { "rs" } else { "txt" };
        let file_path = test_dir.join(format!("file_{}.{}", i, ext));
        tokio::fs::write(&file_path, format!("Content {}", i)).await
            .expect("Failed to create file");
    }
    
    let tool = GlobTool::new();
    let args = json!({
        "path": test_dir.to_str().unwrap(),
        "pattern": "*.rs"
    });
    
    let start = Instant::now();
    let result = tool.execute(args).await;
    let elapsed = start.elapsed();
    
    // Clean up
    let _ = tokio::fs::remove_dir_all(&test_dir).await;
    
    assert!(result.is_ok(), "Glob execution failed: {:?}", result.err());
    
    // Verify performance
    assert!(
        elapsed.as_millis() < PERFORMANCE_THRESHOLD_MS,
        "Glob performance too slow: {}ms (threshold: {}ms)",
        elapsed.as_millis(),
        PERFORMANCE_THRESHOLD_MS
    );
    
    // Verify results
    if let Ok(output) = result {
        let entries: Vec<serde_json::Value> = serde_json::from_str(&output.stdout)
            .expect("Failed to parse JSON output");
        assert!(
            entries.len() >= FILE_COUNT / 2 - 10 && entries.len() <= FILE_COUNT / 2 + 10,
            "Expected ~{} .rs files, got {}",
            FILE_COUNT / 2,
            entries.len()
        );
    }
}

/// Test: Glob pattern matching accuracy
#[tokio::test]
async fn test_glob_pattern_accuracy() {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("test_glob_accuracy");
    
    // Clean up and create directory
    let _ = tokio::fs::remove_dir_all(&test_dir).await;
    tokio::fs::create_dir_all(&test_dir).await
        .expect("Failed to create test directory");
    
    // Create files with various patterns
    let files = vec!["main.rs", "lib.rs", "test.rs", "readme.md", "Cargo.toml"];
    for filename in &files {
        let file_path = test_dir.join(filename);
        tokio::fs::write(&file_path, "test content").await
            .expect("Failed to create file");
    }
    
    let tool = GlobTool::new();
    
    // Test 1: Match all .rs files
    let args_rs = json!({
        "path": test_dir.to_str().unwrap(),
        "pattern": "*.rs"
    });
    let result_rs = tool.execute(args_rs).await;
    
    // Test 2: Match all files (wildcard)
    let args_all = json!({
        "path": test_dir.to_str().unwrap(),
        "pattern": "*"
    });
    let result_all = tool.execute(args_all).await;
    
    // Clean up
    let _ = tokio::fs::remove_dir_all(&test_dir).await;
    
    // Verify .rs pattern
    match result_rs {
        Ok(output) => {
            let entries: Vec<serde_json::Value> = serde_json::from_str(&output.stdout)
                .expect("Failed to parse JSON");
            assert!(
                entries.iter().any(|e| e["name"].as_str().unwrap_or("").ends_with(".rs")),
                "Should find .rs files"
            );
        }
        Err(e) => panic!("Glob .rs pattern failed: {}", e),
    }
    
    // Verify wildcard pattern
    match result_all {
        Ok(output) => {
            let entries: Vec<serde_json::Value> = serde_json::from_str(&output.stdout)
                .expect("Failed to parse JSON");
            assert!(entries.len() >= 3, "Should find at least 3 files with wildcard");
        }
        Err(e) => panic!("Glob wildcard pattern failed: {}", e),
    }
}
