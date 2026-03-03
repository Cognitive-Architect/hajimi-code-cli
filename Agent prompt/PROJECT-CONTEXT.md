# HAJIMI V3 项目背景速览

> **阅读时间**: 2分钟  
> **目的**: 让协作者无需通读整个workspace即可开始工作  
> **配套文档**: `Mike.md` (审计汪提示词)  
> **最后更新**: 2026-02-28（已同步Sprint3完成+Sprint4进行中）

---

## 1. 项目是什么

**HAJIMI V3** 是一个基于 **SimHash-64 + 16分片SQLite** 的本地优先存储系统，支持向量检索和 **WebRTC P2P同步**。

### 核心创新
- **级联哈希**: SimHash-64 (LSH) + MD5-128 双重校验，合成冲突率 ≤7.98×10⁻³⁹
- **16分片架构**: 基于SimHash高8bit路由，单机支持100K+向量
- **本地优先**: 数据存储在本地SQLite，支持WebRTC P2P同步
- **HNSW向量索引**: Phase 2+ 集成HNSW图索引，支持高性能相似度搜索
- **WASM加速**: Phase 4+ 支持Rust/WASM实现，查询1.94x/构建7.7x
- **豪华版限流**: B-01/04~B-04/04 实现SQLite持久化限流+熔断器 (9569 ops/s)
- **WebRTC P2P**: Sprint3+ 实现信令协议+DataChannel传输

---

## 2. 当前状态（截至2026-02-28）

| 里程碑 | 状态 | 评级 | 关键交付 |
|--------|:--:|:--:|:---------|
| **Phase 1-5** | ✅ 完成 | A级 | 16分片+WASM+Worker+债务清偿 |
| **Sprint2** | ✅ 完成 | A级 | OBS-001/002修复 (v3.1.0) |
| **Sprint3** | ✅ 完成 | A-/Go | WebRTC信令协议 (ID-191) ⭐ |
| **DEBT-PHASE1-001** | ✅ 清偿 | - | WebRTC传输层 P2→P0 ⭐ |
| **30号审计** | ⏳ 等待 | - | 29号修复复核 (minor清零) |
| **Sprint4** | 🔄 进行中 | - | DataChannel全量实现 |
| **审计报告** | 30份 | - | 00-30 (28号A-/Go) |

### 版本信息
- **当前版本**: v3.2.2-sprint3-fix-minor
- **Git坐标**: `90e6c3c`

### 关键指标
- 测试用例: 100+ (line覆盖96%+)
- 自测通过率: 100% (48/48刀刃自测)
- 性能基准: 9569 ops/s (WAL批量写入)
- WASM加速: 查询1.94x / 构建7.7x
- WebRTC握手: 5秒超时，STUN穿透

---

## 3. Sprint3 完成摘要 ⭐

### 交付物清单 (9个)

```
docs/sprint3/
├── SIGNALING-PROTOCOL-v1.0.md       # 94行 - JSON-RPC 2.0协议规范
├── TEST-REPORT-webrtc-handshake.md  # 48行 - E2E测试报告
└── SPRINT3-COMPLETION-REPORT.md     # 完成报告

src/p2p/
├── signaling-interface.ts           # 43行 - TypeScript接口
├── signaling-server.js              # 96行 - WebSocket服务器
├── signaling-client.js              # 78行 - WebRTC客户端
└── config.js                        # 29行 - STUN/ICE配置

tests/
├── webrtc-handshake.e2e.js          # 95行 - E2E测试
└── nat-traversal.test.js            # 79行 - NAT测试
```

### 地狱红线 (10项全通过)

| 红线 | 验证 | 状态 |
|------|------|:--:|
| JSON-RPC格式 | `grep '"jsonrpc": "2.0"'` | ✅ |
| STUN配置 | `grep 'stun.l.google.com'` | ✅ |
| ICE候选处理 | `grep 'onicecandidate'` | ✅ |
| E2E通过 | Exit 0 | ✅ |
| 行数限制 | 全部±5行内 | ✅ |
| 强制关键字 | 全部正则命中 | ✅ |
| 无占位符 | 无TODO/FIXME | ✅ |
| P4勾选 | 10项全勾选 | ✅ |
| 刀刃表 | 48项全勾选 | ✅ |
| 无内存泄漏 | timer配对 | ✅ |

