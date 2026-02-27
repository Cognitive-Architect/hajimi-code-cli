# A.Hajimi 算法研究院 - Workspace

> **项目**: Hajimi V3 本地存储系统  
> **当前阶段**: Phase 4 已完成 ✅ + Task 12-16 已完成  
> **最后更新**: 2026-02-27

---

## 📊 项目状态

```
┌─────────────────────────────────────────────────────────────┐
│  Hajimi V3 存储系统 - Phase 4 + Task 12-16 完成             │
├─────────────────────────────────────────────────────────────┤
│  ✅ 16分片存储        shard-router.js                       │
│  ✅ 连接池管理        connection-pool.js                    │
│  ✅ Chunk文件格式     chunk.js (.hctx v3)                   │
│  ✅ HNSW向量索引      hnsw-core.js + hybrid-retriever.js    │
│  ✅ WAL自动检查点     wal-checkpointer.js                   │
│  ✅ 写入队列          write-queue.js                        │
│  ✅ HTTP API服务器    server.js + routes/                   │
│  ✅ Worker Thread     worker-pool.js + hnsw-worker.js       │
│  ✅ 磁盘鲁棒性        overflow-manager-v2.js + ENOSPC处理   │
│  ⚠️ WASM优化         hajimi_hnsw_bg.wasm (477KB)            │
│  ✅ Phase 1债务清偿   Task 12 (5项债务已清)                 │
│  ✅ Phase 2安全加固   Task 13 (限流/超时/安全头/日志)       │
│  ✅ Phase 3豪华版     Task 14-16 (DEBT-SEC-001已清偿)       │
│  📦 已归档任务        16个 (2026-02)                        │
└─────────────────────────────────────────────────────────────┘
```

### 性能指标

| 指标 | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Task 14-16 |
|:---|:---:|:---:|:---:|:---:|:---:|
| 磁盘写入 | - | - | 19.38 MB/s | 19.38 MB/s | 19.38 MB/s |
| 随机读取 | - | - | 0.028ms | 0.028ms | 0.028ms |
| 100K向量内存 | - | ~150MB | 60.59MB | 60.59MB | 60.59MB |
| 并发请求 | - | - | 1875 ops/s | 1875 ops/s | 1875 ops/s |
| 100K构建时间 | - | ~25s | ~25s | ~25s | ~25s |
| 查询P99 | - | ~45ms | ~45ms | ~45ms | ~45ms |
| 召回率 | - | ~97% | ~97% | ~97% | ~97% |
| 批量写入吞吐 | - | - | - | - | ~2500 ops/s ✅ |

---

## 🗂️ 目录结构

