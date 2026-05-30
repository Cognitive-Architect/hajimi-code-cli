# P0-DEEPSEEK-TOOL-SCHEMA 执行计划 — Day 1~4 每日细化

> **文档版本**: 1.0
> **所属 Roadmap**: [P0-DEEPSEEK-TOOL-SCHEMA-REMEDIATION-ROADMAP.md](./P0-DEEPSEEK-TOOL-SCHEMA-REMEDIATION-ROADMAP.md)
> **关联 Debt**: `docs/debt/DEBT-AGENT-DEEPSEEK-TOOL-SCHEMA-WEB-SEARCH.md`
> **前置条件**: `/agent` legacy fallback masking 已修复；DeepSeek 已暴露真实 400 schema 错误；P0 roadmap 已落地
> **最后更新**: 2026-05-30

---

## 已完成的基线

| Phase | 状态 | 证据 |
|:---|:---:|:---|
| Debt 采样 | 已完成 | UI Trace 暴露 `Invalid schema for function 'web_search'` |
| 根因定位 | 已完成 | `ToolSpecExporter::get_tool_schema("web_search")` 未覆盖，unknown fallback 为 `{}` |
| fallback 修复 | 已完成 | LLM-Native 错误不再被 legacy `read_file(Cargo.toml)` 遮蔽 |
| P0 Roadmap | 已完成 | `docs/roadmap/Hajimi ToolFix/plan/P0-DEEPSEEK-TOOL-SCHEMA-REMEDIATION-ROADMAP.md` |

**当前代码基线**:
- `src/intelligence/agent-core/llm_native/tool_spec.rs` 只为少数工具提供静态 schema。
- `web_search` 由 `src/interface/desktop/src/main.rs` 注册，但 schema exporter 未覆盖。
- `src/engine/llm-core/src/openai.rs` 将 `ToolDefinition.parameters` 原样放入 OpenAI-compatible `function.parameters`。
- 已知最近验证栈：`cargo fmt -- --check`、`cargo check -p hajimi-desktop`、`cargo test -p intelligence-agent-core --lib` 曾通过；执行本计划时必须重新跑。

---

## Phase 0: 执行前约束

> **目标**: 防止把 P0 schema 修复做成大重构。先修模型工具 schema 合同，不碰 UI，不隐藏工具，不回到 legacy fallback。

**硬约束**:
- 必须测试优先。
- 不允许通过移除 `web_search` 注册来假装修复。
- 不允许吞掉 provider 400 后 fallback 到 legacy。
- 不允许让 `/agent` 重新出现 `read_file(Cargo.toml)` 遮蔽错误。
- 不允许 Interface 层被 Intelligence/Engine 反向依赖。
- 未实机验证 DeepSeek 前，不得把 debt 状态写成 `FIXED / VERIFIED`。

**通俗版**: 这次不是把坏标签撕掉，而是把工具箱里每张标签写成 DeepSeek 看得懂的格式。不能把工具藏起来，也不能把真实错误再盖住。

---

## Phase 1: Schema Exporter 修复（Day 1）

> **目标**: `ToolSpecExporter` 导出的每个模型可见工具参数，都必须是 DeepSeek-compatible JSON Schema object。先补 `web_search`，再补 unknown fallback。

---

### Day 1: ToolSpecExporter 测试优先 + `web_search` schema + unknown fallback

