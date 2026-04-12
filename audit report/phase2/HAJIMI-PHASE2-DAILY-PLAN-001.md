# Phase 2 详细开发计划（每日级）- HAJIMI-PHASE2-DAILY-PLAN-001

> **审计派单ID**: HAJIMI-PHASE2-AUDIT-001  
> **规划评级**: B（良好，已修正计数矛盾与密度分布）  
> **执行策略**: 横向Sprint 0（Week 9-12核心高频）+ 纵向Sprint 1-4（工具完善）  
> **审计日期**: 2026-04-04  
> **关联**: Master Plan v1.0 + Phase 1 A-级收官

---

## 审计结论与规划修正

### 发现的矛盾点（已修正）

| 矛盾 | 原规划 | 实际审计 | 修正方案 |
|:---|:---|:---|:---|
| **工具总数** | 声称41个 | **实际49个**（README详细列举） | 确认为**49个工具**，修正规划 |
| **Month 3 LSP** | 声称8个 | 实际列举4个 | 确认为**4个LSP工具**（与列举一致） |
| **Month 4密度** | 15个/20天 | 实际17个/20天 | 分散为**Sprint 3（12个）+ Sprint 4（5个）** |
| **执行策略** | 纵向4月堆叠 | ID-270横向4周冲刺 | 采用**"Sprint 0横向+Sprint 1-4纵向"混合策略** |
| **依赖断裂** | semantic_search在Month 1 | 依赖HNSW（Phase 4） | **移至Phase 4**，Month 1改为关键词搜索 |

### 修正后工具清单（49个）

| Sprint | 工具类别 | 数量 | 工作日 | 密度 | 说明 |
|:---:|:---|:---:|:---:|:---:|:---|
| **Sprint 0** | 核心高频工具 | **20** | 20天 | 1.0天/工具 | Week 9-12启动冲刺 |
| **Sprint 1** | 文件/搜索/网络 | **12** | 20天 | 1.67天/工具 | 含编辑工具重构 |
| **Sprint 2** | Git/终端/构建 | **11** | 20天 | 1.82天/工具 | 含测试工具链 |
| **Sprint 3** | LSP/代码智能 | **6** | 20天 | 3.33天/工具 | 重架构，低密度 |
| **Sprint 4** | MCP/代理/高级 | **10** | 20天 | 2.0天/工具 | 并行3 Agent饱和攻击 |

**总计**: 49个工具 / 100工作日 / 平均2.04天/工具

---

## Sprint划分与工具分配

### Sprint 0: Week 9-12 启动冲刺（核心高频20工具）

**目标**: 建立49个工具的骨架实现（Stub+注册+基础测试），验证ToolRegistry承载能力

**选取标准**（ConfigManager 8场景高频交集）：
- minimal场景: 5个（read_file, write_file, bash, grep_files, list_directory）
- daily场景新增: 7个（edit_file, delete_file, glob, git_status, git_diff, git_commit, apply_patch）
- 必要Stub: 8个（lsp_query, mcp_invoke, spawn_agent, semantic_search等占位）

---

## 每日详细计划

### Week 9 - Sprint 0 第1周（文件基础5工具）

