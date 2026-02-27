# A.Hajimi 算法研究院 - 文档索引

> **项目**: Hajimi V3 本地存储系统  
> **当前阶段**: Phase 4 已完成 ✅ (Task 10 部分完成)  
> **最后更新**: 2026-02-26

---

## 📚 核心文档速览

### 🏗️ 架构设计

| 文档 | 类型 | 状态 | 一句话摘要 |
|:-----|:-----|:-----|:-----------|
| [local-storage-v3-design.md](./task01-架构设计/local-storage-v3-design.md) | 架构设计 | ✅ 已完成 | V3 存储系统整体设计摘要（代码已落地） |
| [PHASE1-白皮书-v1.0.md](./task04-Phase1分片实现/PHASE1-白皮书-v1.0.md) | 阶段报告 | ✅ A级 | Phase 1 完整交付：16分片+API+测试 |
| [PHASE2-HNSW-白皮书-v1.0.md](./task06-phase2-hnsw/HAJIMI-PHASE2-HNSW-白皮书-v1.0.md) | 阶段报告 | ✅ A级 | Phase 2 HNSW向量索引：100K构建~25s，召回率97% |
| [PHASE2.1-白皮书-v1.0.md](./task07-phase2.1-debt-clearance/HAJIMI-PHASE2.1-白皮书-v1.0.md) | 债务清偿 | ✅ A级 | WAL自动截断+写入队列+二进制序列化(12x提升) |
| [PHASE3-白皮书-v1.0.md](./task08-phase3-wasm-disk-api/HAJIMI-PHASE3-白皮书-v1.0.md) | 阶段报告 | ✅ A级 | HTTP API服务器+磁盘溢出+WASM框架 |
| [PHASE4-白皮书-v1.0.md](./task09-phase4-wasm-worker-robust/HAJIMI-PHASE4-白皮书-v1.0.md) | 阶段报告 | ✅ A级 | Worker Thread+磁盘鲁棒性+WASM-JS混合 |
| [WASM-COMPILE-白皮书-v1.0.md](./task10-wasm-compile/HAJIMI-WASM-COMPILE-白皮书-v1.0.md) | 编译报告 | ⚠️ 部分完成 | WASM编译过程+性能验证+债务状态更新 |
| [SQLITE-SHARDING-方案对比.md](./task02-技术债务清偿/SQLITE-SHARDING-方案对比.md) | 方案选型 | ✅ 已完成 | 3种分片方案对比，Hash分片胜出 |
| [V3-ROADMAP-v2-CORRECTED.md](./task02-技术债务清偿/V3-ROADMAP-v2-CORRECTED.md) | 路线图 | ✅ 已校正 | 10周工期规划（含WebRTC降级） |

### 💰 技术债务

| 文档 | 类型 | 状态 | 一句话摘要 |
|:-----|:-----|:-----|:-----------|
| [DEBT-CLEARANCE-001-白皮书-v1.0.md](./task02-技术债务清偿/DEBT-CLEARANCE-001-白皮书-v1.0.md) | 债务清偿 | ✅ 6项已清 | 技术债务清偿完整报告 |
| [V3-STORAGE-DEBT-自测表-v1.0.md](./task02-技术债务清偿/V3-STORAGE-DEBT-自测表-v1.0.md) | 自测清单 | ✅ 60项通过 | 债务清偿60项自测清单 |
| [DEBT-HNSW-001-FIX.md](./task02-技术债务清偿/DEBT-HNSW-001-FIX.md) | 内存修正 | ✅ P0已清 | HNSW内存估算校正（400MB声明） |
| [DEBT-LSH-001-REPORT.md](./task02-技术债务清偿/DEBT-LSH-001-REPORT.md) | LSH报告 | ✅ P1已清 | SimHash-64假阳性率验证报告 |
| [DEBT-LSH-001-FIXED.md](./task03-基线修复/DEBT-LSH-001-FIXED.md) | LSH修复 | ✅ R-001修复 | 生产级SimHash实现修复 |
| [DEBT-WEBRTC-IMPLEMENTED.md](./task03-基线修复/DEBT-WEBRTC-IMPLEMENTED.md) | 降级实现 | ✅ R-003实现 | WebRTC降级状态机代码实现 |
| [PHASE1-DEBT-v1.1.md](./task05-Phase1修复/PHASE1-DEBT-v1.1.md) | 债务更新 | ✅ 已更新 | Phase1修复后债务状态（5项） |
| [PHASE2-DEBT-v1.0.md](./task06-phase2-hnsw/HAJIMI-PHASE2-DEBT-v1.0.md) | 债务声明 | ✅ 已声明 | Phase 2 新增5项技术债务 |
| [PHASE2.1-DEBT-CLEARANCE-REPORT.md](./task07-phase2.1-debt-clearance/HAJIMI-PHASE2.1-DEBT-CLEARANCE-REPORT.md) | 清偿报告 | ✅ 3项已清 | WAL+队列+二进制序列化清偿报告 |

