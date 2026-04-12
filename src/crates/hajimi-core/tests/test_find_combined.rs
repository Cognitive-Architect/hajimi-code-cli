//! E2E Test: Find Combined - CORR-W09-04
//! Tests: Combined filtering (name + type + size)

use hajimi_core::{FindTool, Tool};
use serde_json::json;

/// Test: Combined filter with name + type + size
#[tokio::test]
async fn test_find_combined_name_type_size() {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("test_find_combined");
    
    // Clean up and create directory
    let _ = tokio::fs::remove_dir_all(&test_dir).await;
    tokio::fs::create_dir_all(&test_dir).await
        .expect("Failed to create test directory");
    
    // Create test files with different characteristics
    let small_test_file = test_dir.join("test_small.txt");
    tokio::fs::write(&small_test_file, "small").await
        .expect("Failed to create small file");
    
    let large_test_file = test_dir.join("test_large.txt");
    tokio::fs::write(&large_test_file, "A".repeat(10000)).await
        .expect("Failed to create large file");
    
    let other_file = test_dir.join("other.txt");
    tokio::fs::write(&other_file, "other content").await
        .expect("Failed to create other file");
    
    let subdir = test_dir.join("subdir");
    tokio::fs::create_dir(&subdir).await
        .expect("Failed to create subdirectory");
    let subdir_file = subdir.join("test_in_subdir.txt");
    tokio::fs::write(&subdir_file, "nested").await
        .expect("Failed to create nested file");
    
    let tool = FindTool::new();
    
    // Test: Find files with "test" in name, type=file, max_size=5000 bytes
    let args = json!({
        "path": test_dir.to_str().unwrap(),
        "name": "test",
        "file_type": "f",
        "max_size": 5000
    });
    
    let result = tool.execute(args).await;
    
    // Clean up
    let _ = tokio::fs::remove_dir_all(&test_dir).await;
    
    match result {
        Ok(output) => {
            let files: Vec<&str> = output.stdout.lines().collect();
            // Should find test_small.txt and test_in_subdir.txt (small files with "test" in path)
            // Should NOT find test_large.txt (too large) or other.txt (no "test" in name)
            let has_small = files.iter().any(|f| f.contains("test_small"));
            let has_large = files.iter().any(|f| f.contains("test_large"));
            let has_other = files.iter().any(|f| f.contains("other"));
            
            assert!(has_small, "Should find test_small.txt, got: {:?}", files);
            assert!(!has_large, "Should NOT find test_large.txt (too large)");
            assert!(!has_other, "Should NOT find other.txt (name doesn't match)");
        }
        Err(e) => panic!("Combined filter test failed: {}", e),
    }
}

/// Test: Multi-condition filtering with directory type
#[tokio::test]
async fn test_find_multi_condition_directories() {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("test_find_multi");
    
    // Clean up and create directory
    let _ = tokio::fs::remove_dir_all(&test_dir).await;
    tokio::fs::create_dir_all(&test_dir).await
        .expect("Failed to create test directory");
    
    // Create directory structure
    let dirs = vec!["src", "tests", "docs"];
    for dir_name in &dirs {
        let dir_path = test_dir.join(dir_name);
        tokio::fs::create_dir(&dir_path).await
            .expect("Failed to create directory");
        let file_path = dir_path.join("file.txt");
        tokio::fs::write(&file_path, "content").await
            .expect("Failed to create file");
    }
    
    let tool = FindTool::new();
    
    // Test 1: Find only directories
    let args_dirs = json!({
        "path": test_dir.to_str().unwrap(),
        "file_type": "d"
    });
    let result_dirs = tool.execute(args_dirs).await;
    
    // Test 2: Find directories with "test" in name
    let args_test = json!({
        "path": test_dir.to_str().unwrap(),
        "name": "test",
        "file_type": "d"
    });
    let result_test = tool.execute(args_test).await;
    
    // Clean up
    let _ = tokio::fs::remove_dir_all(&test_dir).await;
    
    // Verify directory filtering
    match result_dirs {
        Ok(output) => {
            let entries: Vec<&str> = output.stdout.lines().collect();
            assert!(
                entries.iter().any(|e| e.contains("src")),
                "Should find src directory"
            );
            assert!(
                entries.iter().any(|e| e.contains("tests")),
                "Should find tests directory"
            );
        }
        Err(e) => panic!("Directory filter test failed: {}", e),
    }
    
    // Verify name + type combined filtering
    match result_test {
        Ok(output) => {
            let entries: Vec<&str> = output.stdout.lines().collect();
            assert!(
                entries.iter().any(|e| e.contains("tests")),
                "Should find tests directory"
            );
        }
        Err(e) => panic!("Name+type filter test failed: {}", e),
    }
}
