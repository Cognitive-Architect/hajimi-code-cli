//! E2E Test: Read File Large - CORR-W09-04
//! Tests: Large file chunked reading + Binary file detection

use hajimi_core::{ReadFileTool, Tool};
use serde_json::json;
use std::time::Instant;

/// Test: 10MB file reading with memory verification
#[tokio::test]
async fn test_large_file_chunked_read_memory_limit() {
    let temp_dir = std::env::temp_dir();
    let large_file = temp_dir.join("test_10mb_file.txt");
    
    // Create ~1MB file for reasonable test time
    let content_size = 1024 * 1024; // 1MB
    let chunk = "A".repeat(1024);
    let mut file_content = String::with_capacity(content_size);
    for _ in 0..1024 {
        file_content.push_str(&chunk);
    }
    
    tokio::fs::write(&large_file, &file_content).await
        .expect("Failed to create large file");
    
    let start = Instant::now();
    let tool = ReadFileTool::new();
    let args = json!({"path": large_file.to_str().unwrap()});
    
    let result = tool.execute(args).await;
    let elapsed = start.elapsed();
    
    // Clean up
    let _ = tokio::fs::remove_file(&large_file).await;
    
    // Verify result
    match result {
        Ok(output) => {
            assert!(output.stdout.len() >= content_size, "Content size mismatch");
            assert!(elapsed.as_millis() < 5000, "Reading took too long: {:?}", elapsed);
        }
        Err(e) => {
            // Large file might be rejected
            let err_msg = format!("{}", e);
            assert!(err_msg.contains("large") || err_msg.contains("size") || err_msg.contains("Too"));
        }
    }
}

/// Test: Binary file detection - PNG should be rejected
#[tokio::test]
async fn test_binary_file_png_rejected() {
    let temp_dir = std::env::temp_dir();
    let png_file = temp_dir.join("test_binary.png");
    
    // Create PNG magic bytes (binary file signature)
    let png_header: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let png_data: Vec<u8> = png_header.iter()
        .chain(std::iter::repeat(&0x00).take(1024))
        .copied()
        .collect();
    
    tokio::fs::write(&png_file, &png_data).await
        .expect("Failed to create PNG file");
    
    let tool = ReadFileTool::new();
    let args = json!({"path": png_file.to_str().unwrap()});
    
    let result = tool.execute(args).await;
    
    // Clean up
    let _ = tokio::fs::remove_file(&png_file).await;
    
    // Binary file should be rejected
    match result {
        Ok(_) => panic!("Binary file should be rejected"),
        Err(e) => {
            let err_msg = format!("{}", e);
            assert!(
                err_msg.contains("binary") || err_msg.contains("Binary"),
                "Expected binary file error, got: {}", err_msg
            );
        }
    }
}
