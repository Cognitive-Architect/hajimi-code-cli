# MCP Server Module

HAJIMI IDE 的 MCP（Model Context Protocol）真实 RPC 服务器，基于 TypeScript 实现，通过 stdio 与外部 Agent 通信。

## 职责

- **15+ 真实 RPC 工具**：暴露 `hajimi_search`、`hajimi_read_file`、`hajimi_grep`、`hajimi_git_status`、`hajimi_run_tests`、`hajimi_build`、`hajimi_security_audit`、`hajimi_chat_with_trace` 等工具，所有工具均为真实功能调用，非包装转发。
- **LCR 本地上下文存储**：基于内存 + 文件持久化的轻量级上下文存储（`LCRStore`），支持全文搜索、添加 chunk、统计信息查询；默认存储路径 `~/.hajimi/lcr.db`。
- **输入安全校验**：
  - 输入长度限制（最大 10KB）
  - 控制字符过滤（`\x00-\x1F`）
  - 路径遍历防护（拒绝 `..`、`~/`、`/etc/passwd` 等敏感路径）
  - Meta 对象原型链污染防护（过滤 `__proto__`、`constructor`）
- **协议兼容**：基于 `@modelcontextprotocol/sdk` 实现 JSON-RPC 2.0 通信，支持 `ListTools`、`CallTool`、`ListResources`、`ReadResource` 标准方法。
- **Trace 支持**：`hajimi_chat_with_trace` 与 `hajimi_agent_run` 提供带思考轨迹的 Agent 交互能力。

## 关键文件

- `server.ts` — MCP Server 主入口、工具定义、请求路由
- `handlers/` — 各工具的具体业务实现（文件读取、Git 操作、测试执行、Agent 启动等）
- `capabilities/` — 工具元数据注册与帮助信息生成
- `protocol/` — JSON-RPC 错误定义与类型
- `transport/` — stdio / SSE 传输适配器

## 快速开始

```bash
# 直接启动 MCP Server（stdio 模式）
node --experimental-specifier-resolution=node src/interface/mcp-server/server.ts

# 或通过 npx ts-node
npx ts-node src/interface/mcp-server/server.ts
```

## 测试

```bash
# 运行 MCP 服务器集成测试
node tests/mcp/server.test.mjs

# 或运行目录内测试
npx ts-node src/interface/mcp-server/__tests__/integration.test.ts
```

## 依赖

- `@modelcontextprotocol/sdk` — MCP 官方 SDK（Server / StdioTransport / Types）
- `fs/promises`、`path`、`os` — Node.js 内置模块（文件操作与路径处理）
- `engine-tool-system`（通过 FFI Bridge）— 复用 Rust 引擎层工具实现
