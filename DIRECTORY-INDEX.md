# Hajimi 项目目录索引

> 目录重构于 2026-03-28 | v3.8.0-SPLUS

## 📁 根目录核心文件

| 文件 | 说明 | 大小 |
|------|------|------|
| `README.md` | 项目主文档 (1503行) | 53KB |
| `LICENSE` | Apache 2.0 许可证 | 11KB |
| `package.json` | Node.js 项目配置 | 1.3KB |
| `Cargo.toml` | Rust 工作区配置 | 0.4KB |
| `docker-compose.redis.yml` | Redis 服务配置 | 1KB |

## 📂 源码目录

| 目录 | 内容 | 项目数 |
|------|------|--------|
| `src/` | TypeScript 主源码 | 686 |
| `crates/` | Rust crate 库 | 2,897 |
| `codex-twist/` | Codex Twist FFI 核心 | 36,412 |
| `ts-tests/` | TypeScript 测试 | 5,898 |
| `tests/` | 集成测试 | 61 |

## 📂 文档与审计

| 目录 | 内容 | 项目数 |
|------|------|--------|
| `docs/` | 技术文档 | 298 |
| `audit-report/` | 审计报告 (原"audit report") | 195 |
| `task-audit/` | 任务审计 | 105 |
| `archive/docs/` | 归档文档 (ROLLUP/SYNC) | 11 |

## 📂 构建与部署

| 目录 | 内容 | 项目数 |
|------|------|--------|
| `dist/` | 构建输出 | 173 |
| `scripts/` | 构建脚本 | 22 |
| `docker/` | Docker 配置 | 1 |
| `.github/` | GitHub Actions | 4 |

## 📂 数据与资源

| 目录 | 内容 | 项目数 |
|------|------|--------|
| `data/` | 运行时数据 | 9 |
| `assets/` | 静态资源 | 1 |
| `templates/` | 模板文件 | 8 |
| `test-data/` | 测试数据 | 2 |

## 📂 其他

| 目录 | 内容 | 项目数 |
|------|------|--------|
| `coverage/` | 覆盖率报告 | 25 |
| `logs/` | 日志文件 | 6 |
| `benchmarks/` | 性能基准 | 0 |
| `undefined/` | 未分类文件 | 25 |

## 🧹 重构变更记录

### 已执行清理
- [x] 删除编译产物: `*.rlib`, `memory_tier_test`
- [x] 删除过时日志: `TEST-LOG-*.txt` (6个文件)
- [x] 重命名目录: `audit report` → `audit-report`
- [x] 归档文档: `ROLLUP-*.md`, `SYNC-*.md` → `archive/docs/`
- [x] 添加 Apache 2.0 LICENSE

### 根目录文件变化
- **重构前**: ~35个文件 (混乱)
- **重构后**: 5个核心文件 (整洁)

---
*自动生成于 2026-03-28*
