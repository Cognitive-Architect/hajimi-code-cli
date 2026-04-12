//! Micro压缩层：自动标记替换

use super::{CompressionError, CompressionLayer, CompressionResult, CompressionStats};
use std::collections::HashMap;

/// Micro压缩器
#[derive(Debug, Clone)]
pub struct MicroCompressor { rules: HashMap<String, String> }

impl Default for MicroCompressor {
    fn default() -> Self { Self::new() }
}

impl MicroCompressor {
    /// 创建新的Micro压缩器
    pub fn new() -> Self {
        let mut rules = HashMap::new();
        rules.insert("function ".to_string(), "fn:".to_string());
        rules.insert("return ".to_string(), "ret ".to_string());
        rules.insert("const ".to_string(), "c ".to_string());
        rules.insert("let ".to_string(), "v ".to_string());
        rules.insert("console.log(".to_string(), "log(".to_string());
        rules.insert("implementation".to_string(), "impl".to_string());
        rules.insert("configuration".to_string(), "config".to_string());
        Self { rules }
    }

    /// 添加自定义替换规则
    pub fn add_rule(&mut self, from: String, to: String) { self.rules.insert(from, to); }

    /// 执行压缩
    pub fn compress(&self, input: &str) -> CompressionResult<(String, CompressionStats)> {
        if input.is_empty() { return Err(CompressionError::EmptyInput); }
        let start = std::time::Instant::now();
        let original_len = input.len();
        let mut result = input.to_string();
        let mut rules: Vec<_> = self.rules.iter().collect();
        rules.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        for (from, to) in rules { result = result.replace(from, to); }
        let elapsed = start.elapsed().as_millis() as u64;
        let original_tokens = original_len / 4;
        let compressed_tokens = result.len() / 4;
        let stats = CompressionStats {
            original_tokens, compressed_tokens,
            ratio: compressed_tokens as f64 / original_tokens.max(1) as f64,
            layer: CompressionLayer::Micro, elapsed_ms: elapsed,
        };
        Ok((result, stats))
    }
}

/// 便捷函数：压缩文本
pub fn compress_micro(input: &str) -> CompressionResult<String> {
    MicroCompressor::new().compress(input).map(|(text, _)| text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_compress() {
        let result = compress_micro("function main() { return 0; }");
        assert!(result.is_ok());
        assert!(result.map(|s| s.contains("fn:")).unwrap_or(false));
    }
    #[test]
    fn test_empty() {
        assert!(matches!(compress_micro(""), Err(CompressionError::EmptyInput)));
    }
}