### 🔧 模块设计 (Phase 1)

| 文档 | 模块 | 对应代码 |
|:-----|:-----|:---------|
| [PHASE1-SHARD-ROUTER.md](./task04-Phase1分片实现/PHASE1-SHARD-ROUTER.md) | 路由层 | `src/storage/shard-router.js` |
| [PHASE1-CONN-POOL.md](./task04-Phase1分片实现/PHASE1-CONN-POOL.md) | 连接池 | `src/storage/connection-pool.js` |
| [PHASE1-CHUNK-FORMAT.md](./task04-Phase1分片实现/PHASE1-CHUNK-FORMAT.md) | Chunk格式 | `src/storage/chunk.js` |
| [PHASE1-MIGRATION.md](./task04-Phase1分片实现/PHASE1-MIGRATION.md) | 迁移工具 | `src/storage/migrate.js` |
| [PHASE1-API.md](./task04-Phase1分片实现/PHASE1-API.md) | 统一API | `src/api/storage.js` |

### 🧠 向量索引 (Phase 2/4)

| 文档 | 模块 | 对应代码 |
|:-----|:-----|:---------|
| [HNSW-CORE.md](./task06-phase2-hnsw/HNSW-CORE.md) | HNSW核心 | `src/vector/hnsw-core.js` |
| [HYBRID-RETRIEVER.md](./task06-phase2-hnsw/HYBRID-RETRIEVER.md) | 混合检索 | `src/vector/hybrid-retriever.js` |
| [WASM-BRIDGE.md](./task08-phase3-wasm-disk-api/WASM-BRIDGE.md) | WASM桥接 | `src/wasm/hnsw-bridge.js` |
| [HNSW-INDEX-HYBRID.md](./task09-phase4-wasm-worker-robust/HNSW-INDEX-HYBRID.md) | 混合索引 | `src/vector/hnsw-index-hybrid.js` |

### 📋 测试与交付

| 文档 | 类型 | 说明 |
|:-----|:-----|:-----|
| [DEBT-TEST-UNIFIED.md](./task03-基线修复/DEBT-TEST-UNIFIED.md) | 测试脚本 | `run-debt-tests.sh` 使用文档 |
| [PHASE1-自测表-v1.0.md](./task04-Phase1分片实现/PHASE1-自测表-v1.0.md) | 自测清单 | Phase1 33项自测 |
| [PHASE2-HNSW-自测表-v1.0.md](./task06-phase2-hnsw/HAJIMI-PHASE2-HNSW-自测表-v1.0.md) | 自测清单 | Phase2 7项自测 |
| [PHASE2.1-自测表-v1.0.md](./task07-phase2.1-debt-clearance/HAJIMI-PHASE2.1-自测表-v1.0.md) | 自测清单 | Phase2.1 5项自测 |
| [PHASE3-自测表-v1.0.md](./task08-phase3-wasm-disk-api/HAJIMI-PHASE3-自测表-v1.0.md) | 自测清单 | Phase3 6项自测 |
| [PHASE4-自测表-v1.0.md](./task09-phase4-wasm-worker-robust/HAJIMI-PHASE4-自测表-v1.0.md) | 自测清单 | Phase4 6项自测 |
| [WASM-COMPILE-自测表-v1.0.md](./task10-wasm-compile/HAJIMI-WASM-COMPILE-自测表-v1.0.md) | 自测清单 | WASM编译12项自测 |
| [FIX-REPORT-001.md](./task05-Phase1修复/FIX-REPORT-001.md) | 修复报告 | Phase1修复审计缺陷报告 |
| [Hajimi-Task03-DELIVERY.md](./task03-基线修复/Hajimi-Task03-DELIVERY.md) | 交付报告 | Task03 交付物清单 |
| [Hajimi-Task04-DELIVERY.md](./task04-Phase1分片实现/Hajimi-Task04-DELIVERY.md) | 交付报告 | Task04 交付物清单 |

