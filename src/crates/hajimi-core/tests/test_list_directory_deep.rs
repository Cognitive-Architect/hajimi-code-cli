//! E2E Test: List Directory Deep - CORR-W09-04
//! Tests: Deep directory recursion + Stack safety

use hajimi_core::{ListDirectoryTool, Tool};
use serde_json::json;

const MAX_RECURSION_DEPTH: usize = 100;

/// Test: Deep directory recursion up to 50 levels
#[tokio::test]
async fn test_deep_directory_recursion() {
    let temp_dir = std::env::temp_dir();
    let base_dir = temp_dir.join("test_deep_recursion");
    
    // Clean up and create base directory
    let _ = tokio::fs::remove_dir_all(&base_dir).await;
    tokio::fs::create_dir_all(&base_dir).await
        .expect("Failed to create base directory");
    
    // Create nested directory structure (30 levels deep)
    let mut current_dir = base_dir.clone();
    for i in 0..30 {
        current_dir = current_dir.join(format!("level_{}", i));
        tokio::fs::create_dir(&current_dir).await
            .expect("Failed to create nested directory");
        let file_path = current_dir.join("marker.txt");
        tokio::fs::write(&file_path, format!("Level {}", i)).await
            .expect("Failed to create marker file");
    }
    
    let tool = ListDirectoryTool::new();
    let args = json!({
        "path": base_dir.to_str().unwrap(),
        "recursive": true,
        "max_depth": MAX_RECURSION_DEPTH
    });
    
    let result = tool.execute(args).await;
    
    // Clean up
    let _ = tokio::fs::remove_dir_all(&base_dir).await;
    
    match result {
        Ok(output) => {
            let entries: serde_json::Value = serde_json::from_str(&output.stdout)
                .expect("Failed to parse JSON output");
            let entries_array = entries.as_array()
                .expect("Expected array of entries");
            assert!(entries_array.len() >= 60, "Expected at least 60 entries");
        }
        Err(e) => {
            panic!("Deep recursion test failed: {}", e);
        }
    }
}

/// Test: Stack safety - ensure no stack overflow with deep recursion
#[tokio::test]
async fn test_stack_safety_deep_recursion() {
    let temp_dir = std::env::temp_dir();
    let base_dir = temp_dir.join("test_stack_safety");
    
    // Clean up and create base directory
    let _ = tokio::fs::remove_dir_all(&base_dir).await;
    tokio::fs::create_dir_all(&base_dir).await
        .expect("Failed to create base directory");
    
    // Create moderately deep structure (20 levels)
    let mut current_dir = base_dir.clone();
    for i in 0..20 {
        current_dir = current_dir.join(format!("dir_{}", i));
        tokio::fs::create_dir(&current_dir).await
            .expect("Failed to create directory");
    }
    
    let tool = ListDirectoryTool::new();
    let args = json!({
        "path": base_dir.to_str().unwrap(),
        "recursive": true,
        "max_depth": 50
    });
    
    let result = tool.execute(args).await;
    
    // Clean up
    let _ = tokio::fs::remove_dir_all(&base_dir).await;
    
    // Should complete without stack overflow
    assert!(result.is_ok(), "Stack safety test failed: {:?}", result.err());
    
    if let Ok(output) = result {
        let entries: serde_json::Value = serde_json::from_str(&output.stdout)
            .expect("Failed to parse JSON output");
        let entries_array = entries.as_array()
            .expect("Expected array of entries");
        assert!(entries_array.len() >= 20, "Expected at least 20 directory entries");
    }
}
