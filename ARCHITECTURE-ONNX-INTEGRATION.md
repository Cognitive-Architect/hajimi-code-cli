# ONNX集成架构设计

## 概述

Hajimi项目的ONNX Runtime集成架构，包含模块设计、向后兼容性保证和Dream层集成方式。

## 模块架构

```
src/onnx/
├── mod.rs              # 模块入口，feature切换逻辑
├── real_inference.rs   # 真实ONNX Runtime推理实现
└── adapter.rs          # Mock适配器（默认编译）
```

### 编译条件

```rust
use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "onnx")] {
        pub use real_inference::OnnxInference;
    } else {
        pub use adapter::MockInference as OnnxInference;
    }
}
```

---

## 向后兼容性

### 接口签名

```rust
// 统一接口（两种模式一致）
pub async fn embed(&self, text: &str) -> Result<Vec<f32>, E>;
pub fn embed_blocking(&self, text: &str) -> Result<Vec<f32>, E>;
```

### 默认行为

- `cargo build` → 使用MockInference（零依赖）
- `cargo build --features onnx` → 使用真实ONNX

### 迁移路径

```rust
// 旧代码（无需修改）
let dream = DreamMemory::new("project")?;
let embedding = dream.embed("text")?;

// 新代码（显式使用ONNX引擎）
use onnx::{OnnxInference, EmbeddingEngine};
let engine = OnnxInference::from_path("model.onnx")?;
let embedding = engine.embed("text").await?;
```

---

## Dream层集成

### 当前集成方式

```rust
pub struct DreamMemory {
    embedding_model: OnnxSession, // 占位类型
}

impl DreamMemory {
    pub fn embed(&self, content: &str) -> Result<Vec<f32>, DreamError> {
        Ok(vec![0.0f32; EMBEDDING_DIM]) // Week 32 mock
    }
}
```

### Week 33-34 升级路径

```rust
pub struct DreamMemory {
    embedding_model: Box<dyn EmbeddingEngine>,
}
```

---

## 384维对齐保证

### 编译时常量

```rust
pub const EMBEDDING_DIM: usize = 384;
```

### 运行时验证

```rust
pub fn validate_output(embedding: &[f32]) -> Result<(), OnnxError> {
    if embedding.len() == EMBEDDING_DIM {
        Ok(())
    } else {
        Err(OnnxError::DimensionMismatch { 
            expected: EMBEDDING_DIM, 
            actual: embedding.len() 
        })
    }
}
```

---

## 性能考虑

### Mock模式
- 零外部依赖
- 确定性输出（便于测试）
- 适合CI/CD环境

### ONNX模式（Week 33-34启用）
- 真实语义embedding
- GPU加速支持
- 批处理优化

---

## 依赖配置

```toml
[features]
default = []
onnx = ["dep:ort", "dep:ndarray"]

[dependencies]
ort = { version = "2.0", optional = true }
ndarray = { version = "0.15", optional = true }
cfg-if = "1.0"
```

## 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `HAJIMI_ONNX_ENABLED` | false | 运行时启用ONNX |
| `HAJIMI_ONNX_MODEL_PATH` | - | 模型文件路径 |
| `HAJIMI_ONNX_TIMEOUT_MS` | 500 | 推理超时 |

---

**版本**: 1.0 | **更新日期**: 2026-04-03
