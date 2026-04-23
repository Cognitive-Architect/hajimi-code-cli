# WEEK5-FULL-CLEARANCE-AUDIT-002 建设性审计报告

**审计对象**: Week 5 Task 05 全面清偿交付物（声称 DEBT-MCP-PROXY-001 & DEBT-PERMISSION-FLOW-001 已清偿）  
**审计日期**: 2026-04-16  
**审计官**: 压力怪/审计喵（建设性审计）  
**审计性质**: 债务清偿真实性验证  

---

## 审计结论

| 维度 | 结论 |
|:---|:---|
| **评级** | **D**（返工，严重缺陷） |
| **状态** | **返工** — 债务清偿声明虚假，需立即修复 |
| **与自检报告一致性** | **严重偏离**（声称清偿，实际残留） |

**建设性评语**: 🔴 **"重来"（D 级：债务清偿声明虚假，代理代码残留，安全红线）**

> **不是，这 DEBT-MCP-PROXY-001 哪里清偿了？！**
>
> 注释写得天花乱坠："fully cleared"、"zero external calls"、"pure local via registry"，结果呢？
>
> ```rust
> // mcp.rs:45-53 - 这玩意儿还在！
> async fn mcp_request(endpoint: &str, method: &str, params: Value) -> Result<Value, ToolError> {
>     let client = reqwest::Client::new();  // <-- reqwest！外部 HTTP！
>     let resp = client.post(endpoint)...   // <-- 外部调用！
> }
> ```
>
> `McpInitTool` 和 `McpInvokeTool` 还在用 `mcp_request` 调用外部服务器！这叫"fully cleared"？这叫"proxy 模式残留 + 本地实现新增"，是**双轨制**！
>
> **债务清偿的诚实性在哪里？**
> - 声称：DEBT-MCP-PROXY-001 fully cleared
> - 实际：代理代码一点没删，只是新增了 `McpServer` 本地实现
> - 这是**标记为清偿但实际残留**，比未清偿更恶劣！
>
> **权限流程也有问题**：
> - `confirm_permission` 用了 `std::io::stdin().read_line()` 阻塞读取
> - **没有超时机制**（tokio::time::timeout 包裹）
> - 在 async runtime 中阻塞 IO 可能导致整个服务器冻结
> - 默认拒绝逻辑正确（非 "y" 则拒绝），但阻塞风险高
>
> **唯一的好消息**：
> - `McpServer` 本地实现确实存在（`handle_tools_list/call` 使用 registry）
> - 0 新文件承诺达成
> - 编译通过，测试通过
>
> **返工要求**：
> 1. **删除** `mcp_request` 函数和 `McpInitTool`/`McpInvokeTool` 的外部调用逻辑，或标记为 legacy 并默认禁用
> 2. **修复** `confirm_permission` 添加超时机制（tokio::time::timeout）
> 3. **更新** 债务声明：如保留代理代码，应标记为 DEBT-MCP-PROXY-002（遗留SSE模式），而非声称已清偿
>
> 压力怪盖章: D 级，债务声明虚假，立即返工！

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 | 关键证据 |
|:---|:---:|:---|:---|
| **本地 MCP 实现度** | **A** | `McpServer` 存在，`handle_tools_list/call` 本地实现 | mcp.rs:296-323 |
| **零外部 HTTP** | **D** | `mcp_request` + `reqwest::Client` 仍然存在 | V1=4，mcp.rs:45-53 |
| **权限流程完整性** | **C** | `confirm_permission` 存在但无超时，阻塞风险 | mcp.rs:273-284 |
| **工具本地路由** | **B** | `McpServer` 本地路由，但 `McpInitTool/McpInvokeTool` 仍为代理 | 混合模式 |
| **零新文件承诺** | **A** | 无新 .rs 文件 | V5=4（最近编辑，非新建） |
| **债务清偿诚实性** | **D** | 声称 fully cleared，实际代理代码残留 | 注释 vs 代码严重不符 |
| **编译清洁度** | **A** | cargo check 0 错误 | V6=0 |

**整体健康度**: **D**（债务声明虚假，安全红线）

---

## 关键疑问回答（Q1-Q4）

### Q1: 零外部 HTTP 是否真实？

**审计结论**: 🔴 **虚假声明（D 级）**

**验证结果**:
```powershell
V1: Select-String 'reqwest::Client|mcp_request|hyper::client' = 4
```

**实际代码** (mcp.rs:45-53):
```rust
async fn mcp_request(endpoint: &str, method: &str, params: Value) -> Result<Value, ToolError> {
    let client = reqwest::Client::new();  // reqwest 外部 HTTP 客户端
    let resp = client.post(endpoint)...   // 向外部服务器发送 POST
```