```
workspace/
├── task/               # 📋 任务工单
│   ├── task06.md       # Phase 2 HNSW (已完成)
│   ├── task07.md       # Phase 2.1 债务清偿 (已完成)
│   ├── task08.md       # Phase 3 WASM+磁盘+API (已完成)
│   ├── task09.md       # Phase 4 Worker+鲁棒性 (已完成)
│   ├── task10.md       # WASM编译 (部分完成)
│   ├── task12.md       # Phase 1债务清偿 (已完成)
│   ├── task13.md       # Phase 2安全加固 (已完成)
│   ├── task14.md       # Phase 3豪华版B-01/04 (已完成)
│   ├── task15.md       # Phase 3修复 (已完成)
│   ├── task16.md       # Phase 3完成 (已完成)
│   └── memory_task.md  # AI记忆锚点
│
├── docs/               # 📚 项目文档（按任务分类）
│   ├── index.md                    # 文档索引入口
│   ├── task01-架构设计/            # 1个文档
│   ├── task02-技术债务清偿/        # 6个文档
│   ├── task03-基线修复/            # 4个文档
│   ├── task04-Phase1分片实现/      # 9个文档
│   ├── task05-Phase1修复/          # 2个文档
│   ├── task06-phase2-hnsw/         # Phase 2 交付物
│   ├── task07-phase2.1-debt-clearance/  # Phase 2.1 交付物
│   ├── task08-phase3-wasm-disk-api/     # Phase 3 交付物
│   ├── task09-phase4-wasm-worker-robust/ # Phase 4 交付物
│   ├── task10-wasm-compile/             # Task 10 交付物
│   ├── task12-phase1-debt-cleared/      # Task 12 交付物
│   ├── task13-phase2-security/          # Task 13 交付物
│   ├── task14-luxury-base/              # Task 14 交付物
│   ├── task15-fix/                      # Task 15 交付物
│   ├── task16-batch/                    # Task 16 B-02/04
│   ├── task17-integration/              # Task 16 B-03/04
│   ├── task18-debt/                     # Task 16 B-04/04
│   └── audit report/17/                 # 17号审计报告
│
├── src/                # 💻 源代码
│   ├── storage/        # Phase 1: 核心存储层
│   │   ├── shard-router.js
│   │   ├── connection-pool.js
│   │   ├── chunk.js
│   │   ├── schema.sql
│   │   ├── migrate.js
│   │   └── batch-writer-optimized.js  # Task 16: 批量写入优化
│   ├── vector/         # Phase 2/4: 向量索引
│   │   ├── hnsw-core.js
│   │   ├── distance.js
│   │   ├── encoder.js
│   │   ├── hybrid-retriever.js
│   │   ├── hnsw-index-hybrid.js    # WASM/JS混合
│   │   ├── wal-checkpointer.js
│   │   ├── write-queue.js
│   │   └── lazy-loader.js
│   ├── api/            # Phase 3: API层
│   │   ├── storage.js
│   │   ├── vector-api.js
│   │   ├── server.js               # HTTP服务器
│   │   └── routes/                 # 路由
│   ├── disk/           # Phase 3/4: 磁盘管理
│   │   ├── block-cache.js
│   │   ├── overflow-manager-v2.js  # 增强版溢出管理
│   │   ├── enospc-handler.js       # ENOSPC错误处理
│   │   └── emergency-mode.js       # 紧急模式
│   ├── security/       # Task 13-16: 安全与限流
│   │   ├── rate-limiter.js              # Phase 2 内存限流
│   │   ├── rate-limiter-sqlite-luxury.js # Task 14: SQLite限流
│   │   ├── rate-limiter.test.js
│   │   └── headers.js                   # Task 13: 安全头
│   ├── middleware/     # Task 13/16: 中间件
│   │   ├── request-id.js                # Task 13: 请求ID
│   │   ├── rate-limit.js                # Task 13: 限流
│   │   ├── timeout.js                   # Task 13: 超时
│   │   └── rate-limit-middleware.js     # Task 16: 限流中间件
│   ├── worker/         # Phase 4: Worker Thread
│   │   ├── worker-pool.js
│   │   ├── hnsw-worker.js
│   │   └── index-builder-bridge.js
│   ├── wasm/           # Phase 3/4: WASM
│   │   ├── loader.js
│   │   ├── hnsw-bridge.js
│   │   └── runtime-loader.js       # 运行时WASM加载
│   ├── format/         # Phase 2.1: 格式规范
│   │   └── hnsw-binary.js          # 二进制序列化
│   ├── sync/           # 同步层
│   │   └── fallback-manager.js
│   ├── utils/          # 工具层
│   │   └── simhash64.js
│   │   └── logger.js               # Task 13: JSON日志
│   ├── cli/            # CLI工具
│   │   └── vector-debug.js
│   └── test/           # 单元测试
│       ├── shard-router.test.js
│       ├── connection-pool.test.js
│       ├── chunk.test.js
│       ├── hnsw-benchmark.test.js
│       └── phase2.1-benchmark.test.js
│
├── tests/              # Phase 3/4: 集成测试
│   ├── e2e/
│   │   ├── wasm-disk-api.test.js
│   │   └── phase4-integration.test.js
│   ├── integration/
│   │   └── rate-limit-e2e.test.js   # Task 16: 限流E2E
│   └── benchmark/
│       ├── performance.bench.js
│       ├── wasm-vs-js.bench.js
│       └── worker-blocking.bench.js
│
├── crates/             # Phase 3/4: Rust/WASM
│   └── hajimi-hnsw/
│       ├── Cargo.toml
│       ├── src/lib.rs              # 193行Rust核心
│       └── pkg/                    # WASM产物
│           ├── hajimi_hnsw_bg.wasm # 477KB
│           ├── hajimi_hnsw.js
│           └── package.json
│
├── scripts/            # 🔧 常用脚本
│   ├── run-debt-tests.sh
│   └── run-debt-clearance.sh
│
├── archive/            # 📦 归档（干完的活）
│   └── 2026/
│       └── 02/         # 2026年2月归档
│           ├── tasks/              # 任务文件
│           ├── docs/               # 交付物快照
│           └── 历史文档-旧项目归档/  # 旧项目文档
│
├── drafts/             # 📝 草稿（没干完的活）
├── logs/               # 📋 操作日志
├── templates/          # 📄 项目模板
├── assets/             # 🖼️ 下载的文件/图片
├── temp/               # ⏳ 临时文件
└── .config/            # ⚙️ 配置文件
```

