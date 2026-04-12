#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! ```
use std::fs;

fn main() {
    let src_dir = "src/crates/hajimi-codex-twist/src";
    let mut total = 0;
    let mut files = vec![];
    
    for entry in fs::read_dir(src_dir).expect("无法读取目录") {
        let entry = entry.expect("无法读取条目");
        let path = entry.path();
        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let content = fs::read_to_string(&path).expect("无法读取文件");
            let lines = content.lines().count();
            total += lines;
            files.push((path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "unknown".to_string()), lines));
        }
    }
    
    // 按文件名排序，确保一致性
    files.sort_by(|a, b| a.0.cmp(&b.0));
    
    println!("=== CODEX-TWIST LINE COUNT ===");
    println!("Source: {}", src_dir);
    println!("");
    for (name, lines) in &files {
        println!("{:20} {:>5}", name, lines);
    }
    println!("{:-<26}", "");
    println!("{:20} {:>5}", "TOTAL", total);
    println!("=== END ===");
}