| 日期 | 星期 | 工单ID | 目标工具 | 交付物路径 | 验收标准 | 依赖 | 债务预声明 |
|:---:|:---:|:---:|:---|:---|:---|:---|:---|
| Day 1 | Mon | B-W09-01 | **read_file**（增强） | `src/tool/fs.rs` | 大文件分块读取>1MB | Phase 1基础 | DEBT-001: 二进制文件检测延后 |
| Day 2 | Tue | B-W09-02 | **write_file**（增强） | `src/tool/fs.rs` | 原子写入验证+备份机制 | Day 1 | DEBT-002: 并发写入锁延后 |
| Day 3 | Wed | B-W09-03 | **list_directory**（重构） | `src/tool/directory.rs` | 递归遍历+权限检查 | Phase 1基础 | 无 |
| Day 4 | Thu | B-W09-04 | **glob**（新建） | `src/tool/directory.rs` | 支持**/*.rs模式+性能测试 | Day 3 | 无 |
| Day 5 | Fri | B-W09-05 | **find**（新建） | `src/tool/search.rs` | 递归查找+类型过滤+修改时间 | Day 3 | DEBT-003: 内容搜索集成grep延后 |
| Day 6 | Sat | - | **审计日/缓冲日** | 代码审查 | 5工单A/B级 | - | - |
| Day 7 | Sun | - | 休息日 | - | - | - | - |

### Week 9 验收标准
- ToolRegistry注册5工具全部通过
- `cargo test tool::fs` + `tool::directory` 全绿
- ConfigManager切换minimal场景延迟<3秒

---

### Week 10 - Sprint 0 第2周（搜索+编辑7工具）

| 日期 | 星期 | 工单ID | 目标工具 | 交付物路径 | 验收标准 | 依赖 | 债务预声明 |
|:---:|:---:|:---:|:---|:---|:---|:---|:---|
| Day 8 | Mon | B-W10-01 | **grep_files**（新建） | `src/tool/search.rs` | regex支持+多文件+上下文行 | Week 9基础 | 无 |
| Day 9 | Tue | B-W10-02 | **grep_code**（新建） | `src/tool/search.rs` | 代码感知（跳过注释/字符串） | Day 8 | DEBT-004: AST解析延后 |
| Day 10 | Wed | B-W10-03 | **edit_file**（重构） | `src/tool/edit.rs` | diff应用+冲突检测 | Phase 1基础 | DEBT-005: 3-way merge延后 |
| Day 11 | Thu | B-W10-04 | **apply_patch**（重构） | `src/tool/edit.rs` | unified diff解析+应用 | Day 10 | 无 |
| Day 12 | Fri | B-W10-05 | **multi_file_edit**（新建） | `src/tool/edit.rs` | 批量编辑+事务回滚 | Day 11 | DEBT-006: 跨文件原子操作延后 |
| Day 13 | Sat | - | **审计日** | 代码审查 | 5工单评级 | - | - |
| Day 14 | Sun | - | 休息日 | - | - | - | - |

---

### Week 11 - Sprint 0 第3周（Git 5工具 + Ink UI并行启动）

| 日期 | 星期 | 工单ID | 目标工具/任务 | 交付物路径 | 验收标准 | 依赖 | 并行Agent |
|:---:|:---:|:---:|:---|:---|:---|:---|:---:|
| Day 15 | Mon | B-W11-01 | **git_status** | `src/tool/git.rs` | porcelain解析+颜色支持 | bash工具 | 1 |
| Day 16 | Tue | B-W11-02 | **git_diff** | `src/tool/git.rs` | diff格式+stat输出 | Day 15 | 1 |
| Day 17 | Wed | B-W11-03 | **git_log** | `src/tool/git.rs` | 格式化+图形化分支 | Day 15 | 1 |
| Day 18 | Thu | B-W11-04 | **git_commit** | `src/tool/git.rs` | 自动message生成（可选） | Day 15-17 | 1 |
| Day 19 | Fri | B-W11-05 | **git_branch** | `src/tool/git.rs` | 分支切换/创建/删除 | Day 15 | 1 |
| Day 20 | Sat | UI-W11-01 | **Ink UI框架**（并行） | `ui/terminal/` | React+Ink项目搭建 | - | **2** |
| Day 21 | Sun | - | 休息日 | - | - | - | - |

**Week 11并行策略**: 工具开发（1 Agent）+ Ink UI启动（2 Agent）饱和攻击

---

### Week 12 - Sprint 0 第4周（终端3工具 + Ink UI基础 + 记忆系统启动）

| 日期 | 星期 | 工单ID | 目标工具/任务 | 交付物路径 | 验收标准 | 依赖 | 并行Agent |
|:---:|:---:|:---:|:---|:---|:---|:---|:---:|
| Day 22 | Mon | B-W12-01 | **bash**（重构） | `src/tool/shell.rs` | 超时机制+信号处理+输出截断 | Phase 1基础 | 1 |
| Day 23 | Tue | B-W12-02 | **exec**（新建） | `src/tool/shell.rs` | 通用执行+环境变量控制 | Day 22 | 1 |
| Day 24 | Wed | B-W12-03 | **script**（新建） | `src/tool/shell.rs` | 多行脚本+shebang支持 | Day 22 | 1 |
| Day 25 | Thu | UI-W12-01 | **Ink布局组件** | `ui/terminal/components/` | Layout+Input+Output组件 | UI-W11-01 | **2** |
| Day 26 | Fri | MEM-W12-01 | **Session Memory** | `src/memory/session.rs` | 持久化+加载+清理 | - | **1** |
| Day 27 | Sat | - | **审计日** | Sprint 0验收 | 20工具A/B级 | - | - |
| Day 28 | Sun | - | 休息日 | - | - | - | - |

**Week 12饱和攻击**: 3波次并行（工具/ UI/ 记忆）

---

## Sprint 0 验收标准（Week 12结束）

| 指标 | 目标 | 验收命令 |
|:---|:---:|:---|
| 工具骨架 | 20个Stub | `grep -c "impl Tool for" src/tool/*.rs` = 20 |
| ToolRegistry承载 | 注册全部通过 | `cargo test tool::registry::test_register_all` |
| 测试覆盖 | ≥60个新增 | `cargo test 2>&1 | grep "test result" | sum` = 230+ |
| Ink UI框架 | 可运行 | `cd ui/terminal && npm start` 无错误 |
| Session Memory | 基础功能 | `cargo test memory::session` 全绿 |
| 零债务 | 0新增 | `grep -r "unwrap\|panic" src/tool/ src/memory/ --include="*.rs" | grep -v test | wc -l` = 0 |

---

## Sprint 1-4 概要（纵向补全阶段）

### Sprint 1: Month 2 文件完善+网络（12工具，20天，1.67天/工具）

| 批次 | 工具 | 天数 | 说明 |
|:---:|:---|:---:|:---|
| Week 13 | delete_file, view_image | 4天 | 文件操作完善 |
| Week 14 | web_search, fetch_url, api_request | 6天 | 网络工具链 |
| Week 15 | generate_docs, update_readme | 4天 | 文档工具 |
| Week 16 | refactor（重构）, complexity, dependency_graph | 6天 | 代码分析（轻量版） |

**依赖处理**:
- semantic_search: **移至Phase 4**（依赖HNSW向量库）
- symbol_search: **移至Sprint 3**（依赖LSP客户端）

---

### Sprint 2: Month 3 构建+测试（11工具，20天，1.82天/工具）

| 批次 | 工具 | 天数 | 说明 |
|:---:|:---|:---:|:---|
| Week 17 | npm_run, cargo_build, make, cmake | 8天 | 构建工具链 |
| Week 18 | run_tests, coverage, benchmark | 6天 | 测试工具链 |
| Week 19 | powershell（Windows适配）, security_audit（轻量） | 6天 | 平台适配+安全 |

**技术债务预声明**:
- DEBT-S2-001: coverage工具仅支持Rust（其他语言延后）
- DEBT-S2-002: security_audit仅基础正则检测（AST分析延后）

---

### Sprint 3: Month 4 LSP重架构（6工具，20天，3.33天/工具）

**关键认知**: LSP客户端是重架构，非简单工具

| 批次 | 任务 | 天数 | 说明 |
|:---:|:---|:---:|:---|
| Week 21-22 | LSP Client基础架构 | 10天 | tokio+lsp-types协议实现 |
| Week 23 | lsp_query, go_to_definition | 5天 | 基础查询 |
| Week 24 | find_references, type_check | 5天 | 高级查询 |

**并行策略**: 3 Agent饱和攻击（1架构+2实现）

**依赖处理**:
- symbol_search: **在LSP基础上实现**（非独立工具）

---

### Sprint 4: Month 4-5 MCP+代理+收尾（10工具，20天，2.0天/工具）

| 批次 | 工具 | 天数 | 说明 |
|:---:|:---|:---:|:---|
| Week 25 | mcp_invoke, mcp_resource, mcp_tool | 9天 | MCP协议栈实现 |
| Week 26 | spawn_agent, close_agent, send_input | 6天 | 代理生命周期管理 |
| Week 27 | coverage（完善）, benchmark（完善）, security_audit（完善） | 5天 | Sprint 2债务清偿 |

**风险控制**:
- MCP协议栈: 如Week 25未完成，**DEBT-S4-001**标记至Phase 3
- 代理系统: 如Week 26未完成，**DEBT-S4-002**标记至Phase 3

---

## 关键依赖路径（DAG，已修正）

```
Week 9-12 Sprint 0（基础骨架）
├── read_file/write_file（Day 1-2）
│   └── list_directory/glob/find（Day 3-5）
│       └── grep_files/grep_code（Week 10）
│           └── [semantic_search移至Phase 4]
├── bash（Week 12 Day 22）
│   └── git_status/git_diff/git_log（Week 11）
│       └── git_commit/git_branch（Week 11）
│           └── npm_run/cargo_build（Sprint 2）
│               └── run_tests/coverage（Sprint 2）
└── edit_file/apply_patch（Week 10）
    └── multi_file_edit（Week 10）
        └── refactor（Sprint 1）

Sprint 3 LSP重架构（依赖分离）
├── LSP Client基础（Week 21-22，独立）
│   ├── lsp_query（Week 23）
│   ├── go_to_definition（Week 23）
│   ├── find_references（Week 24）
│   └── type_check（Week 24）
│       └── [symbol_search作为LSP查询模式，非独立工具]

Sprint 4 MCP+代理（独立波次）
├── MCP协议栈（Week 25，3 Agent并行）
│   ├── mcp_invoke
│   ├── mcp_resource
│   └── mcp_tool
└── 代理系统（Week 26，2 Agent并行）
    ├── spawn_agent
    ├── close_agent
    └── send_input
```

---

## 风险缓冲与审计日安排

### 固定审计日（每周六）

| Sprint | 审计日 | 审查范围 | 产出 |
|:---:|:---:|:---|:---|
| Sprint 0 | Week 9-12每周六 | 当周5工单 | A/B/C/D评级+债务标记 |
| Sprint 1-4 | 每月第2/4周六 | Sprint交付物 | 里程碑验收报告 |

### 缓冲日安排

| 位置 | 天数 | 用途 |
|:---:|:---:|:---|
| 每月最后1周五 | 1天 | Sprint回顾+债务清偿 |
| Month 4最后一周 | 4天 | LSP/MCP重架构缓冲 |
| Phase 2结束前 | 5天 | 验收测试+文档补全 |

---

## 压力怪评语（规划审计结论）

🥁 **"还行吧"**（B级规划，修正后可行，Month 4密度仍需监控）

> "49个工具（不是41个），Month 1-2密度1.0-1.8天/工具可接受，Month 3 LSP重架构3.33天/工具给了足够空间，Month 4 MCP+代理2天/工具需要3 Agent并行饱和攻击。
>
> 关键修正做得对：semantic_search移到Phase 4（依赖HNSW），symbol_search作为LSP查询模式（非独立工具），这样依赖链就理顺了。
>
> Sprint 0的横向冲刺策略（Week 9-12先啃20个高频工具）+ Sprint 1-4纵向补全，这种混合策略比纯纵向4个月堆叠更合理。
>
> 风险点：Month 4的MCP协议栈如果 Week 25完不成，要有DEBT标记到Phase 3的勇气，别硬撑。
>
> 给B级，规划修正后可行，执行时监控Month 4密度。"

---

## 归档与下一步

- **计划文档**: `audit report/phase2/HAJIMI-PHASE2-DAILY-PLAN-001.md`
- **关联状态**: ID-270（准备态）→ ID-271（Phase 2 Sprint 0执行中）
- **下次审计**: Week 9 Day 6（首个审计日）
- **关键监控**: 
  - Week 12 Sprint 0验收（20工具骨架）
  - Month 4 Week 25 MCP协议栈完成度
  - 咕咕 gorgeous申报偏差（承诺≤5行）

---

*规划审计完成时间: 2026-04-04 11:20*  
*审计官: 压力怪（建设性审计模式）*  
*修正内容: 工具数41→49，Month 3 LSP 8→4，Month 4密度分散，依赖链修正*  
*规划评级: B（良好，可执行）*
