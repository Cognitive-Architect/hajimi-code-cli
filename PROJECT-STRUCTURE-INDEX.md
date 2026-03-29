# 🗂️ Hajimi Code CLI 项目完整目录索引

> **版本**: v3.8.0-SPLUS  
> **生成时间**: 2026-03-28  
> **总目录数**: 40+  
> **项目根路径**: `F:\Hajimi Code Ultra\Hajimi CLI\workspace\hajimi-code-cli`

---

## 📊 目录总览（按重要性排序）

| 优先级 | 目录 | 大小 | 文件数 | 作用 | 状态 |
|:------:|------|:----:|:------:|------|:----:|
| ⭐⭐⭐ | [src/](#1-src-目录) | 172 MB | 686 | **TypeScript源码核心** | 🟢 活跃 |
| ⭐⭐⭐ | [crates/](#2-crates-目录) | 791 MB | 2897 | **Rust Crates** | 🟡 需清理 |
| ⭐⭐ | [tests/](#3-tests-目录) | 100 MB | 61 | **JavaScript测试** | 🟢 活跃 |
| ⭐⭐ | [ts-tests/](#4-ts-tests-目录) | 55 MB | 5898 | **TypeScript FFI测试** | 🟡 需清理 |
| ⭐⭐ | [docs/](#5-docs-目录) | 77 KB | 30 | **技术文档** | 🟢 活跃 |
| ⭐ | [audit-report/](#6-audit-report-目录) | 0.86 MB | 104 | **审计报告** | 🟢 活跃 |
| ⭐ | [task-audit/](#7-task-audit-目录) | 0.94 MB | 97 | **任务审计** | 🟢 活跃 |
| ⭐ | [archive/](#8-archive-目录) | 1.38 MB | 314 | **历史归档** | 🟢 保留 |
| ⭐ | [scripts/](#9-scripts-目录) | 1.65 MB | 22 | **构建脚本** | 🟢 活跃 |
| ⭐ | [codex-twist/](#10-codex-twist-目录) | 11.5 GB | 31837 | **Codex Twist子项目** | 🟢 子模块 |

---

## 1️⃣ src/ 目录 - TypeScript源码核心

**作用**: 项目主源码，包含Hajimi所有TypeScript/Node.js实现  
**状态**: 🟢 活跃开发中  
**大小**: 172.88 MB | **文件数**: 686

### 子目录详情

| 子目录 | 核心作用 | 关键文件 | 大小 | 状态 |
|--------|----------|----------|:----:|:----:|
| **adapters/** | EVM工具适配器 | `slither-adapter.ts`, `foundry-adapter.ts` | 103 KB | 🟢 |
| **api/** | REST API服务 | `server.js`, `vector-api.js`, `storage.js` | 49 KB | 🟢 |
| **bench/** | 性能基准测试 | `p2p-sync-benchmark.js` | 25 KB | 🟢 |
| **cli/** | 命令行工具 | `vector-debug.js` | 13 KB | 🟢 |
| **disk/** | 磁盘存储管理 | `memory-mapped-store.js`, `enospc-handler.js` | 35 KB | 🟢 |
| **format/** | 数据格式定义 | `hnsw-binary.js`, `hctx-v3-hnsw-extension.md` | 15 KB | 🟢 |
| **mcp/** | MCP协议服务器 | `server.ts` | 9 KB | 🟢 |
| **middleware/** | Express中间件 | `rate-limit-middleware.js` | 5 KB | 🟢 |
| **migration/** | 数据库迁移 | `migrator.js`, `v1-to-v2.js` | 15 KB | 🟢 |
| **p2p/** | P2P网络同步引擎 | `crdt-engine.ts`, `turn-client.ts`, `bidirectional-sync*.ts` | 101 KB | 🟢 |
| **security/** | 安全模块 | `rate-limiter-*.js` | 39 KB | 🟢 |
| **storage/** | 存储层适配 | `leveldb-optimized.ts`, `batch-writer-optimized.js` | 49 KB | 🟢 |
| **sync/** | 同步回退策略 | `fallback-manager.js` | 22 KB | 🟢 |
| **test/** | 测试辅助工具 | `debt-clearance-validator.js` | 88 KB | 🟢 |
| **utils/** | 通用工具函数 | `logger.js`, `simhash64.js` | 6 KB | 🟢 |
| **vector/** | 向量搜索核心(HNSW) | `hnsw-core.js`, `hnsw-index-hybrid.js` | 633 KB | 🟢 |
| **wasm/** | WebAssembly绑定 ⚠️ | `hnsw-bridge.js`, `loader.js` | **171 MB** | 🔴 需清理 |
| **worker/** | Web Worker池 | `hnsw-worker.js`, `worker-pool.js` | 20 KB | 🟢 |

### ⚠️ wasm/ 目录问题

该目录占用 **171.68 MB**，包含大量Rust构建产物：
- `.rlib` 文件: 70.32 MB (20个)
- `.rmeta` 文件: 36.73 MB (32个)  
- `.pdb` 文件: 41.28 MB (20个)
- `.dll` 文件: 12.38 MB (5个)

**建议**: 将 `src/wasm/target/` 添加到 `.gitignore`

---

## 2️⃣ crates/ 目录 - Rust Crate结构

**作用**: Rust工作区，包含核心算法和FFI绑定  
**状态**: 🟡 构建产物过大，需清理  
**大小**: 790.99 MB | **文件数**: 2897

### Crate详情

| Crate | 用途 | 源码数 | 总大小 | 实际源码大小 |
|-------|------|:------:|:------:|:------------:|
| **codex-twist** | CLI核心(FFI/存储/TURN) | 22 | 13.81 MB | ~500 KB |
| **evm-bench-adapter** | EVM基准测试适配 | 6 | 54.30 MB | ~200 KB |
| **hajimi-hnsw** | HNSW向量搜索 ⚠️ | 9 | **722.87 MB** | ~300 KB |

### 🔴 严重问题: hajimi-hnsw

- **2169个文件** 中只有 **9个** 是Rust源码
- **722.87 MB** 几乎全是 `target/` 构建产物
- 单个 `.rlib` 文件最大达 **31.86 MB**

**建议**: 
```bash
# 清理所有target目录
crates/*/target/
```

---

## 3️⃣ tests/ 目录 - JavaScript测试

**作用**: 项目测试套件，包含单元测试、集成测试、E2E测试  
**状态**: 🟢 活跃  
**大小**: 100.25 MB | **文件数**: 61

### 测试分类

| 子目录 | 测试类型 | 文件数 | 大小 | 说明 |
|--------|----------|:------:|:----:|------|
| **adapters/** | EVM适配器测试 | 2 | 10 KB | Slither/Foundry适配测试 |
| **bench/** | 性能测试 | 4 | 7 KB | P2P同步基准 |
| **benchmark/** | 基准测试 | 3 | 19 KB | 存储性能测试 |
| **e2e/** | 端到端测试 | 6 | 32 KB | 完整链路测试 |
| **fixtures/** | 测试夹具 ⚠️ | 2 | **100 MB** | 大型测试数据文件 |
| **helpers/** | 测试辅助 | 1 | 4 KB | 通用测试工具 |
| **integration/** | 集成测试 | 1 | 6 KB | 模块集成测试 |
| **mcp/** | MCP服务器测试 | 4 | 35 KB | 协议测试 |
| **p2p/** | P2P网络测试 | 5 | 23 KB | WebRTC/CRDT测试 |
| **rc/** | RC版本测试 | 2 | 13 KB | 发布候选测试 |
| **unit/** | 单元测试 | 1 | 2 KB | 基础单元测试 |
| *(根目录)* | 独立测试 | 18 | 110 KB | 各类独立测试文件 |

### ⚠️ fixtures/ 问题
- 占用 **100 MB** 空间
- 包含大型二进制测试数据
- **建议**: 使用 Git LFS 或外部存储

---

## 4️⃣ ts-tests/ 目录 - TypeScript FFI测试

**作用**: TypeScript FFI绑定测试，napi-rs集成测试  
**状态**: 🟡 node_modules可清理  
**大小**: 55.29 MB | **文件数**: 5898

### 内容分析

| 文件/目录 | 大小 | 说明 | 建议 |
|-----------|:----:|------|------|
| `node_modules/` | 54.97 MB | 测试依赖 | 🟡 可清理，重新安装 |
| `TEST-LOG-*.txt` (7个) | 199 KB | FFI测试日志 | 🟢 保留最近5个 |
| `poc.test.ts` | 8 KB | PoC概念验证测试 | 🟢 保留 |
| `real-ffi.test.ts` | 10 KB | 真实FFI集成测试 | 🟢 保留 |
| `jest.config.js` | 2 KB | Jest配置 | 🟢 保留 |
| `tsconfig.json` | 1 KB | TypeScript配置 | 🟢 保留 |

---

## 5️⃣ docs/ 目录 - 技术文档

**作用**: 项目技术文档，架构设计、API文档、审计报告  
**状态**: 🟢 活跃  
**大小**: 77 KB | **文件数**: 30（精简后）

### 文档结构

| 子目录 | 用途 | 关键文档 |
|--------|------|----------|
| **architecture/** | 架构设计 | `CRDT-STRATEGY.md`, `P2P-SYNC-PROTOCOL-v1.0.md`, `YJS-INTEGRATION-v1.0.md`, `TURN-INTEGRATION-v1.0.md` |
| **audit-report/** | 审计报告 | `19/`, `21/`, `211/` - 当前活跃审计 |
| **archive/** | 文档存档 | 已归档的历史文档 |
| **bench/** | 性能基准 | `HAJIMI-P2P-BENCHMARK-v1.0.md` |
| **learning/** | 学习资料 | `PHASE0-RUST-BOOTCAMP.md` |
| **mcp/** | MCP协议 | `COVERAGE.md`, `PERF.md`, `SECURITY.md`, `SETUP.md`, `STRESS.md` |
| **rc/** | 发布候选 | `FINAL-REPORT-35.md`, `HARDWARE-SPEC.md`, `STABILITY-REPORT.md` |

---

## 6️⃣ audit-report/ 目录 - 审计报告

**作用**: 项目审计报告，质量评估和债务追踪  
**状态**: 🟢 活跃  
**大小**: 0.86 MB | **文件数**: 104

### 审计报告分类

| 目录 | 审计范围 | 报告数量 |
|------|----------|:--------:|
| `01-30/` | Sprint 1-30 审计 | 30+ |
| `30-240/` | Phase 2-3 审计 | 50+ |
| `241-250/` | 最新审计 (SPLUS) | 10 |
| `196/` | Intel可行性审计 | 1 |
| `progress/` | 进度审计 | 2 |
| `其他/` | 特殊审计 | 10+ |

---

## 7️⃣ task-audit/ 目录 - 任务审计

**作用**: 任务级审计追踪，工单执行记录  
**状态**: 🟢 活跃  
**大小**: 0.94 MB | **文件数**: 97

### 审计范围

| 子目录 | 任务范围 |
|--------|----------|
| `01-20/` | 任务1-20审计 |
| `01-42/` | 任务1-42审计（复制） |
| `21-30/` | 任务21-30审计 |
| `logfile/` | 执行日志（任务39-43） |
| `other/` | 特殊审计（196, ID59等） |

---

## 8️⃣ archive/ 目录 - 历史归档

**作用**: 归档历史文档、任务、审计报告  
**状态**: 🟢 保留  
**大小**: 1.38 MB | **文件数**: 314

### 归档结构

| 路径 | 内容 |
|------|------|
| `2026/02/` | 2026年2月归档 |
| `2026/02/tasks/` | 历史任务（task01-task15） |
| `2026/02/docs/` | 债务清理、设计文档 |
| `docs/` | 各类历史报告 |
| *(新归档)* | ROLLUP/SYNC/task等 |

---

## 9️⃣ scripts/ 目录 - 构建脚本

**作用**: 项目构建、测试、部署脚本  
**状态**: 🟢 活跃  
**大小**: 1.65 MB | **文件数**: 22

### 脚本分类

| 文件/目录 | 用途 |
|-----------|------|
| `setup/check-env.mjs` | 环境检查脚本 |
| `setup/install-evm-toolchain.*` | EVM工具链安装（Windows/Linux） |
| `rc/` | RC版本相关脚本 |
| `convert-evmbench-data.js` | 基准数据转换 |
| `install-wrtc.*` | WebRTC安装脚本 |
| `migrate*.js` | 数据迁移脚本 |
| `run-*-test.sh` | 各类测试运行器 |

---

## 🔟 codex-twist/ 目录 - Codex Twist子项目

**作用**: OpenAI Codex CLI的本地化版本，完整子项目  
**状态**: 🟢 子模块（可独立运行）  
**大小**: 11.5 GB | **文件数**: 31837

### ⚠️ 重要说明

这是一个**完整的独立子项目**，包含：
- Rust CLI工具 (`codex-rs/`)
- TypeScript API (`codex-cli/`)
- Python SDK (`sdk/python/`)
- 完整文档 (`docs/`)
- 测试套件
- CI/CD配置

**建议**: 考虑将其作为Git子模块或独立仓库管理

---

## 1️⃣1️⃣ 其他重要目录

### 构建输出目录

| 目录 | 大小 | 文件数 | 用途 | 建议 |
|------|:----:|:------:|------|------|
| **dist/** | 349 KB | 173 | TypeScript编译输出 | 🟢 保留 |
| **target/** | - | - | Rust编译输出 | 🔴 添加到.gitignore |
| **coverage/** | 927 KB | 25 | 代码覆盖率报告 | 🟢 保留最近报告 |

### 数据和资源

| 目录 | 大小 | 文件数 | 用途 | 建议 |
|------|:----:|:------:|------|------|
| **data/** | 123 KB | 9 | 测试数据库文件 | 🟢 保留 |
| **assets/** | 68 B | 1 | 静态资源（空） | 🟡 填充或删除 |
| **snippets/** | 36 KB | 3 | 代码片段 | 🟢 保留 |
| **templates/** | 2.5 KB | 8 | 项目模板 | 🟢 保留 |
| **contracts/** | 474 B | 2 | 漏洞合约示例 | 🟢 保留 |
| **test-data/** | - | 1 | 测试数据 | 🟢 保留 |

### 配置和日志

| 目录 | 大小 | 文件数 | 用途 | 建议 |
|------|:----:|:------:|------|------|
| **logs/** | 27 KB | 6 | RC测试日志 | 🟢 保留最近日志 |
| **.github/** | 0.01 MB | 3 | GitHub Actions | 🟢 保留 |
| **.config/** | 0 | 1 | 配置文件 | 🟢 保留 |
| **.claude/** | 0 | 1 | Claude配置 | 🟢 保留 |
| **docker/** | 726 B | 1 | E2E Dockerfile | 🟢 保留 |

### 空/待处理目录

| 目录 | 状态 | 建议 |
|------|:----:|------|
| **benchmarks/** | 空 | 🟡 填充或删除 |
| **demo/** | 3文件 | 🟢 保留演示配置 |
| **drafts/** | 2文件 | 🟢 保留草稿 |
| **undefined/** | 25文件/2MB | 🔴 需清理或重命名 |

---

## 📋 清理建议汇总

### 🔴 高优先级（立即处理）

```bash
# 1. 添加.gitignore规则
crates/*/target/
src/wasm/target/
ts-tests/node_modules/
target/

# 2. 清理大型构建产物
rm -rf crates/*/target/
rm -rf src/wasm/target/
rm -rf ts-tests/node_modules/
```

### 🟡 中优先级（下次提交前）

```bash
# 3. 处理大型测试夹具
# 方案A: 使用Git LFS
git lfs track "tests/fixtures/*.bin"
git lfs track "tests/fixtures/*.dat"

# 方案B: 移动到外部存储
mv tests/fixtures/ s3://hajimi-test-fixtures/

# 4. 重命名undefined目录
mv undefined/ misc/
# 或分析内容后归档
```

### 🟢 低优先级（定期维护）

```bash
# 5. 清理旧日志
find logs/ -name "*.log" -mtime +30 -delete

# 6. 清理旧覆盖率报告
find coverage/ -name "*.html" -mtime +7 -delete

# 7. 填充或删除空目录
rmdir benchmarks/ 2>/dev/null || echo "非空，需手动检查"
```

---

## 📊 空间占用分布

```
crates/       ████████████████████████████████████████  791 MB (40%)
src/          █████████                                 173 MB (9%)
codex-twist/  ████████████████████████████████████████████████████████████████████████████████████████████████████  11.5 GB (56%) ⚠️ 子项目
tests/        █████                                      100 MB (0.5%)
ts-tests/     ███                                         55 MB (0.3%)
其他          █                                           ~5 MB (<0.1%)
```

**总计**: ~12.6 GB（其中11.5 GB为codex-twist子项目）

---

## 🔗 相关文档

- [DIRECTORY-INDEX.md](./DIRECTORY-INDEX.md) - 目录索引（简化版）
- [README.md](./README.md) - 项目主文档
- [LICENSE](./LICENSE) - Apache 2.0许可证

---

*自动生成于 2026-03-28 | 共扫描 40+ 目录 | 195 个文档已归档*
