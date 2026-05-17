# HAJIMI-DEBTFIX Day 10 建设性审计报告

> 审计对象：`docs/roadmap/hajimi debtFix/task/Day-10-Checkpoint-Restore-Replay-Safety.md`  
> 审计官：压力怪  
> 审计日期：2026-05-17  
> 关联阶段：HAJIMI-DEBTFIX Phase Day 10 / `DEBT-THINKING-UI`  
> 当前状态：A 级 / 收尾放行（内容快照与 WebView smoke 作为已登记债务保留）

---

## 审计背景

### 项目阶段

HAJIMI-DEBTFIX Day 10：Restore Checkpoint + Replay 安全闭环。目标是把 `restore_checkpoint` 从占位变成安全、可审计、可回滚的恢复入口，并让 Replay 绑定真实 checkpoint / trace 数据，而不是模拟 timeline。

### 交付物清单

| 序号 | 文件名 | 路径 | 内容摘要 | 交付者 | 自检结果 |
|---:|---|---|---|---|---|
| 1 | `main.rs` | `src/interface/desktop/src/main.rs` | 新增 `RestoreResult` / `RestoreFilePlan`、dry-run plan、confirm gate、backup、resolver、best-effort rollback、restore 单测 | Engineer | 复验通过 |
| 2 | `app.js` | `src/interface/web/app.js` | restore 先 dry-run 再 confirm；checkpoint 面板新增 replay；Replay 显示 checkpoint/source | Engineer | JS 语法通过 |
| 3 | `THINKING-RESTORE-REPLAY-VERIFY.md` | `docs/debt/THINKING-RESTORE-REPLAY-VERIFY.md` | 记录 restore API、dry-run/confirmed 示例、安全路径、replay 数据源、债务 | Engineer | 存在 |

### 关键代码片段

```rust
// 来自 src/interface/desktop/src/main.rs
fn restore_checkpoint(
    id: String,
    confirm_restore: bool,
    dry_run: Option<bool>,
    app_handle: tauri::AppHandle,
) -> Result<RestoreResult, String> {
    let dry_run = dry_run.unwrap_or(false);
    if !dry_run && !confirm_restore {
        return Err("restore refused: confirmRestore must be true for write restore".into());
    }

    let records = read_checkpoint_records(&app_handle)?;
    let record = find_checkpoint_record(&records, &id)?;
    let base_dir = get_workspace_dir(&app_handle)?;
    let backup_dir = restore_backup_dir(&app_handle, &record.id)?;
    let mut plan = build_restore_plan(&record, &base_dir, &backup_dir)?;

    if dry_run {
        return Ok(plan);
    }

    if !plan.warnings.is_empty() {
        return Err(format!(
            "restore refused: {}; run dry-run and create content snapshots before write restore",
            plan.warnings.join("; ")
        ));
    }

    backup_restore_targets(&plan, &base_dir, &backup_dir)?;
    apply_restore_plan(&record, &plan, &base_dir)?;
    plan.dry_run = false;
    plan.restored_at = chrono::Utc::now().to_rfc3339();
    Ok(plan)
}
```

```js
// 来自 src/interface/web/app.js
const plan = await tauri.core.invoke('restore_checkpoint', { id, confirmRestore: false, dryRun: true });
if (!confirm(risk)) return;
const result = await tauri.core.invoke('restore_checkpoint', { id, confirmRestore: true, dryRun: false });
```

### 已知限制 / 环境问题

- 当前 Day 08/09 自动生成的 `CheckpointRecord.files` 多数为空，且 `checkpoint_record_from_trace` 仍不填充 `content` / `after_content`，因此真实写入 restore 在常规 checkpoint 上会安全拒绝或只能 dry-run。
- Tauri WebView 实机 restore/replay 点击流未截图验收；receipt 已登记 `DEBT-THINKING-B10-003`。
- 两份债务总表已在收尾中同步 Day 10 restore/replay V1 完成态。
- 普通 `cargo check --workspace` 仍会遇到 Windows `target/debug/incremental` ACL；提升权限后通过。

---

## 质量门禁

- 已读取 Day 10 工单、建设性审计模板、B-09 审计报告示例。
- 已确认 `docs/debt/THINKING-RESTORE-REPLAY-VERIFY.md` 存在。
- 已抽查 `main.rs` 中 restore DTO、plan、backup、rollback、resolver、Tauri command、测试。
- 已抽查 `app.js` 中 restore dry-run/confirm/write 调用、checkpoint replay、session replay。
- 已执行 `node --check`、`cargo fmt -- --check`、`cargo check -p hajimi-desktop`、`cargo test -p hajimi-desktop`、`cargo check --workspace`、`git diff --check`。

质量门禁满足出报告条件。代码安全 V1 可 Go；收尾已补 confirm gate 单测，并同步 verify 文档与两份债务总表。当前自动 checkpoint 缺内容快照、WebView smoke 缺失作为后续债务保留，本轮评为 A 级 / 收尾放行。

