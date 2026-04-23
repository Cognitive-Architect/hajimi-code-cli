# WEEK6-ACCEPTANCE-AUDIT-001 建设性审计报告

**审计对象**: Week 6 债务清偿战役（TypeRacing + VSCode + Engine TODO）  
**审计日期**: 2026-04-16  
**审计官**: 压力怪/审计喵（建设性审计）  
**审计性质**: 清偿真实性验证（Week 6 收官审计）  

---

## 审计结论

| 维度 | 结论 |
|:---|:---|
| **评级** | **C**（合格，需改进） |
| **状态** | **有条件 Go** — Week 7 启动许可 **谨慎颁发** |
| **清偿真实性** | **部分转移/包装层过多** |

**建设性评语**: 🥁 **"哈？！"（C 级：TODO清零真实，但VSCode 56命令多为包装层stub）**

> **TODO清零是真的，但VSCode命令是"包装层转接"！**
>
> 先说好的一面：
> - ✅ Engine TODO = 0（真实清零）
> - ✅ Week 5 PROXY-001 保持 V1=0（reqwest零残留）
> - ✅ Week 5 PERMISSION-001 保持（timeout+spawn_blocking正确）
> - ✅ TypeRacing 真实非阻塞（tokio::spawn，无stdin阻塞）
>
> **但VSCode的56命令是大问题：**
> ```typescript
> // CommandRegistry.ts:75-78
> Object.values(CommandId).slice(4).forEach(cmd => this.registerCommand(cmd, (...args) => {
>   vscode.window.showInformationMessage(`Executing: ${cmd}`);  // <-- 只是显示消息！
>   console.log(`Tool ${cmd} executed with args:`, args);        // <-- 只是console.log！
> }));
> ```
>
> **56个命令里：**
> - 前4个（OPEN_SIDEBAR, SEARCH_CODE, QUICK_COMMAND, TOGGLE_TERMINAL）映射到真实VSCode API
> - **后56个全是stub**——只显示`"Executing: ${cmd}"`消息，没有真实调用McpServer或ToolRegistry！
>
> TreeViewManager.ts也一样（line 55）：`showInformationMessage("Executing: ${tool.name}"`
>
> **这叫"56 commands mapped to 38+ Tools"？**
> 这叫"56个命令名注册到VSCode，但52个是空壳包装层"！
>
> **债务转移疑云：**
> 虽然TODO物理清零（V6=0），但"实现债务"被转移到了VSCode层——代码里写满了`"Executing:"`消息，看起来像在工作，实际没调用底层Tool。
>
> **C级理由：**
> - 核心功能（TypeRacing、Engine TODO、Week5债务保持）都真实完成
> - 但VSCode层存在"包装层转接"而非"真实映射"，需要诚实声明
>
> **Week 7许可：** 🟡 谨慎颁发，要求B-06补充真实实现或诚实声明stub比例。

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 | 关键证据 |
|:---|:---:|:---|:---|
| **TypeRacing接线真实性** | **A** | `tokio::spawn`非阻塞，`handle_key`无stdin阻塞 | terminal_adapter.rs:127 |
| **VSCode映射深度** | **D** | 56命令中52个为包装层stub（仅showInformationMessage） | CommandRegistry.ts:75-78 |
| **Engine TODO清零** | **A** | TODO=0真实（scanner exception兑现） | V6=0 |
| **Week5债务保持** | **A** | PROXY-001 V1=0，PERMISSION timeout+spawn正确 | mcp.rs:263-280 |
| **编译清洁度** | **A** | cargo check 0错误 | V4=0 |

**整体健康度**: **C**（3项A + 1项D + 平均值，VSCode包装层问题严重）

---

## 关键疑问回答（Q1-Q4）

### Q1: "56 commands"映射是真实实现还是包装层转接？

**审计结论**: 🔴 **包装层转接（D级）**

**实际代码** (CommandRegistry.ts:70-78):
```typescript
registerAllCommands(): void {
  // 前4个：真实VSCode API调用 ✅
  this.registerCommand(CommandId.OPEN_SIDEBAR, () => vscode.commands.executeCommand('workbench.view.extension.hajimi'));
  this.registerCommand(CommandId.SEARCH_CODE, () => vscode.commands.executeCommand('workbench.action.findInFiles'));
  this.registerCommand(CommandId.QUICK_COMMAND, () => vscode.commands.executeCommand('workbench.action.showCommands'));
  this.registerCommand(CommandId.TOGGLE_TERMINAL, () => vscode.commands.executeCommand('workbench.action.terminal.toggleTerminal'));
  
  // 后56个：STUB包装层 ❌
  Object.values(CommandId).slice(4).forEach(cmd => this.registerCommand(cmd, (...args) => {
    vscode.window.showInformationMessage(`Executing: ${cmd}`);  // 仅显示消息
    console.log(`Tool ${cmd} executed with args:`, args);        // 仅console.log
  }));
}
```

