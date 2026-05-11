# DEBT-LINES-GRAPH-001: Graph核心引擎实现

## 债务申报

**申报日期**: 2026-04-14  
**申报人**: Agent A (Architect)  
**工单编号**: B-05/A

## 当前状态

- **当前行数**: 324行 (graph.rs)
- **目标行数**: 320±5行 (315-325)
- **状态**: ✅ 符合初始标准

## 交付物清单

### 文件1: `src/intelligence/memory/src/graph.rs` (324行)
- [x] `pub fn extract_entities` - Regex+Heuristic混合NER
- [x] `pub fn store_entity` - SQLite持久化
- [x] `pub struct KnowledgeGraph` - 图存储核心结构
- [x] `impl Serialize for Entity` - Serde序列化
- [x] `impl Deserialize for Entity` - Serde反序列化
- [x] 14项测试用例实现

### 文件2: `src/intelligence/memory/src/mod.rs` (114行)
- [x] 新增接口导出 (+21行，目标+20行)
- [x] 类型别名定义 (`GraphResult`, `EntityResult`)
- [x] 模块常量定义

### 依赖更新: `src/intelligence/memory/Cargo.toml`
- [x] 添加 `uuid = { version = "1.6", features = ["v4", "serde"] }`
- [x] 添加 `regex = "1.10"`

## 正则验证结果

| 验证项 | 命令 | 结果 |
|--------|------|------|
| V1 | `grep -E "pub fn extract_entities\|pub fn store_entity\|pub struct KnowledgeGraph"` | ✅ 命中3处 |
| V2 | `grep -E "impl.*Entity.*Serialize\|impl.*Entity.*Deserialize"` | ✅ 命中2处 |
| V3 | `grep -c "reqwest::\|hyper::client\|openai::"` | ✅ 0处 |

## 刀刃表自测 (14项)

| ID | 类别 | 验证命令 | 通过标准 | 状态 |
|:---|:---|:---|:---|:---:|
| FUNC-001 | FUNC | `cargo test test_extract_entities_basic` | 返回Vec长度≥1 | [ ] 待环境恢复 |
| FUNC-002 | FUNC | `cargo test test_confidence_scoring` | confidence∈[0.0,1.0] | [ ] 待环境恢复 |
| FUNC-003 | FUNC | `cargo test test_persist_entity_sqlite` | 插入后SELECT成功 | [ ] 待环境恢复 |
| CONST-001 | CONST | `cargo test test_entity_serde_roundtrip` | 序列化→反序列化相等 | [ ] 待环境恢复 |
| NEG-001 | NEG | `cargo test test_empty_text_no_panic` | 空字符串返回Ok([]) | [ ] 待环境恢复 |
| NEG-002 | NEG | `cargo test test_malformed_unicode_handling` | 不panic，返回Err | [ ] 待环境恢复 |
| NEG-003 | NEG | `cargo test test_concurrent_entity_insert` | 100线程插入无竞态 | [ ] 待环境恢复 |
| UX-001 | UX | `cargo test test_entity_display_format` | Display trait符合"{label}@{span}" | [ ] 待环境恢复 |
| E2E-001 | E2E | `cargo test test_graph_session_integration` | Session→Graph→查询端到端 | [ ] 待环境恢复 |
| E2E-002 | E2E | `cargo test test_entity_search_latency` | 1K实体查询<50ms | [ ] 待环境恢复 |
| High-001 | High | `cargo test test_entity_relationship_cyclic` | 循环引用检测不栈溢出 | [ ] 待环境恢复 |
| High-002 | High | `cargo test test_large_graph_memory_usage` | 10K节点Peak RSS<150MB | [ ] 待环境恢复 |
| RG-001 | RG | `cargo test test_entity_update_idempotent` | 重复插入UUID不变 | [ ] 待环境恢复 |
| RG-002 | RG | `cargo test test_graph_edge_consistency` | 边删除后节点同步更新 | [ ] 待环境恢复 |

## 地狱红线检查

| 红线项 | 状态 | 说明 |
|--------|------|------|
| 行数欺诈 | ✅ 通过 | 324行 ∈ [315-325] |
| 架构破坏 | ✅ 通过 | Gateway未修改 |
| 外部依赖违规 | ✅ 通过 | 无reqwest/hyper/openai |
| 算法未实现 | ✅ 通过 | NER已实现完整逻辑 |
| 序列化缺失 | ✅ 通过 | Entity已实现Serialize/Deserialize |
| 并发灾难 | ⏳ 待验证 | 代码已使用Arc<Mutex<>>保护 |
| 内存泄漏 | ⏳ 待验证 | 代码无泄漏风险模式 |
| 底座污染 | ⏳ 待验证 | 等待网络恢复运行测试 |
| 熔断逃避 | ✅ 通过 | 首次提交符合标准 |
| 债务隐瞒 | ✅ 通过 | 已申报本DEBT文件 |

## 阻塞原因

当前环境网络限制 (`SSL connect error`)，无法连接到 crates.io 下载以下新依赖：
- `uuid = "1.6"`
- `regex = "1.10"`

## 清偿计划

1. **P0-网络恢复**: 等待网络连接恢复后运行完整测试套件
2. **P1-底座验证**: 验证46/46现有测试无失败
3. **P2-刀刃验证**: 运行14项自测并更新状态
4. **P3-代码审查**: 审查人进行架构Review

## 代码实现摘要

### NER实现 (Regex+Heuristic)
```rust
pub fn extract_entities(text: &str) -> Result<Vec<Entity>, NerError>
```
- 支持人名、组织、产品实体识别
- Heuristic置信度计算（长度、大小写奖励）
- 空输入和非法Unicode处理

### 图存储实现
```rust
pub struct KnowledgeGraph {
    pub nodes: HashMap<Uuid, Node>,
    pub edges: HashMap<(Uuid, Uuid), Edge>,
    db: Arc<Mutex<rusqlite::Connection>>,
}
```
- SQLite内存/文件存储
- 循环引用检测
- 并发安全（Arc<Mutex<>>）

### Serde实现
- 手动实现 `Serialize` 和 `Deserialize` for `Entity`
- Uuid字符串序列化
- 错误处理完善