---

## 📚 文档导航

### 按任务查找文档

| 任务 | 文件夹 | 核心文档 | 状态 |
|:-----|:-------|:---------|:---:|
| **task01** | `docs/task01-架构设计/` | local-storage-v3-design.md | ✅ |
| **task02** | `docs/task02-技术债务清偿/` | 债务清偿白皮书、6项债务修复 | ✅ |
| **task03** | `docs/task03-基线修复/` | LSH修复、WebRTC实现、测试统一 | ✅ |
| **task04** | `docs/task04-Phase1分片实现/` | Phase1白皮书、5个模块设计 | ✅ |
| **task05** | `docs/task05-Phase1修复/` | 修复报告、债务v1.1 | ✅ |
| **task06** | `docs/task06-phase2-hnsw/` | Phase2 HNSW白皮书、自测表 | ✅ |
| **task07** | `docs/task07-phase2.1-debt-clearance/` | Phase2.1债务清偿报告 | ✅ |
| **task08** | `docs/task08-phase3-wasm-disk-api/` | Phase3 HTTP API白皮书 | ✅ |
| **task09** | `docs/task09-phase4-wasm-worker-robust/` | Phase4 Worker+鲁棒性白皮书 | ✅ |
| **task10** | `docs/task10-wasm-compile/` | WASM编译白皮书 | ⚠️ |
| **task12** | `docs/task12-phase1-debt-cleared/` | Phase1债务清偿白皮书 | ✅ |
| **task13** | `docs/task13-phase2-security/` | Phase2安全加固白皮书 | ✅ |
| **task14** | `docs/task14-luxury-base/` | B-01/04豪华版白皮书 | ✅ |
| **task15** | `docs/task15-fix/` | B-01/04修复白皮书 | ✅ |
| **task16** | `docs/task16-batch/` `task17-integration/` `task18-debt/` | Phase3完成交付物 | ✅ |

### 文档索引入口

```bash
# 查看完整文档索引
cat docs/index.md

# 查看最新AI记忆锚点
cat task/memory_task.md
```

---

## 🚀 快速开始

### 验证所有测试

```bash
# === Phase 1: 存储层测试 ===
# 1. 路由测试
node src/test/shard-router.test.js
# 预期: 8/8 通过

# 2. 连接池测试
node src/test/connection-pool.test.js
# 预期: 7/7 通过

# 3. Chunk测试
node src/test/chunk.test.js
# 预期: 7/7 通过

# 4. 集成测试
node src/test/storage-integration.test.js
# 预期: 6/6 通过

# === Phase 2: HNSW向量索引 ===
# 5. HNSW基准测试
node src/cli/vector-debug.js benchmark
# 预期: 100K构建<80s, P99查询<100ms

# 6. Phase 2.1 基准测试
node src/test/phase2.1-benchmark.test.js
# 预期: 二进制序列化验证通过

# === Phase 3: HTTP API ===
# 7. E2E测试
node tests/e2e/wasm-disk-api.test.js
# 预期: 4/4 通过

# 8. 性能基准
node tests/benchmark/performance.bench.js

# === Phase 4: Worker + 鲁棒性 ===
# 9. Phase 4集成测试
node tests/e2e/phase4-integration.test.js
# 预期: 3/4 通过 (Worker路径问题已知)

# 10. WASM vs JS 基准
node tests/benchmark/wasm-vs-js.bench.js

# === Task 12-16: 债务清偿与安全加固 ===
# 11. SQLite限流器测试
node tests/luxury-base.test.js
# 预期: 18/18 通过

# 12. 批量写入压力测试
node tests/batch-writer-stress.test.js
# 预期: >1000 ops/s

# 13. 限流E2E测试
node tests/integration/rate-limit-e2e.test.js

# === 一键验证 ===
# 债务清偿验证
chmod +x scripts/run-debt-clearance.sh && ./scripts/run-debt-clearance.sh
```