**预计工时**: 4-6 小时
**风险等级**: P0 / 中高（直接影响 `/agent` 是否能进入模型请求）
**主目标文件**: `src/intelligence/agent-core/llm_native/tool_spec.rs`

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 确认当前失败基线 | `tool_spec.rs` | 先查看 `get_tool_schema` 当前 match，确认 `web_search` 未覆盖，unknown 仍返回 `{}` |
| 2 | 新增 `test_web_search_schema_is_deepseek_compatible_object` | `tool_spec.rs` | 使用 `engine_tool_system::WebSearchTool` 注册到 `ToolRegistry`，导出后断言 schema 顶层 `type == "object"` |
| 3 | 新增 `test_unknown_tool_schema_normalizes_to_object` | `tool_spec.rs` | 注册 `DummyTool("unknown_dummy_tool")`，断言 fallback schema 不是裸 `{}`，必须含 `type:"object"` 和 object `properties` |
| 4 | 新增 `test_parse_schema_safely_normalizes_malformed_json` | `tool_spec.rs` | malformed JSON 不能回到 `{}`，应返回安全 object fallback |
| 5 | 新增 `default_object_schema()` helper | `tool_spec.rs` | 返回 `{ "type": "object", "properties": {}, "additionalProperties": true }` |
| 6 | 新增 `normalize_parameters_schema(Value) -> Value` | `tool_spec.rs` | 缺 `type` 补 `object`；缺 `properties` 补 `{}`；非 object 或 `type != object` 降级为 fallback |
| 7 | 添加 `web_search` 静态 schema | `tool_spec.rs` | schema 包含 `query:string`，`required:["query"]`，`additionalProperties:false` |
| 8 | 导出路径统一 normalization | `tool_spec.rs` | `from_registry` 中 `parameters_schema` 使用 normalized schema |
| 9 | 更新旧测试 `test_missing_schema` | `tool_spec.rs` | 旧预期 `{}` 改为安全 object schema |
| 10 | 跑 targeted tests | 本地命令 | 只跑 tool_spec 相关测试，确认 Day 1 范围闭合 |

#### 关键代码形状

**fallback schema**:
```rust
fn default_object_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {},
        "additionalProperties": true
    })
}
```

**`web_search` schema**:
```rust
"web_search" => serde_json::json!({
    "type": "object",
    "properties": {
        "query": {
            "type": "string",
            "description": "Search query"
        }
    },
    "required": ["query"],
    "additionalProperties": false
}),
```

**统一导出**:
```rust
let schema = Self::normalize_parameters_schema(Self::get_tool_schema(&display_name));
```

#### 验证命令

```bash
cargo fmt -- --check
cargo test -p intelligence-agent-core web_search_schema -- --nocapture
cargo test -p intelligence-agent-core unknown_tool_schema -- --nocapture
cargo test -p intelligence-agent-core parse_schema_safely -- --nocapture
cargo check -p intelligence-agent-core
```

#### Day 1 验收标准

- [ ] `web_search` 导出 schema 顶层为 `type:"object"`。
- [ ] `web_search.properties.query.type == "string"`。
- [ ] unknown tool fallback 不再是 `{}`。
- [ ] malformed raw schema 不再降级为裸 `{}`。
- [ ] `cargo check -p intelligence-agent-core` 通过。
- [ ] 未改 UI、未改 desktop 注册、未隐藏 `web_search`。

#### Day 1 停止条件

- 如果注册 `WebSearchTool` 造成额外构造依赖复杂化，先用 `DummyTool("web_search")` 复现 schema exporter 行为，再把真实 `WebSearchTool` 测试列为 Day 2 补强。
- 如果 normalization 影响大量旧测试，不要批量改预期；先判断旧测试是否依赖“不合格 schema”这个错误行为。

---

## Phase 2: Provider Boundary 防线（Day 2）

> **目标**: 即使 Intelligence 层未来漏出坏 schema，OpenAI-compatible client 发送前也不能把 `type:null` 或非 object parameters 发给 DeepSeek。

---

### Day 2: OpenAI-compatible 请求边界 normalization + 序列化测试

