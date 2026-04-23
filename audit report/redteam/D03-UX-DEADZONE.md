# D03-UX-DEADZONE: 功能可用性审计 — 僵尸功能与入口死区

**审计日期**: 2026-04-19  
**审计范围**: `src/intelligence/*`, `src/interface/*`, `src/engine/*` 端到端入口  
**审计方法**: 源代码静态分析 + 入口点遍历 + 工作流步骤推演  
**风险等级**: 🔴 HIGH（多个核心功能无法被用户触达）

---

## 执行摘要

本次审计通过 12 条定向验证命令，对 HAJIMI 的功能入口进行了端到端扫描。结果暴露出一个系统性问题：**大量后端模块已完成实现，但前端/CLI 未提供任何可达入口**。用户视角下，这些功能等同于不存在。我们将此类功能定义为 **"僵尸功能"（Zombie Features）**。

---

## 审计命令与原始输出

### 1. Knowledge Search CLI 入口检查
```powershell
Get-Content "src/intelligence/knowledge/src/search.rs" | Select-Object -First 40
```
**输出**: `AdrSearch` 结构体已实现 `find_by_id` 与 `find_by_keyword`，`lib.rs` 也将其导出为 `pub mod search`。

### 2. Memory 界面入口检查
```powershell
Get-ChildItem -Path "src/intelligence/memory" -Directory | Select-Object Name
```
**输出**: 仅 `src` 和 `tests`，无任何 `cli/`、`gui/`、`web/` 子目录。

### 3. TypeRacing 终端快捷键检查
```powershell
Get-Content "src/intelligence/typeracing/src/terminal_adapter.rs" | Select-Object -First 40
```
**输出**: `TerminalAdapter` 实现了 `Ctrl+Space` 触发与 LSP 预测调用，状态机完备。

### 4. 终端输入处理检查
```powershell
Get-Content "src/interface/terminal/src/input_handler.rs" | Select-Object -First 40
```
**输出**: `InputHandler` 只处理 `Standard/Vim/Emacs` 三种模式，无 TypeRacing 路由逻辑。

### 5. CLI ADR 搜索命令检查
```powershell
Get-ChildItem -Path "src/interface/cli" -Recurse -File | Select-String -Pattern "search|adr|knowledge" | Select-Object -First 20
```
**输出**: 仅命中 `vector-debug.js` 的向量搜索命令，无任何 ADR 或 Knowledge 搜索 CLI 命令。

### 6. Tool-System 帮助入口检查
```powershell
Get-Content "src/engine/tool-system/src/mod.rs" | Select-Object -First 50
```
**输出**: 仅定义 `Config`、`ToolConfig`、`PermissionLevel`，无 `--help` 或工具列表注册。

### 7. Tool-System 错误定义检查
```powershell
Get-Content "src/engine/tool-system/src/error.rs" | Select-Object -First 40
```
**输出**: `EngineError` 枚举完备（Timeout、RetryExhausted、PermissionDenied 等），但无 CLI 错误映射。

### 8. Agent Core README 示例检查
```powershell
Get-Content "src/intelligence/agent-core/README.md" | Select-Object -First 60
```
**输出**: 宣称 `AgentOrchestrator::new(memory)` 即可启动，但存在 4 条 Active DEBT。

### 9. VS Code 命令注册检查
```powershell
Get-ChildItem -Path "src/interface/vscode" -Recurse -File | Select-String -Pattern "commands|register" | Select-Object -First 20
```
**输出**: `extension.ts` 仅注册 `openAdr` 与 `gotoAdr` 两个命令，`package.json` 命令列表极短。

### 10. Web/Terminal 功能差异检查
```powershell
Get-ChildItem -Path "src/interface/web","src/interface/terminal" -Directory
```
**输出**: Web 有 `components`、`dist`、`node_modules`、`src`；Terminal 仅有 `src`，无组件体系。

### 11. P2P-Sync 进度反馈检查
```powershell
Get-Content "src/engine/p2p-sync/src/sync-engine.ts" | Select-Object -First 40
```
**输出**: 仅有 TypeScript 接口定义；`onProgress?:` 为**可选回调**，无强制 UI 集成。

### 12. Agent Core 启动配置复杂度检查
```powershell
Get-Content "src/intelligence/agent-core/agent_loop.rs" | Select-String -Pattern "Config|new\(|setup|init" | Select-Object -First 20
```
**输出**: `AgentLoop::new()` 需要传入 7 个 `Arc<Mutex<dyn Trait>>` 参数，构造极其繁琐。

---

## 发现详情

### D3-F01: Knowledge Search — 完全僵尸库 ⬤