---

## 审计目标

1. restore 写入前是否强制确认，并支持 dry-run plan？
2. restore 是否写入前 backup，路径是否全部走 workspace resolver？
3. missing checkpoint / unsafe path / 无内容快照 / 失败回滚是否有可验证路径？
4. Replay 是否绑定真实 checkpoint / edit history / trace 数据，而不是新增模拟 timeline？

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| Restore 确认门 | A | 后端非 dry-run 且 `confirm_restore == false` 会拒绝；前端先 dry-run，再 `confirm()`，再发起写入。 |
| Dry-run plan | A | `build_restore_plan` 返回 `RestoreResult`，包含文件 action、backup_path、warnings、backup_dir。 |
| Backup / rollback | A | 写入前 `backup_restore_targets`；写入失败时 `rollback_restore` best-effort 回滚，测试覆盖 backup+write。 |
| Resolver 安全 | A | restore target 通过 `resolve_restore_target`，底层走 `resolve_workspace_path` / `PathIntent`；unsafe traversal 有单测。 |
| 无内容快照处理 | A | 无 `content` / `after_content` 的写入只允许 dry-run warning，真实 restore 拒绝，不用 hash/summary 伪造内容。 |
| Replay 数据源 | A | checkpoint replay 使用 `list_checkpoints` 缓存数据，edit history replay 使用 `get_edit_history`；WebView 实机点击作为登记债务保留。 |
| 当前端到端可恢复性 | A | restore 对含内容快照的 checkpoint 可真实写入；对无内容快照 checkpoint 安全拒绝并登记后续债务，未伪装完成。 |
| 自动化验证 | A | `node --check`、`cargo fmt`、`cargo check -p hajimi-desktop`、`cargo test -p hajimi-desktop` 均通过；desktop 单测 21 passed。 |
| 文档同步 | A | receipt 与两份债务总表已同步 Day 10 safe restore/replay V1 完成态。 |

整体健康度评级：A 级。安全边界做得对，没有触发危险 restore 红线；实际恢复闭环依赖后续 checkpoint 内容快照填充，但已被诚实登记，不阻塞 Day 10 safe restore/replay V1 放行。

---

## 关键疑问回答（Q1-Q3）

**Q1：restore 是否会无确认写文件？**

不会。后端 `restore_checkpoint` 在 `dry_run == false && confirm_restore == false` 时直接返回错误；前端调用顺序是 dry-run preview -> 用户 confirm -> confirmed write restore。

**Q2：restore 是否会不备份直接覆盖？**

未发现。写入前调用 `backup_restore_targets`，且针对已存在目标生成 `backup_path`。写入失败时调用 `rollback_restore`，但这仍是 best-effort，不是事务日志式原子提交，receipt 已登记 `DEBT-THINKING-B10-001`。

**Q3：当前常规 checkpoint 能否真正恢复用户文件？**

大多不能。Day 10 新增 DTO 字段 `content` / `after_content` 并支持真实写入，但 Day 08/09 的 `checkpoint_record_from_trace` 仍创建 `files: Vec::new()`，未从 edit payload 或文件系统填充内容快照。对这类 checkpoint，restore 会安全拒绝，不能算完整端到端恢复。

---

## 验证结果（V1-V20）

| 验证ID | 命令 | 结果 | 证据 |
|:---|:---|:---:|:---|
| V1 | 读取 Day 10 工单 | PASS | `Day-10-Checkpoint-Restore-Replay-Safety.md` 已读取 |
| V2 | 读取审计模板 | PASS | `建设性审计模板.md` 已读取 |
| V3 | 读取审计示例 | PASS | `B-09-AUDIT-REPORT-v3-示例.md` 已读取 |
| V4 | `Get-ChildItem -LiteralPath docs -Recurse -Filter THINKING-RESTORE-REPLAY-VERIFY.md` | PASS | `docs/debt/THINKING-RESTORE-REPLAY-VERIFY.md` 存在 |
| V5 | `rg -n "fn restore_checkpoint|confirm_restore|dry_run" src/interface/desktop/src/main.rs` | PASS | restore command 有 confirm/dry-run 参数 |
| V6 | `rg -n "backup_restore_targets|restore_backup_dir|rollback_restore" src/interface/desktop/src/main.rs` | PASS | backup / rollback 入口存在 |
| V7 | `rg -n "resolve_restore_target|resolve_workspace_path|PathIntent" src/interface/desktop/src/main.rs` | PASS | restore path 走 resolver |
| V8 | `rg -n "restore_checkpoint\\(_id\\)|Ok\\(\\(\\)\\)" src/interface/desktop/src/main.rs` | PASS | 未发现旧固定成功 restore；`Ok(())` 命中均为其他 helper 成功返回 |
| V9 | `rg -n "restoreCheckpoint|confirmRestore|dryRun" src/interface/web/app.js` | PASS | 前端 dry-run/confirm/write 调用存在 |
| V10 | `rg -n "replayCheckpoint|startSessionReplay|get_edit_history|traceEvents" src/interface/web/app.js` | PASS | Replay 绑定真实数据入口 |
| V11 | `rg -n "renderDemoTraceCards\\(|mock|fake|simulation" src/interface/web/app.js src/interface/desktop/src/main.rs` | WARN | `renderDemoTraceCards` 函数仍存在但未自动调用；测试 helper `sample_*` 仅在 Rust tests 中 |
| V12 | `node --check src/interface/web/app.js` | PASS | 退出码 0 |
| V13 | `cargo fmt -- --check` | PASS | 退出码 0 |
| V14 | `cargo check -p hajimi-desktop` | PASS | 退出码 0 |
| V15 | `cargo test -p hajimi-desktop` | PASS | 21 passed，含 restore unsafe path / no content / backup write / missing parent / confirmation gate 测试 |
| V16 | `cargo check --workspace` | PASS after escalation | 普通沙盒 ACL 失败；提升权限后通过 |
| V17 | `git diff --check` | PASS | 无 whitespace error，仅 CRLF warning |
| V18 | `rg -n "checkpoint_record_from_trace|files: Vec::new" src/interface/desktop/src/main.rs` | WARN | checkpoint 自动生成仍不填充可恢复文件内容 |
| V19 | 债务总表同步检查 | PASS | 两份 `HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` 已同步 Day 10 safe restore/replay V1 完成态 |
| V20 | receipt 债务声明 | PASS | `DEBT-THINKING-B10-001/002/003` 已记录 |