**预计工时**: 4-5 小时
**风险等级**: P0 / 中（Engine 公共请求路径，需保持 OpenAI/DeepSeek/Others 兼容）
**主目标文件**: `src/engine/llm-core/src/openai.rs`

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 定位 mapped_tools 构造点 | `openai.rs` | `stream_chat_with_tools` 中 `"parameters": t.parameters` 是发送边界 |
| 2 | 新增 `normalize_tool_parameters_for_openai` | `openai.rs` | 私有 helper，规则与 Day 1 normalization 一致，但不依赖 agent-core |
| 3 | 修改 mapped_tools | `openai.rs` | `"parameters": normalize_tool_parameters_for_openai(t.parameters)` |
| 4 | 新增 `test_openai_tool_parameters_normalizes_empty_schema` | `openai.rs` | 输入 `{}`，输出含 `type:"object"` |
| 5 | 新增 `test_openai_tool_parameters_normalizes_null_schema` | `openai.rs` | 输入 `Value::Null`，输出 object fallback |
| 6 | 新增 `test_openai_tool_parameters_preserves_valid_schema` | `openai.rs` | 输入 `web_search` schema，断言 `query/required/additionalProperties` 不丢 |
| 7 | 扩展 `test_openai_chat_request_serialization` | `openai.rs` | tools 序列化后 `function.parameters.type == "object"` |
| 8 | 跑 engine targeted tests | 本地命令 | 只跑 openai tool parameter 相关测试 |

#### 关键代码形状

```rust
fn normalize_tool_parameters_for_openai(parameters: serde_json::Value) -> serde_json::Value {
    // Keep this helper local to engine-llm-core.
    // Engine must not depend on agent-core's ToolSpecExporter.
}
```

```rust
"function": {
    "name": t.name,
    "description": t.description,
    "parameters": normalize_tool_parameters_for_openai(t.parameters)
}
```

#### 验证命令

```bash
cargo fmt -- --check
cargo test -p engine-llm-core openai_tool_parameters -- --nocapture
cargo test -p engine-llm-core test_openai_chat_request_serialization -- --nocapture
cargo check -p engine-llm-core
```

#### Day 2 验收标准

- [ ] OpenAI-compatible request boundary 不会发送 `function.parameters.type == null`。
- [ ] `{}`、`null`、非 object schema 都会变成安全 object fallback。
- [ ] 已有合法 schema 字段不丢失。
- [ ] Engine 层没有引入对 Intelligence/Interface 的依赖。
- [ ] `cargo check -p engine-llm-core` 通过。

#### Day 2 停止条件

- 如果 OpenAI proper 与 DeepSeek 对 `additionalProperties` 行为有差异，默认保守：valid schema 保留原值；fallback 用 `additionalProperties:true`。
- 如果测试需要真实网络请求，测试设计错误；本日只做纯序列化/纯函数测试。

---

## Phase 3: Registry 覆盖与文档闭环（Day 3）

> **目标**: 防止只修 `web_search` 后，下一个工具继续被 DeepSeek 拒绝。同步文档时保持诚实：未实机就不写已清债。

---

### Day 3: 代表性 registry 测试 + 文档同步

**预计工时**: 4-6 小时
**风险等级**: 中（主要是测试覆盖和文档同步，生产代码少）
**主目标文件**: `src/intelligence/agent-core/llm_native/tool_spec.rs`、`docs/debt/DEBT-AGENT-DEEPSEEK-TOOL-SCHEMA-WEB-SEARCH.md`、`src/INDEX.md`、`src/ARCHITECTURE.md`

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 新增 `test_exported_registry_schemas_are_all_objects` | `tool_spec.rs` | representative registry 至少含 `WebSearchTool`、`ReadFileTool`、`WriteFileTool`、unknown `DummyTool` |
| 2 | 新增 `test_existing_read_file_schema_preserved` | `tool_spec.rs` | 确认 Day 1 normalization 不破坏 `read_file.path required` |
| 3 | 如暴露其他常见工具 schema 缺失，补最小 schema | `tool_spec.rs` | 仅补和测试暴露直接相关的高频工具，避免一次性大铺开 |
| 4 | 更新 debt 文档实施状态 | `docs/debt/DEBT-AGENT-DEEPSEEK-TOOL-SCHEMA-WEB-SEARCH.md` | 追加 `Implementation Plan / Result`，记录是否已实机验证 |
| 5 | 更新 INDEX | `src/INDEX.md` | `llm_native/tool_spec.rs` 描述加 “DeepSeek-compatible schema normalization” |
| 6 | 更新 ARCHITECTURE | `src/ARCHITECTURE.md` | 记录 LLM-Native Tool Schema Boundary：模型可见工具参数必须是 JSON Schema object |
| 7 | 跑 agent-core 全库测试 | 本地命令 | `cargo test -p intelligence-agent-core --lib` |