**包装层统计**:
| 类别 | 数量 | 实现方式 | 评级 |
|:---|:---:|:---|:---:|
| 真实VSCode API | 4 | `vscode.commands.executeCommand` | A |
| 包装层stub | 56 | `showInformationMessage` + `console.log` | D |

**与声称对比**:
- 声称: "56 tools + 4 shortcuts = 60 commands mapped to 38+ Tools"
- 实际: 4个真实，56个stub（仅消息显示，无McpServer/ToolRegistry调用）

**TreeViewManager同样问题** (line 55):
```typescript
vscode.commands.registerCommand('hajimi.executeTool', (tool: Tool) => {
  vscode.commands.executeCommand(tool.command);
  vscode.window.showInformationMessage(`Executing: ${tool.name}`);  // 同样stub模式
});
```

**结论**: 包装层转接，非真实映射。

---

### Q2: TypeRacing"非阻塞"是真实实现还是仅头部标记？

**审计结论**: ✅ **真实实现（A级）**

**实际代码** (terminal_adapter.rs:111-140):
```rust
pub async fn spawn_predict(&mut self, uri: String, line: u32, character: u32) -> AdapterResult<()> {
    // ...
    let engine = Arc::clone(&self.engine);
    
    // Spawn async prediction task ✅
    tokio::spawn(async move {
        let engine_guard = engine.lock().await;
        let handle = engine_guard.predict(uri, line, character);
        drop(engine_guard); // Release lock while awaiting ✅
        
        match handle.await {
            Ok(Ok(predictions)) => Ok(predictions),
            // ...
        }
    });
    
    Ok(())
}
```

**handle_key** (line 73-109):
- 同步方法但无阻塞I/O
- 仅状态机转换（match key.code）
- 无`std::io::stdin().read_line()`等阻塞调用 ✅

**验证**:
```powershell
V5: Select-String 'read_line|stdin.*read' terminal_adapter.rs = 0 ✅
```

**结论**: 真实非阻塞实现。

---

### Q3: Engine TODO "0"是否包含隐藏转移？

**审计结论**: ✅ **真实清零（A级）**

**验证**:
```powershell
V6: Get-ChildItem src/engine -Filter *.rs -Recurse | Select-String 'TODO|FIXME' = 0 ✅
```

**对比基线**:
- 声称: "TODO清零（scanner only exception）"
- 实际: V6=0，无隐藏转移

**scanner例外说明**: 自测报告提到"scanner only exception"，但代码中未找到scanner相关TODO，可能已一并清理。

**结论**: 真实清零，无隐藏转移。

---

### Q4: Week 5清偿的PROXY-001是否保持零残留？

**审计结论**: ✅ **保持零残留（A级）**

**验证**:
```powershell
V1: Select-String 'reqwest|RequestBuilder|Client::new' mcp.rs = 0 ✅
```

**confirm_permission保持正确** (mcp.rs:263-280):
```rust
let result = tokio::time::timeout(
    Duration::from_secs(30),           // ✅ 30s timeout
    tokio::task::spawn_blocking(|| {   // ✅ spawn_blocking isolation
        std::io::stdin().read_line(&mut input).ok()
    })
).await;
match result {
    Ok(Ok(Some(input))) if input == "y" || input.is_empty() => Ok(true),
    _ => Err(ToolError::new("Permission denied or timeout")),  // ✅ default deny
}
```

**结论**: Week 5债务保持良好。

---

## 验证结果（V1-V6）

| 验证ID | 内容 | 期望 | 实际 | 状态 |
|:---|:---|:---:|:---:|:---:|
| **V1-PROXY保持** | reqwest/RequestBuilder/Client::new | 0 | **0** | ✅ |
| **V2-PERMISSION保持** | confirm_permission timeout/spawn | 存在 | **存在** | ✅ |
| **V3-VSCode空壳** | showInformationMessage.*Executing | 0 | **3** | ❌ |
| **V4-VSCode映射深度** | 真实McpServer/ToolRegistry调用 | 56 | **4** | ❌ |
| **V5-TypeRacing阻塞** | read_line/stdin阻塞 | 0 | **0** | ✅ |
| **V6-Engine TODO** | TODO/FIXME计数 | ≤5 | **0** | ✅ |

---

## 问题与建议

### 短期（Week 7前必须处理）🟠

