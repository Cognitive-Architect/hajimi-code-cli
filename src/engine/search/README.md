# Search 模块

HAJIMI IDE 的全文搜索引擎，基于 Tantivy 构建 16 分片索引架构，支持 SimHash-64 路由与 Jieba 中文分词。

## 职责

- **16 分片 Tantivy 索引**：`TantivyIndexManager` 管理 16 个独立分片（`ShardIndex`），每个分片包含独立的 `Index`、`IndexWriter`、`IndexReader`
- **SimHash-64 路由**：文档按 `id` 的 SimHash 值均匀路由到对应分片，确保相同 `id` 始终命中同一分片，1000 条文档测试验证分片分布均匀且无空分片
- **中文分词支持**：自定义 `JiebaTokenizer` 实现 Tantivy `Tokenizer` trait，集成 `jieba-rs` 进行中文切词；`has_chinese` 检测自动识别中英文混合内容
- **Schema 设计**：`id`（STRING | STORED）、`title` / `body` / `code`（Text + Jieba 分词 + 存储）、`symbol`（STRING | STORED）、`shard_id`（INDEXED | STORED）
- **异步文档管理**：`add_document` 按 SimHash 路由写入对应分片并实时 commit + reload；`commit_all` 批量提交全部分片

## 测试

运行搜索模块全部测试：

```bash
cargo test -p hajimi-engine
```

测试覆盖 SimHash 路由一致性、分片均匀分布、Jieba 分词正确性、Manager 创建与文档写入、分片访问边界。

## 依赖

```toml
[dependencies]
foundation-hash = { path = "../../foundation/hash" }
tantivy = "0.21"
jieba-rs = "0.6"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
lru = "0.12"
```

`dev-dependencies`：`tempfile`（测试用临时目录）。

## 关键文件

| 文件 | 说明 |
|------|------|
| `src/tantivy_index.rs` | `TantivyIndexManager`、`ShardIndex`、`JiebaTokenizer`、`JiebaTokenStream` 实现 |
| `src/debug_test.rs` | 调试二进制入口 |

## 架构

```
┌─────────────────────────────┐
│   TantivyIndexManager       │
│  ┌─────────────────────┐    │
│  │ ShardIndex[0..15]   │    │
│  │ - index             │    │
│  │ - writer            │    │
│  │ - reader            │    │
│  └─────────────────────┘    │
│  JiebaTokenizer (中文)      │
└─────────────────────────────┘
```
