# HAJIMI V3 贡献指南

> **目标**: 帮助开发者快速理解代码结构并参与开发  
> **适用对象**: 核心开发者、审计员、功能扩展者

---

## 🚀 快速开始

### 环境要求
```bash
# Node.js
node --version  # >= 18.x

# Rust
cargo --version  # >= 1.75

# 可选
redis-server --version  # 用于限流测试
foundry --version       # 用于 EVM 测试
```

### 安装依赖
```bash
# Node.js 依赖
npm ci

# Rust 依赖
cargo fetch
```

### 运行测试
```bash
# TypeScript 编译检查
npx tsc --noEmit

# Rust 编译检查
cargo check --manifest-path src/chimera/chimera-repl/Cargo.toml

# 单元测试
npm run test:unit

# P2P 测试
node src/tests/webrtc-handshake.e2e.js
```

---

## 📂 代码组织原则

### 目录命名规范
| 类型 | 命名 | 示例 |
|------|------|------|
| 核心模块 | 名词单数 | `storage`, `vector` |
| 适配器 | `*-adapter` | `foundry-adapter.ts` |
| 管理器 | `*-manager` | `datachannel-manager.js` |
| 接口 | `*-interface.ts` | `queue-db-interface.ts` |
| 工具 | `*-utils.ts` | `logger.js` |

### 文件命名规范
| 类型 | 命名 | 示例 |
|------|------|------|
| 实现文件 | 小写+连字符 | `shard-router.js` |
| 类型定义 | 大驼峰 | `ICrdtEngine.ts` |
| 测试文件 | `*.test.js` | `chunk.test.js` |
| E2E 测试 | `*.e2e.js` | `webrtc-handshake.e2e.js` |
| 基准测试 | `*.bench.js` | `sab-overhead.bench.js` |

---

## 🎯 开发工作流

### 1. 添加新功能

**步骤 1: 确定位置**
```
新功能类型 → 对应目录
├── 存储相关 → storage/
├── P2P 相关 → p2p/
├── AI/LLM 相关 → chimera/ 或 crates/hajimi-codex-twist/
├── 向量检索 → vector/ 或 crates/hajimi-hnsw/
├── 安全相关 → security/
└── 外部适配 → adapters/
```

**步骤 2: 实现接口**
```typescript
// 1. 定义接口
interface MyFeature {
    doSomething(): Promise<Result>;
}

// 2. 实现类
export class MyFeatureImpl implements MyFeature {
    async doSomething(): Promise<Result> {
        // 实现
    }
}

// 3. 单元测试
// my-feature.test.ts
describe('MyFeature', () => {
    it('should do something', async () => {
        const feature = new MyFeatureImpl();
        const result = await feature.doSomething();
        expect(result).toBeDefined();
    });
});
```

**步骤 3: 更新索引**
- 修改 `src/INDEX.md` 添加新模块描述
- 如有架构变更，更新 `src/ARCHITECTURE.md`

### 2. 修改现有代码

**检查清单**:
- [ ] 是否影响 `chimera-repl/src/lib.rs` 冻结行数（≤70行）
- [ ] 是否更新相关测试
- [ ] 是否更新文档注释
- [ ] 是否通过 `cargo check` 或 `tsc --noEmit`

**提交规范**:
```bash
# 格式: <type>(<scope>): <description>
# 示例:
feat(storage): add Redis cluster support
fix(p2p): resolve WebRTC connection leak
docs(vector): update HNSW algorithm description
refactor(chimera): extract InputSource trait
```

### 3. 审计流程

**触发条件**:
- 新模块超过 100 行
- 修改核心接口
- 新增外部依赖

**审计步骤**:
1. 工程师编写自测报告：`docs/self-audit/XX-ENGINEER-SELF-AUDIT.md`
2. 审计员阅读 `Agent prompt/Mike.md` 了解规范
3. 审计员执行审计，输出到 `audit report/XX/XX-AUDIT-XXX.md`
4. 评级：S/A/B/C/D

---

## 🔍 代码阅读指南

### 按角色阅读

**新开发者**:
1. 阅读 `src/INDEX.md` - 了解全貌
2. 阅读 `src/ARCHITECTURE.md` - 理解设计
3. 从 `storage/shard-router.js` 开始 - 最简单核心逻辑
4. 看 `vector/hnsw-core.js` - 了解向量检索
5. 看 `p2p/signaling-server.js` - 了解 P2P

