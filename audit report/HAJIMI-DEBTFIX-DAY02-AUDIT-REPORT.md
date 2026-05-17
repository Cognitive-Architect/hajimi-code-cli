# HAJIMI-DEBTFIX Day 02 建设性审计报告

> 审计对象：`docs/roadmap/hajimi debtFix/task/Day-02-Secure-Workspace-Resolver.md`
> 审计官：压力怪
> 审计日期：2026-05-16
> 关联阶段：HAJIMI-DEBTFIX Phase Day 02
> 当前状态：修复后复审 A 级 / Go（详见文末“修复后复审结论”）

---

## 审计背景

### 项目阶段

HAJIMI-DEBTFIX Day 02：安全 workspace 路径解析函数。目标是修复 `CS-HAJIMI-002` workspace symlink / nonexistent path 逃逸风险，在 `src/interface/desktop/src/main.rs` 中引入统一 resolver，并替换 `read_file`、`write_file`、`list_dir` 的旧路径校验。

### 交付物清单

| 序号 | 文件名 | 路径 | 内容摘要 | 交付者 | 自检结果 |
|---:|---|---|---|---|---|
| 1 | `main.rs` | `src/interface/desktop/src/main.rs` | 新增 `PathIntent` / `resolve_workspace_path`，替换 read/write/list resolver，新增 6 个单元测试 | Engineer | 声明通过 |
| 2 | `HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | 将 `CS-HAJIMI-002` 推进到 `VERIFY`，记录 Day 2 修复与验证证据 | Engineer | 声明通过 |

### 关键代码片段

```rust
// 来自 src/interface/desktop/src/main.rs:167-177
enum PathIntent {
    ExistingFile,
    ExistingDir,
    NewFile,
    NewDir,
    AnyExisting,
}
```

```rust
// 来自 src/interface/desktop/src/main.rs:205-229
let canonical = match intent {
    PathIntent::ExistingFile | PathIntent::ExistingDir | PathIntent::AnyExisting => {
        resolved.canonicalize().map_err(|e| format!("无法解析目标路径: {}", e))?
    }
    PathIntent::NewFile | PathIntent::NewDir => {
        let parent = resolved.parent().ok_or_else(|| "无法获取父目录".to_string())?;
        if !parent.exists() {
            return Err(format!("父目录不存在: {}", parent.display()));
        }
        let canonical_parent = parent.canonicalize().map_err(|e| format!("无法解析父目录: {}", e))?;
        canonical_parent.join(resolved.file_name().ok_or_else(|| "无法获取文件名".to_string())?)
    }
};
```

```rust
// 来自 src/interface/desktop/src/main.rs:1977-1986
let temp = std::env::temp_dir().join(format!("hajimi-test-{}", std::process::id()));
let _ = std::fs::remove_dir_all(&temp);
...
let _ = std::fs::remove_dir_all(temp);
```

### 已知限制/环境问题

- 默认沙箱内部分 Cargo 命令会因 Windows `target` 写入权限出现 `os error 5`；本审计按规则提权复跑关键命令。
- 当前工作区仍有既有 `src/MEMORY.md` tracked 修改，与 Day 02 resolver 无直接关系。
- `docs/roadmap/` 与 `audit report/` 被 `.gitignore` 忽略，提交文档时需要 `git add -f`。

---

## 质量门禁

- 已读取 Day 02 工单、建设性审计模板、B-09 审计报告示例。
- 已读取并抽查 `src/interface/desktop/src/main.rs` 的 resolver 实现与测试。
- 已读取债务总表 `CS-HAJIMI-002` 更新段落。
- 已独立执行 `cargo check -p hajimi-desktop`、`cargo fmt -- --check`、`cargo test -p hajimi-desktop`、`cargo clippy -p hajimi-desktop -- -D warnings`。
- 已验证 `unwrap_or(resolved)` 旧 fallback 无命中。

质量门禁满足出报告条件，但不满足 A 级放行条件。

---

## 审计目标

1. `PathIntent` / `resolve_workspace_path` 是否按工单实现？
2. `read_file`、`write_file`、`list_dir` 是否切到新 resolver？
3. Day 02 的 BUILD / FMT / TEST / LINT 质量闸门是否真实可复现？
4. symlink / junction 外跳风险是否有测试或手动 receipt 证据？

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| 主实现方向 | B | 新 resolver 已实现，existing 目标 canonicalize、新文件 canonicalize 父目录、旧 fallback 删除，方向正确。 |
| 调用点替换 | A | `read_file` 使用 `ExistingFile`，`write_file` 使用 `NewFile`，`list_dir` 使用 `ExistingDir`。 |
| 自动化测试 | C | `cargo test -p hajimi-desktop` 默认并行执行失败；串行可过，说明测试夹具存在竞态。 |
| LINT 闸门 | C | `cargo clippy -p hajimi-desktop -- -D warnings` 因 `NewDir` / `AnyExisting` dead_code 失败。 |
| 安全负面证据 | C | 有 traversal、missing parent、absolute outside 测试，但缺少 symlink/junction 外跳测试或手动 receipt。 |
| 文档闭环 | C | 债务总表推进到 `VERIFY` 合理，但写入段落残留旧代码块，且“cargo test 全部通过”与默认命令复核不一致。 |

整体健康度评级：C 级。实现骨架可救，但 Day 02 的验收铁律里 TEST、LINT、symlink 证据三项没有达到可放行标准。

---

## 关键疑问回答（Q1-Q3）

**Q1：旧 `canonicalize().unwrap_or(resolved)` fallback 是否删除？**

是。`rg -n "unwrap_or\\(resolved\\)|validate_path_within_workspace\\(" src/interface/desktop/src/main.rs` 无命中。read/write/list 三处也已切到 `resolve_workspace_path`。

**Q2：resolver 单元测试是否真实 6/6 通过？**

默认不是。`cargo test -p hajimi-desktop` 提权复跑结果为 4/6，通过项外有 `test_resolve_existing_file` 与 `test_resolve_new_file` 失败。失败原因来自测试夹具：所有测试共用 `std::process::id()` 生成同一个 temp 根目录，并在 setup/cleanup 中互相删除。串行运行 `cargo test -p hajimi-desktop test_resolve -- --test-threads=1` 才能 6/6 通过。

**Q3：symlink / junction 外跳是否已经被验收覆盖？**

没有。当前 6 个测试覆盖 existing file、existing dir、new file、missing parent、`..` traversal、absolute outside，但没有创建 workspace 内 symlink/junction 指向 workspace 外的 case，也没有看到手动 receipt。因此 Day 02 最核心的 symlink 逃逸风险还缺一条直接证据。

---

## 验证结果（V1-V12）

| 验证ID | 命令 | 结果 | 证据 |
|:---|:---|:---:|:---|
| V1 | `git status --short` | PASS | `M src/MEMORY.md`，`M src/interface/desktop/src/main.rs` |
| V2 | `rg -n "enum PathIntent|fn resolve_workspace_path" src/interface/desktop/src/main.rs` | PASS | `PathIntent` 在 167 行，resolver 在 181 行 |
| V3 | `rg -n "PathIntent::ExistingFile|PathIntent::NewFile|PathIntent::ExistingDir" src/interface/desktop/src/main.rs` | PASS | read/write/list 三处已替换 |
| V4 | `rg -n "unwrap_or\\(resolved\\)|validate_path_within_workspace\\(" src/interface/desktop/src/main.rs` | PASS | 无命中 |
| V5 | `cargo check -p hajimi-desktop` | PASS | 退出码 0，但有 `NewDir` / `AnyExisting` dead_code warning |
| V6 | `cargo fmt -- --check` | PASS | 退出码 0 |
| V7 | `cargo test -p hajimi-desktop` | FAIL | 4/6；`test_resolve_existing_file`、`test_resolve_new_file` 失败 |
| V8 | `cargo test -p hajimi-desktop test_resolve -- --test-threads=1` | PASS | 6/6，证明默认失败主要是测试夹具竞态 |
| V9 | `cargo clippy -p hajimi-desktop -- -D warnings` | FAIL | `NewDir` / `AnyExisting` never constructed，被 `-D warnings` 提升为 error |
| V10 | `rg -n "symlink|junction|std::os::windows|std::os::unix" src/interface/desktop/src/main.rs` | FAIL | 未发现 symlink/junction 测试 |
| V11 | `rg -n "CS-HAJIMI-002|Day 2 修复|cargo test" "docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md"` | FAIL | 文档声称 `cargo test` 全部通过，但默认命令复核失败 |
| V12 | 人工阅读债务总表 4.2 节 | FAIL | Day 2 插入后残留旧代码片段 `if !canonical.starts_with...`，Markdown 结构破损 |

---

## 问题与建议

### 必须返工

1. 修复测试夹具竞态：每个测试使用唯一临时目录，例如加入测试名、线程安全随机后缀或 `SystemTime` 纳秒；不要用单一 `process::id()` 目录并在并行测试中互删。
2. 让 `cargo test -p hajimi-desktop` 默认通过，而不是只在 `--test-threads=1` 下通过。
3. 修复 `cargo clippy -p hajimi-desktop -- -D warnings`：要么让 `NewDir` / `AnyExisting` 在非测试路径真实使用，要么用明确的局部 allow 并说明 Day 3 将消费。
4. 增加 symlink/junction 外跳验证。若 Windows 自动创建 symlink 需要权限，就写手动 receipt；至少覆盖 `workspace/link -> outside` 后 `write_file("link/new.txt")` 或直接 resolver `NewFile` 被拒绝。
5. 修正债务总表 4.2 节：不要保留断裂的旧代码块；把 `cargo test` 证据改成真实状态，返工后再写通过。

### 建议补强

- `ExistingFile` / `ExistingDir` 目前只 canonicalize，不主动验证 file type / dir type。虽然后续 `read_to_string` / `read_dir` 会失败，但 resolver 既然引入了 intent，建议在 resolver 内统一返回更清晰错误。
- `input.contains("..")` 会拒绝合法文件名如 `notes..txt`。安全上偏保守可以接受，但如果产品侧有文件名兼容要求，后续可改为按 path component 判断 `ParentDir`。

---

## 评级结论

- 评级：C 级
- 状态：返工
- 与自测报告一致性：部分一致，关键测试结论偏离
- 地狱红线触发：是，TEST/LINT 不可复现，且缺少 symlink/junction 证据
- 是否需要返工：需要

---

## 压力怪评语

“方向是对的，但验收不是只看方向。安全 resolver 的第一天，最不能糊弄的就是测试证据：默认 cargo test 会炸、clippy 会炸、symlink 证据还没上桌。把这三处补齐，再回来冲 A。”

---

## 归档建议

- 审计报告归档：`audit report/HAJIMI-DEBTFIX-DAY02-AUDIT-REPORT.md`
- 关联状态：HAJIMI-DEBTFIX Day 02
- 下一步建议：先返工 Day 02，再进入 Day 03；否则 Day 03 的 file ops commands 会建立在未验收稳定的 resolver 上。

---

## 修复后复审结论（2026-05-16）

- **复审评级**：A 级
- **状态**：Go
- **与自测报告一致性**：一致

### 修复闭环

| 原问题 | 修复结果 | 证据 |
|:---|:---:|:---|
| `cargo test -p hajimi-desktop` 默认并行失败 | 已修复测试夹具竞态，每个测试使用唯一临时目录 | `cargo test -p hajimi-desktop`：8 passed |
| `cargo clippy -p hajimi-desktop -- -D warnings` dead_code 失败 | 已对 Day 3 预留 intent 做局部说明，并补 `NewDir` 测试覆盖 | `cargo clippy -p hajimi-desktop -- -D warnings`：通过 |
| 缺少 symlink/junction 外跳证据 | 已新增 parent symlink/junction 外跳拒绝测试 | `test_resolve_new_file_rejects_parent_symlink_escape ... ok` |
| 债务总表 4.2 节文档断裂 | 已重写 Day 2 段落，状态保持 `VERIFY`，证据改为真实复跑结果 | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` |

### 复审验证

| 验证ID | 结果 | 证据 |
|:---|:---:|:---|
| RV1 | PASS | `cargo fmt -- --check` 退出码 0 |
| RV2 | PASS | `cargo check -p hajimi-desktop` 退出码 0 |
| RV3 | PASS | `cargo test -p hajimi-desktop`：8 passed，0 failed |
| RV4 | PASS | Windows 本地创建 junction，`outside-link/newfile.txt` 被 resolver 拒绝 |
| RV5 | PASS | `cargo clippy -p hajimi-desktop -- -D warnings` 退出码 0 |
| RV6 | PASS | `rg -n "unwrap_or\\(resolved\\)|validate_path_within_workspace\\(" src/interface/desktop/src/main.rs` 无命中 |

### A 级评语

Day 02 已从“实现方向正确但证据破损”修到“默认测试可复现、lint 可复现、symlink/junction 风险有直接证据”。`resolve_workspace_path` 现在可以作为 Day 03 文件操作命令的安全入口继续复用。保持债务状态为 `VERIFY` 是正确的，等 Day 03 把 create/rename/delete 全接进来后再考虑关闭。
