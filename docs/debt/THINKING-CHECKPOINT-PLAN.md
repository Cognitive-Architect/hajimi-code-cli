# THINKING-CHECKPOINT-PLAN — Day 08 Trace Link + Checkpoint Model

> 工单: B-08/15  
> 日期: 2026-05-16  
> 分支: `v3.8.0-batch-1`  
> HEAD: `d697414f42584a0d0c9c85346a6a692e691c4dad`

## 1. 结论

`DEBT-THINKING-UI` 仍保持 `PARTIAL`，但 Day 08 已完成第一步真实契约闭环:

- Trace 链路源码入口已确认: `AgentLoop::trace_tx()` 注入 `AppState.trace_tx`，`subscribe_agent_trace` 订阅 broadcast 后同时发送 Tauri `Channel<TraceEvent>` 与全局事件 `agent:trace`。
- 前端接收入口已确认: `app.js` 订阅 `agent:trace` 并写入 `traceEvents` / trace panel / timeline。
- Checkpoint DTO 已落地到 `src/interface/desktop/src/main.rs`，结构名为 `CheckpointRecord`。
- Checkpoint 存储位置明确为 workspace-local: `<workspace>/.hajimi/checkpoints/*.json`。
- Desktop 现在会把 edit trace 事件，以及 details 大小写无关包含 `checkpoint` 的 `TraceStepType::Store` 事件写成 `CheckpointRecord`。
- Day 08 不实现 restore；`restore_checkpoint` 明确返回未执行错误。
- Day 09 的 `export_checkpoint` / `compare_checkpoints` 合约已定义，但本日不把固定 `{}` / `false` 伪装成实现。

## 2. Trace Receipt

### 源码链路

| 环节 | 证据 |
|---|---|
| 后端 trace sender | `src/interface/desktop/src/main.rs` 中 `trace_tx: Mutex<Option<broadcast::Sender<TraceEvent>>>` |
| 注入点 | `agent_loop.trace_tx()` 后调用 `state.set_trace_tx(tx)` |
| 订阅命令 | `subscribe_agent_trace(on_event: Channel<TraceEvent>, state, app)` |
| 前端事件 | `subscribe_agent_trace` 内 `on_event.send(event.clone())` 和 `app.emit("agent:trace", &event)` |
| 前端接收 | `src/interface/web/app.js` 中 `agent:trace` / `traceEvents` / `renderTrace` |

### 触发方式

真实触发应来自 AgentLoop 运行时发送的 `TraceEvent`。Day 08 未使用模拟 trace，也未新增假事件源。

手动验收步骤:

1. 启动 Tauri dev。
2. 打开 Agent Trace 侧栏。
3. 执行一个真实 agent 命令，使 AgentLoop 产生 Observe/Plan/Act/Store 等事件。
4. 验证前端 trace panel 出现与后端 `TraceEvent` 字段一致的事件。
5. 执行涉及 edit 或 Agent checkpoint store 的动作后，检查 `<workspace>/.hajimi/checkpoints/*.json` 是否产生 `CheckpointRecord`。

### 当前 blocker

`DEBT-THINKING-B08-001`: 本轮没有运行完整 `cargo tauri dev` WebView 实机 smoke，因此 E2E 前端截图/控制台 receipt 暂缺。源码链路和编译检查已完成；实机触发验收留给 Day 09 前置 smoke。

Day 08 收尾补强:

- Store checkpoint 捕获改为大小写无关匹配，覆盖 `AgentLoop` 的 `Storing checkpoint for iteration ...` 和 `WorkflowOrchestrator` 的 `Checkpoint ... after apply` 两类文案。
- 新增 `test_checkpoint_detail_matches_agent_loop_lowercase_store_event`，防止普通 AgentLoop Store checkpoint 事件再次漏捕。
- 前端 trace 订阅不可用或失败时不再自动填充 demo trace，避免把演示数据误判为真实 trace receipt。
- WebView 实机 smoke 仍由 `DEBT-UX-B07-001` / `DEBT-THINKING-B08-001` 追踪，不在 Day 08 中伪装完成。

## 3. Checkpoint DTO

Rust DTO:

- `CheckpointRecord`
- `CheckpointFileRef`
- `CheckpointDiffSummary`
- `CheckpointMetadata`

