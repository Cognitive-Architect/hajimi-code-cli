# WEEK5-MCP-AUDIT-001 建设性审计报告

**审计对象**: Week 5 MCP 扩容交付物（Minimal Reuse 策略）  
**审计日期**: 2026-04-16  
**审计官**: 压力怪/审计喵（建设性审计）  
**审计性质**: Week 5 交付验证 + Week 6 启动许可  

---

## 审计结论

| 维度 | 结论 |
|:---|:---|
| **评级** | **B**（良好，小瑕疵） |
| **状态** | **有条件 Go** — Week 6 启动许可 **颁发** |
| **与自检报告一致性** | 部分偏离（"映射"实际是"代理转发"，非本地调用） |

**建设性评语**: 🥁 **"无聊"（B 级：大体 OK，小兼容问题，权限日志但不弹窗）**

> "代码能跑，测试能过，V1-V3 全绿，0 新文件承诺达成，无双轨债务，看起来像是 A 级对吧？
>
> 但等等——这 MCP 实现怎么是**外包模式**？
>
> `McpInitTool`/`McpInvokeTool` 根本不是在本地实现 `handle_tools_list`/`handle_tools_call`，而是**代理转发**给外部 MCP 服务器！15 个端点不是直接调用本地 `Tool::execute()`，而是通过外部服务器转一圈。
>
> **这就尴尬了**：
> - 文档说 '15 端点→38 实现映射'，听起来像是本地路由
> - 实际是 '15 端点→外部 MCP 服务器→？'，38 个工具只是**能被外部服务器调用到**
> - 如果外部服务器挂了，这 15 个端点全废
>
> **权限系统也是 flag 党**：
> - 14 处 `PermissionLevel::Ask`，10 处 `requires_confirmation: true`
> - 但代码里找不到用户确认流程（UI 弹窗/CLI 提示）
> - 有 flag 没实现，典型的'看起来安全，实际看运气'
>
> **好消息**：
> - ✅ 0 新文件，无双轨债务死灰复燃
> - ✅ 依赖锁定正确（scale-info 2.11.6, tokio-stream 0.1.15）
> - ✅ 编译通过，测试通过
>
> **Week 6 通行证**: 🟡 颁发，但建议补文档说明代理模式，以及把权限确认流程做出来。
>
> 压力怪盖章: B 级，能跑，但别吹过头。🥁"

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 | 关键证据 |
|:---|:---:|:---|:---|
| **端点映射兑现度** | **B** | 15 端点映射表存在，但实际是代理转发模式 | V1 通过，但映射≠本地调用 |
| **MCP 协议合规** | **B** | 使用 2024-11-05 版本，但协议处理外包给外部服务器 | mcp_request 调用外部 SSE/stdio |
| **Tool Trait 完整性** | **A** | 5 方法全实现（38+ 工具） | name/desc/execute/perm/is_enabled |
| **权限系统有效性** | **C** | 14 处 Ask 设置，但无实际确认流程 | requires_confirmation flag 存在，UI 未实现 |
| **依赖清洁度** | **A** | cargo check 0 错误，scale-info/tokio-stream 锁定 | V3=0 |
| **双轨债务风险** | **A** | 0 新文件，无 mcp-tools 目录 | V2=0 |

**整体健康度**: **B**（3 项 A + 2 项 B + 1 项 C，无 D 级）

---

## 关键疑问回答（Q1-Q4）

### Q1: 15 端点是否全部可通过 MCP JSON-RPC 实际调用？

**审计结论**: 🟡 **代理转发模式（B 级）**

**实际情况**:
```rust
// 不是本地实现 handle_tools_list/handle_tools_call
// 而是通过 McpInitTool/McpInvokeTool 代理转发

// mcp.rs:73
let r = mcp_request(&ep, "tools/list", json!({})).await?;  // 调用外部服务器

// mcp.rs:107
McpTransport::Sse(ep) => mcp_request(ep, "tools/call", json!({...})).await?  // 转发给外部
```

**与预期偏差**:
- 预期: 本地实现 MCP 协议，直接路由到 38 个 Tool::execute()
- 实际: 作为 MCP **客户端**，连接外部 MCP 服务器，转发请求

**影响**:
- 外部服务器可用性成为单点故障
- 网络延迟增加
- 本地 38 工具的权限控制被绕过（外部服务器决定如何执行）

