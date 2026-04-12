//! Compact压缩层：手动压缩API

use super::{CompressionError, CompressionLayer, CompressionResult, CompressionStats};
use std::collections::HashSet;

/// 压缩级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel { Light = 1, Medium = 2, Heavy = 3, Extreme = 4 }

impl CompressionLevel {
    pub fn retention_ratio(&self) -> f64 {
        match self { Self::Light => 0.8, Self::Medium => 0.6, Self::Heavy => 0.4, Self::Extreme => 0.25 }
    }
}

impl Default for CompressionLevel {
    fn default() -> Self { CompressionLevel::Medium }
}

/// Compact压缩选项
#[derive(Debug, Clone)]
pub struct CompactOptions {
    pub level: CompressionLevel, pub preserve_comments: bool,
    pub preserve_words: HashSet<String>, pub max_output: Option<usize>,
}

impl Default for CompactOptions {
    fn default() -> Self { Self::new() }
}

impl CompactOptions {
    pub fn new() -> Self { Self { level: CompressionLevel::Medium, preserve_comments: false, preserve_words: HashSet::new(), max_output: None } }
    pub fn with_level(mut self, level: CompressionLevel) -> Self { self.level = level; self }
    pub fn with_preserve_word(mut self, word: String) -> Self { self.preserve_words.insert(word); self }
    pub fn with_max_output(mut self, length: usize) -> Self { self.max_output = Some(length); self }
}

/// Compact压缩器
#[derive(Debug, Clone)]
pub struct CompactCompressor { options: CompactOptions }

impl CompactCompressor {
    pub fn new(options: CompactOptions) -> Self { Self { options } }

    pub fn compress(&self, input: &str) -> CompressionResult<(String, CompressionStats)> {
        if input.is_empty() { return Err(CompressionError::EmptyInput); }
        let start = std::time::Instant::now();
        let original_len = input.len();
        let retention = self.options.level.retention_ratio();
        let mut result = if self.options.preserve_comments { input.to_string() }
        else { input.lines().map(|l| l.split("//").next().unwrap_or("").trim_end()).collect::<Vec<_>>().join("\n") };
        result = self.simplify_whitespace(&result);
        let target_len = (original_len as f64 * retention) as usize;
        if result.len() > target_len { result.truncate(target_len); result.push_str(" ..."); }
        if let Some(max) = self.options.max_output { if result.len() > max { result.truncate(max); result.push_str("[truncated]"); } }
        let elapsed = start.elapsed().as_millis() as u64;
        let original_tokens = original_len / 4;
        let compressed_tokens = result.len() / 4;
        let stats = CompressionStats {
            original_tokens, compressed_tokens,
            ratio: compressed_tokens as f64 / original_tokens.max(1) as f64,
            layer: CompressionLayer::Compact, elapsed_ms: elapsed,
        };
        Ok((result, stats))
    }

    fn simplify_whitespace(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut prev_space = false;
        for ch in text.chars() {
            if ch.is_whitespace() { if !prev_space { result.push(' '); prev_space = true; } }
            else { result.push(ch); prev_space = false; }
        }
        result.trim().to_string()
    }
}

impl Default for CompactCompressor {
    fn default() -> Self { Self::new(CompactOptions::new()) }
}

pub fn compact(input: &str, options: CompactOptions) -> CompressionResult<String> {
    CompactCompressor::new(options).compress(input).map(|(text, _)| text)
}

pub fn compact_with_stats(input: &str, options: CompactOptions) -> CompressionResult<(String, CompressionStats)> {
    CompactCompressor::new(options).compress(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_level() { assert_eq!(CompressionLevel::Heavy.retention_ratio(), 0.4); }
    #[test]
    fn test_empty() { assert!(matches!(compact("", CompactOptions::new()), Err(CompressionError::EmptyInput))); }
    #[test]
    fn test_remove_comments() {
        let result = CompactCompressor::new(CompactOptions::new()).compress("code // comment");
        assert!(result.is_ok());
        assert!(result.map(|(s, _)| !s.contains("//")).unwrap_or(false));
    }
}