**审计员**:
1. 必读 `Agent prompt/Mike.md` - 审计规范
2. 必读 `Agent prompt/PROJECT-CONTEXT.md` - 项目背景
3. 查看 `src/INDEX.md` - 快速定位模块
4. 重点关注：
   - `security/rate-limiter-*.js` - 安全策略
   - `p2p/datachannel-manager.js` - 传输安全
   - `chimera/chimera-repl/src/lib.rs` - 行数合规

**性能优化者**:
1. 查看 `vector/hnsw-*.js` - 向量检索性能
2. 查看 `storage/batch-writer-*.js` - 写入性能
3. 查看 `wasm/` - WASM 边界跨越
4. 查看 `worker/` - 线程池利用率

### 关键代码路径

**Chunk 写入**:
```
api/storage.js → storage/chunk.js → storage/shard-router.js 
    → storage/connection-pool.js → SQLite WAL
    → vector/hnsw-index-wasm-v3.js → WASM HNSW
```

**P2P 同步**:
```
p2p/sync-engine.ts → p2p/crdt-engine.ts → Yjs
    → p2p/datachannel-manager.js → WebRTC
    → p2p/signaling-server.js (if needed)
```

**AI 对话**:
```
chimera/src/repl.rs → chimera/src/codex_bridge.rs
    → crates/hajimi-codex-twist/src/thread.rs
    → crates/hajimi-codex-twist/src/memory/memory_gateway.rs
```

---

## 🐛 调试技巧

### Rust 调试
```bash
# 编译检查
cd src/chimera/chimera-repl
cargo check

# 详细错误
cargo check 2>&1 | head -50

# 格式化代码
cargo fmt

# 运行测试
cargo test
```

### Node.js 调试
```bash
# 内存分析
node --max-old-space-size=512 --inspect src/tests/xxx.test.js

# 性能分析
npx clinic flame -- node src/tests/bench/xxx.bench.js

# 网络调试
DEBUG=* node src/p2p/signaling-server.js
```

### 常见问题

**问题 1**: `Error: Cannot find module './codex-twist'`
- **解决**: 这是预期行为，项目使用本地 `hajimi-codex-twist` 替代

**问题 2**: `cargo check` 失败，找不到 `codex-protocol`
- **解决**: 已修复，使用 `codex_twist::thread::ThreadId` 替代

**问题 3**: TypeScript 编译错误
- **解决**: `npm run test:unit` 先确保基础功能正常

---

## 📈 性能优化指南

### 1. 向量检索优化
```javascript
// 使用批量搜索（减少 WASM 边界跨越）
const results = index.searchBatch(queries, queryCount, k);

// 使用零拷贝（避免内存分配）
const results = index.searchBatchZeroCopy(float32Array, dim, k);
```

### 2. 存储优化
```javascript
// 使用批量写入
await batchWriter.write(chunks);

// 使用 WAL 模式
const db = new SQLite(':memory:', { wal: true });
```

### 3. P2P 优化
```javascript
// 使用 State Vector 差量同步
const diff = Y.encodeStateVector(doc);

// 使用 64KB 分片
const chunks = chunkData(data, 64 * 1024);
```

---

## 📝 文档维护

### 必须同步更新的文档
| 变更类型 | 更新文档 |
|----------|----------|
| 新增模块 | `src/INDEX.md` |
| 架构变更 | `src/ARCHITECTURE.md` |
| 接口变更 | 相关接口注释 + `src/INDEX.md` |
| 性能数据 | `src/INDEX.md` 性能表格 |
| 审计报告 | `audit report/XX/` |

### 文档模板

**模块描述模板**:
```markdown
### module-name/
**技术栈**: TypeScript/Rust  
**代码规模**: ~XXX行  
**状态**: 稳定/开发中/已废弃

| 文件 | 功能 |
|------|------|
| `file.ts` | 一句话描述 |

**关键特性**:
- 特性 1
- 特性 2
```

---

## 🎓 学习资源

### 核心技术
- **SimHash**: [Google SimHash 论文](https://static.googleusercontent.com/media/research.google.com/zh-CN//pubs/archive/33026.pdf)
- **HNSW**: [Efficient and robust approximate nearest neighbor search using Hierarchical Navigable Small World graphs](https://arxiv.org/abs/1603.09320)
- **Yjs CRDT**: [Yjs Documentation](https://docs.yjs.dev/)
- **WebRTC**: [WebRTC for the Curious](https://webrtcforthecurious.com/)
- **MCP**: [Model Context Protocol Spec](https://modelcontextprotocol.io/)

### 项目特定
- **HAJIMI V3 设计**: 见 `docs/deepresearch/`
- **审计历史**: 见 `audit report/`
- **任务工单**: 见 `task-audit/`

---

*本指南与代码同步维护，最后更新于 2026-04-02*