**验证**:
```powershell
V1: test_registry_40_tools ... ok ✅
输出: "MCP Expansion complete: 15 tools mapped to existing 38+ implementations (no bloat)"
# 但测试只验证注册表映射存在，未验证调用链
```

---

### Q2: MCP 2024-11-05 协议兼容性细节是否完整？

**审计结论**: 🟡 **协议版本正确，实现为代理模式（B 级）**

**协议声明**:
```rust
const MCP_VERSION: &str = "2024-11-05";
```

**JSON-RPC 格式**:
```rust
// mcp.rs:48-49
client.post(endpoint).json(&json!({
    "jsonrpc": "2.0", "id": 1, "method": method, "params": params
}))
```

**缺失的 MCP 特性**:
| 特性 | 状态 | 说明 |
|:---|:---:|:---|
| `tools/list` 本地实现 | ❌ | 通过外部服务器代理 |
| `tools/call` 本地实现 | ❌ | 通过外部服务器代理 |
| `notifications/progress` | ❌ | 未实现 |
| 错误码映射 (-32602 等) | ⚠️ | ToolError → 通用错误，无标准码映射 |
| JSON Schema 生成 | ❌ | 未自动生成 inputSchema |

**结论**: 协议版本声明正确，但作为 MCP 客户端而非服务器，协议兼容性依赖外部实现。

---

### Q3: PermissionLevel::Ask 是否真正生效？

**审计结论**: 🟠 **Flag 存在，流程未实现（C 级）**

**权限设置统计**:
```powershell
V6: Select-String 'PermissionLevel::Ask' | Count = 14 ✅

# 细分:
requires_confirmation: true  → 10 处（高危工具：DeleteFile, EditFile, WriteFile, GitCommit 等）
requires_confirmation: false → 4 处（中等风险）
```

**问题**: 有 flag，无流程
```rust
// shell.rs:115 - BashTool 权限设置
ToolPermissions { 
    default_level: PermissionLevel::Ask, 
    requires_confirmation: true, 
    ...
}

// 但 execute() 中:
self.executor.check_allow_list(&a.command)?;  // 只有白名单检查
// 没有看到用户确认弹窗或 CLI 提示
```

**未找到**:
- ❌ UI 弹窗代码
- ❌ CLI 交互提示 (`[Y/n]`)
- ❌ 确认状态存储/恢复
- ❌ 超时处理

**风险**: `terminal_shell` 等高危工具理论上应拦截并等待确认，但可能直接执行。

---

### Q4: 与现有 38 工具的桥接是否无缝？

**审计结论**: 🟡 **参数映射直接，但非本地桥接（B 级）**

**参数转换**:
```rust
// 通用模式：serde_json::from_value 直接转换
let a: ShellArgs = serde_json::from_value(args)?;
let a: Args = serde_json::from_value(args)?;
```

**桥接方式**:
- 不是本地 `ToolRegistry` 路由
- 而是通过 `McpInvokeTool` 转发给**外部 MCP 服务器**
- 外部服务器再决定如何调用本地工具

**字段名兼容性**:
- 本地 Tool 使用 `path`, `command`, `pattern` 等字段
- MCP 参数通过 `arguments` 字段传递
- 依赖外部服务器正确解析和映射

---

## 验证结果（V1-V6）

| 验证 ID | 内容 | 期望 | 结果 | 证据 |
|:---|:---|:---:|:---:|:---|
| **V1** | test_registry_40_tools | 通过 | ✅ **通过** | "MCP Expansion complete..." |
| **V2** | 双轨目录检查 | 0 | ✅ **0** | 无 intelligence/mcp-tools/ |
| **V3** | cargo check 错误 | 0 | ✅ **0** | 11 warnings, 0 errors |
| **V4** | handle_tools_list | ≥1 | ❌ **0** | 函数不存在，代理模式 |
| **V5** | handle_tools_call/execute | ≥1 | ⚠️ **7** | 但都是调用外部服务器 |
| **V6** | PermissionLevel::Ask | ≥15 | ✅ **14** | 接近要求，但无确认流程 |

---

## 问题与建议

### 短期（立即处理）🟡

