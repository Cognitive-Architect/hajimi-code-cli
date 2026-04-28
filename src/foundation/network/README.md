# Network Module

HAJIMI IDE 地基层的 WebSocket 服务器，基于 Tokio 与 `tokio-tungstenite` 实现，采用 JSON-RPC 2.0 作为应用层协议。

## 职责

- **WebSocket 连接管理**：`WebSocketServer` 绑定 TCP 端口（默认 `127.0.0.1:8080`），接受异步连接并分配唯一 `connection_id`；`ConnectionManager` 通过 `RwLock<HashMap>` 维护活跃连接池，支持最大连接数限制。
- **心跳与超时检测**：每个连接启动独立的 `tokio::spawn` 任务，通过 `tokio::select!` 同时处理：
  - 定时 Ping 心跳（默认 30 秒间隔）
  - Pong 响应超时检测（默认 60 秒无响应断开）
  - 内部消息通道（`mpsc::UnboundedSender<WsMessage>`）转发
  - 客户端 WebSocket 帧接收与处理
- **JSON-RPC 2.0 协议解析**：`protocol.rs` 定义标准请求/响应/错误结构（`JsonRpcRequest`、`JsonRpcResponse`、`JsonRpcError`），包含标准错误码（`-32700` ~ `-32000`）；消息反序列化失败时返回 `ParseError`。
- **连接状态机**：`ConnectionState` 定义 `Connecting / Connected / Reconnecting / Disconnected` 四种状态，支持 `Display` 输出。
- **指数退避重连策略**：`ExponentialBackoff` 提供可配置的重连间隔（初始 1 秒，最大 30 秒，倍数 2.0，最大重试 10 次）。
- **Handler 注册表**：通过 `HandlerRegistry` 将 JSON-RPC `method` 路由到具体业务处理器，实现请求分发。

## 关键文件

- `src/lib.rs` — `WebSocketServer`、`ConnectionManager`、`Connection` 核心实现
- `src/protocol.rs` — JSON-RPC 2.0 协议类型、错误码、`ServerConfig`、`ExponentialBackoff`
- `src/handlers.rs` — `HandlerRegistry` 与请求路由
- `tests/type_verification.rs` — Rust/TypeScript 类型一致性验证测试

## 快速开始

```rust
use ws_server::{WebSocketServer, protocol::ServerConfig};

let config = ServerConfig::default();
let server = WebSocketServer::new(config, handler_registry);
server.run().await.unwrap();
```

## 测试

```bash
# 运行 WebSocket 服务器模块测试
cargo test -p ws_server
```

## 依赖

- `tokio` — 异步运行时（TCP Listener、spawn、select、time）
- `tokio-tungstenite` — WebSocket 协议实现（`accept_async`、`Message`、`SinkExt` / `StreamExt`）
- `serde` / `serde_json` — JSON-RPC 序列化与反序列化
- `futures` — `SinkExt`、`StreamExt` 扩展
- `thiserror` — 协议错误类型定义
- `tracing` / `tracing-subscriber` — 结构化日志（`info!`、`warn!`、`error!`、`debug!`）
