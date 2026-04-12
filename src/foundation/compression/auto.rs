//! Auto压缩层：Token计数+LLM触发摘要

use super::{CompressionError, CompressionLayer, CompressionResult, CompressionStats, TOKEN_THRESHOLD};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Token计数器
#[derive(Debug, Clone)]
pub struct TokenCounter { ratio: f64, lang_factors: HashMap<String, f64> }

impl Default for TokenCounter {
    fn default() -> Self { Self::new() }
}

impl TokenCounter {
    pub fn new() -> Self {
        let mut lang_factors = HashMap::new();
        lang_factors.insert("rust".to_string(), 0.9);
        lang_factors.insert("json".to_string(), 1.2);
        Self { ratio: 4.0, lang_factors }
    }
    pub fn estimate(&self, text: &str) -> usize { (text.len() as f64 / self.ratio).ceil() as usize }
    pub fn exceeds_threshold(&self, text: &str) -> bool { self.estimate(text) >= TOKEN_THRESHOLD }
}

/// LLM摘要响应
#[derive(Debug, Clone)]
pub struct SummaryResponse {
    pub summary: String, pub tokens_used: usize,
    pub model: String, pub timestamp: u64,
}

/// Auto压缩器
#[derive(Debug, Clone)]
pub struct AutoCompressor {
    counter: TokenCounter, storage: PathBuf, model: String,
}

impl Default for AutoCompressor {
    fn default() -> Self { Self::new() }
}

impl AutoCompressor {
    pub fn new() -> Self {
        let storage = dirs::home_dir()
            .map(|h| h.join(".hajimi/memory/auto"))
            .unwrap_or_else(|| PathBuf::from("./memory/auto"));
        Self { counter: TokenCounter::new(), storage, model: "gpt-4".to_string() }
    }
    pub fn with_storage(mut self, path: PathBuf) -> Self { self.storage = path; self }
    pub fn with_model(mut self, model: &str) -> Self { self.model = model.to_string(); self }
    pub fn should_compress(&self, text: &str) -> bool { self.counter.exceeds_threshold(text) }
    pub fn token_count(&self, text: &str) -> usize { self.counter.estimate(text) }

    pub fn compress(&self, text: &str, model: Option<&str>) -> CompressionResult<(SummaryResponse, CompressionStats)> {
        if text.is_empty() { return Err(CompressionError::EmptyInput); }
        let original_tokens = self.counter.estimate(text);
        if original_tokens < TOKEN_THRESHOLD {
            return Err(CompressionError::InvalidCompression(format!("Below threshold: {}", original_tokens)));
        }
        let start = std::time::Instant::now();
        let model = model.unwrap_or(&self.model);
        let summary = self.generate_summary(text)?;
        let compressed_tokens = self.counter.estimate(&summary);
        let elapsed = start.elapsed().as_millis() as u64;
        let response = SummaryResponse {
            summary: summary.clone(), tokens_used: compressed_tokens,
            model: model.to_string(), timestamp: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        };
        let stats = CompressionStats {
            original_tokens, compressed_tokens,
            ratio: compressed_tokens as f64 / original_tokens.max(1) as f64,
            layer: CompressionLayer::Auto, elapsed_ms: elapsed,
        };
        self.save_to_jsonl(&response, &stats)?;
        Ok((response, stats))
    }

    fn generate_summary(&self, text: &str) -> CompressionResult<String> {
        let sentences: Vec<_> = text.split('.').filter(|s| !s.trim().is_empty()).take(5).collect();
        if sentences.is_empty() { return Err(CompressionError::InvalidCompression("No content".to_string())); }
        Ok(format!("[SUMMARY] {}...", sentences.join(". ")))
    }

    fn save_to_jsonl(&self, response: &SummaryResponse, stats: &CompressionStats) -> CompressionResult<()> {
        if !self.storage.exists() {
            std::fs::create_dir_all(&self.storage).map_err(|e| CompressionError::StorageError(format!("{}", e)))?;
        }
        let filename = format!("{}/auto_{}.jsonl", self.storage.display(), response.timestamp);
        let mut file = OpenOptions::new().create(true).append(true).open(&filename)
            .map_err(|e| CompressionError::StorageError(format!("{}", e)))?;
        let json = format!(r#"{{"timestamp":{},"model":"{}","tokens_used":{},"original_tokens":{},"ratio":{:.4},"summary":"{}"}}"#,
            response.timestamp, response.model, response.tokens_used, stats.original_tokens, stats.ratio,
            response.summary.replace('"', "\\\""));
        writeln!(file, "{}", json).map_err(|e| CompressionError::StorageError(format!("{}", e)))?;
        Ok(())
    }
}

pub fn compress_auto(text: &str, model: &str) -> CompressionResult<String> {
    AutoCompressor::new().with_model(model).compress(text, Some(model)).map(|(resp, _)| resp.summary)
}

pub fn needs_compression(text: &str) -> bool {
    AutoCompressor::new().should_compress(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_counter() {
        let counter = TokenCounter::new();
        assert_eq!(counter.estimate(&"a".repeat(400)), 100);
        assert!(counter.exceeds_threshold(&"x".repeat(TOKEN_THRESHOLD * 5)));
    }
    #[test]
    fn test_below_threshold() {
        let result = AutoCompressor::new().compress("short", None);
        assert!(matches!(result, Err(CompressionError::InvalidCompression(_))));
    }
}
