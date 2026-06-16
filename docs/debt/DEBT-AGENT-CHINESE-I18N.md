# DEBT-AGENT-CHINESE-I18N: Agent 核心链路仅支持英文输入，中文用户意图丢失

> **ID**: `AD-017` / `DEBT-AGENT-CHINESE-I18N`  
> **Priority**: **P0**  
> **Date**: 2026-05-28  
> **Status**: `CLOSED` (2026-05-30, LLM-Native Migration Day 8-23)  
> **Cluster**: Agent Core Execution (Day 1-8) 中文本地化  
> **关联审计**: `AGENT-LOOP-EXECUTION-001-DAY-07-AUDIT-REPORT.md`  

---

## 1. 问题摘要

Hajimi IDE 的前端界面已全面中文化，但 **Agent 核心执行链路和斜杠命令系统仅支持英文关键词匹配**。当用户使用中文自然语言输入时，Planner 的规则映射无法识别意图，导致生成错误的执行计划，最终工具执行失败。

具体表现为：
- 用户输入 `/agent 创建一个名为 hello-agent.txt 的文件，内容是 hello-from-real-agent-loop`
- Planner `decompose_rule_based` 无法识别中文"创建"，生成通用 subgoals `["Research", "Execute", "Validate"]`
- Task description 被改写为英文 "Research"，与原始用户意图完全无关
- `legacy_act` 规则映射将 "Research" 错误映射为 `read_file("Cargo.toml")`
- 文件不存在 → `os error 2` → Stop-Loss → `Aborted`

---

## 2. 根因分析

### 2.1 Planner `decompose_rule_based` 仅检查英文关键词

```rust
// src/intelligence/agent-core/planner.rs:182
fn decompose_rule_based(&self, goal: &Goal) -> Vec<SubGoal> {
    let desc = goal.description.to_lowercase();
    let patterns: Vec<&str> = if desc.contains("implement") || desc.contains("create") {
        vec!["Analyze requirements", "Design", "Implement", "Test"]
    } else if desc.contains("fix") {
        vec!["Reproduce", "Identify cause", "Apply fix", "Verify"]
    } else {
        vec!["Research", "Execute", "Validate"]  // ← 中文指令永远走进这里
    };
    // ...
}
```

**问题**：中文"创建"不匹配英文 `create`，中文"修复"不匹配英文 `fix`，所有中文指令都被归为通用的 "Research/Execute/Validate"，**用户原始意图完全丢失**。

### 2.2 Planner `generate_tasks_for` 同样仅检查英文

```rust
// src/intelligence/agent-core/planner.rs:209
fn generate_tasks_for(&self, sg: &SubGoal) -> Vec<Task> {
    let desc = sg.description.to_lowercase();
    let items: Vec<&str> = if desc.contains("implement") {
        vec!["Write code", "Check compilation"]
    } else if desc.contains("test") {
        vec!["Run tests", "Review"]
    } else {
        vec![&sg.description]  // ← "Research" → Task "Research"
    };
    // ...
    tool_calls: Vec::new(),  // ← Task 永远没有 tool_calls
}
```

**问题**：Subgoal "Research" 不匹配 `implement` 或 `test`，Task description 直接复制 subgoal description。到 `legacy_act` 时，Task description 已不是用户原始输入。

### 2.3 `legacy_act` 规则映射修复被上游绕过

```rust
// src/intelligence/agent-core/agent_loop.rs:584-600 (Day 7 修复)
let tool_name = if desc.contains("read") || desc.contains("analyze")
    || desc.contains("读") || desc.contains("查看") || desc.contains("分析")
{ "read_file" }
else if desc.contains("write") || desc.contains("edit") || desc.contains("create")
    || desc.contains("写") || desc.contains("编辑") || desc.contains("创建")
{ "write_file" }
// ...
```

**问题**：Day 7 在 `legacy_act` 添加了中文关键词支持，但 Task description 已被上游 Planner 改为英文 "Research"，**中文关键词永远不会被匹配到**。

### 2.4 斜杠命令（Slash Commands）仅支持英文 trigger

```javascript
// src/interface/web/app.js:2423-2435
getSlashCommands() {
  return [
    { id: 'tools', trigger: '/tools', title: 'List tools', ... },
    { id: 'providers', trigger: '/providers', title: 'List providers', ... },
    { id: 'tool', trigger: '/tool', title: 'Run tool', ... },
    { id: 'chat', trigger: '/chat', title: 'Chat with provider', ... },
    { id: 'mcp', trigger: '/mcp', title: 'MCP command', ... },
    { id: 'agent', trigger: '/agent', title: 'Run agent task', ... },
    // ...
  ];
}
```