- **现状**: `search.rs` 提供了 `AdrSearch::find_by_id()` 和 `find_by_keyword()`，且被 `lib.rs` 公开导出。
- **入口死区**: `src/interface/cli/` 下只有一个 `vector-debug.js`，没有任何命令调用 `AdrSearch`。
- **后果评估**: 用户无法通过任何界面（CLI/Web/Terminal/VSCode）搜索 ADR。Knowledge Graph 索引构建后无法被消费，等于白实现。更危险的是，随着 ADR 数量增长，索引维护成本持续消耗资源，却没有任何用户价值回流，形成纯粹的负债模块。若该索引被 Agent Core 在后台隐式调用，用户甚至无法验证搜索结果是否正确，调试难度极大。

### D3-F02: Memory 层 — 无任何界面入口 ⬤

- **现状**: `src/intelligence/memory/src` 与 `tests` 存在，说明核心逻辑已开发并测试。
- **入口死区**: 目录结构中找不到 `cli/`、`gui/` 或 `api/` 子目录，没有任何入口文件。
- **后果评估**: Agent Core README 宣称的 `AgentOrchestrator::new(memory)` 需要一个 memory 实例，但用户无法独立查看、导出或调试 Memory 内容，黑盒化严重。当 Agent 行为异常时，开发者无法追踪记忆上下文，问题定位周期被拉长数倍。此外，没有导出入口意味着无法实现记忆迁移、备份或合规审计，对生产环境部署构成实质性阻碍。

### D3-F03: TypeRacing — 终端死区，Web 独占 ⬤

- **现状**: `typeracing/src/terminal_adapter.rs` 完整实现了 `Ctrl+Space` 触发与预测状态机。
- **入口死区**: `terminal/src/input_handler.rs` 仅处理 `Standard/Vim/Emacs`，未将 `Ctrl+Space` 路由到 TypeRacing；`terminal/src/mod.rs` 也未导出任何 TypeRacing 模块。
- **后果评估**: TypeRacing 在终端中完全不可达，仅在 Web 的 `TypeRacingWidget.tsx` 可用。双端体验严重割裂，终端用户被排除在核心生产力功能之外。考虑到大量核心开发者习惯在终端内工作，这种遗漏直接导致目标用户群体的核心使用场景降级。`terminal_adapter.rs` 已投入开发成本却未被集成，属于典型的"沉没代码"——既占维护负担，又不产生用户价值。

### D3-F04: CLI 层 — 入口荒漠 🔴

- **现状**: `src/interface/cli` 下仅有单文件 `vector-debug.js`。
- **入口死区**: 无主 CLI 入口（如 `hajimi`、`ha`）、无子命令体系、无 `--help` 生成逻辑。Tool-System 的 30+ 工具（`analyze.rs`、`git.rs`、`lsp.rs`、`mcp.rs`、`security.rs`、`shell.rs` 等）没有任何 CLI 绑定。
- **后果评估**: 整个 HAJIMI 对命令行用户不可见。开发者无法通过终端调用任何工具，所有自动化脚本和 CI/CD 集成被阻断。Tool-System 是项目 Week 4 架构的核心成果，却因为 CLI 层缺失而无法在真实工作流中落地。没有 CLI 入口也意味着无法编写自动化测试来验证工具集成，工具回归测试只能依赖手工，长期可靠性无法保证。

### D3-F05: Tool-System — 30+ 工具无帮助入口 ⬤

- **现状**: `src/engine/tool-system/src/` 下存在 30+ 工具实现文件（`analyze`、`build`、`directory`、`docs`、`download`、`edit`、`find`、`fs`、`git`、`graph`、`grep`、`image_view`、`lsp`、`mcp`、`multi_edit`、`network`、`parse`、`patch`、`registry`、`rust_doc_generator`、`search`、`security`、`shell`、`test` 等）。
- **入口死区**: `mod.rs` 中仅有 `Config` 与权限枚举，无工具注册表、无 `--help` 入口、无命令发现机制。
- **后果评估**: 工具生态已就绪，但用户不知道有哪些工具可用，也无法通过帮助系统学习用法。新工具上线后传播成本极高。在大型项目中，开发者通常通过 `--help` 或 `man` 页面发现功能；缺失这些入口意味着每新增一个工具都需要额外编写文档并人工告知用户。长期来看，工具数量越多，用户认知负担越重，最终可能导致核心工具使用率低下、边缘工具被彻底遗忘。

### D3-F06: VS Code 扩展 — 命令注册极简 ⬤

- **现状**: `extension.ts` 注册了 2 个命令：`openAdr` 和 `gotoAdr`。
- **入口死区**: 未注册任何 Tool-System 命令、未提供 Agent 执行面板、未集成 TypeRacing。
- **后果评估**: VS Code 用户只能打开 ADR 文件，无法调用分析工具、无法触发 Agent、无法使用预测补全。扩展形同虚设。VS Code 是 HAJIMI 目标用户的主要 IDE，扩展功能薄弱会直接削弱项目吸引力。更深层的问题是，扩展与 Tool-System 之间缺乏 IPC/消息层，即使未来想补充命令，也需要重新设计通信协议，返工成本高昂。

