# Crates 目录

保留的 Rust Crate 目录，用于存放需要独立 crate 形态或对外提供 C ABI / WASM 绑定的模块，同时保持与主 workspace 的兼容。

## 职责

- **`hajimi-codex-twist`**：AI 记忆核心（Codex Thread/Turn 架构）的 Rust 移植层，crate-type 为 `["cdylib", "rlib"]`，可编译为动态库供外部调用或作为 rlib 被 Rust 代码依赖
- **Workspace 兼容**：`Cargo.toml` 使用 `version.workspace = true`、`edition.workspace = true` 等字段，与主 workspace 保持版本一致
- **本地存储适配**：轻量级 Codex Thread/Turn 架构，支持 LCR（Local Context Repository）本地存储格式
- **对外重导出**：`lib.rs` 已标记为 `#[deprecated(since = "3.8.0")]`，建议直接使用 `intelligence::codex_twist`；当前保留向后兼容的 `pub use intelligence::codex_twist::*`

## 测试

运行 codex-twist crate 测试：

```bash
cargo test -p codex-twist
```

## 依赖

`hajimi-codex-twist/Cargo.toml`：

```toml
[dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
tokio = { workspace = true }
once_cell = "1.19"
unicode-segmentation = "=1.10.1"
zstd = "0.13"
memmap2 = "0.9"
lru = "0.12"
```

内部依赖：`intelligence`（通过 `pub use intelligence::codex_twist::*`）。

## 目录结构

```
src/crates/
└── hajimi-codex-twist/
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

## 使用说明

> **注意**：`hajimi-codex-twist` 自 3.8.0 起已标记为 deprecated。新项目请直接依赖 `intelligence::codex_twist`。

如需保留独立 crate 形态编译：

```bash
cargo build -p codex-twist --release
```