**问题**：所有斜杠命令的 `trigger` 都是英文（`/tools`、`/agent`、`/chat` 等），没有中文别名。中国用户需要记住英文命令才能调用 Agent。

### 2.5 `handleChatCommand` 仅解析英文斜杠命令

```javascript
// src/interface/web/app.js:2567-2580
async handleChatCommand(text) {
  if (text === '/agent') { /* ... */ }
  if (text.startsWith('/agent ')) { /* ... */ }
  if (text === '/tools') { /* ... */ }
  if (text === '/providers') { /* ... */ }
  // ...
}
```

**问题**：即使前端添加了中文 trigger，`handleChatCommand` 也不会识别 `/代理`、`/工具` 等中文命令。

---

## 3. 影响评估

| 维度 | 影响 | 严重程度 |
|:---|:---|:---:|
| **功能完整性** | 中文用户无法通过自然语言让 Agent 执行任何有意义的任务 | 🔴 高 |
| **用户体验** | 界面全中文，但核心 Agent 功能必须输入英文，产生割裂感 | 🔴 高 |
| **产品价值** | 对中文用户而言，Agent 功能基本不可用 | 🔴 高 |
| **国际化** | 仅支持英文输入，无法服务中文母语用户 | 🔴 高 |
| **斜杠命令** | 用户必须记忆英文命令名 | 🟡 中 |

---

## 4. 受影响代码路径

| 文件 | 函数/结构 | 问题 |
|:---|:---|:---|
| `src/intelligence/agent-core/planner.rs` | `decompose_rule_based()` | 只检查英文 `implement`/`create`/`fix` |
| `src/intelligence/agent-core/planner.rs` | `generate_tasks_for()` | 只检查英文 `implement`/`test` |
| `src/intelligence/agent-core/agent_loop.rs` | `legacy_act()` 规则映射 | 中文修复被上游绕过 |
| `src/interface/web/app.js` | `getSlashCommands()` | 斜杠命令无中文 trigger |
| `src/interface/web/app.js` | `handleChatCommand()` | 不解析中文斜杠命令 |

---

## 5. 关闭条件

### 5.1 必须满足（P0 关闭门槛）

1. **Planner 规则映射支持中文关键词**
   - `decompose_rule_based` 识别中文"创建"/"实现"/"修复"/"分析"等
   - `generate_tasks_for` 识别中文关键词并生成正确的 Task description
   - 或：保留原始 goal description 供下游规则映射使用

2. **`legacy_act` 规则映射能访问原始用户输入**
   - Task 增加 `source_goal_description` 字段，或
   - Blackboard 中存储原始 goal，规则映射时优先参考

3. **斜杠命令支持中文别名**
   - `/agent` → 也支持 `/代理`
   - `/tools` → 也支持 `/工具`
   - `/chat` → 也支持 `/聊天`
   - `/tool` → 也支持 `/运行工具`
   - `/providers` → 也支持 `/模型`
   - `/search` → 也支持 `/搜索`
   - 其他命令保持英文或添加合适的中文别名

4. **`handleChatCommand` 解析中文斜杠命令**
   - 识别 `/代理 <目标>`、 `/工具`、 `/模型` 等中文命令
   - 中文命令与英文命令行为完全一致

### 5.2 验证方法

```bash
# 1. 中文 Agent 任务
/agent 创建一个名为 test-chinese.txt 的文件，内容是 hello

# 2. 中文斜杠命令
/代理 创建一个名为 test-chinese.txt 的文件，内容是 hello
/工具
/模型

# 3. 验证文件真实创建
ls test-chinese.txt
cat test-chinese.txt  # hello
```

---

## 6. 修复方向与建议

### 6.1 短期（最小可行修复）

1. **Planner 层中文关键词**
   ```rust
   // decompose_rule_based
   if desc.contains("implement") || desc.contains("create")
       || desc.contains("创建") || desc.contains("实现") || desc.contains("生成")
   {
       vec!["Analyze requirements", "Design", "Implement", "Test"]
   } else if desc.contains("fix") || desc.contains("修复") || desc.contains("修改") {
       vec!["Reproduce", "Identify cause", "Apply fix", "Verify"]
   }
   ```

2. **斜杠命令中文别名**
   ```javascript
   { id: 'agent', trigger: '/agent', title: 'Run agent task', 
     keywords: ['代理', '智能体', 'agent'], ... },
   { id: 'tools', trigger: '/tools', title: 'List tools',
     keywords: ['工具', 'tool'], ... },
   ```

