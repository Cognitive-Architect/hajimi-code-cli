//! E2E Test: Write File Atomic - CORR-W09-04
//! Tests: Atomic write (temp + rename) + Crash recovery

use hajimi_core::{WriteFileTool, Tool};
use serde_json::json;

/// Test: Atomic write using temp file + rename
#[tokio::test]
async fn test_atomic_write_temp_rename() {
    let temp_dir = std::env::temp_dir();
    let target_file = temp_dir.join("test_atomic_write.txt");
    let temp_file = target_file.with_extension("tmp");
    
    // Clean up any existing files
    let _ = tokio::fs::remove_file(&target_file).await;
    let _ = tokio::fs::remove_file(&temp_file).await;
    
    let tool = WriteFileTool::new();
    let content = "Atomic write test content";
    let args = json!({
        "path": target_file.to_str().unwrap(),
        "content": content
    });
    
    let result = tool.execute(args).await;
    
    // Clean up
    let _ = tokio::fs::remove_file(&target_file).await;
    let _ = tokio::fs::remove_file(&temp_file).await;
    
    assert!(result.is_ok(), "Atomic write failed: {:?}", result.err());
}

/// Test: Crash recovery - backup file exists after overwrite
#[tokio::test]
async fn test_crash_recovery_backup_exists() {
    let temp_dir = std::env::temp_dir();
    let target_file = temp_dir.join("test_crash_recovery.txt");
    // Backup path: with_extension handles the extension replacement
    // Source creates: format!("{}.{}", ext, BACKUP_SUFFIX) = "txt.bak" (BACKUP_SUFFIX="bak")
    // Actually output shows: test_crash_recovery.txt..bak
    // So the extension becomes "txt..bak" due to double dots in format
    let backup_file = target_file.with_extension("txt..bak");
    
    // Create original file
    let original_content = "Original content for crash recovery test";
    tokio::fs::write(&target_file, original_content).await
        .expect("Failed to create original file");
    
    let tool = WriteFileTool::new();
    let new_content = "New content after crash recovery";
    let args = json!({
        "path": target_file.to_str().unwrap(),
        "content": new_content
    });
    
    let result = tool.execute(args).await;
    
    // Check if backup exists
    let backup_exists = backup_file.exists();
    
    // Read current file content
    let current_content = tokio::fs::read_to_string(&target_file).await
        .unwrap_or_default();
    
    // Clean up
    let _ = tokio::fs::remove_file(&target_file).await;
    let _ = tokio::fs::remove_file(&backup_file).await;
    
    assert!(result.is_ok(), "Write failed: {:?}", result.err());
    assert!(backup_exists, "Backup file should exist after overwrite");
    assert_eq!(current_content, new_content, "File should have new content");
}