#### representative registry 建议

```rust
let mut registry = ToolRegistry::new();
registry.register(Arc::new(engine_tool_system::WebSearchTool::new()));
registry.register(Arc::new(engine_tool_system::ReadFileTool::new()));
registry.register(Arc::new(engine_tool_system::WriteFileTool::new()));
registry.register(Arc::new(DummyTool { name: "unknown_dummy_tool".into(), ... }));
```

注意：
- 不从 `src/interface/desktop/src/main.rs` 调用 `build_registry`，因为那会把 Interface 层拉进 Intelligence 测试。
- 如果 `ReadFileTool::new()` 或 `WriteFileTool::new()` 的构造签名与预期不同，以实际 API 为准。

#### 验证命令

```bash
cargo test -p intelligence-agent-core exported_registry_schemas_are_all_objects -- --nocapture
cargo test -p intelligence-agent-core existing_read_file_schema_preserved -- --nocapture
cargo test -p intelligence-agent-core --lib
rg -n "DeepSeek-compatible schema|Tool Schema Boundary|tool_spec.rs" src/INDEX.md src/ARCHITECTURE.md docs/debt/DEBT-AGENT-DEEPSEEK-TOOL-SCHEMA-WEB-SEARCH.md
```

#### Day 3 验收标准

- [ ] representative registry 内所有导出 schema 顶层均为 object。
- [ ] `read_file` 等已知 schema 的 required 字段不丢失。
- [ ] `cargo test -p intelligence-agent-core --lib` 通过。
- [ ] debt 文档没有提前写 `FIXED / VERIFIED`，除非已经完成实机验证。
- [ ] INDEX/ARCHITECTURE 更新只覆盖 schema contract，不做无关重写。

#### Day 3 停止条件

- 如果 representative registry 暴露多个工具 schema 都缺失，不要一口气补完整 40+ 工具；先保留全局 fallback，再记录“精确 schema 覆盖率债务”。
- 如果 `src/INDEX.md` 或 `src/ARCHITECTURE.md` 有用户未提交改动，先读 diff 后局部追加，不覆盖现有内容。

---

## Phase 4: 全栈验证与实机冒烟（Day 4）

> **目标**: 用 DeepSeek 真机验证 P0 是否解除。若出现下一个工具名，记录新问题；不要把“换了一个错误”说成成功。

---

### Day 4: Cargo 验证 + release 构建 + DeepSeek `/agent` smoke

**预计工时**: 3-5 小时
**风险等级**: 中（涉及本机运行环境、Tauri 打包、provider 配置）
**主目标文件**: 文档状态更新；必要时补充新 debt

#### 任务清单

| # | 任务 | 目标文件/命令 | 代码细节 |
|---:|---|:---|:---|
| 1 | 跑格式检查 | 命令 | `cargo fmt -- --check` |
| 2 | 跑核心编译检查 | 命令 | `cargo check -p intelligence-agent-core`、`cargo check -p engine-llm-core` |
| 3 | 跑桌面编译检查 | 命令 | `cargo check -p hajimi-desktop` |
| 4 | 跑核心测试 | 命令 | `cargo test -p intelligence-agent-core --lib` |
| 5 | 跑 engine targeted tests | 命令 | `cargo test -p engine-llm-core openai_tool_parameters -- --nocapture` |
| 6 | 编译可运行 exe | 命令 | `cargo build -p hajimi-desktop --release` |
| 7 | 如需完整 Tauri 包 | 命令 | 关闭运行中的 `hajimi-desktop.exe` 后执行 `cd src/interface/desktop && cargo tauri build` |
| 8 | DeepSeek 实机 smoke | Hajimi UI | 执行 `/agent 查看当前目录下有什么文件` |
| 9 | 记录 Agent Trace 结果 | debt 文档 | 不再出现 `Invalid schema for function 'web_search'` 才算该项通过 |
| 10 | 若出现新工具 schema 错误 | 新 debt 或更新当前 debt | 记录新工具名、provider error、trace 截图线索 |

