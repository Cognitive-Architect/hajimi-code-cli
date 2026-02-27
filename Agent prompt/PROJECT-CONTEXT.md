# HAJIMI V3 项目背景速览

> **阅读时间**: 2分钟  
> **目的**: 让协作者无需通读整个workspace即可开始工作  
> **配套文档**: `Mike.md` (审计汪提示词)  
> **最后更新**: 2026-02-27

---

## 1. 项目是什么

**HAJIMI V3** 是一个基于 **SimHash-64 + 16分片SQLite** 的本地优先存储系统，支持向量检索和P2P同步。

### 核心创新
- **级联哈希**: SimHash-64 (LSH) + MD5-128 双重校验，合成冲突率 ≤7.98×10⁻³⁹
- **16分片架构**: 基于SimHash高8bit路由，单机支持100K+向量
- **本地优先**: 数据存储在本地SQLite，支持WebRTC P2P同步
- **HNSW向量索引**: Phase 2+ 集成HNSW图索引，支持高性能相似度搜索
- **WASM加速**: Phase 4+ 支持Rust/WASM实现，目标5倍性能提升
- **豪华版限流**: B-01/04~B-04/04 实现SQLite持久化限流+熔断器 (9569 ops/s)

---

## 2. 当前状态（截至2026-02-27）

| 里程碑 | 状态 | 评级 | 关键交付 |
|--------|:--:|:--:|:---------|
| **Phase 1** | ✅ 完成 | A级 | 16分片存储 + 连接池 + Chunk格式 + CRUD API |
| **Phase 2** | ✅ 完成 | A级 | HNSW向量索引集成 + SimHash编码 |
| **Phase 2.1** | ✅ 完成 | A级 | 债务清偿（WAL截断+写入队列+二进制序列化） |
| **Phase 3** | ✅ 完成 | A级 | 磁盘溢出 + HTTP API + 版本迁移 |
| **Phase 4** | ✅ 完成 | A级 | Worker Thread + 磁盘鲁棒性 + WASM框架 |
| **B-01/04** | ✅ 完成 | A级 | LuxurySQLiteRateLimiter (16号审计) |
| **B-02/04** | ✅ 完成 | A级 | BatchWriterOptimized 9569 ops/s (17号审计) |
| **B-03/04** | ✅ 完成 | A级 | RateLimitMiddleware+熔断器 (17号审计) |
| **B-04/04** | ✅ 完成 | A级 | 债务归档+清偿证明 (17号审计) |
| **Task 10** | ⚠️ 部分 | B级 | WASM编译477KB产出，wasm-bindgen-cli待完善 |
| **DEBT-SEC-001** | ✅ 清偿 | - | SQLite持久化限流，18/18测试全绿 |
| **10个任务** | ✅ 已归档 | - | task01-10 全部完成 |

### 关键指标
- 测试用例: 100+ (line覆盖96%+)
- 自测通过率: 100% (48/48刀刃自测)
- 性能基准: 9569 ops/s (WAL批量写入)
- 审计报告: 17份 (00-10, 12-17)
- **Phase 3封顶** ✅ ID-180可冻结

---

## 3. 技术栈速查

```
语言: Node.js (v24+), Rust (WASM)
存储: SQLite (16分片+WAL模式), 二进制HNSW索引
向量: SimHash-64 (LSH), HNSW图索引
限流: Token Bucket + SQLite持久化 + 熔断器
WASM: wasm32-unknown-unknown, wasm-bindgen
Worker: Node.js Worker Threads
同步: WebRTC DataChannel (设计中) / FILE_EXPORT降级
测试: 原生Node.js assert + 自定义runner
```

---

## 4. 目录结构（新）

