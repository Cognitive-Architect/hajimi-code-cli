# DEBT-ONNX-API-W28-W32

## 债务状态

**状态**: P1-活动中  
**启动**: Week 32  
**预计清偿**: Week 34  
**负责人**: ONNX清偿工程师

---

## Week 32 完成内容

### 接口迁移（Mock → 真实推理）

1. **ONNX模块架构**
   - 创建 `src/onnx/` 模块目录
   - 实现 `mod.rs` 主入口（40行）
   - 实现 `real_inference.rs` 真实推理（150行）
   - 实现 `adapter.rs` 适配器（60行）

2. **Feature Flag 切换**
   - 默认编译：`cargo build` → 使用Mock
   - ONNX模式：`cargo build --features onnx` → 使用真实ONNX
   - `cfg_if` 条件编译实现无缝切换

3. **向后兼容保证**
   - `embed()` 签名保持不变
   - Dream层集成无侵入式修改
   - 384维强制校验（运行时+编译时）

4. **代码质量**
   - 零unwrap新增（使用`?`或`match`）
   - 完整错误类型定义（`OnnxError`/`MockError`）
   - 单元测试覆盖（维度/一致性/归一化）

---

## Week 33-34 清偿路径

### 模型优化阶段

1. **ONNX Runtime集成**
   - 启用 `ort = "2.0"` 依赖
   - 加载真实ONNX模型文件
   - 实现Tokenizer预处理

2. **推理优化**
   - 批处理支持（batch inference）
   - 异步会话管理
   - 内存池优化

3. **测试验证**
   - 真实模型端到端测试
   - 性能基准测试
   - 内存泄漏检测

4. **文档更新**
   - 部署指南（模型文件路径）
   - 性能调优手册
   - 故障排查指南

---

## 向后兼容说明

### 编译模式

```bash
# 默认模式 - 使用Mock推理
cargo build

# ONNX模式 - 使用真实推理
cargo build --features onnx

# 测试两种模式
cargo test --features onnx
cargo test
```

### Dream层集成

```rust
// 使用方式不变
use memory::dream::DreamMemory;

let dream = DreamMemory::new("project")?;
let embedding = dream.embed("text")?; // 返回Vec<f32>
assert_eq!(embedding.len(), 384);
```

### 环境变量

```bash
# 运行时切换（可选）
export HAJIMI_ONNX_ENABLED=true
export HAJIMI_ONNX_MODEL_PATH=/path/to/model.onnx
export HAJIMI_ONNX_TIMEOUT_MS=500
```

---

## 文件清单

| 文件 | 行数 | 说明 |
|------|------|------|
| `src/onnx/mod.rs` | 40 | 模块入口，feature切换 |
| `src/onnx/real_inference.rs` | 150 | 真实ONNX推理 |
| `src/onnx/adapter.rs` | 60 | Mock适配器 |
| `src/memory/Cargo.toml` | +3 | ort依赖（注释状态） |
| `ARCHITECTURE-ONNX-INTEGRATION.md` | 100 | 架构文档 |

---

## 验收标准

- [x] Week 32：接口迁移完成
- [ ] Week 33：ONNX Runtime集成
- [ ] Week 34：模型优化+测试验证
- [ ] 最终：债务状态迁移至P0-已清偿

---

**最后更新**: 2026-04-03
