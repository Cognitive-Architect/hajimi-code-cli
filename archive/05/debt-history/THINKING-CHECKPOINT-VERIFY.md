# THINKING-CHECKPOINT-VERIFY — Day 09 Export / Compare + Operation Diff

> 工单: B-09/15  
> 日期: 2026-05-16  
> 分支: `v3.8.0-batch-1`  
> HEAD: `d697414f42584a0d0c9c85346a6a692e691c4dad`

## 1. 结论

Day 09 已把 checkpoint export/compare 从占位推进为真实可审计输出:

- `export_checkpoint(id)` 读取 workspace-local `<workspace>/.hajimi/checkpoints/*.json`，支持单个 checkpoint 和 `id == "all"` bundle。
- `compare_checkpoints(id_a, id_b)` 返回结构化 `CheckpointCompareResult`，包含 `files_added` / `files_modified` / `files_removed` / `summary` / `data_source`。
- 找不到 checkpoint 会返回明确错误: `checkpoint not found: <id>`。
- 前端 checkpoint 面板新增最小 compare 展示，export/compare 错误会显示 toast，不吞掉后端错误。
- Operation Summary 的 diff preview 不再生成 `新建文件 #1` / `修改文件 #1` 这类虚拟 diff；没有文件级数据时明确显示数据源为 `TraceEvent.operation_summary` 和降级说明。
- Day 09 未实现 restore，`restore_checkpoint` 仍保持 Day 10 延后错误。

## 2. Export 示例

单个 checkpoint:

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
      "after_hash": "sha256:after"
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

全部导出 bundle:

```json
{
  "schema_version": 1,
  "exported_at": "2026-05-16T12:40:00Z",
  "workspace": "C:\\Users\\<user>\\Documents\\hajimi-workspace",
  "checkpoints": []
}
```

## 3. Compare 示例

```json
{
  "id_a": "chk_before",
  "id_b": "chk_after",
  "same": false,
  "files_added": [
    {
      "path": "added.txt",
      "before_status": null,
      "after_status": "created",
      "before_hash": null,
      "after_hash": "new"
    }
  ],
  "files_removed": [
    {
      "path": "removed.txt",
      "before_status": "modified",
      "after_status": null,
      "before_hash": "old",
      "after_hash": null
    }
  ],
  "files_modified": [
    {
      "path": "changed.txt",
      "before_status": "modified",
      "after_status": "modified",
      "before_hash": "old",
      "after_hash": "new"
    }
  ],
  "summary": "1 added, 1 modified, 1 removed",
  "data_source": "checkpoint.files"
}
```

当 Day 08/09 checkpoint 仍没有文件级 `files` 时，compare 不伪造文件 diff，会返回:

```json
{
  "same": false,
  "files_added": [],
  "files_removed": [],
  "files_modified": [],
  "summary": "No file-level diff data; checkpoint summary or trace metadata changed",
  "data_source": "checkpoint.diff_summary+metadata"
}
```

## 4. 数据来源

| 功能 | 来源 | 说明 |
|---|---|---|
| export single | `<workspace>/.hajimi/checkpoints/<id>.json` | 真实 checkpoint 文件反序列化后 pretty JSON 输出 |
| export all | `<workspace>/.hajimi/checkpoints/*.json` | 按 timestamp 倒序读取后 bundle 输出 |
| compare file diff | `CheckpointRecord.files` | 以 path 为 key，比较 status / before_hash / after_hash |
| compare fallback | `diff_summary` + `trace_event_ids` + `metadata` | 无文件级数据时明确降级，不生成假 diff |
| Operation Summary preview | `TraceEvent.operation_summary` | 仅展示真实字段或明确“无文件级 diff 数据” |

## 5. 验证记录

已执行:

```powershell
node --check src/interface/web/app.js
cargo fmt -- --check
cargo check -p hajimi-desktop
cargo test -p hajimi-desktop
cargo check --workspace
git diff --check
rg -n 'Ok\("\{\}"\)|Ok\(false\)' src/interface/desktop/src/main.rs
rg -n 'fn export_checkpoint|fn compare_checkpoints|CheckpointCompareResult|files_added|files_modified|files_removed|export_checkpoint|compare_checkpoints|renderOperationDiffPreview' src/interface/desktop/src/main.rs src/interface/web/app.js
```

结果:

- `node --check`: 通过。
- `cargo fmt -- --check`: 通过。
- `cargo check -p hajimi-desktop`: 普通沙盒因 Windows `target/debug/incremental` ACL 失败；提升权限复验通过。
- `cargo test -p hajimi-desktop`: 普通沙盒因 Windows `target/debug/incremental` ACL 失败；提升权限复验通过，16 passed。
- `cargo check --workspace`: 普通沙盒因同类 ACL 问题失败；提升权限复验通过。
- `git diff --check`: 通过，仅 CRLF warning。
- 固定 `{}` / `false` 扫描: 无命中。
- export/compare/Operation Diff 证据: 有命中。

环境说明:

- Windows incremental 编译目录 ACL 仍会使普通沙盒 cargo 写入失败；本轮已用提升权限复验排除源码失败。

## 6. 债务声明

`DEBT-THINKING-B09-001`: diff 粒度当前是 V1 summary。若 checkpoint `files` 为空，只比较 `diff_summary` / trace metadata，不伪造文件级 diff。Day 10 或后续应从 edit payload / git diff / file hash 生成更完整的 `CheckpointRecord.files`。

`DEBT-THINKING-B09-002`: 本轮未完成 Tauri WebView 实机 smoke，前端 compare/export UI 仍需在真实桌面会话中点验。

## 7. Day 10 前置

restore/replay 仍必须满足:

- dry-run。
- 恢复前 backup checkpoint。
- 用户确认。
- workspace path resolver。
- 失败可审计，不硬编码成功。
