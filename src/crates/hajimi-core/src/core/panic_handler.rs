//! Panic处理与结构化崩溃日志（DEBT-EXPERIENCE-W37清偿）
//! 功能：捕获全局panic，记录JSON Lines到~/.hajimi/logs/crashes.jsonl

use std::panic;
use std::io::Write;
use std::fs::{OpenOptions, create_dir_all};
use std::path::PathBuf;
use chrono::Local;
use serde_json::json;

/// 安装全局panic hook，应在main函数开头调用
/// 
/// # Panics
/// 不会panic，所有错误均静默处理以确保panic hook本身可靠
pub fn install_hook() {
    panic::set_hook(Box::new(|info| {
        let timestamp = Local::now().to_rfc3339();
        let location = info.location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_else(|| "unknown".to_string());
        let message = info.payload().downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".to_string());
        let thread = std::thread::current().name().unwrap_or("main").to_string();
        
        let entry = json!({
            "timestamp": timestamp,
            "location": location,
            "message": message,
            "thread": thread,
            "version": env!("CARGO_PKG_VERSION"),
        });
        
        if let Some(dir) = dirs::data_dir() {
            let path = dir.join("hajimi/logs/crashes.jsonl");
            let _ = create_dir_all(path.parent().unwrap_or(&path));
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
                let _ = writeln!(f, "{}", entry);
            }
        }
        eprintln!("Panicked at {}: {}", location, message);
    }));
}

/// 获取崩溃日志路径（供诊断使用）
pub fn crash_log_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("hajimi/logs/crashes.jsonl"))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hook_installation() {
        install_hook();
    }
    
    #[test]
    fn test_json_format() {
        let entry = json!({
            "timestamp": "2026-01-01T00:00:00+00:00",
            "location": "test.rs:10",
            "message": "test panic",
            "thread": "test_thread",
            "version": "0.1.0",
        });
        let s = entry.to_string();
        assert!(s.contains("timestamp") && s.contains("location") && s.contains("message"));
    }
    
    #[test]
    fn test_crash_log_path() {
        let path = crash_log_path();
        assert!(path.is_some());
        assert!(path.unwrap().to_string_lossy().contains("crashes.jsonl"));
    }
}