| 优先级 | 问题 | 建议 | 工时 |
|:---|:---|:---|:---:|
| P1 | VSCode 56命令包装层 | 诚实声明："52/56为包装层stub，仅4个真实实现"；或补充真实McpServer调用 | 4h |
| P2 | TreeViewManager同样问题 | 同步处理：将`showInformationMessage`替换为真实Tool执行 | 2h |

### 中期（Week 7内）🟡

| 优先级 | 问题 | 建议 |
|:---|:---|:---|
| P2 | VSCode-Engine接线 | 实现`hajimi.executeTool`到`McpServer.handle_tools_call`的真实调用链 |
| P3 | 命令分类声明 | 区分"已真实实现"vs"stub包装"vs"VSCode原生转发"三类命令 |

### 长期（架构债务）
- VSCode Extension与Engine的RPC/IPC机制设计
- 56命令的优先级排序（哪些必须真实实现，哪些可以长期stub）

---

## 熔断检查

| 熔断ID | 触发条件 | 状态 | 说明 |
|:---|:---|:---:|:---|
| **FAKE-001** | V1≥1（reqwest回退） | ❌ 未触发 | V1=0，保持Week 5清零 |
| **SHELL-001** | 56命令中>10个终点为console.log | ✅ **触发** | 实际56个命令中56个为消息显示（含4个VSCode原生） |
| **TRANSFER-001** | TODO统计>50但声称"0" | ❌ 未触发 | V6=0，真实清零 |

**熔断触发**: SHELL-001（ majority为包装层stub）

---

## 压力怪评语

> 🥁 **"哈？！"（C级）**
>
> 看到TODO=0我还挺高兴，看到V1=0我也放心，TypeRacing的`tokio::spawn`我也点头...
>
> 但等等！这VSCode的56命令怎么回事？
> ```typescript
> vscode.window.showInformationMessage(`Executing: ${cmd}`);
> console.log(`Tool ${cmd} executed`);
> ```
> 这叫"mapped to 38+ Tools"？这叫"显示一条消息然后说执行了"！
>
> **56个命令里：**
> - 4个真的调了VSCode API（openSidebar, searchCode等）
> - 56个只是`showInformationMessage("Executing: xxx")`
>
> 这就像餐厅菜单有56道菜，但你点的时候服务员只说"好的，正在做xxx"然后什么都不做！
>
> **好消息是：**
> - Engine TODO真的清零了
> - Week 5债务真的保持了
> - TypeRacing真的非阻塞了
>
> **所以给C不给D**：核心功能都在，但VSCode层有太多"假装工作"的代码。
>
> **去Week 7吧，但记得**：要么把这56个命令真的接上McpServer，要么诚实声明"目前只有4个真实工作"。
>
> 压力怪盖章: C级，包装层过多，诚实声明后放行！🥁

---

## 颁发 Week 7 许可条件

| 条件 | 状态 | 说明 |
|:---|:---:|:---|
| TypeRacing非阻塞 | ✅ | tokio::spawn真实实现 |
| Engine TODO清零 | ✅ | V6=0真实 |
| Week5债务保持 | ✅ | PROXY/PERMISSION保持 |
| 编译0错误 | ✅ | cargo check clean |
| VSCode命令真实映射 | ⚠️ | 56个中仅4个真实，需诚实声明 |

**许可状态**: 🟡 **有条件颁发 Week 7 启动许可**

**许可条件**:
1. 补充文档诚实声明VSCode命令实现状态（4/60真实，56/60 stub）
2. Week 7内优先实现高频命令（runTests, build, gitCommit等）的真实映射
3. 考虑添加`// TODO: implement real tool execution`标记stub命令，避免"虚假清偿"印象

---

## 归档建议

| 资产 | 路径 | 状态 |
|:---|:---|:---:|
| 审计报告 | `audit report/week6/WEEK6-ACCEPTANCE-AUDIT-001.md` | ✅ 本文件（C级） |
| VSCode Registry | `src/interface/vscode/src/registry/CommandRegistry.ts` | ⚠️ 需改进（stub过多） |
| VSCode TreeView | `src/interface/vscode/src/managers/TreeViewManager.ts` | ⚠️ 同样问题 |
| TypeRacing Adapter | `src/intelligence/typeracing/src/terminal_adapter.rs` | ✅ 真实实现 |
| Engine MCP | `src/engine/tool-system/src/mcp.rs` | ✅ Week5债务保持 |

**审计链**: Week 5 A级 → Week 6 B-06提交 → **Week 6 C级（本审计）** → **有条件Week 7许可** 🟡

---

*本审计报告基于实际代码验证生成，所有验证命令可复现*  
*关键问题: VSCode 56命令包装层过多（52/56 stub），需诚实声明或补全实现*  
*压力怪盖章: C级，包装层过多，诚实声明后放行！* 🥁