```
workspace/
├── Agent prompt/              # 【提示词+背景】
│   ├── Mike.md               # 审计汪提示词
│   ├── PROJECT-CONTEXT.md    # 本文件
│   └── memory.md             # 记忆锚点
├── task-audit/               # 【任务输入】01.md, 02.md...
├── audit report/             # 【报告输出】00-17/
│   ├── 00-10/               # Phase 1-4 + Task 10
│   ├── 12/                  # 建设性探索审计 (A级)
│   ├── 13/                  # Phase 1债务清偿审计 (A级)
│   ├── 14/                  # Phase 2安全加固审计 (A级)
│   ├── 15/                  # B-01/04建设性审计 (B级→修复)
│   ├── 16/                  # FIX-001修复验收审计 (A级)
│   └── 17/                  # Phase 3 Final封顶审计 (A级)
├── docs/                     # 📚 项目文档（按任务分类）
│   ├── task01-10/           # Phase 1-4文档
│   ├── task14-luxury-base/  # B-01/04交付物
│   ├── task15-fix/          # FIX-001修复文档
│   ├── task16-batch/        # B-02/04批量优化
│   ├── task17-integration/  # B-03/04业务集成
│   └── task18-debt/         # B-04/04债务归档
├── src/                      # 💻 源代码
│   ├── storage/             # 存储层（BatchWriterOptimized）
│   ├── api/                 # API层
│   ├── security/            # 安全层（限流器+熔断器）
│   ├── vector/              # 向量索引
│   ├── worker/              # Worker线程
│   ├── disk/                # 磁盘管理
│   ├── wasm/                # WASM加载器
│   ├── middleware/          # 中间件（限流+熔断）
│   └── test/                # 测试
├── crates/                   # 🦀 Rust代码
│   └── hajimi-hnsw/         # WASM HNSW核心
├── scripts/                  # 脚本工具
├── archive/                  # 📦 归档任务
└── logs/、temp/、assets/     # 其他
```

---

## 5. 核心模块速查

| 模块 | 路径 | 一句话说明 |
|------|------|-----------|
| **ShardRouter** | `src/storage/shard-router.js` | SimHash→分片00-15路由 |
| **ShardConnectionPool** | `src/storage/connection-pool.js` | 每分片8连接，并发管理 |
| **ChunkStorage** | `src/storage/chunk.js` | .hctx v3格式文件存储 |
| **StorageV3** | `src/api/storage.js` | 统一CRUD: put/get/delete/stats |
| **HNSWIndex** | `src/vector/hnsw-core.js` | JS版HNSW图索引实现 |
| **LuxurySQLiteRateLimiter** | `src/security/rate-limiter-sqlite-luxury.js` | SQLite持久化限流器，18/18测试 |
| **BatchWriterOptimized** | `src/storage/batch-writer-optimized.js` | WAL批量写入，9569 ops/s |
| **RateLimitMiddleware** | `src/middleware/rate-limit-middleware.js` | API限流+熔断器(CLOSED/OPEN/HALF_OPEN) |
| **OverflowManager** | `src/disk/overflow-manager.js` | 内存>180MB自动溢出到磁盘 |
| **ENOSPCHandler** | `src/disk/enospc-handler.js` | 磁盘满优雅降级处理 |

---

## 6. 文档查找（按任务）

| 任务 | 文件夹 | 核心文档 | 状态 |
|:-----|:-------|:---------|:---:|
| **task01-10** | `docs/task01~10/` | Phase 1-4白皮书 | ✅ |
| **B-01/04** | `docs/task14-luxury-base/` | Luxury Base白皮书、自测表 | ✅ A级 |
| **B-02/04** | `docs/task16-batch/` | 批量优化白皮书、自测表 | ✅ A级 |
| **B-03/04** | `docs/task17-integration/` | 业务集成白皮书、自测表 | ✅ A级 |
| **B-04/04** | `docs/task18-debt/` | 债务归档白皮书、自测表 | ✅ A级 |
| **审计报告** | `audit report/12~17/` | 建设性审计报告 | ✅ |

---

## 7. 工作流约定（必读）

### 7.1 任务管理流程

```
1. 创建新任务 → task-audit/XX.md
2. 执行审计  → 我读取任务并输出报告
3. 任务完成  → 归档到 archive/2026/02/tasks/
4. 文档整理  → 放入 docs/taskXX-任务名/
```

### 7.2 审计报告输出