---

## 🎯 快速导航

### 如果你是开发者

1. **整体了解** → [PHASE4-白皮书-v1.0.md](./task09-phase4-wasm-worker-robust/HAJIMI-PHASE4-白皮书-v1.0.md)
2. **架构设计** → [local-storage-v3-design.md](./task01-架构设计/local-storage-v3-design.md)
3. **模块细节** → 查看 `PHASE1-XXX.md`、`HNSW-CORE.md` 系列
4. **运行测试** → 
   ```bash
   # Phase 1
   node src/test/shard-router.test.js        # 8/8
   node src/test/connection-pool.test.js     # 7/7
   node src/test/chunk.test.js               # 7/7
   node src/test/storage-integration.test.js # 6/6
   
   # Phase 2
   node src/cli/vector-debug.js benchmark    # HNSW基准
   node src/test/phase2.1-benchmark.test.js  # 债务清偿验证
   
   # Phase 3-4
   node tests/e2e/phase4-integration.test.js # 4项E2E测试
   node tests/benchmark/wasm-vs-js.bench.js  # WASM性能对比
   ```

### 如果你是审计员

1. **债务清偿** → [DEBT-CLEARANCE-001-白皮书-v1.0.md](./task02-技术债务清偿/DEBT-CLEARANCE-001-白皮书-v1.0.md)
2. **Phase 2.1清偿** → [PHASE2.1-DEBT-CLEARANCE-REPORT.md](./task07-phase2.1-debt-clearance/HAJIMI-PHASE2.1-DEBT-CLEARANCE-REPORT.md)
3. **自测清单** → [V3-STORAGE-DEBT-自测表-v1.0.md](./task02-技术债务清偿/V3-STORAGE-DEBT-自测表-v1.0.md)
4. **修复验证** → [FIX-REPORT-001.md](./task05-Phase1修复/FIX-REPORT-001.md)
5. **WASM编译** → [WASM-COMPILE-白皮书-v1.0.md](./task10-wasm-compile/HAJIMI-WASM-COMPILE-白皮书-v1.0.md)

### 如果你是项目经理

1. **进度跟踪** → [V3-ROADMAP-v2-CORRECTED.md](./task02-技术债务清偿/V3-ROADMAP-v2-CORRECTED.md)
2. **债务状态** → [PHASE1-DEBT-v1.1.md](./task05-Phase1修复/PHASE1-DEBT-v1.1.md)
3. **阶段交付** → [PHASE4-白皮书-v1.0.md](./task09-phase4-wasm-worker-robust/HAJIMI-PHASE4-白皮书-v1.0.md)
4. **性能指标** → [PHASE2-HNSW-白皮书-v1.0.md](./task06-phase2-hnsw/HAJIMI-PHASE2-HNSW-白皮书-v1.0.md)

---

## 📊 项目概览

### 项目信息

- **项目代号**: HAJIMI-V3-PHASE4
- **当前版本**: v3.0-PHASE4 ✅
- **核心创新**: 16分片SQLite + HNSW向量索引 + Worker Thread + WASM优化
- **存储目标**: 100K+ 向量分片，单分片6.25K记录
- **性能目标**: 100K构建<80s, P99查询<100ms, 召回率>95%

### 关键数据

| 指标 | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|:-----|:---:|:---:|:---:|:---:|
| 分片数 | 16 | 16 | 16 | 16 |
| 测试用例 | 33项 | 7项 | 6项 | 6项 |
| 代码文件 | 9个 | 15+ | 25+ | 35+ |
| 技术债务 | 6项已清 | 5项新增 | 3项已清 | 1项部分 |
| 总代码行 | ~3000 | ~8000 | ~12000 | ~14000 |

### 性能指标

| 指标 | 数值 | 目标 |
|:---|---:|:---:|
| 磁盘写入 | 19.38 MB/s | - |
| 随机读取 | 0.028ms | <100ms ✅ |
| 100K向量内存 | 60.59MB | <200MB ✅ |
| 并发请求 | 1875 ops/s | >100/s ✅ |
| 100K构建时间 | ~25s | <80s ✅ |
| 查询P99 | ~45ms | <100ms ✅ |
| 召回率 | ~97% | >95% ✅ |

