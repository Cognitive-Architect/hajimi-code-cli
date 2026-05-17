# THINKING-RESTORE-REPLAY-VERIFY — Day 10 Safe Restore + Replay Binding

> 工单: B-10/15  
> 日期: 2026-05-17  
> 分支: `v3.8.0-batch-1`  
> HEAD: `d697414f42584a0d0c9c85346a6a692e691c4dad`

## 1. 结论

Day 10 已完成安全 V1 restore/replay 闭环:

- `restore_checkpoint(id, confirmRestore, dryRun)` 真实读取 workspace-local checkpoint。
- 后端支持 dry-run restore plan；写入 restore 必须 `confirmRestore == true`。
- 写入前必须先生成 backup 到 `<workspace>/.hajimi/checkpoints/backups/restore_<id>_<timestamp>/...`。
- 每个 restore path 都通过 `resolve_workspace_path` / `PathIntent` 派生的 `resolve_restore_target`，拒绝 traversal、workspace 外路径、目录目标和缺失父目录。
- 没有内容快照的 checkpoint 只允许 dry-run，真实写入会被拒绝，不用 hash/summary 伪造文件内容。
- 前端 restore 改为先 dry-run，再展示文件数量、backup 路径、warnings，用户确认后才发起写入 restore。
- Checkpoint 面板新增 checkpoint replay，replay event 来源为真实 `list_checkpoints` 返回数据；Edit History replay 继续使用真实 `get_edit_history`。

## 2. Restore API

命令:

```text
restore_checkpoint(id: String, confirm_restore: bool, dry_run: Option<bool>) -> RestoreResult
```

Dry-run 示例:

```json
{
  "checkpoint_id": "chk_trace_3_editapplied_1778912345678",
  "restored_at": "2026-05-17T12:00:00Z",
  "dry_run": true,
  "backup_dir": "C:\\Users\\<user>\\Documents\\hajimi-workspace\\.hajimi\\checkpoints\\backups\\restore_chk_trace_3_editapplied_1778912345678_1778990000000",
  "files": [
    {
      "path": "src/interface/web/app.js",
      "action": "write",
      "target_exists": true,
      "backup_path": "C:\\Users\\<user>\\Documents\\hajimi-workspace\\.hajimi\\checkpoints\\backups\\restore_chk_trace_3_editapplied_1778912345678_1778990000000\\src\\interface\\web\\app.js",
      "reason": "checkpoint status 'modified'"
    }
  ],
  "warnings": []
}
```

Confirmed restore 成功后:

```json
{
  "checkpoint_id": "chk_trace_3_editapplied_1778912345678",
  "restored_at": "2026-05-17T12:00:02Z",
  "dry_run": false,
  "backup_dir": "C:\\Users\\<user>\\Documents\\hajimi-workspace\\.hajimi\\checkpoints\\backups\\restore_chk_trace_3_editapplied_1778912345678_1778990000000",
  "files": [],
  "warnings": []
}
```

## 3. 安全路径

已覆盖失败路径:

| 场景 | 行为 |
|---|---|
| missing checkpoint | `checkpoint not found: <id>` |
| `confirmRestore == false` 且非 dry-run | `restore refused: confirmRestore must be true for write restore` |
| checkpoint 无 file-level 数据 | `checkpoint <id> has no file-level restore data` |
| checkpoint file 无内容快照 | dry-run 返回 warning；真实 restore 拒绝 |
| unsafe path / traversal | `restore path rejected for '<path>'` |
| missing parent | restore plan 阶段拒绝，写入前停止 |
| backup 失败 | 写入前停止 |
| 写入失败 | 尝试用 backup 回滚已处理目标 |

## 4. Replay 数据源

| UI | 数据源 | 说明 |
|---|---|---|
| Checkpoint 回放 | `list_checkpoints` 返回的 `CheckpointRecord` | 生成单条 replay event，包含 `checkpoint_id`、`trace_event_ids`、`diff_summary`、`files` |
| Edit History 回放 | `get_edit_history` 返回的真实 edit history | 保留 `checkpoint_id` 并在 replay entry 中展示 source |
| Trace panel | `traceEvents` | 仍来自 `agent:trace` / `subscribe_agent_trace` |

未新增 mock/fake replay timeline。

## 5. 自动化验证

已执行:

```powershell
node --check src/interface/web/app.js
cargo fmt -- --check
cargo check -p hajimi-desktop
cargo test -p hajimi-desktop
rg -n "fn restore_checkpoint|confirm_restore|confirmRestore|backup|dry_run|dryRun|resolve_restore_target|resolve_workspace_path|replayCheckpoint" src/interface/desktop/src/main.rs src/interface/web/app.js
```

结果:

- `node --check`: 通过。
- `cargo fmt -- --check`: 通过。
- `cargo check -p hajimi-desktop`: 普通沙盒因 Windows `target/debug/incremental` ACL 失败；提升权限复验通过。
- `cargo test -p hajimi-desktop`: 复验通过，21 passed。
- restore/replay rg 证据: 有命中。

新增/覆盖测试:

- `test_restore_plan_rejects_unsafe_path`
- `test_restore_plan_warns_without_content_snapshot`
- `test_apply_restore_plan_backs_up_and_writes_content_snapshot`
- `test_restore_confirmation_required_for_write_restore`
- `test_restore_plan_rejects_missing_parent_before_writes`

## 6. 债务声明

`DEBT-THINKING-B10-001`: restore 原子性为 V1。当前策略是全量预检、写入前 backup、失败时 best-effort rollback；尚不是跨文件事务日志式原子提交。

`DEBT-THINKING-B10-002`: 当前 Day 08/09 生成的 checkpoint 多数没有文件内容快照，因此真实写入 restore 会被安全拒绝，只能 dry-run。后续需要在生成 checkpoint 时填充 `content` 或 `after_content`。

`DEBT-THINKING-B10-003`: WebView 实机 replay/restore 点击流未完成截图验收，仍需真实桌面 smoke。

## 7. 回滚策略

如 restore 误覆盖用户文件:

1. 查找返回的 `backup_dir`。
2. 按 backup 内相对路径复制回 workspace 对应路径。
3. 若代码层面回滚，回退 `src/interface/desktop/src/main.rs` 和 `src/interface/web/app.js`。
