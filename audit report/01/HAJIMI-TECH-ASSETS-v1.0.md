# HAJIMI 技术资产清单 v1.0

> **来源**: DEBT-CLEARANCE-001 审计提取  
> **日期**: 2026-02-22  
> **资产类型**: 方法论 + 代码 + 流程

---

## 概述

本清单从 DEBT-CLEARANCE-001 债务清偿集群中提炼可复用的技术积累，供后续项目参考。

```
╔══════════════════════════════════════════════════════════════╗
║                    技术资产统计                               ║
╠══════════════════════════════════════════════════════════════╣
║  方法论资产: 3 项                                              ║
║  代码资产:   2 项                                              ║
║  流程资产:   1 项                                              ║
║  ─────────────────────────────────                          ║
║  总计:       6 项可复用技术点                                 ║
╚══════════════════════════════════════════════════════════════╝
```

---

## 1. 方法论资产 (3项)

### ASSET-001: 内存诚实声明方法论

**来源**: `docs/DEBT-HNSW-001-FIX.md`

**核心要点**:
1. **四组件分解法**: vectorData + index + cache + runtime
2. **实测验证**: Android 13 Termux OOM 阈值实测
3. **风险显式声明**: 明确声明"最低需求"和"系统杀后台风险"

**可复用场景**:
- 任何移动端内存敏感功能设计
- 资源需求文档编写
- 技术方案评审清单

**复用模板**:
```markdown
## 内存需求声明

### 四组件分解
| 组件 | 计算方式 | 大小 |
|------|----------|------|
| 主数据 | ... | ... |
| 索引 | ... | ... |
| 缓存 | ... | ... |
| 运行时 | ... | ... |

### 最低需求
- 前台: X MB
- 后台: Y MB

### 风险提示
⚠️ 场景 Z 下可能被系统终止
```

**引用章节**: DEBT-HNSW-001-FIX.md 第 2.1、3、4 节

---

### ASSET-002: 概率算法验证方法论

**来源**: `docs/DEBT-LSH-001-REPORT.md` + `src/test/lsh-collision-sim.js`

**核心要点**:
1. **理论计算先行**: 泊松分布/生日悖论数学推导
2. **模拟验证**: 蒙特卡洛模拟 (100K 向量 × 1K 查询)
3. **调参预案**: 超标时的参数调整方案

**可复用场景**:
- 哈希碰撞概率评估
- Bloom Filter 参数设计
- 任何概率型数据结构验证

**复用步骤**:
```
步骤 1: 理论计算 P(事件)
   ↓
步骤 2: 简化模型模拟
   ↓
步骤 3: 对比理论 vs 实测
   ↓
步骤 4: 准备调参方案 (预防性)
```

**引用章节**: DEBT-LSH-001-REPORT.md 第 2、3、5 节

---

### ASSET-003: 分片架构对比决策法

**来源**: `docs/SQLITE-SHARDING-方案对比.md`

**核心要点**:
1. **六维度评分**: 锁竞争、并发、查询、备份、工时、扩展性
2. **权重明确**: 根据业务特点分配权重 (查询模式 15%, 锁竞争 20%)
3. **量化评分**: 1-10 分制，加权求和

**可复用场景**:
- 数据库分片方案选择
- 存储架构决策
- 技术选型评估

**复用模板**:
```markdown
| 维度 | 权重 | 方案A | 方案B | 方案C |
|------|------|-------|-------|-------|
| 维度1 | W% | X/10 | Y/10 | Z/10 |
| 维度2 | W% | X/10 | Y/10 | Z/10 |
| **总分** | 100% | **Σ** | **Σ** | **Σ** |

推荐方案: [最高分方案] + 理由
```

**引用章节**: SQLITE-SHARDING-方案对比.md 第 2、3 节

---

## 2. 代码资产 (2项)

### ASSET-004: LSH 假阳性率模拟器

**来源**: `src/test/lsh-collision-sim.js`

**资产内容**:
- 完整 LSH 索引实现 (支持 SimHash-64)
- 汉明距离高效计算 (Brian Kernighan 算法)
- 理论 FPR 计算 (泊松近似)
- 可配置 CLI (--vectors, --queries, --verbose)

**可复用场景**:
- LSH 参数调优
- 向量检索系统测试
- 教学/演示用途

**复用方式**:
```bash
# 基础使用
node lsh-collision-sim.js --vectors 10000 --queries 100

# 输出 JSON 报告
node lsh-collision-sim.js --json > report.json

# 集成到 CI
node lsh-collision-sim.js --vectors 1000 --queries 50 || exit 1
```