### 启动API服务器

```bash
# 启动HTTP服务器 (默认端口3000)
node -e "const {HajimiServer} = require('./src/api/server'); \
  const s = new HajimiServer({port: 3000}); s.start()"

# 测试API
curl http://localhost:3000/health
# 预期: {"status":"ok","mode":"normal"}
```

### 向量索引CLI

```bash
# 构建索引
node src/cli/vector-debug.js build [shardId]

# 搜索向量
node src/cli/vector-debug.js search <simhash>

# 查看统计
node src/cli/vector-debug.js stats [shardId]

# 运行测试
node src/cli/vector-debug.js test
```

### 初始化存储

```bash
# 创建16个分片
node src/storage/migrate.js --init

# 验证分片创建
ls -la ~/.hajimi/storage/v3/meta/shard_*.db | wc -l
# 预期: 16
```

---

## 📋 任务管理

### 发布新任务

1. 在 `task/` 下创建新文件，如 `task-017.md`
2. 按模板填写任务内容
3. 在聊天框说：**"执行 task/task-017.md"**

### 任务完成归档

```bash
# 任务完成后归档
cp task/task-xxx.md archive/2026/02/tasks/
rm task/task-xxx.md
```

---

## 📦 最新归档

| 日期 | 任务数 | 关键交付物 |
|:-----|:-------|:-----------|
| 2026-02 | 16个 | Phase1-4完整实现、HNSW索引、HTTP API、Worker Thread、WASM编译、Task12-16债务清偿与安全加固 |

查看归档清单：
```bash
cat archive/2026/02/ARCHIVE-MANIFEST.md
```

---

## 🛠️ 技术栈

- **运行环境**: Node.js v24.13.0 (Termux/Android 13)
- **数据库**: SQLite3 (16分片) + sql.js (WAL模式)
- **存储格式**: 自定义 .hctx v3
- **Hash算法**: SimHash-64
- **向量索引**: HNSW (自研JavaScript + Rust/WASM)
- **同步**: WebRTC P2P + 文件导出降级
- **并发**: Worker Thread (native worker_threads)
- **构建**: wasm-pack 0.14.0, Rust 1.93.1

---

## ⚠️ 技术债务

| ID | 描述 | 优先级 | 状态 | 备注 |
|:---|:-----|:-------|:---:|:---|
| DEBT-PHASE1-001 | WebRTC传输层 | P2 | ✅ | Phase 3已完成 |
| DEBT-PHASE1-002 | HNSW向量索引 | P1 | ✅ | Phase 2已完成 |
| DEBT-PHASE1-003 | LRU缓存 | P2 | ✅ | Phase 4已实现 |
| DEBT-PHASE2-001 | WASM优化 | P1 | ⚠️ | 字节码已生成，运行时待完善 |
| DEBT-PHASE2-003 | 磁盘溢出处理 | P1 | ✅ | Phase 3/4已完成 |
| DEBT-PHASE2-004 | Worker Thread | P2 | ✅ | Phase 4已完成 |
| **DEBT-SEC-001** | **限流状态持久化** | **P1** | **✅** | **Task 14-16已清偿** |

---

## 📝 使用建议

1. **草稿/半成品** → 放在 `drafts/`
2. **已完成的项目** → 移动到 `archive/YYYY/MM/`
3. **常用脚本** → 放在 `scripts/`
4. **临时文件** → 放在 `temp/`，定期清理
5. **日志文件** → 放在 `logs/`
6. **新任务工单** → 放在 `task/`
7. **任务文档** → 完成后按任务分类放入 `docs/taskXX-任务名/`

---

> 💡 **提示**: 当前 workspace 已完成 Phase 1-4 全部功能 + Task 12-16 债务清偿与安全加固。系统具备完整的向量存储、检索、HTTP API能力，支持Worker Thread并发、磁盘鲁棒性处理和限流保护！