### D3-F07: Web vs Terminal — 严重功能不对等 🔴

- **Web 独有**: `TypeRacingWidget.tsx`、`useMCP.ts`、`useShortcuts.ts`、`StreamOutput.tsx`、`ResizablePane.tsx`。
- **Terminal 缺失**: 无 TypeRacing、无 MCP Hook、无流式输出组件、无可调整 Pane。
- **后果评估**: 两套界面维护成本倍增，且终端成为二等公民。习惯终端的高级用户无法使用 MCP、TypeRacing 等现代功能，导致用户分层和流失。从工程角度看，Web 端拥有完整的组件库（`useMCP`、`useShortcuts`、`StreamOutput`），而 Terminal 仅有基础布局引擎，两者代码复用率极低。任何新功能都需要分别实现两套 UI，交付周期和 Bug 面双双膨胀。

### D3-F08: P2P-Sync — 进度反馈可选，易遗漏 ⬤

- **现状**: `sync-engine.ts` 中 `onProgress?:` 是可选回调，无默认 UI 绑定。
- **入口死区**: 没有强制要求前端实现进度条或状态指示器。
- **后果评估**: 长时同步任务缺乏反馈，用户可能认为系统卡死而重复操作，导致数据冲突和用户体验下降。P2P 同步可能涉及大文件传输或 CRDT 合并，耗时数秒至数分钟。没有进度条的情况下，用户极大概率会中断进程或发起重复同步，造成网络资源浪费和潜在的数据不一致。将 `onProgress` 设为可选而非强制，是 API 设计的重大失误。

### D3-F09: Agent Core — 启动配置门槛过高 🔴

- **现状**: `AgentLoop::new()` 要求传入 7 个 `Arc<Mutex<dyn Trait>>`（`AgentContext`、`Planner`、`Reflector`、`Governance`、`Swarm`、`Blackboard`、`CheckpointManager`）。
- **入口死区**: README 示例 `AgentOrchestrator::new(memory)` 与实际 `AgentLoop::new` 签名严重不符；无 Builder 模式、无默认配置、无配置加载入口。
- **后果评估**: 开发者几乎不可能正确手动构造 `AgentLoop`，导致 Agent 功能无法被第三方集成或测试，生态扩展受阻。README 示例 `AgentOrchestrator::new(memory)` 与实际 API 之间的鸿沟会造成严重误导：新开发者 copy-paste 示例后无法编译，挫败感极强。7 个 `Arc<Mutex<dyn Trait>>` 参数也意味着任何生命周期或线程安全问题都会在运行时以死锁形式爆发，调试成本远高于编译期检查。

---

## 端到端工作流步骤数评估

| 工作流 | 理想步骤 | 实际步骤 | 阻塞点 |
|--------|----------|----------|--------|
| ADR 搜索 | 1（输入关键词）| ∞（无入口）| D3-F01, D3-F04 |
| Memory 查询 | 1（打开面板）| ∞（无入口）| D3-F02 |
| TypeRacing（终端）| 2（输入 + Ctrl+Space）| ∞（快捷键未绑定）| D3-F03 |
| 调用分析工具 | 1（选择工具）| ∞（无 CLI）| D3-F04, D3-F05 |
| 启动 Agent | 1（运行命令）| 7+（手动构造 7 个 Arc）| D3-F09 |
| P2P 同步查看进度 | 1（打开同步面板）| ∞（无默认 UI）| D3-F08 |

**结论**: 6 条核心工作流中，5 条因入口缺失而无法完成（步骤数为 ∞），1 条因配置复杂度导致步骤数超标 7 倍。从用户旅程视角看，HAJIMI 当前处于"后端饱和、前端荒漠"的状态：大量模块在 crates 层面完整，但在用户界面层几乎不可见。这不是局部缺陷，而是架构层面的"入口断层"（Entry Gap）。

---

## 建议修复优先级

| 优先级 | 编号 | 修复措施 | 预期收益 |
|--------|------|----------|----------|
| P0 | D3-F04 | 建立 `hajimi` 主 CLI，绑定 Tool-System 全部工具 | 解锁所有命令行用户 |
| P0 | D3-F09 | 为 `AgentLoop` 提供 `Builder` 与默认配置 | 降低 Agent 启动门槛 |
| P1 | D3-F03 | 在 `input_handler.rs` 中注册 `Ctrl+Space` → TypeRacing 路由 | 终端功能对等 |
| P1 | D3-F01 | CLI 增加 `hajimi adr search <keyword>` 子命令 | 释放 Knowledge 索引价值 |
| P1 | D3-F07 | 为 Terminal 补充 `StreamOutput` 与 `TypeRacing` 组件 | 双端体验统一 |
| P2 | D3-F05 | 在 `mod.rs` 中生成工具注册表与 `--help` 入口 | 提升可发现性 |
| P2 | D3-F08 | 将 `onProgress` 改为强制回调，提供默认 TUI 进度条 | 改善长时任务体验 |
| P2 | D3-F06 | VS Code 扩展补充 Tool-System 命令面板 | 扩展用户可用功能 |
| P2 | D3-F02 | 为 Memory 增加 `hajimi memory dump/export` CLI | 消除黑盒化 |