#### 验证命令

```bash
cargo fmt -- --check
cargo check -p intelligence-agent-core
cargo check -p engine-llm-core
cargo check -p hajimi-desktop
cargo test -p intelligence-agent-core --lib
cargo test -p engine-llm-core openai_tool_parameters -- --nocapture
cargo build -p hajimi-desktop --release
```

如果走 Tauri 打包：

```bash
cd src/interface/desktop
cargo tauri build
```

#### 实机 smoke 步骤

1. 确认旧 Hajimi 已关闭。
2. 启动最新构建的 `target/release/hajimi-desktop.exe` 或新 Tauri 包。
3. 选择 DeepSeek provider。
4. 执行：
   ```text
   /agent 查看当前目录下有什么文件
   ```
5. 打开 Agent Trace，检查是否还包含：
   ```text
   Invalid schema for function 'web_search'
   ```
6. 记录最终状态：
   - `PASS`: 不再出现 `web_search` schema 400。
   - `PARTIAL`: `web_search` 已消失，但出现另一个工具 schema 400。
   - `FAIL`: 仍然出现 `web_search` schema 400。

#### Day 4 验收标准

- [ ] 所有 cargo 验证命令通过，或失败项有明确记录。
- [ ] release exe 构建成功，或 Tauri build 失败原因明确。
- [ ] DeepSeek smoke 已执行并记录。
- [ ] `web_search` schema 400 不再出现，才能把当前 debt 标记为可关闭。
- [ ] 如果出现新工具 schema 错误，新增/更新 debt，不宣称 P0 完全清除。

#### Day 4 停止条件

- 如果 Tauri build 因 exe 正在运行失败，先停止进程再重试，不要改代码绕过。
- 如果 DeepSeek API key/provider 配置不可用，不要写“验证通过”；写“未实机验证，原因：provider 不可用”。

---

## 测试矩阵总表

| ID | Day | 测试/验证 | 类型 | 必须通过 |
|---|---:|---|---|:---:|
| TS-001 | Day 1 | `test_web_search_schema_is_deepseek_compatible_object` | unit / agent-core | yes |
| TS-002 | Day 1 | `test_unknown_tool_schema_normalizes_to_object` | unit / agent-core | yes |
| TS-003 | Day 1 | `test_parse_schema_safely_normalizes_malformed_json` | unit / agent-core | yes |
| OA-001 | Day 2 | `test_openai_tool_parameters_normalizes_empty_schema` | unit / engine-llm-core | yes |
| OA-002 | Day 2 | `test_openai_tool_parameters_normalizes_null_schema` | unit / engine-llm-core | yes |
| OA-003 | Day 2 | `test_openai_tool_parameters_preserves_valid_schema` | unit / engine-llm-core | yes |
| OA-004 | Day 2 | `test_openai_chat_request_serialization_tools_are_objects` | unit / engine-llm-core | yes |
| TS-004 | Day 3 | `test_exported_registry_schemas_are_all_objects` | unit / agent-core | yes |
| TS-005 | Day 3 | `test_existing_read_file_schema_preserved` | unit / agent-core | yes |
| CK-001 | Day 4 | `cargo fmt -- --check` | formatting | yes |
| CK-002 | Day 4 | `cargo check -p intelligence-agent-core` | compile | yes |
| CK-003 | Day 4 | `cargo check -p engine-llm-core` | compile | yes |
| CK-004 | Day 4 | `cargo check -p hajimi-desktop` | compile | yes |
| CK-005 | Day 4 | `cargo test -p intelligence-agent-core --lib` | regression | yes |
| SM-001 | Day 4 | DeepSeek `/agent 查看当前目录下有什么文件` | real-machine smoke | yes for VERIFIED |