3. **`handleChatCommand` 中文映射**
   ```javascript
   const commandMap = {
     '/代理': '/agent', '/工具': '/tools', '/模型': '/providers',
     '/聊天': '/chat', '/运行工具': '/tool', '/搜索': '/search',
   };
   ```

### 6.2 中期（架构完善）

1. **Task 携带原始 goal description**
   - `Task` 结构增加 `source_goal: String` 字段
   - `legacy_act` 规则映射优先使用 `task.source_goal` 而不是 `task.description`
   - 这样 Planner 怎么拆分都不影响下游意图识别

2. **规则映射改为 LLM-based**
   - 用轻量级本地模型（如 all-MiniLM）做意图分类
   - 或调用 LLM 将中文 goal 直接转化为 ToolCallV1

### 6.3 长期（架构重塑：学习 Codex）

> **核心洞察**：Codex (OpenAI) 没有"中文输入问题"，因为他们**根本没有本地规则映射**。用户输入直接作为 `UserInput::Text` 原样发给 LLM，LLM 通过 `tool_choice: "auto"` 直接生成 ToolCall。意图识别完全由 LLM 负责，本地零规则映射。
>
> **Hajimi 的困境**：三层英文-only 规则映射（`decompose_rule_based` → `generate_tasks_for` → `legacy_act`）是中文输入问题的根源。每增加一种语言就要加一套关键词，永远追不完。

1. **淘汰本地规则映射，学习 Codex 架构**
   - **目标**：用户输入（任意语言）→ system prompt + available_tools + history → LLM → ToolCall
   - **删除**：`decompose_rule_based()`、`generate_tasks_for()`、`legacy_act()` 中的规则映射 fallback
   - **替代**：LLM 直接根据用户输入生成 `ToolCallV1`，写入 `BB_NEXT_TOOL`
   - **参考**：`docs/codex-twist-source/codex-rs/core/src/client.rs:759` (`tool_choice: "auto"`)

2. **混合架构（过渡期）**
   - LLM 路径作为**主路径**（默认启用 `HAJIMI_AGENT_LLM_BOOTSTRAP_ENABLED=true`）
   - 本地规则映射降级为**离线 fallback**（LLM 不可用时启用）
   - 规则映射保留中文关键词支持，但不再主动使用

3. **系统级多语言支持**
   - 所有 prompt、命令解析支持多语言
   - 根据用户系统语言自动切换关键词库
   - 支持日文、韩文等其他语言

---

## 9. 参考代码（Codex）

| 文件 | 关键代码 | 说明 |
|:---|:---|:---|
| `codex-rs/core/src/client.rs:759` | `tool_choice: "auto".to_string()` | LLM 自动选择工具 |
| `codex-rs/core/src/agent/control.rs:1277-1286` | `Op::UserInput { items }` | 用户输入原样传递 |
| `codex-rs/core/src/agent/builtins/awaiter.toml` | `developer_instructions` | system prompt 定义行为 |
| `codex-rs/core/src/tools/router.rs` | `ToolRouter` | 工具路由（无规则映射）|

**Codex 与 Hajimi 架构对比**：
```
Codex:  用户输入 → LLM(tool_choice=auto) → ToolCall → 执行
Hajimi: 用户输入 → 规则映射 → Task → 规则映射 → ToolCall → 执行
                ↑ 英文-only 瓶颈
```

---

## 7. 相关债务

| 现有债务 | 状态 | 与本债务的关系 |
|:---|:---|:---|
| `DEBT-AGENT-LOOP-LLM-NO-OP` | **CLOSED** | Day 1-7 已修复执行链路，但中文输入问题未被覆盖 |
| `DEBT-COMPLEXITY-DAY05-001` | OPEN | `bootstrap_first_tool_call` 145 行，超限 |

---

## 8. 验收记录

| 日期 | 验收人 | 结果 | 备注 |
|:---|:---|:---|:---|
| 2026-05-28 | 用户实机验收 | **发现问题** | 中文 `/agent 创建一个...` 指令 → Aborted，`read_file` 错误 |

---

> **压力怪评语**: "界面全中文，核心链路只认英文——Planner 的 `decompose_rule_based` 把中文'创建'当空气，生成个'Research'就打发用户；斜杠命令全是 `/agent` `/tools`，中国用户还得背英文单词。这不是本地化，这是假中文。P0，必须修。"