---

## 🔧 技术关键词

```
16分片存储 (shard_00.db ~ shard_0f.db)    SimHash-64 高8bit路由
SQLite + 文件系统混合存储                 .hctx v3 文件格式
WebRTC P2P + 文件导出降级                  connection-pool (8连接/分片)
ChunkStorage (SHA256校验)                  StorageV3 API (put/get/delete)

HNSW向量索引 (M=16, ef=200)               混合检索 (HNSW+LSH)
WAL自动检查点 (<110MB)                     写入队列 (并发安全)
二进制序列化 (12x加速)                     懒加载 (内存<60MB)

HTTP API服务器 (/health, /vector/*)       磁盘溢出管理 (ENOSPC处理)
Worker Thread (native worker_threads)      紧急模式 (内存-only)
WASM优化 (477KB, 目标5x加速)              WASM-JS混合 (自动降级)
```

---

## ⚠️ 技术债务状态

### 已清偿 (Phase 1-2.1)

- ✅ DEBT-HNSW-001: 内存估算校正
- ✅ DEBT-LSH-001: 假阳性率验证
- ✅ DEBT-SQLITE-001: 分片架构选型
- ✅ DEBT-WEBRTC-001: NAT穿透降级设计
- ✅ DEBT-ROADMAP-001: 工期校正
- ✅ DEBT-PHASE2-006: WAL无限膨胀 → 自动截断
- ✅ DEBT-PHASE2-007: 并发安全风险 → 队列化保护
- ✅ DEBT-PHASE2-005: JSON序列化慢 → 二进制(12x)
- ✅ DEBT-PHASE2-003: 磁盘溢出 → 已实现
- ✅ DEBT-PHASE2-004: Worker Thread → 已实现

### 待处理

| ID | 描述 | 优先级 | 状态 | 计划 |
|:---|:-----|:-------|:---:|:---|
| DEBT-PHASE2-001 | WASM优化 | P1 | ⚠️ 85% | 字节码已生成，运行时待完善 |

---

## 📁 文档分类

```
docs/
├── 架构设计/
│   ├── local-storage-v3-design.md
│   ├── PHASE1-白皮书-v1.0.md
│   ├── PHASE2-HNSW-白皮书-v1.0.md
│   ├── PHASE2.1-白皮书-v1.0.md
│   ├── PHASE3-白皮书-v1.0.md
│   ├── PHASE4-白皮书-v1.0.md
│   ├── WASM-COMPILE-白皮书-v1.0.md
│   └── SQLITE-SHARDING-方案对比.md
├── 模块设计/
│   ├── PHASE1-SHARD-ROUTER.md
│   ├── PHASE1-CONN-POOL.md
│   ├── PHASE1-CHUNK-FORMAT.md
│   ├── PHASE1-MIGRATION.md
│   ├── PHASE1-API.md
│   ├── HNSW-CORE.md
│   ├── HYBRID-RETRIEVER.md
│   └── WASM-BRIDGE.md
├── 债务文档/
│   ├── DEBT-CLEARANCE-001-白皮书-v1.0.md
│   ├── V3-STORAGE-DEBT-自测表-v1.0.md
│   ├── PHASE1-DEBT-v1.1.md
│   ├── PHASE2-DEBT-v1.0.md
│   ├── PHASE2.1-DEBT-CLEARANCE-REPORT.md
│   └── DEBT-汇总.md
├── 自测清单/
│   ├── PHASE1-自测表-v1.0.md
│   ├── PHASE2-HNSW-自测表-v1.0.md
│   ├── PHASE2.1-自测表-v1.0.md
│   ├── PHASE3-自测表-v1.0.md
│   ├── PHASE4-自测表-v1.0.md
│   └── WASM-COMPILE-自测表-v1.0.md
├── 路线图/
│   └── V3-ROADMAP-v2-CORRECTED.md
└── 交付报告/
    ├── FIX-REPORT-001.md
    ├── Hajimi-Task03-DELIVERY.md
    └── Hajimi-Task04-DELIVERY.md
```

---

> 💡 **提示**: 本文档索引与 workspace/README.md 保持同步，反映 Hajimi V3 本地存储系统 Phase 1-4 的完整交付状态。
