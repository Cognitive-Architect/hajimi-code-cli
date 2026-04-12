//! ONNX推理模块
//! DEBT-ONNX-API-W28: P1-活动中（Week 32启动）
pub mod adapter;
pub mod real_inference;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "onnx")] {
        pub use real_inference::OnnxInference;
        pub use real_inference::OnnxError;
    } else {
        pub use adapter::MockInference as OnnxInference;
        pub use adapter::MockError as OnnxError;
    }
}

/// 维度常量 - 必须严格对齐
pub const EMBEDDING_DIM: usize = 384;

/// 推理引擎trait - 向后兼容接口
pub trait EmbeddingEngine: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// 生成文本的embedding向量
    fn embed(&self, text: &str) -> impl std::future::Future<Output = Result<Vec<f32>, Self::Error>> + Send;

    /// 验证embedding维度
    fn validate_embedding(embedding: &[f32]) -> Result<(), DimensionError> {
        if embedding.len() == EMBEDDING_DIM {
            Ok(())
        } else {
            Err(DimensionError {
                expected: EMBEDDING_DIM,
                actual: embedding.len(),
            })
        }
    }
}

/// 维度错误
#[derive(Debug, Clone)]
pub struct DimensionError {
    pub expected: usize,
    pub actual: usize,
}

impl std::fmt::Display for DimensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "维度不匹配: 期望 {}, 实际 {}", self.expected, self.actual)
    }
}

impl std::error::Error for DimensionError {}

/// 工厂函数 - 创建默认推理引擎
pub fn create_engine() -> OnnxInference {
    OnnxInference::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_dim_constant() {
        assert_eq!(EMBEDDING_DIM, 384);
    }

    #[test]
    fn test_validate_embedding_ok() {
        let valid = vec![0.0f32; 384];
        assert!(<OnnxInference as EmbeddingEngine>::validate_embedding(&valid).is_ok());
    }

    #[test]
    fn test_validate_embedding_err() {
        let invalid = vec![0.0f32; 100];
        assert!(<OnnxInference as EmbeddingEngine>::validate_embedding(&invalid).is_err());
    }
}
