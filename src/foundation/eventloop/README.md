# EventLoop Module

HAJIMI IDE 地基层的轻量级异步运行时工具库，提供统一的异步任务调度接口。

## 职责

- **跨平台 `spawn` 抽象**：
  - 原生目标（`native` feature）：使用 `tokio::spawn` 多线程调度，返回 `JoinHandle`
  - WASM32 目标（`wasm` feature）：使用 `wasm_bindgen_futures::spawn_local` 单线程调度
- **`block_on` 阻塞执行**：基于 `tokio::runtime::Handle::current().block_on`，仅支持非 WASM 环境。
- **协作式调度**：`yield_now()` 将控制权交还运行时调度器；`sleep(duration)` 基于 `tokio::time::sleep` 的异步延迟。
- **超时包装**：`timeout(duration, future)` 为任意 Future 添加超时限制，超时返回 `Err(())`。
- **任务生命周期管理**：`AbortHandle` 包装 `tokio::task::AbortHandle`，支持 `abort()` 与 `is_finished()` 查询。
- **零上层依赖**：作为 Foundation 层模块，不依赖 Engine / Intelligence / Interface 任何上层 crate。

## 关键文件

- `src/lib.rs` — `spawn`、`block_on`、`yield_now`、`sleep`、`timeout`、`AbortHandle` 实现

## 快速开始

```rust
use foundation_eventloop::{spawn, sleep, timeout};
use std::time::Duration;

// 启动异步任务
let handle = spawn(async { 42 });

// 带超时的异步操作
let result = timeout(Duration::from_secs(5), sleep(Duration::from_millis(100))).await;
```

## 测试

```bash
# 运行事件循环模块测试
cargo test -p foundation-eventloop
```

## 依赖

- `tokio`（`native` feature / 非 WASM）— 异步运行时，使用 workspace 统一版本
- `wasm-bindgen-futures`（`wasm` feature / WASM32）— WebAssembly 异步任务调度

## Feature 标志

- `native`（默认）：启用 `tokio` 多线程支持
- `wasm`：启用 `wasm_bindgen_futures` 单线程支持