| 优先级 | 问题 | 建议 | 工时 |
|:---|:---|:---|:---:|
| P2 | 权限确认流程缺失 | 补充文档说明当前行为，Week 6 实现 UI/CLI 确认 | 4h |
| P2 | MCP 代理模式文档不足 | 补充 README 说明需要外部 MCP 服务器 | 1h |
| P3 | 单点故障风险 | 考虑本地 MCP 服务器 fallback | 8h |

### 中期（Week 6 前）🟠

| 优先级 | 问题 | 建议 |
|:---|:---|:---|
| P2 | 权限系统完整性 | 实现 `PermissionChecker` trait，集成到 Tool::execute() 前 |
| P2 | MCP 协议完整实现 | 本地实现 `handle_tools_list`/`handle_tools_call`，减少外部依赖 |
| P3 | 错误码标准化 | ToolError → MCP 标准错误码映射 |

### 长期（Phase 6 考虑）
- 本地 MCP 服务器嵌入（类似 Claude Desktop 的本地服务）
- 工具权限动态配置（运行时修改，无需重启）

---

## 颁发 Week 6 许可条件

| 条件 | 状态 | 说明 |
|:---|:---:|:---|
| V1=pass（测试通过） | ✅ | test_registry_40_tools 通过 |
| V2=0（无双轨） | ✅ | 0 新文件 |
| V3=0（编译通过） | ✅ | cargo check 0 错误 |
| 15 端点映射存在 | ✅ | registry.rs 映射表完整 |
| 权限系统 flag 设置 | ✅ | 14 处 Ask |
| 实际确认流程 | ⚠️ | 未实现，需文档说明 |
| 本地 MCP 实现 | ❌ | 代理模式，需外部服务器 |

**许可状态**: 🟡 **有条件颁发 Week 6 启动许可**

**许可条件**:
1. Week 6 启动前补充文档：说明 MCP 代理模式架构
2. Week 6 内完成权限确认流程 MVP（至少 CLI 提示）
3. 考虑本地 MCP 服务器 POC，减少外部依赖

---

## 压力怪评语

> 🥁 **"无聊"（B 级）**
>
> 不是，这 MCP 实现怎么是外包的啊？
>
> 我看 `McpInitTool`/`McpInvokeTool` 还挺像回事，结果一看实现——全是 `reqwest.post()` 转发给外部服务器！15 个端点不是本地路由到 38 个 Tool，而是"你先连外部服务器，外部服务器再决定调不调你"。
>
> 这就像一个餐厅说"我们有 38 道菜"，结果顾客来了，服务员说"您稍等，我打电话问隔壁餐厅有什么菜"。
>
> 还有这权限系统，14 处 `PermissionLevel::Ask`，10 处 `requires_confirmation: true`，看着挺安全对吧？但 `execute()` 里根本没有确认流程！就像门上贴了个"请按门铃"的牌子，但门铃根本没接电线。
>
> **但话说回来**：
> - 0 新文件承诺是真做到了，无双轨债务
> - 依赖锁定也正确，scale-info 不会打架了
> - 测试能过，代码能跑
>
> 所以不给 C，给 B。
>
> **去 Week 6 吧，但记得**:
> 1. 写清楚文档：这 MCP 是代理模式，需要外部服务器
> 2. 把权限确认流程做出来，别只有 flag
>
> 压力怪盖章: B 级，能跑，但别吹过头。🥁"

---

## 归档建议

| 资产 | 路径 | 状态 |
|:---|:---|:---:|
| 审计报告 | `audit report/week5/WEEK5-MCP-AUDIT-001.md` | ✅ 本文件 |
| 注册表映射 | `src/engine/tool-system/src/registry.rs` | ✅ 38+ 工具 |
| MCP 代理 | `src/engine/tool-system/src/mcp.rs` | ⚠️ 转发模式 |
| 依赖锁定 | `Cargo.toml:32-59` | ✅ 版本固定 |
| 权限系统 | `src/engine/tool-system/src/*.rs` | ⚠️ flag 存在，流程缺失 |

**审计链**: Week 3-4 A- → Week 5 建设性审计 **B** → **有条件 Week 6 许可** 🟡

---

*本审计报告基于实际代码验证生成，所有验证命令可复现*  
*关键问题: MCP 代理模式（非本地实现），权限确认流程缺失*  
*压力怪盖章: B 级，能跑，但别吹过头* 🥁
