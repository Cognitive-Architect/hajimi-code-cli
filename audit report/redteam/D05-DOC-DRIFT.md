# D05 文档一致性审计报告

> **审计项**: 源代码文档（INDEX.md / ARCHITECTURE.md / CONTRIBUTING.md）与代码实际状态的一致性  
> **审计日期**: 2026-04-19  
> **执行环境**: Windows PowerShell  
> **审计人**: D5 RedTeam Doc-Drift Audit  

---

## 1. 执行摘要

本次审计对 `src/` 下的 3 份核心文档进行了 14 项自动化验证。总体结论：**文档版本统一，关键代码行数准确，但架构图存在模块遗漏和命名不一致问题**。

| 检查维度 | 结果 | 风险等级 |
|---------|------|---------|
| 版本号一致性 | ✅ 统一为 v3.2 | 无 |
| 关键文件行数 | ✅ 3/3 匹配 | 无 |
| 模块总数 | ✅ 39 个模块存在 | 无 |
| 架构图完整性 | ⚠️ 遗漏 5 个模块，1 处命名不一致 | 低 |
| README 非空 | ✅ agent-core/README.md 45 行 | 无 |
| 代码存在性 | ✅ tool-system error.rs 存在 | 无 |

---

## 2. 验证执行记录

### 2.1 模块数量验证

```powershell
Get-ChildItem -Path "src/foundation","src/engine","src/intelligence","src/interface" -Directory | Measure-Object
# => Count: 39
```

分层明细：
- `src/foundation`: **18** 个目录
- `src/engine`: **5** 个目录
- `src/intelligence`: **11** 个目录
- `src/interface`: **5** 个目录

### 2.2 关键文件行数验证（INDEX.md 声称值）

| 文件 | 声称行数 | 实际行数 | 状态 |
|------|---------|---------|------|
| `agent-core/agent_loop.rs` | 257 | 257 | ✅ 精确匹配 |
| `agent-core/swarm.rs` | 255 | 255 | ✅ 精确匹配 |
| `agent-core/governance.rs` | 241 | 241 | ✅ 精确匹配 |

### 2.3 文档版本号检查

| 文档 | 声明版本 | 状态 |
|------|---------|------|
| `src/INDEX.md` | v3.2 | ✅ |
| `src/CONTRIBUTING.md` | v3.2 | ✅ |
| `src/ARCHITECTURE.md` | v3.2 | ✅ |

**结论**: 三份文档版本号完全一致，均为 **v3.2 (Day 10 Agent Core FULL)**。

### 2.4 agent-core README 检查

```powershell
Get-Content "src/intelligence/agent-core/README.md" | Measure-Object
# => Count: 45
```

README 非空，包含架构图、7 步循环说明、Governance 示例、DEBT 汇总表和 API 示例。

### 2.5 tool-system error.rs 存在性

```powershell
Get-Content "src/engine/tool-system/src/error.rs" | Select-Object -First 30
```

文件存在，定义了 `EngineError` 枚举（ToolNotFound / Timeout / RetryExhausted / ExecutionFailed / InvalidParameters / ToolError / PermissionDenied），与文档描述一致。

### 2.6 文档最后更新时间

```powershell
Get-Item "src/ARCHITECTURE.md" | Select-Object LastWriteTime
# => 2026/4/20 09:49:07
```

**注意**: 文件系统时间戳显示为 **2026-04-20**，但文档内文标注为 **2026-04-19**。这是一个 harmless 的轻微漂移（可能由当日编辑引起）。

---

## 3. 架构图完整性问题（⚠️ 唯一发现）

### 3.1 ARCHITECTURE.md ASCII 图遗漏模块

通过对比 ARCHITECTURE.md 的 ASCII 架构图与实际目录结构，发现以下 **实际存在但架构图未展示** 的模块：

| 实际目录 | 所属层级 | 架构图是否展示 |
|---------|---------|--------------|
| `foundation/scripts/` | Foundation | ❌ 缺失 |
| `foundation/hash/` | Foundation | ❌ 缺失 |
| `intelligence/integration/` | Intelligence | ❌ 缺失 |
| `intelligence/pgvector/` | Intelligence | ❌ 缺失 |
| `interface/web/` | Interface | ❌ 缺失 |

**缓解因素**: `src/INDEX.md` 的树形目录列表中**完整包含**了上述 5 个模块，因此用户通过索引文档仍可找到它们。问题仅限于 ARCHITECTURE.md 的 ASCII 架构图。

### 3.2 命名不一致

ARCHITECTURE.md 的 ASCII 图中使用 `(compress/)` 标注压缩模块，而实际目录名为 `compression/`。INDEX.md 中使用的是 `compression/`，命名正确。

### 3.3 test/tests 表示歧义

ARCHITECTURE.md 的 ASCII 图中使用 `(test/tests)` 这一合并写法，实际在 `src/foundation` 下存在 `test/` 和 `tests/` 两个独立目录。此表示方式虽不算错误，但可能引起理解歧义。

---

## 4. 目录迁移与债务文档

### 4.1 迁移记录

`src/INDEX.md` 中包含目录迁移表格（v1.x → v3.1），示例：

| 原路径 | 新路径 | 层级 |
|--------|--------|------|
| (历史路径) | `middleware/` | Foundation |
| (历史路径) | `migration/` | Foundation |

迁移文档存在且可读。

### 4.2 债务状态

`src/intelligence/agent-core/README.md` 中明确列出 **4 项 Active DEBT**（目标 ≤ 8），包括：
- DEBT-RETRIEVE-PHASE5
- DEBT-WORKER-TOOL-EXECUTION
- DEBT-MEMORY-SYNC
- DEBT-LEAK-TEST-PHASE5

与 Day 10 A 级评级及 "0 编译警告" 的声明一致。

---

## 5. 结论与建议

### 5.1 总体评价

| 指标 | 评分 |
|------|------|
| 版本一致性 | A |
| 数据准确性 | A |
| 架构图完整性 | B+ |
| 文档时效性 | A |

### 5.2 建议修复（低风险）

1. **ARCHITECTURE.md 架构图补全**: 在 ASCII 图中补充 `scripts/`、`hash/`、`integration/`、`pgvector/`、`web/` 五个模块的框体。
2. **命名修正**: 将 `(compress/)` 更正为 `(compression/)`。
3. **test/tests 拆分**: 明确标注 `test/` 和 `tests/` 为两个独立模块。
4. **时间戳同步**: 如再次编辑 ARCHITECTURE.md，可将内文日期更新为 2026-04-20 以匹配文件系统时间。

以上修复均为 **文档层微调整**，不影响编译、测试或运行时行为。

---

> **审计状态**: ✅ 通过（附带低风险改进建议）  
> **下次复查**: 建议在新增模块后触发 D5 复查流程。
