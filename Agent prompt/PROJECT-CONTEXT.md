# HAJIMI V3 项目背景速览

> **阅读时间**: 2分钟  
> **目的**: 让协作者无需通读整个workspace即可开始工作  
> **配套文档**: `Mike.md` (审计汪提示词)  
> **最后更新**: 2026-02-26

---

## 1. 项目是什么

**HAJIMI V3** 是一个基于 **SimHash-64 + 16分片SQLite** 的本地优先存储系统，支持向量检索和P2P同步。

### 核心创新
- **级联哈希**: SimHash-64 (LSH) + MD5-128 双重校验，合成冲突率 ≤7.98×10⁻³⁹
- **16分片架构**: 基于SimHash高8bit路由，单机支持100K+向量
- **本地优先**: 数据存储在本地SQLite，支持WebRTC P2P同步
- **HNSW向量索引**: Phase 2+ 集成HNSW图索引，支持高性能相似度搜索
- **WASM加速**: Phase 4+ 支持Rust/WASM实现，目标5倍性能提升

---

## 2. 当前状态（截至2026-02-26）

| 里程碑 | 状态 | 评级 | 关键交付 |
|--------|:--:|:--:|:---------|
| **Phase 1** | ✅ 完成 | A级 | 16分片存储 + 连接池 + Chunk格式 + CRUD API |
| **Phase 2** | ✅ 完成 | A级 | HNSW向量索引集成 + SimHash编码 |
| **Phase 2.1** | ✅ 完成 | A级 | 债务清偿（WAL截断+写入队列+二进制序列化） |
| **Phase 3** | ✅ 完成 | A级 | 磁盘溢出 + HTTP API + 版本迁移 |
| **Phase 4** | ✅ 完成 | A级 | Worker Thread + 磁盘鲁棒性 + WASM框架 |
| **Task 10** | ⚠️ 部分 | B级 | WASM编译477KB产出，wasm-bindgen-cli待完善 |
| **10个任务** | ✅ 已归档 | - | task01-10 全部完成 |

### 关键指标
- 测试用例: 100+ (line覆盖96%+)
- 自测通过率: 95%+ (22/24刀刃自测)
- P4检查: 10/10 (100%)
- **当前无待处理任务** ✅

---

## 3. 技术栈速查

```
语言: Node.js (v24+), Rust (WASM)
存储: SQLite (16分片), 二进制HNSW索引
向量: SimHash-64 (LSH), HNSW图索引
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
├── audit report/             # 【报告输出】00-10/
│   ├── 00/                  # 早期通用审计
│   ├── 01-05/               # Phase 1相关
│   ├── 06/                  # Phase 2 HNSW
│   ├── 07/                  # Phase 2.1 债务清偿
│   ├── 08/                  # Phase 3 WASM+API
│   ├── 09/                  # Phase 4 Worker+磁盘
│   └── 10/                  # Task 10 WASM编译
├── docs/                     # 📚 项目文档（按任务分类）
│   ├── task01-架构设计/
│   ├── task02-技术债务清偿/
│   ├── task03-基线修复/
│   ├── task04-Phase1分片实现/
│   ├── task05-Phase1修复/
│   ├── task06-phase2-hnsw/
│   ├── task07-phase2.1-debt-clearance/
│   ├── task08-phase3-wasm-disk-api/
│   ├── task09-phase4-wasm-worker-robust/
│   └── task10-wasm-compile/
├── src/                      # 💻 源代码
│   ├── storage/             # 存储层（路由/连接池/Chunk）
│   ├── api/                 # API层（HTTP服务器/路由）
│   ├── vector/              # 向量索引（HNSW核心/Hybrid索引）
│   ├── worker/              # Worker线程（池/桥接）
│   ├── disk/                # 磁盘管理（溢出/缓存/ENOSPC）
│   ├── wasm/                # WASM加载器
│   ├── migration/           # 版本迁移
│   └── test/                # 测试
├── crates/                   # 🦀 Rust代码
│   └── hajimi-hnsw/         # WASM HNSW核心
│       ├── src/lib.rs       # 193行Rust代码
│       └── pkg/             # 477KB WASM产出
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
| **ChunkStorage** | `src/storage/chunk.js` | .hctx v3格式文件存储(128字节header+SHA256) |
| **StorageV3** | `src/api/storage.js` | 统一CRUD: put/get/delete/stats |
| **SimHash-64** | `src/utils/simhash64.js` | 生产级实现，汉明距离峰值32 |
| **HNSWIndex** | `src/vector/hnsw-core.js` | JS版HNSW图索引实现 |
| **HybridHNSWIndex** | `src/vector/hnsw-index-hybrid.js` | WASM/JS自动降级混合索引 |
| **IndexBuilderBridge** | `src/worker/index-builder-bridge.js` | Worker↔主线程索引构建桥接 |
| **OverflowManager** | `src/disk/overflow-manager.js` | 内存>180MB自动溢出到磁盘 |
| **ENOSPCHandler** | `src/disk/enospc-handler.js` | 磁盘满优雅降级处理 |
| **Migrator** | `src/migration/migrator.js` | V0→V1→V2版本迁移协调器 |

---

## 6. 文档查找（按任务）

| 任务 | 文件夹 | 核心文档 |
|:-----|:-------|:---------|
| **task01** | `docs/task01-架构设计/` | local-storage-v3-design.md |
| **task02** | `docs/task02-技术债务清偿/` | 债务清偿白皮书、6项债务修复 |
| **task03** | `docs/task03-基线修复/` | LSH修复、WebRTC实现、测试统一 |
| **task04** | `docs/task04-Phase1分片实现/` | Phase1白皮书、5个模块设计 |
| **task05** | `docs/task05-Phase1修复/` | 修复报告、债务v1.1 |
| **task06** | `docs/task06-phase2-hnsw/` | Phase2 HNSW白皮书、自测表 |
| **task07** | `docs/task07-phase2.1-debt-clearance/` | Phase2.1债务清偿白皮书 |
| **task08** | `docs/task08-phase3-wasm-disk-api/` | Phase3 WASM+API白皮书 |
| **task09** | `docs/task09-phase4-wasm-worker-robust/` | Phase4 Worker+磁盘白皮书 |
| **task10** | `docs/task10-wasm-compile/` | WASM编译白皮书、477KB产出 |

---

## 7. 工作流约定（必读）

### 7.1 任务管理流程

```
1. 创建新任务 → task/task-XXX.md
2. 执行审计  → 我读取任务并输出报告
3. 任务完成  → 归档到 archive/2026/02/tasks/
4. 文档整理  → 放入 docs/taskXX-任务名/
```

### 7.2 审计报告输出

```
task/task-XXX.md          ← 用户放任务文件在这里
         ↓
    我读取并执行审计
         ↓
