# Tool System 模块

HAJIMI IDE 的工具系统引擎，提供 40+ 种工具的注册、发现与执行能力，以及带白名单参数化的安全 Shell 执行环境。

## 职责

- **统一工具接口**：定义 `Tool` trait（`name` / `description` / `permissions` / `execute`），所有工具必须实现该接口
- **ToolRegistry**：工具的注册与动态发现中心，支持按配置启用/禁用工具
- **安全 Shell（P0 级别）**：`ShellTool` 实现严格的命令白名单（38 个允许命令，如 `git`、`cargo`、`npm`、`node` 等），禁止 `rm`、`sudo` 等危险命令；使用参数化 `Command::new` 执行，禁止 `bash -c` 拼接；元字符过滤拒绝 `;`、`&`、`|` 等注入字符
- **多工具集群**：文件操作（`ReadFileTool`、`WriteFileTool`、`DeleteFileTool`）、Git（`GitCommitTool`、`GitStatusTool`、`SmartCommitTool`）、编辑（`EditFileTool`、`apply_patch`、跨文件事务 `MultiEditTransaction`）、搜索（`GrepTool`、`FindTool`）、LSP（`LspDefinitionTool`、`LspHoverTool`）、MCP（`McpInitTool`、`McpInvokeTool`）、构建（`CargoBuildTool`、`NpmRunTool`）、分析（`AnalyzeTool`、`GraphTool`）等
- **权限分级**：`PermissionLevel::Deny / Ask / Allow` + `ToolPermissions` 路径隔离
- **错误体系**：`ToolErrorKind` 覆盖权限拒绝、执行失败、参数非法、超时、Patch 冲突、Git 错误、网络错误等 17 种错误类型

## 测试

运行工具系统全部测试（含 Shell 白名单安全测试）：

```bash
cargo test -p engine-tool-system -- test_allow_list
cargo test -p engine-tool-system
```

Shell 白名单测试验证：允许 `git status`、`cargo check`、`ls -la`；拒绝 `rm -rf /` 和带元字符的注入命令（如 `echo ; rm -rf /`）。

## 依赖

```toml
[dependencies]
tokio = { version = "1.37", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"
regex = "1.10"
reqwest = { version = "0.12", features = ["stream", "json"] }
syn = { version = "2.0", features = ["full", "visit"] }
petgraph = "0.6"
lsp-types = "0.94"
uuid = { version = "1.10", features = ["v4"] }
which = "6.0"
image = "0.24"
```

内部依赖：`foundation-hash`（SimHash 分片）。

## 关键文件

| 文件 | 说明 |
|------|------|
| `src/mod.rs` | `Tool` trait、`ToolError`、`ToolPermissions` 定义与全部模块导出 |
| `src/shell.rs` | `ShellTool` / `BashTool` / `PowerShellTool`：跨平台 Shell 执行 + 白名单校验 + 元字符过滤 |
| `src/registry.rs` | `ToolRegistry`：工具注册表 |
| `src/edit.rs` | `EditFileTool`：单文件编辑 |
| `src/patch.rs` | `apply_patch` / `fuzzy_match`：带冲突检测的 Patch 应用 |
| `src/mcp.rs` | MCP 协议工具集群（`McpInitTool`、`McpInvokeTool`、`SpawnAgentTool` 等） |
| `src/lsp.rs` | LSP 基础工具集群（定义/引用/悬停） |
