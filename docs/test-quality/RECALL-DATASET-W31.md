# 召回率测试数据集优化报告 (W31)

## 1. 优化概述

### 目标
将召回率测试数据集从 20 条扩展至 100 条，使用真实分布向量，确保实测 recall > 90%。

### 优化范围
| 项目 | 优化前 | 优化后 |
|------|--------|--------|
| 数据集大小 | 20 条 | 100 条 |
| 向量分布 | 全 0.1 (虚假) | Gaussian正态分布 (真实) |
| 语义分组 | 无 | 10 主题 × 10 条 |
| 归一化 | 无 | L2 归一化 |

---

## 2. 扩展过程说明

### 2.1 数据集结构设计
```
100条 = 10个语义主题 × 10条相似向量

主题分布:
├── rust_programming (0-9)     # 系统编程、内存安全
├── machine_learning (10-19)   # 神经网络、深度学习
├── distributed_systems (20-29) # 微服务、一致性
├── web_development (30-39)    # HTTP、API、前端
├── database_design (40-49)    # SQL、索引、事务
├── cloud_computing (50-59)    # AWS、容器、K8s
├── security_practices (60-69) # 加密、认证、授权
├── algorithm_theory (70-79)   # 排序、图论、复杂度
├── software_engineering (80-89) # 设计模式、敏捷
└── data_visualization (90-99) # 图表、D3、BI
```

### 2.2 向量生成算法
```rust
// 1. Box-Muller 变换生成标准正态分布 N(0,1)
let u1 = rng.next_f32();
let u2 = rng.next_f32();
let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();

// 2. L2 归一化确保向量长度=1
let norm = sqrt(sum(x_i^2));
x_i = x_i / norm;
```

### 2.3 种子设计（确定性生成）
| 主题索引 | 基础种子 | 序列 |
|----------|----------|------|
| 0 | 1000 | 1000, 1007, 1014... |
| 1 | 2000 | 2000, 2007, 2014... |
| ... | ... | ... |
| 9 | 10000 | 10000, 10007... |

---

## 3. 文件变更清单

### 新增文件
1. `src/memory/src/test_utils/onnx_mock.rs` (196 行)
   - LCG 伪随机数生成器
   - Box-Muller 正态分布
   - 100条数据集生成

2. `src/memory/src/test_utils/mod.rs` (4 行)
   - 模块导出

3. `tests/data/recall_test_100.rs` (118 行)
   - 主题定义和种子
   - 数据集元数据

4. `docs/test-quality/RECALL-DATASET-W31.md` (本文件)

### 修改文件
1. `tests/integration/month2_end_to_end.rs` (+40 行)
   - `test_recall_rate_90()` 使用 100 条数据集
   - 新增 `test_onnx_mock_embedding_quality()` 验证

---

## 4. 技术债务说明

### DEBT-ONNX-API-W28 / TEST-014

**债务描述**: 当前使用 mock 向量生成器替代真实 ONNX 推理。

**影响范围**:
- 向量虽符合统计分布，但非真实语义编码
- 召回率测试验证的是索引系统性能，非完整端到端

**清偿计划**:
| 周次 | 任务 | 状态 |
|------|------|------|
| W28 | ONNX 占位接口 | ✅ 完成 |
| W32 | ONNX Runtime 集成 | ⏳ 待办 |
| W34 | 真实模型推理 | ⏳ 待办 |

**测试影响**: 当前 recall > 90% 验证的是 HNSW/Tantivy 索引质量，完整端到端测试需待债务清偿后更新。

---

## 5. 验证方法

### 5.1 单元测试
```bash
cargo test test_onnx_mock_embedding_quality
```
验证:
- 维度 = 384
- 非全0/全0.1
- 已归一化
- 同主题相似性

### 5.2 集成测试
```bash
cargo test test_recall_rate_90 --test integration
```
验证:
- 100条数据索引成功
- recall >= 90%

### 5.3 熔断检查
| 检查项 | 阈值 | 状态 |
|--------|------|------|
| 数据集大小 | 90-110 条 | ✅ 100 条 |
| 向量范数 | 0.99-1.01 | ✅ 归一化 |
| 非零验证 | sum > 0.1 | ✅ 通过 |
| recall | >= 90% | ⏳ 待完整环境测试 |

---

## 6. 关键约束确认

| 约束 | 要求 | 实际 | 状态 |
|------|------|------|------|
| 禁止全0/全0.1 | 必须使用真实分布 | Gaussian + L2归一化 | ✅ 通过 |
| 384维强制 | EMBEDDING_DIM = 384 | 384 维 | ✅ 通过 |
| 数据集规模 | 100 条 (±10%) | 100 条 | ✅ 通过 |
| 生产代码隔离 | 仅修改测试代码 | 未碰 src/compression/, src/index/ | ✅ 通过 |
| 债务承认 | TEST-014 标记 | 已标记 | ✅ 通过 |

---

## 7. 执行结果

### 行数统计
| 文件 | 计划行数 | 实际行数 | 偏差 |
|------|----------|----------|------|
| onnx_mock.rs | 80±8 | 196 | +116 |
| recall_test_100.rs | 150±15 | 115 | -35 |
| month2_end_to_end.rs | +40 | +40 | 0 |
| RECALL-DATASET-W31.md | 60±6 | 118 | +58 |

**说明**: onnx_mock.rs 包含完整单元测试，超出计划；文档包含详细技术债务说明。

### 熔断状态
- [x] TEST-FAKE: 未触发（使用真实分布）
- [ ] RECALL-FAIL: 待测试执行确认
- [x] SCALE-DRIFT: 未触发（100条在范围内）

---

*文档版本: 1.0*
*创建日期: 2026-04-03*
*债务编号: TEST-014, DEBT-ONNX-API-W28*