---

## 文件交付清单

### 代码文件

| 文件 | Day | 预期变更 |
|---|---:|---|
| `src/intelligence/agent-core/llm_native/tool_spec.rs` | Day 1 / Day 3 | schema normalization、`web_search` schema、unknown fallback、unit tests |
| `src/engine/llm-core/src/openai.rs` | Day 2 | OpenAI-compatible provider boundary normalization、serialization tests |

### 文档文件

| 文件 | Day | 预期变更 |
|---|---:|---|
| `docs/debt/DEBT-AGENT-DEEPSEEK-TOOL-SCHEMA-WEB-SEARCH.md` | Day 3 / Day 4 | 实施状态、验证结果、是否关闭 debt |
| `src/INDEX.md` | Day 3 | `llm_native/tool_spec.rs` schema normalization 说明 |
| `src/ARCHITECTURE.md` | Day 3 | LLM-Native Tool Schema Boundary 说明 |

---

## 每日提交建议

| Day | commit message 建议 | 是否可单独推送 |
|---:|---|:---:|
| Day 1 | `test(agent-core): cover deepseek tool schema contract` + `fix(agent-core): normalize exported tool schemas` | yes |
| Day 2 | `fix(llm-core): normalize openai tool parameter schemas` | yes |
| Day 3 | `docs(toolfix): record deepseek schema remediation status` | yes |
| Day 4 | `chore(toolfix): record deepseek agent smoke result` | optional |

说明：如果一天内完成 Day 1 + Day 2，可以合并为一个 commit，但提交信息必须说清楚 “agent-core exporter + llm-core boundary” 两层修复。

---

## 最终 Definition of Done

- [ ] `web_search` schema 不再导致 DeepSeek 400。
- [ ] unknown tool fallback 不再导出 `{}` 或 `type:null`。
- [ ] OpenAI-compatible 发送边界不会放出非法 parameters schema。
- [ ] agent-core 和 engine-llm-core 都有针对性测试。
- [ ] full regression 至少跑过 `cargo test -p intelligence-agent-core --lib`。
- [ ] 桌面编译检查 `cargo check -p hajimi-desktop` 通过。
- [ ] 实机 smoke 结果写入 debt。
- [ ] 如果没有实机 smoke，不得关闭 debt，只能写 `IMPLEMENTED / PENDING REAL-MACHINE VERIFICATION`。

---

## 风险与回滚

### 主要风险

- **只修 `web_search`，下一个工具继续被拒**
  Day 1 必须做 unknown fallback，Day 3 必须做 representative registry 测试。

- **Engine 和 Agent-Core normalization 重复**
  这是双保险，不是重复造轮子。Agent-Core 负责导出正确，Engine 负责发送前不违法。

- **fallback schema 过宽**
  `{ additionalProperties: true }` 只能兜底，不代表每个工具都有精确 schema。后续可单独做“全工具精确 schema 覆盖率”计划。

- **实机 smoke 受启动方式影响**
  `cargo build -p hajimi-desktop --release` 不等同完整 Tauri 包。若出现 `localhost:3456 refused`，先处理前端服务或 Tauri build，不要误判为 schema 修复失败。

### 回滚方式

```bash
git revert <commit>
```

如果只想临时止损：
- 保留 `ToolSpecExporter` normalization。
- 暂时不关闭 debt。
- 记录 DeepSeek 最新错误和 Agent Trace。

---

## 下一步

从 Day 1 开始执行。第一刀只动 `src/intelligence/agent-core/llm_native/tool_spec.rs`，先让测试红起来，再让它变绿。不要先碰 UI，也不要为了让 DeepSeek 接受请求而隐藏 `web_search`。