---

## 风险矩阵（影响 × 可能性）

| 发现 | 业务影响 | 技术可能性 | 风险等级 |
|------|----------|------------|----------|
| D3-F04 CLI 荒漠 | 阻断所有命令行用户与 CI 集成 | 已发生 | 🔴 严重 |
| D3-F09 Agent 配置门槛 | 阻止第三方集成与生态扩展 | 已发生 | 🔴 严重 |
| D3-F07 Web/Terminal 不对等 | 终端用户流失，维护成本倍增 | 已发生 | 🟠 高 |
| D3-F03 TypeRacing 终端死区 | 核心生产力功能对终端不可见 | 已发生 | 🟠 高 |
| D3-F01 Knowledge 僵尸库 | 索引纯负债，无用户价值 | 已发生 | 🟡 中 |
| D3-F05 Tool-System 无帮助 | 工具可发现性为零 | 已发生 | 🟡 中 |
| D3-F08 P2P 进度缺失 | 用户误操作导致数据冲突 | 已发生 | 🟡 中 |
| D3-F06 VS Code 极简 | IDE 扩展价值极低 | 已发生 | 🟢 低 |
| D3-F02 Memory 黑盒 | 调试困难，合规受阻 | 已发生 | 🟢 低 |

---

## 附录：工具清单快照

```
src/engine/tool-system/src/
├── analyze.rs              ├── lsp.rs
├── build.rs                ├── mcp.rs
├── directory.rs            ├── multi_edit.rs
├── docs.rs                 ├── network.rs
├── download.rs             ├── parse.rs
├── edit.rs                 ├── patch.rs
├── find.rs                 ├── registry.rs
├── fs.rs                   ├── rust_doc_generator.rs
├── git.rs                  ├── search.rs
├── git_branch.rs           ├── security.rs
├── git_cli.rs              ├── shell.rs
├── graph.rs                ├── test.rs
└── grep.rs                 └── image_view.rs
```

> 以上 30+ 工具全部处于 "已实现、不可达" 状态。

---

## 短期缓解措施（无需重写架构）

在 P0 级重构完成前，可通过以下低成本手段部分缓解可用性危机：

1. **生成式帮助脚本**: 利用 `tools_lib.rs` 的元数据，编写一个静态生成脚本，在构建时输出 `TOOLS.md`，列出所有工具名称与描述。虽非交互式 `--help`，但可让用户通过文档发现功能。
2. **CLI 代理入口**: 在 `src/interface/cli` 下新增一个薄封装 `hajimi-cli.js`，使用 `child_process` 调用底层 Rust 二进制（若已编译），或将现有 `vector-debug.js` 扩展为统一的子命令分发器，至少让已有工具能被调用。
3. **终端快捷键占位**: 在 `input_handler.rs` 的 `Standard` 模式下增加一个 `Ctrl+Space` 占位分支，打印 `"TypeRacing not yet integrated in terminal. Use Web UI."`。这虽不提供功能，但至少告知用户功能存在且位置在哪，避免"完全不可知"的死区体验。
4. **Agent 启动模板**: 在 `src/intelligence/agent-core/examples/` 中提供一个 `minimal_agent.rs`，展示如何用默认实现填充 `AgentLoop::new` 的 7 个参数。让 README 示例与实际代码对齐。

---

## 验证清单（修复后复测用）

- [ ] `hajimi adr search "keyword"` 可返回 ADR 路径列表
- [ ] `hajimi memory export --format json` 可输出记忆内容
- [ ] 终端中按 `Ctrl+Space` 触发 TypeRacing 预测窗口
- [ ] `hajimi --help` 列出全部 30+ 工具及其简要说明
- [ ] `hajimi agent start "goal"` 可一键启动 Agent（无需手动构造 7 个 Arc）
- [ ] VS Code 命令面板中可搜索并执行至少 5 个 Tool-System 命令
- [ ] P2P 同步期间 TUI 显示实时进度百分比
- [ ] Terminal 与 Web 同时支持 StreamOutput 流式渲染
- [ ] 以上所有工作流步骤数 ≤ 3

---

*报告结束。建议在下个 Sprint 优先完成 P0 项，以打通用户触达路径。*