---

## 4. Sprint4 进行中内容 ⭐

### DataChannel协议规范 (docs/sprint4/)

| 功能 | 规格 | 状态 |
|------|------|:--:|
| 文件传输 | 64KB分片，SHA256校验 | 🔄 |
| 文本消息 | AES-256-GCM加密 | 🔄 |
| 断点续传 | BLAKE3校验，范围请求 | 🔄 |
| 拥塞控制 | 滑动窗口，RTT测量 | 🔄 |

### 错误码定义

| 错误码 | 描述 | 处理策略 |
|--------|------|----------|
| E_DC_001 | Channel not open | 重连后重试 |
| E_DC_002 | Chunk checksum mismatch | 请求重传 |
| E_DC_003 | Decryption failed | 丢弃消息 |
| E_DC_004 | Timeout | 触发断点续传 |

---

## 5. 技术栈速查

```
语言: Node.js (v24+), Rust (WASM), TypeScript (接口)
存储: SQLite (16分片+WAL模式), 二进制HNSW索引
向量: SimHash-64 (LSH), HNSW图索引
限流: Token Bucket + SQLite持久化 + 熔断器
WASM: wasm32-unknown-unknown, wasm-bindgen
Worker: Node.js Worker Threads
P2P: WebRTC DataChannel + WebSocket信令
加密: AES-256-GCM, scrypt密钥派生
测试: 原生Node.js assert + 自定义runner
```

---

## 6. 目录结构

```
workspace/
├── Agent prompt/              # 【提示词+背景】
│   ├── Mike.md               # 审计汪提示词
│   ├── PROJECT-CONTEXT.md    # 本文件
│   └── memory.md             # 记忆锚点
├── task-audit/               # 【任务输入】01.md ~ 28.md
├── audit report/             # 【报告输出】00-30/
│   ├── 28/                   # Sprint3 FINAL审计 (A-/Go)
│   └── 30/                   # 29号修复复核
├── docs/                     # 📚 项目文档
│   ├── sprint3/              # Sprint3完成产出 ⭐
│   ├── sprint4/              # Sprint4进行中 ⭐
│   ├── deepresearch/         # Task21深度研究
│   └── task01-23/            # 历史任务文档
├── src/                      # 💻 源代码
│   ├── storage/              # Phase 1: 存储层
│   ├── vector/               # Phase 2: 向量索引
│   ├── p2p/                  # Sprint3/4: WebRTC P2P ⭐
│   │   ├── signaling-server.js
│   │   ├── signaling-client.js
│   │   └── datachannel-manager.js
│   ├── cli/                  # CLI工具
│   └── sync/                 # 同步管理
├── tests/                    # 🧪 测试
│   ├── webrtc-handshake.e2e.js
│   └── datachannel-transfer.e2e.js
└── scripts/                  # 🔧 脚本
    └── install-wrtc.bat
```

---

## 7. 核心模块速查

| 模块 | 路径 | 一句话说明 |
|------|------|-----------|
| **ShardRouter** | `src/storage/shard-router.js` | SimHash→分片00-15路由 |
| **HNSWIndex** | `src/vector/hnsw-core.js` | JS版HNSW图索引 |
| **HNSWIndexWASMV3** | `src/vector/hnsw-index-wasm-v3.js` | WASM版HNSW (1.94x) |
| **LuxurySQLiteRateLimiter** | `src/security/rate-limiter-sqlite-luxury.js` | SQLite持久化限流 |
| **SignalingServer** | `src/p2p/signaling-server.js` | WebSocket信令服务器 ⭐ |
| **SignalingClient** | `src/p2p/signaling-client.js` | WebRTC客户端 ⭐ |
| **DataChannelManager** | `src/p2p/datachannel-manager.js` | DataChannel管理 ⭐ |
| **FallbackManager** | `src/sync/fallback-manager.js` | 同步降级管理 ⭐ |

---

## 8. 文档查找（按任务）

