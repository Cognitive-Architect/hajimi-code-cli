//! 四层压缩架构模块 (Week 29)

pub mod auto;
pub mod compact;
pub mod micro;

pub use auto::{compress_auto, AutoCompressor, TokenCounter};
pub use compact::{compact, CompactCompressor, CompactOptions, CompressionLevel};
pub use micro::{compress_micro, MicroCompressor};

use std::fmt;

/// Auto层Token触发阈值（50k tokens）
pub const TOKEN_THRESHOLD: usize = 50000;

/// 压缩层级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionLayer { Micro, Auto, Compact, #[cfg(feature = "p2")] Cascade }

impl fmt::Display for CompressionLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            CompressionLayer::Micro => "micro",
            CompressionLayer::Auto => "auto",
            CompressionLayer::Compact => "compact",
            #[cfg(feature = "p2")]
            CompressionLayer::Cascade => "cascade",
        })
    }
}

/// 压缩错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum CompressionError {
    EmptyInput, TokenCountFailed(String), LlmApiError(String),
    InvalidCompression(String), StorageError(String), UnsupportedLayer(String),
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionError::EmptyInput => write!(f, "Input text is empty"),
            CompressionError::TokenCountFailed(msg) => write!(f, "Token count failed: {}", msg),
            CompressionError::LlmApiError(msg) => write!(f, "LLM API error: {}", msg),
            CompressionError::InvalidCompression(msg) => write!(f, "Invalid compression: {}", msg),
            CompressionError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            CompressionError::UnsupportedLayer(msg) => write!(f, "Unsupported layer: {}", msg),
        }
    }
}

impl std::error::Error for CompressionError {}

/// 压缩结果类型
pub type CompressionResult<T> = Result<T, CompressionError>;

/// 压缩统计信息
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    pub original_tokens: usize, pub compressed_tokens: usize,
    pub ratio: f64, pub layer: CompressionLayer, pub elapsed_ms: u64,
}

impl CompressionStats {
    pub fn tokens_saved(&self) -> usize { self.original_tokens.saturating_sub(self.compressed_tokens) }
    pub fn savings_percent(&self) -> f64 {
        if self.original_tokens == 0 { return 0.0; }
        (self.tokens_saved() as f64 / self.original_tokens as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_layer_display() { assert_eq!(CompressionLayer::Micro.to_string(), "micro"); }
    #[test]
    fn test_stats() {
        let s = CompressionStats { original_tokens: 1000, compressed_tokens: 400, ..Default::default() };
        assert_eq!(s.tokens_saved(), 600);
        assert_eq!(s.savings_percent(), 60.0);
    }
}