---

## 问题与建议

### 初审必须收尾

1. 同步两份债务总表：
   - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`
   - `docs/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`
   - 应改为 Day 10 safe restore/replay V1 已完成，剩余事务级原子 restore、内容快照填充、WebView smoke。
2. 明确 `checkpoint_record_from_trace` 与 restore 的关系：
   - 当前恢复能力只对带 `content` / `after_content` 的 checkpoint 真写入。
   - 普通 trace checkpoint 仍没有可恢复内容快照，后续需在 checkpoint 生成阶段填实。

以上两项已在 Day 10 收尾中完成；另补 `validate_restore_confirmation` 单测锁住后端确认门。

### 建议补强

- 给后端补一个直接覆盖 `confirm_restore == false` 的可测试 helper，避免只靠 command 入口人工判断。
- 前端 restore 成功后可以展示 files 数量与 backup 路径的可复制文本，而不仅是 toast。
- WebView smoke 应至少覆盖：dry-run preview、用户取消、用户确认但无内容快照拒绝、checkpoint replay、edit-history replay prev/next。

---

## 评级结论

- 评级：A 级
- 状态：收尾放行
- 与自测报告一致性：收尾后一致
- 地狱红线触发：未触发
- 是否需要返工：不需要；内容快照、事务级原子 restore、WebView smoke 作为已登记后续债务

---

## 收尾复审

收尾日期：2026-05-17

已完成修正：

- `src/interface/desktop/src/main.rs` 新增 `validate_restore_confirmation`，并由 `restore_checkpoint` 调用，确认门不再只靠人工读命令体。
- `src/interface/desktop/src/main.rs` 新增 `test_restore_confirmation_required_for_write_restore`，覆盖非 dry-run 未确认拒绝、dry-run 未确认允许、确认写入允许。
- `docs/debt/THINKING-RESTORE-REPLAY-VERIFY.md` 已同步 21 passed 和新增确认门测试。
- `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` 与 `docs/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` 已同步 Day 10 safe restore/replay V1 完成态。

收尾复核：

- `node --check src/interface/web/app.js`：PASS。
- `cargo fmt -- --check`：PASS。
- `cargo check -p hajimi-desktop`：PASS。
- `cargo test -p hajimi-desktop`：PASS，21 passed。
- `git diff --check`：PASS，仅 CRLF warning。

剩余债务：

- `DEBT-THINKING-B10-001`：restore 原子性仍是 V1，尚非事务日志式跨文件原子提交。
- `DEBT-THINKING-B10-002`：常规 checkpoint 仍需填充 `content` / `after_content` 才能端到端真实写入恢复。
- `DEBT-THINKING-B10-003`：WebView restore/replay 实机点击流仍待 smoke。

---

## 压力怪评语

“这次最重要的是没有乱写用户文件，这点很好。restore 先 dry-run、后确认、先 backup、再写入，路径也走 resolver，安全骨架是立住了。上游内容快照还没完全喂进来，但账本已经说清楚，不装作完成。Day 10 可以放行。”

---

## 归档建议

- 审计报告归档：`audit report/HAJIMI-DEBTFIX-DAY10-AUDIT-REPORT.md`
- 关联状态：HAJIMI-DEBTFIX Day 10 / `DEBT-THINKING-UI`
- 下一步建议：进入 Day 11，同时把 checkpoint 内容快照链路和 WebView restore/replay smoke 作为前置债务继续追踪。
