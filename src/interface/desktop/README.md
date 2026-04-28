# Desktop Backend Module

HAJIMI IDE 的 Tauri v2 桌面后端，负责桥接 Rust 引擎与纯 HTML/CSS/JS 前端。

## 职责

- **工具注册中心**：在启动时注册 38+ 个引擎层工具（文件操作、Git、LSP、MCP、构建、搜索等），通过 `ToolRegistry` 暴露给前端调用。
- **LLM 提供商管理**：支持 Anthropic、OpenAI、Ollama 及自定义 OpenAI-compatible 提供商；API Key 使用 OS Keyring（Windows Credential Manager / macOS Keychain / Linux Secret Service）安全存储，`providers.json` 仅保存元数据。
- **流式聊天**：通过 Tauri Channel 向前端推送 LLM 流式响应（`StreamEvent`），集成审计日志记录每次调用的状态。
- **工作目录沙箱**：文件读写操作限定在用户文档目录下的 `hajimi-workspace` 内，禁止 `..` 路径穿越，Unix 权限 `0o600` / Windows `icacls` ACL 限制。
- **命令白名单**：Shell 命令仅允许 `git`、`cargo`、`npm`、`node`、`python3` 等 15 个白名单命令；参数过滤元字符 `;`、`&`、`|` 等。
- **Profile 系统**：支持多 Profile 隔离配置与密钥，全局/工作区两级配置覆盖。
- **备份加密**：使用 AES-256-GCM + PBKDF2（100,000 轮）加密导出文件。

## 关键文件

- `src/main.rs` — Tauri 命令注册、AppState、LLM 客户端初始化
- `src/audit.rs` — 密钥使用审计日志（`audit.jsonl`，自动轮转 10MB）

## 快速开始

```bash
# 开发模式（前端自动从 src/interface/web/ 加载）
cd src/interface/desktop && cargo tauri dev

# 构建 release
cd src/interface/desktop && cargo tauri build
```

## 测试

```bash
# 运行桌面后端单元测试
cargo test -p hajimi-desktop
```

## 依赖

- `tauri` / `tauri-build` — 桌面应用框架
- `tokio` — 异步运行时
- `engine-tool-system` — 工具注册与执行（引擎层）
- `engine-llm-core` — LLM 客户端抽象（引擎层）
- `keyring` / `secrecy` — OS Keyring 安全存储与内存脱敏
- `aes-gcm` / `pbkdf2` / `sha2` / `rand` — 备份加密
- `chrono` — 时间戳与审计日志
- `serde` / `serde_json` — 序列化