**关键函数**:
| 函数 | 用途 | 行号 |
|------|------|------|
| `simhash64()` | 计算 SimHash | 35-61 |
| `hammingDistance()` | 汉明距离 | 66-75 |
| `LSHIndex.queryCandidates()` | LSH 查询 | 126-148 |
| `calculateTheoreticalFPR()` | 理论 FPR | 208-226 |

---

### ASSET-005: 同步降级状态机模式

**来源**: `src/sync/fallback-strategy.md`

**资产内容**:
- 5 状态同步状态机 (IDLE → DISCOVERING → CONNECTING → ...)
- 3 触发降级条件 (ICE_FAILED, TIMEOUT, PEER_NOT_FOUND)
- 明确超时配置 (gatheringTimeout=5s, connectionTimeout=10s)
- 用户提示设计规范

**可复用场景**:
- 任何"主路径失败 → 降级路径"场景
- 网络传输模块设计
- 多策略自动切换系统

**复用模板**:
```javascript
class FallbackManager extends EventEmitter {
  constructor(options) {
    this.state = 'IDLE';
    this.config = {
      primaryTimeout: options.primaryTimeout || 10000,
      enableFallback: true
    };
  }
  
  async execute(primaryFn, fallbackFn) {
    try {
      this.state = 'TRYING_PRIMARY';
      return await Promise.race([
        primaryFn(),
        this.timeout(this.config.primaryTimeout)
      ]);
    } catch (error) {
      if (this.config.enableFallback) {
        this.emit('fallback', { error });
        this.state = 'FALLBACK';
        return await fallbackFn();
      }
      throw error;
    }
  }
}
```

**引用章节**: fallback-strategy.md 第 2、3、5 节

---

## 3. 流程资产 (1项)

### ASSET-006: 技术债务清偿 SOP

**来源**: `docs/DEBT-CLEARANCE-001-白皮书-v1.0.md`

**核心要点**:
1. **债务分级**: P0(致命)/P1(高)/P2(中) 三级分类
2. **四工单并行**: 按依赖关系拆分，AI Agent 并行处理
3. **60项自测**: 功能+边界+文档三维覆盖
4. **质量门禁**: 自测全绿 → Mike 审计 → 最终批准

**可复用场景**:
- 大规模技术债务清偿项目
- 代码重构流程规范
- 交付物质量保障

**复用流程**:
```
项目启动
   ↓
债务识别 → 分级 (P0/P1/P2)
   ↓
工单拆分 → 并行处理
   ↓
自测 (功能/边界/文档)
   ↓
质量门禁检查
   ↓
审计员审计
   ↓
批准/返工
```

**关键交付物模板**:
| 交付物 | 用途 | 模板位置 |
|--------|------|----------|
| 债务白皮书 | 整合所有修复成果 | 白皮书第 1-3 节 |
| 自测表 | 60项检查清单 | 自测表第 2-5 节 |
| 单债务报告 | 单项债务详细修复 | HNSW/LSH/...-FIX.md |

**引用章节**: DEBT-CLEARANCE-001-白皮书-v1.0.md 全文

---

## 4. 资产索引速查

| 场景 | 推荐资产 | 快速引用 |
|------|----------|----------|
| 移动端内存设计 | ASSET-001 | 四组件分解法 |
| 概率算法验证 | ASSET-002 | 理论+模拟双验证 |
| 分片架构决策 | ASSET-003 | 六维度评分表 |
| 向量检索测试 | ASSET-004 | lsh-collision-sim.js |
| 降级策略设计 | ASSET-005 | 状态机模式 |
| 债务清偿项目 | ASSET-006 | 60项自测 SOP |

---

## 5. 资产维护建议

### 持续积累

- **代码资产**: 将验证脚本统一放入 `src/test/lib/`，供其他项目引用
- **方法论资产**: 在 wiki 或 notion 建立"技术决策模式"知识库
- **流程资产**: 将 SOP 固化为项目模板，新项目一键复用

### 版本管理

建议后续资产清单按 SEMVER 管理：
- MAJOR: 新增资产类别
- MINOR: 新增单项资产
- PATCH: 资产内容修正

---

**清单版本**: v1.0  
**资产数量**: 6 项  
**覆盖领域**: 内存管理、算法验证、架构决策、代码实现、流程规范  
**维护人**: Mike (代码审计汪)

---

> **使用提示**: 本清单所有资产均可直接引用，无需授权。建议在项目文档中标注引用来源，便于追溯和同步更新。