| 任务 | 文件夹 | 核心文档 |
|:-----|:-------|:---------|
| **Sprint3完成** | `docs/sprint3/` | SIGNALING-PROTOCOL-v1.0.md |
| **Sprint4进行** | `docs/sprint4/` | DATACHANNEL-PROTOCOL-v1.0.md |
| **Sprint3审计** | `audit report/28/` | 28-AUDIT-SPRINT3-FINAL.md |
| **29号修复** | `audit report/30/` | 30-AUDIT-FIX-MINOR.md |
| **Deep Research** | `docs/deepresearch/` | 技术债务根治白皮书 |

---

## 9. 技术债务（已更新 2026-02-28）

| ID | 描述 | 优先级 | 排期 | 状态 |
|:---|:---|:---:|:---:|:---:|
| DEBT-PHASE1-001 | WebRTC传输层 (P2→P0) | P2 | Sprint3 | ✅ **已清偿** |
| DEBT-PHASE2-001 | WASM方案 (477KB) | P1 | - | ⚠️ 部分85% |
| OBS-001 | wasm-loader零拷贝 | 低 | Sprint2 | ✅ 已修复 |
| OBS-002 | Redis超时防护 | 低 | Sprint2 | ✅ 已修复 |
| RISK-01 | SAB环境检测 | P1 | - | ✅ 已修复 |
| RISK-02 | searchBatch批量 | P1 | - | ✅ 已修复 |
| RISK-03 | Redis主动重连 | P1 | - | ✅ 已修复 |

---

## 10. 关键验证命令

```bash
# Sprint3 E2E测试
node tests/webrtc-handshake.e2e.js        # 预期: Exit 0

# NAT穿透测试
node tests/nat-traversal.test.js          # 预期: 真实wrtc加载

# DataChannel传输测试
node tests/datachannel-transfer.e2e.js    # Sprint4验证

# 心跳测试
node tests/heartbeat.e2e.js

# 回归测试
npm run test:regression                   # 48项全绿
```

---

## 11. 快速导航

| 你想做什么 | 看这里 |
|-----------|--------|
| 了解Sprint3完成 | `docs/sprint3/SPRINT3-COMPLETION-REPORT.md` |
| 了解信令协议 | `docs/sprint3/SIGNALING-PROTOCOL-v1.0.md` |
| 了解DataChannel | `docs/sprint4/DATACHANNEL-PROTOCOL-v1.0.md` |
| 查看最新审计 | `audit report/28/28-AUDIT-SPRINT3-FINAL.md` |
| 查看29号修复 | `audit report/30/30-AUDIT-FIX-MINOR.md` |

---

## 12. 审计链连续性

```
23 (Phase5 RISK修复 A-/Go)
  ↓
27 (Sprint2完成 A/Go)
  ↓
28 (Sprint3 FINAL A-/Go) ← DEBT-PHASE1-001清偿
  ↓
29 (地狱修复 - 3项minor)
  ↓
30 (minor清零闭环验证) ← 当前等待终局
  ↓
31+ (Sprint4 DataChannel进行中)
```

---

## 13. 新人上手 checklist

1. ✅ **必读**: `Agent prompt/Mike.md` - 审计规范和工作流
2. ✅ **必读**: `Agent prompt/PROJECT-CONTEXT.md` - 本文件，项目全貌
3. ✅ **必读**: `Agent prompt/memory.md` - 记忆锚点，快速恢复状态
4. 📋 **根据任务**: `docs/sprint3/` - Sprint3完成文档
5. 📋 **根据任务**: `docs/sprint4/` - Sprint4进行中文档
6. 🎯 **执行审计**: 读取 `task-audit/28.md`，输出到 `audit report/28/`

---

## 14. 一句话总结

> **HAJIMI V3 = SimHash路由的16分片SQLite + Chunk文件存储 + HNSW向量索引 + Worker线程 + 豪华版限流(9569 ops/s) + WASM加速(1.94x/7.7x) + WebRTC P2P同步。Phase 1-5全部A级完成，Sprint2 OBS修复完成(v3.1.0)，Sprint3 WebRTC信令完成(A-/Go, DEBT-PHASE1-001已清偿)，30号审计等待minor清零终局，Sprint4 DataChannel进行中。**

---

*本文档与 `Mike.md` 配套使用，审计时先读此背景，再执行审计任务。*
*最后更新: 2026-02-28，已同步Sprint3完成+Sprint4进行中最新进展。*