audit report/XX/XX-AUDIT-XXX-REPORT.md  ← 我输出报告到这里
```

### 7.3 任务文件格式

```markdown
项目代号：HAJIMI-XXX

[CODE]
## file: src/xxx.js
```js
// 代码内容，或写"见文件 /src/xxx.js"
```
[/CODE]

[ARCH]
- 关键约束：...
- 数据流：...
[/ARCH]

[PRD]
- 目标：...
- 范围：...
[/PRD]

【重点关注】
- 性能/并发/一致性等特定风险点
```

---

## 8. 技术债务（已知）

| ID | 描述 | 优先级 | 排期 | 状态 |
|:---|:---|:---:|:---:|:---:|
| DEBT-PHASE2-001 | WASM方案（477KB产出，cli待完善） | P1 | - | ⚠️ 部分清偿85% |
| DEBT-PHASE2-002 | SimHash→Dense编码损失 | P1 | - | ✅ 已缓解 |
| DEBT-PHASE2-003 | Termux内存限制（磁盘溢出已缓解） | P0-if | - | ✅ 已缓解 |
| DEBT-PHASE2-004 | Worker Thread（已实现，Termux路径问题已知） | P2 | - | ✅ 已清偿 |

> ⚠️ **审计时注意**: 检查是否引入新的P0/P1债务

---

## 9. 关键验证命令（复制即用）

```bash
# Phase 1 基础测试
node src/test/shard-router.test.js        # 预期: 8/8通过
node src/test/chunk.test.js               # 预期: 7/7通过
node src/test/storage-integration.test.js # 预期: 6/6通过

# Phase 2 HNSW测试
node src/cli/vector-debug.js test         # 预期: 5/5通过
node src/cli/vector-debug.js benchmark    # 预期: 构建<30s

# Phase 2.1 债务清偿验证
node src/test/debt-clearance-validator.js # 预期: 3/3清偿

# Phase 3/4 E2E测试
node tests/e2e/wasm-disk-api.test.js      # 预期: 3/4通过
node tests/e2e/phase4-integration.test.js # 预期: 3/4通过

# WASM降级验证
node -e "const {HybridHNSWIndex}=require('./src/vector/hnsw-index-hybrid'); new HybridHNSWIndex({dimension:128}).init().then(i=>console.log('Mode:',i.getMode()))"
# 预期: Mode: javascript (WASM待cli时自动降级)

# 初始化16分片
node src/storage/migrate.js --init
ls ~/.hajimi/storage/v3/meta/shard_*.db | wc -l  # 预期: 16
```

---

## 10. 常见陷阱

1. **SimHash验证**: `0xFF00n` 不等于 `0xFF00000000000000n`，后者才是有效的64位测试值
2. **Chunk header**: 创建用128字节，解析也必须用128字节（不是60字节）
3. **BigInt处理**: JS中`number`精度只有2^53-1，超大文件必须用`bigint`
4. **分片ID**: `(simhash_hi >> 56n) % 16`，不是直接取高4位
5. **WASM降级**: Termux环境WASM自动降级到JS是预期行为，非错误
6. **Worker路径**: Termux中Worker Threads有路径解析限制，使用fallback机制

---

## 11. 快速导航

| 你想做什么 | 看这里 |
|-----------|--------|
| 了解整体架构 | `docs/task04-Phase1分片实现/PHASE1-白皮书-v1.0.md` |
| 查看HNSW实现 | `docs/task06-phase2-hnsw/HAJIMI-PHASE2-HNSW-白皮书-v1.0.md` |
| 查看最新审计 | `audit report/10/10-AUDIT-WASM-COMPILE.md` |
| 查技术债务 | `docs/task02-技术债务清偿/DEBT-CLEARANCE-001-白皮书-v1.0.md` |
| 执行测试 | `scripts/run-debt-tests.sh` |
| 了解级联哈希 | `docs/task03-基线修复/DEBT-LSH-001-FIXED.md` |
| 新任务模板 | `task/` (参考README.md中的任务管理章节) |
| 完整文档索引 | `docs/index.md` |

---

## 12. 一句话总结

> **HAJIMI V3 = SimHash路由的16分片SQLite + Chunk文件存储 + HNSW向量索引 + Worker线程 + WASM加速（部分完成），Phase 1-4已完成（A级），10个任务已归档，当前待命状态。**

---

*本文档与 `Mike.md` 配套使用，审计时先读此背景，再执行审计任务。*
