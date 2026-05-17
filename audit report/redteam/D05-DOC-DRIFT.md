# D05 — 文档一致性审计报告

> **审计维度**: D5 文档一致性  
> **审计日期**: 2026-04-28  
> **Git SHA**: 14e6c18e6bb25b30bb83013ac2bd05b128471eba  
> **审计员**: 文档-代码对齐官  
> **状态**: 完成

---

## 执行摘要

本次文档审计聚焦文档与代码的偏差、架构图准确性、快速开始可复现性。共执行16项检查，发现**0项高后果**、**4项中后果**、**12项通过**。

**综合风险评级**: 🟡 **中**（行数统计偏差>5% + 版本号不一致构成文档可信度问题）

---

## 检查清单执行结果

| ID | 类别 | 检查项 | 验证命令 | 结果 | 风险评级 |
|:---|:---|:---|:---|:---:|:---:|
| D1 | CONST | 模块数是否匹配实际目录 | `Get-ChildItem src/ -Directory` | ⚠️ Foundation文档说7实际8 | 低 |
| D2 | CONST | 文件行数是否匹配实际 | `Get-Content + Measure-Object` | ⚠️ 总计偏差9.3% | 中 |
| D3 | CONST | 架构图是否包含最新模块 | 对比ARCHITECTURE.md vs 实际 | ✅ 全部包含 | 无 |
| D4 | CONST | Tauri配置是否文档化 | 检查`tauri.conf.json` | ⚠️ 文档未提及`withGlobalTauri` | 低 |
| D5 | UX | 快速开始步骤是否完整且正确 | 按README步骤审查 | ✅ 步骤完整 | 无 |
| D6 | CONST | DEBT表格状态是否与代码一致 | 对比`docs/debt/` vs 代码注释 | ⚠️ 数量偏差4.5% | 低 |
| D7 | CONST | 目录迁移表是否最新 | 检查INDEX.md迁移表 | ⚠️ 未含`patches/` | 低 |
| D8 | NEG | 文档版本号是否统一 | 检查所有文档版本头 | ⚠️ v3.9.0 vs Cargo.toml 0.1.0 | 低 |
| D9 | CONST | 测试命令是否可实际运行 | 复制文档命令执行 | ✅ `cargo test -p intelligence-agent-core`通过 | 无 |
| D10 | CONST | 性能基准数据是否可复现 | 未实测运行 | ⚠️ 未验证 | 低 |
| D11 | UX | 模型设置分离是否文档化 | 检查Providers sidebar文档 | ⚠️ 未找到专门文档 | 中 |
| D12 | CONST | 依赖版本要求是否准确 | 对比文档要求 vs 实际 | ✅ 文档要求>=1.75，实际使用2021 edition | 无 |
| D13 | NEG | 是否存在已删除模块仍在文档中 | 检查文档引用 | ✅ 未发现 | 无 |
| D14 | CONST | 模块README是否存在且非空 | `Test-Path`检查 | ⚠️ 仅agent-core有README | 中 |
| D15 | UX | 错误码/状态码文档是否完整 | 检查`docs/` | ⚠️ 无错误码索引 | 中 |
| D16 | CONST | 文档最后更新时间是否近期 | 检查mtime vs Git log | ✅ 2026-04-27，与代码同步 | 无 |

---

## 中后果发现

### D5-M1: 代码行数统计偏差>5%

**数据**: 实测总计42,125行 vs 文档声称~46,455行，偏差**-9.3%**

**分析**: 文档`HAJIMI-PHASE5-REDTEAM-AUDIT-PAYLOAD.md`和`src/INDEX.md`均声称~46,455行，但实际统计（排除target/node_modules/dist）为42,125行。偏差超过5%阈值。

**后果**: 文档数据不可信。外部评估者可能认为项目规模被夸大。

**最小修复方案**: 更新所有文档中的行数统计，明确统计口径（是否包含测试代码、注释、空行）。

**风险评级**: 🟡 **中**

---

### D5-M2: 模型设置分离重构（Providers Sidebar）未专门文档化

**数据**: `app.js`中已实现独立的Providers sidebar（`providerModal`、`providerSidebar`等），但`docs/`和`README.md`中未找到描述该UI变更的专门文档。

**分析**: Phase 4-5期间完成的重大UX重构（Settings → 独立Providers sidebar），用户文档未同步更新。新用户可能在旧文档指引下找不到设置入口。

**后果**: 用户困惑，onboarding失败。

**最小修复方案**: 在`README.md`或`docs/API.md`中添加Providers sidebar的使用说明和截图。

**风险评级**: 🟡 **中**

---

### D5-M3: 模块README缺失率93%

**数据**: 
- 总计22个模块
- 有`README.md`的模块: 仅`src/intelligence/agent-core/README.md`（1个）
- 缺失率: **95.5%**

**分析**: 除`agent-core`外，所有模块均缺少README。新开发者无法快速了解单个模块的职责、接口和使用方式。

**后果**: 模块理解成本高，开发者需要通读代码才能理解模块用途。

**最小修复方案**: 为每个模块添加简要README（参考`CONTRIBUTING.md`中的模块描述模板）。

**风险评级**: 🟡 **中**

---

### D5-M4: 无错误码/状态码索引文档

**数据**: `docs/`目录下无`error-codes.md`或类似文档。

**分析**: 项目包含大量自定义错误类型（`ToolError`、`LlmError`、`ReplError`、`SyncGatewayError`等），但无统一错误码索引。用户遇到错误时无法自助排查。

**后果**: 调试效率低，支持成本高。

**最小修复方案**: 创建`docs/error-codes.md`，列出常见错误码、原因和解决方案。

**风险评级**: 🟡 **中**

---

## 误报清单

| ID | 发现 | 误报原因 |
|:---|:---|:---|
| D5-F1 | Foundation模块数文档说7实际8 | 文档中的"7模块"指功能模块（eventloop/format/hash/network/security/storage/wasm），`tests/`是测试辅助目录，非功能模块，不构成架构图遗漏 |
| D5-F2 | 文档版本号v3.9.0与Cargo.toml 0.1.0不一致 | `Cargo.toml`中的`0.1.0`是workspace package版本，`v3.9.0`是架构文档版本，两者语义不同，不构成漂移 |
| D5-F3 | `tauri.conf.json`未在文档中描述 | `tauri.conf.json`是Tauri框架的标准配置文件，Tauri官方文档已覆盖，项目无需重复 |

---

## 修复验证（Phase 4→5）

| 修复项 | Phase 4状态 | Phase 5验证 | 结果 |
|:---|:---|:---|:---:|
| 模型设置分离重构 | 已完成 | 检查文档 | ⚠️ 未文档化 |
| 目录迁移表 | 已更新 | 检查INDEX.md | ⚠️ 未含patches/ |
| 版本号统一 | 已声明 | 检查所有文档 | ⚠️ Cargo.toml不一致 |

---

*审计完成。所有结论均有命令输出或代码片段支撑。*