**使用位置**:
- `McpInitTool::execute` (mcp.rs:72): `mcp_request(&ep, "tools/list", ...)`
- `McpInvokeTool::execute` (mcp.rs:106): `mcp_request(ep, "tools/call", ...)`

**与声明对比**:
| 声明 | 实际 |
|:---|:---|
| "DEBT-MCP-PROXY-001 fully cleared" | 代理代码一点未删 |
| "zero reqwest in local paths" | reqwest 调用仍然存在 |
| "handle_tools_list/call now pure local" | 新增本地实现，但代理代码仍在 |

**结论**: 这是**新增本地实现 + 保留代理代码**的混合模式，不是"清偿债务"。

---

### Q2: 权限确认流程是否真正阻塞+超时？

**审计结论**: 🟠 **阻塞但无超时（C 级）**

**实际代码** (mcp.rs:273-284):
```rust
pub async fn confirm_permission(name: &str, args: &Value) -> Result<bool, ToolError> {
    println!("Permission required...");
    println!("Continue? [Y/n]: ");
    let mut input = String::new();
    if let Ok(_) = std::io::stdin().read_line(&mut input) {  // 阻塞同步 IO！
        let input = input.trim().to_lowercase();
        if input == "y" || input.is_empty() {
            return Ok(true);
        }
    }
    Err(ToolError::new("Permission denied"))
}
```

**问题**:
| 要求 | 实际 | 状态 |
|:---|:---|:---:|
| 阻塞 [Y/n] | `stdin().read_line()` | 符合 |
| 超时机制 | 无 `tokio::time::timeout` | 缺失 |
| 默认拒绝 | 非 "y" 则拒绝 | 符合 |
| async 安全 | 阻塞 IO 在 tokio runtime 中 | 风险 |

**风险**: `std::io::stdin().read_line()` 是**阻塞同步调用**，在 tokio async runtime 中可能导致：
- 当前线程阻塞
- 其他任务无法执行
- 服务器整体响应延迟

**修复建议**:
```rust
// 应使用 spawn_blocking + timeout
let result = tokio::time::timeout(
    Duration::from_secs(30),
    tokio::task::spawn_blocking(|| {
        std::io::stdin().read_line(&mut input)
    })
).await;
```

---

### Q3: 15 工具是否全部本地路由？

**审计结论**: 🟡 **部分本地，部分代理（B 级）**

**本地路由** (McpServer):
```rust
// mcp.rs:296-323
impl McpServer {
    pub fn handle_tools_list(&self) -> Value {
        let tools = self.registry.list();  // 本地注册表
        // ...
    }
    
    pub async fn handle_tools_call(&self, name: &str, arguments: Value) -> Result<Value, ToolError> {
        if let Some(tool) = self.registry.get(name) {  // 本地路由
            let output = tool.execute(arguments).await?;  // 本地执行
        }
    }
}
```

**代理路由** (仍然存在):
```rust
// McpInitTool (mcp.rs:58-86) 和 McpInvokeTool (mcp.rs:88-115)
// 仍然使用 mcp_request 调用外部服务器
McpTransport::Sse(ep) => mcp_request(&ep, "tools/list", ...).await?,
McpTransport::Sse(ep) => mcp_request(ep, "tools/call", ...).await?,
```

**结论**: 
- 新增 `McpServer` 提供纯本地路由
- 但 `McpInitTool`/`McpInvokeTool` 仍为代理模式
- 这是**双轨并存**，不是"完全清偿"

---

### Q4: UI 弹窗模式是否实现？

**审计结论**: 🔴 **未实现（D 级缺失）**

**实际**: 仅 CLI `[Y/n]` 阻塞输入，无 UI 弹窗模式。

**Week 5 审计要求**: CLI + UI 双模式  
**实际交付**: CLI only

---

## 验证结果（V1-V6）

| 验证 ID | 内容 | 期望 | 实际 | 状态 |
|:---|:---|:---:|:---:|:---:|
| **V1** | 外部 HTTP 检查 | 0 | **4** | 代理代码残留 |
| **V2** | handle_tools_list 本地 | >=1 | **1** | `self.registry.list()` |
| **V3** | handle_tools_call 本地 | >=1 | **1** | `self.registry.get()` |
| **V4** | confirm_permission | 阻塞+超时 | **阻塞无超时** | 异步风险 |
| **V5** | 新文件检查 | <=4 | **4** | 编辑非新建 |
| **V6** | 测试通过 | pass | **pass** | 测试通过 |

---

## 问题与建议

### 立即返工（D 级触发条件）🔴