```
task-audit/XX.md          ← 用户放任务文件在这里
         ↓
    我读取并执行审计
         ↓
audit report/XX/XX-AUDIT-XXX-REPORT.md  ← 我输出报告到这里
```

---

## 8. 技术债务（已更新）

| ID | 描述 | 优先级 | 排期 | 状态 |
|:---|:---|:---:|:---:|:---:|
| DEBT-PHASE2-001 | WASM方案（477KB产出，cli待完善） | P1 | - | ⚠️ 部分清偿85% |
| DEBT-SEC-001 | 限流状态持久化（内存→SQLite） | P1 | Phase 3 | ✅ 已清偿 |

> ✅ **DEBT-SEC-001清偿**: 16号审计A级确认，18/18测试全绿，队列优先修复完成

---

## 9. 关键验证命令（复制即用）

```bash
# Phase 1-4 基础测试
node src/test/shard-router.test.js        # 预期: 8/8通过
node src/test/chunk.test.js               # 预期: 7/7通过

# B-01/04 Luxury Base测试
node src/security/rate-limiter.test.js    # 预期: 7/7通过
node tests/luxury-base.test.js            # 预期: 18/18通过

# B-02/04 批量写入压力测试
node tests/batch-writer-stress.test.js    # 预期: 9569+ ops/s

# B-03/04 E2E集成测试
node tests/integration/rate-limit-e2e.test.js  # 预期: 通过

# DEBT-SEC-001清偿验证
node -e "
const {LuxurySQLiteRateLimiter}=require('./src/security/rate-limiter-sqlite-luxury');
const fs=require('fs');
const p='./test.db';
(async()=>{
  const l1=new LuxurySQLiteRateLimiter({dbPath:p});
  await l1.init();
  await l1.saveBucket('1.2.3.4',5,Date.now());
  await l1._flushBatch();
  await l1.close();
  const l2=new LuxurySQLiteRateLimiter({dbPath:p});
  await l2.init();
  console.log('DEBT-SEC-001:',l2.getBucket('1.2.3.4')?'✅清偿':'❌失败');
  fs.unlinkSync(p);
})();
"
```

---

## 10. 常见陷阱

1. **SimHash验证**: `0xFF00n` 不等于 `0xFF00000000000000n`，后者才是有效的64位测试值
2. **Chunk header**: 创建用128字节，解析也必须用128字节（不是60字节）
3. **BigInt处理**: JS中`number`精度只有2^53-1，超大文件必须用`bigint`
4. **分片ID**: `(simhash_hi >> 56n) % 16`，不是直接取高4位
5. **WASM降级**: Termux环境WASM自动降级到JS是预期行为，非错误
6. **限流队列**: `getBucket`优先检查`writeQueue`再读DB，避免数据不一致
7. **熔断器状态**: CLOSED→OPEN→HALF_OPEN→CLOSED，需正确实现状态转换

---

## 11. 快速导航

| 你想做什么 | 看这里 |
|-----------|--------|
| 了解整体架构 | `docs/task04-Phase1分片实现/PHASE1-白皮书-v1.0.md` |
| 查看最新审计 | `audit report/17/17-AUDIT-PHASE3-FINAL-建设性审计报告.md` |
| 查技术债务 | `docs/debt/DEBT-PHASE3-FINAL-CLEARANCE.md` |
| 限流器实现 | `src/security/rate-limiter-sqlite-luxury.js` |
| 批量写入优化 | `src/storage/batch-writer-optimized.js` |
| 熔断器中间件 | `src/middleware/rate-limit-middleware.js` |
| 完整文档索引 | `docs/index.md` |

---

## 12. 一句话总结

> **HAJIMI V3 = SimHash路由的16分片SQLite + Chunk文件存储 + HNSW向量索引 + Worker线程 + 豪华版限流(9569 ops/s) + WASM加速，Phase 1-4及B-01/04~B-04/04全部A级完成，DEBT-SEC-001已清偿，Phase 3封顶ID-180可冻结。**

---

*本文档与 `Mike.md` 配套使用，审计时先读此背景，再执行审计任务。*
