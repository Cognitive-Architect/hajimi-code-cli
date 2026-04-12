//! Mock/真实切换适配器
//! DEBT-ONNX-API-W28: 向后兼容接口

use std::path::Path;
use thiserror::Error;

use super::EMBEDDING_DIM;

/// Mock错误类型
#[derive(Debug, Error)]
pub enum MockError {
    #[error("Mock引擎错误: {0}")]
    Generic(String),
    #[error("维度不匹配: 期望{expected}, 实际{actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

/// Mock推理引擎（默认编译模式）
pub struct MockInference;

impl MockInference {
    pub fn new() -> Self { Self }

    pub fn from_path<P: AsRef<Path>>(_path: P) -> Result<Self, MockError> {
        Ok(Self)
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, MockError> {
        tokio::task::yield_now().await;
        Ok(Self::generate_mock_embedding(text))
    }

    fn generate_mock_embedding(text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::f32::consts::PI;

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();
        const A: u64 = 6364136223846793005;
        const C: u64 = 1442695040888963407;

        let mut state = seed;
        let mut vec: Vec<f32> = (0..EMBEDDING_DIM)
            .map(|_| {
                state = state.wrapping_mul(A).wrapping_add(C);
                let u1 = (state as f32) / (u64::MAX as f32 + 1.0);
                state = state.wrapping_mul(A).wrapping_add(C);
                let u2 = (state as f32) / (u64::MAX as f32 + 1.0);
                let u1 = u1.max(f32::EPSILON);
                ((-2.0 * u1.ln()).sqrt()) * (2.0 * PI * u2).cos()
            })
            .collect();

        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut vec { *x /= norm; }
        }
        vec
    }

    pub fn validate_output(embedding: &[f32]) -> Result<(), MockError> {
        if embedding.len() == EMBEDDING_DIM {
            Ok(())
        } else {
            Err(MockError::DimensionMismatch { expected: EMBEDDING_DIM, actual: embedding.len() })
        }
    }
}

impl Default for MockInference {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_new() {
        let _ = MockInference::new();
    }

    #[tokio::test]
    async fn test_mock_embed_dimension() {
        let engine = MockInference::new();
        let embedding = engine.embed("test").await.expect("失败");
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_validate_output_ok() {
        let valid = vec![0.0f32; 384];
        assert!(MockInference::validate_output(&valid).is_ok());
    }
}