字段说明:

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | string | checkpoint 主键，当前由 trace event id 派生 |
| `timestamp` | string | RFC3339 时间，来自真实 `TraceEvent.timestamp` |
| `label` | string | UI 可展示标签，例如 `EditApplied iteration 3` |
| `files` | array | 文件列表，Day 08 默认为空，Day 09 从 edit payload/diff 填充 |
| `diff_summary` | object | diff 聚合摘要，Day 08 使用 operation summary/detail |
| `trace_event_ids` | array | 与 checkpoint 关联的 trace event id |
| `metadata` | object | 来源、agent id、iteration、step_type、confidence、schema_version |

JSON 示例:

```json
{
  "id": "chk_trace_3_editapplied_1778912345678",
  "timestamp": "2026-05-16T12:34:05.678Z",
  "label": "EditApplied iteration 3",
  "files": [
    {
      "path": "src/interface/web/app.js",
      "status": "modified",
      "before_hash": null,
      "after_hash": null
    }
  ],
  "diff_summary": {
    "files_changed": 1,
    "hunks": null,
    "additions": null,
    "deletions": null,
    "summary": "24 diff lines reported by trace operation summary"
  },
  "trace_event_ids": ["trace_3_editapplied_1778912345678"],
  "metadata": {
    "source": "desktop-trace",
    "agent_id": null,
    "iteration": 3,
    "step_type": "EditApplied",
    "confidence": 0.94,
    "schema_version": 1
  }
}
```

## 4. 存储位置

选择: workspace-local `<workspace>/.hajimi/checkpoints`.

理由:

- 符合 local-first，checkpoint 随当前 workspace 隔离。
- 不写项目根目录，不污染源码树。
- Day 09 export/compare 需要按 workspace 找到同一组 checkpoint。
- Tauri app data 更适合全局配置；checkpoint 与代码工作区强绑定，放 workspace 更可解释。

当前 desktop workspace 由 `get_workspace_dir(app_handle)` 提供，实际目录为用户文档目录下的 `hajimi-workspace`。

## 5. Day 09 Export Contract

`export_checkpoint(id: String) -> Result<String, String>`

计划返回:

- `id == "all"`: 返回 `CheckpointExportBundle` JSON。
- 单个 id: 返回单个 `CheckpointRecord` JSON。

建议 bundle:

```json
{
  "schema_version": 1,
  "exported_at": "2026-05-16T12:40:00Z",
  "workspace": "F:/path/to/workspace",
  "checkpoints": []
}
```

Day 08 当前行为: 返回明确错误，避免固定 `{}` 假实现。

## 6. Day 09 Compare Contract

`compare_checkpoints(id_a: String, id_b: String) -> Result<bool, String>`

Day 09 应升级为结构化返回，建议新增:

```json
{
  "id_a": "chk_a",
  "id_b": "chk_b",
  "same": false,
  "files_added": [],
  "files_removed": [],
  "files_modified": [],
  "summary": "2 files modified"
}
```

为保持现有 Tauri 命令兼容，Day 09 可先保留 bool API，同时新增 `compare_checkpoint_details`。

Day 08 当前行为: 返回明确错误，避免固定 `false` 假实现。

## 7. Day 10 Restore / Replay Contract

Day 08 不实现 restore。

Day 10 restore 前置条件:

- 必须有 dry-run。
- 必须创建恢复前 backup checkpoint。
- 必须列出将覆盖/删除/新增的文件。
- 必须要求用户确认。
- 必须拒绝 workspace 外路径。
- restore 失败必须可回滚或至少保留 backup。

当前 `restore_checkpoint` 返回错误: `restore_checkpoint is intentionally deferred to Day 10; no restore is performed`。

## 8. 验证命令

执行/应执行:

```powershell
rg -n "subscribe_agent_trace|trace_tx|agent:trace" src/interface/desktop/src/main.rs src/interface/web/app.js
rg -n "Checkpoint.*Dto|CheckpointRecord|checkpoint_store_dir|\\.hajimi|export_checkpoint|compare_checkpoints|restore_checkpoint" src/interface/desktop/src/main.rs docs/debt/THINKING-CHECKPOINT-PLAN.md
cargo fmt -- --check
cargo check -p hajimi-desktop
cargo test -p hajimi-desktop
node --check src/interface/web/app.js
rg -n "renderDemoTraceCards" src/interface/web/app.js
```

## 9. 范围外

- Day 09: export / compare 的真实文件读取和差异计算。
- Day 09: 从 edit payload 提取文件路径、hunk、additions、deletions。
- Day 10: restore / replay 的 dry-run、确认、backup、回滚。
- 实机 WebView trace receipt: 当前记录为 blocker，不清偿 `DEBT-THINKING-UI`。