| 优先级 | 问题 | 修复要求 | 工时 |
|:---|:---|:---|:---:|
| **P0** | 债务声明虚假 | 删除/禁用 `mcp_request` 和代理调用，或更正债务声明为"部分清偿" | 4h |
| **P0** | 双轨制风险 | 明确区分 `McpServer`（本地）vs `McpInitTool`（代理），避免混淆 | 2h |

### 短期（返工后补正）🟠

| 优先级 | 问题 | 建议 |
|:---|:---|:---|
| P1 | `confirm_permission` 无超时 | 添加 `tokio::time::timeout` + `spawn_blocking` |
| P1 | UI 弹窗缺失 | Week 6 实现 GUI 确认 trait |
| P2 | 文档不一致 | 更新 `.mcp.json` 和注释，诚实说明代理模式遗留 |

### 中期（Week 6）
- 完整的 MCP 服务器本地嵌入（类似 Claude Desktop）
- 权限系统 GUI 支持

---

## 返工指令

### 返工范围

1. **删除或禁用代理代码**:
   ```rust
   // 选项 A: 删除 mcp_request 和代理调用（真正清偿）
   // 选项 B: 标记为 #[deprecated] 并默认禁用（承认遗留）
   ```

2. **更正债务声明**:
   ```rust
   // 当前（虚假）:
   //! DEBT-MCP-PROXY-001 fully cleared
   
   // 更正（诚实）:
   //! DEBT-MCP-PROXY-001: Local McpServer added, legacy proxy code retained for SSE fallback
   //! DEBT-MCP-PROXY-002: Remove legacy mcp_request when SSE no longer needed
   ```

3. **修复 confirm_permission**:
   ```rust
   pub async fn confirm_permission(name: &str, args: &Value) -> Result<bool, ToolError> {
       println!("Permission required for tool '{}'", name);
       println!("Continue? [Y/n] (timeout 30s): ");
       
       let result = tokio::time::timeout(
           Duration::from_secs(30),
           tokio::task::spawn_blocking(|| {
               let mut input = String::new();
               std::io::stdin().read_line(&mut input).ok()?;
               Some(input.trim().to_lowercase())
           })
       ).await;
       
       match result {
           Ok(Ok(Some(input))) if input == "y" || input.is_empty() => Ok(true),
           _ => Err(ToolError::new("Permission denied or timeout")),
       }
   }
   ```

### 返工验证

返工后必须全部通过：
```bash
# V1 必须为 0（或遗留代码明确标记为 deprecated）
grep -c 'reqwest::Client\|mcp_request' src/engine/tool-system/src/mcp.rs  # 0

# V4 必须有超时
grep -c 'timeout\|spawn_blocking' src/engine/tool-system/src/mcp.rs  # >=2

# 测试仍然通过
cargo test -p engine-tool-system test_registry_40_tools --lib  # pass
```

---

## 压力怪评语

> 🔴 **"重来"（D 级）**
>
> 我看注释写得挺好："DEBT-MCP-PROXY-001 fully cleared"、"zero external calls"，差点就信了。
>
> 然后一 grep，`mcp_request` 还在！`reqwest::Client::new()` 还在！`McpInitTool` 还在调用外部服务器！
>
> 这叫"fully cleared"？这叫"新增了本地实现但代理代码一点没动"！这是**债务清偿声明虚假**！
>
> 更恶劣的是，这会让其他开发者误以为代理债务已清，结果在架构决策时踩坑。这比"没清偿"更危险！
>
> `confirm_permission` 也是，`std::io::stdin().read_line()` 直接放在 async fn 里，整个 tokio runtime 都会被阻塞。这是 async Rust 基础常识啊！
>
> **返工，立即！**
> 1. 要么真的删掉代理代码（真正清偿）
> 2. 要么诚实声明"部分清偿，遗留代码仍存在"
> 3. `confirm_permission` 加上 timeout 和 spawn_blocking
>
> D 级，没得商量。

---

## 归档建议

| 资产 | 路径 | 状态 |
|:---|:---|:---:|
| 审计报告 | `audit report/week5/WEEK5-FULL-CLEARANCE-AUDIT-002.md` | 本文件 |
| 初验报告 | `audit report/week5/WEEK5-MCP-AUDIT-001.md` | B 级 |
| MCP 代码 | `src/engine/tool-system/src/mcp.rs` | 需返工 |
| 债务声明 | 代码注释行 2-7 | 虚假 |

**审计链**: Week 5 B 级（初验）→ **Week 5 D 级（清偿复验）** → **返工** → 复验 → Week 6

---

*本审计报告基于实际代码验证生成，所有验证命令可复现*  
*关键失败: V1=4（代理代码残留），债务声明虚假*  
*压力怪盖章: D 级，立即返工！*
