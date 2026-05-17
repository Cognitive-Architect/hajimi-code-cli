# Day 02 派单: 安全 workspace 路径解析函数

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 2，是 `CS-HAJIMI-002` workspace symlink / nonexistent path 逃逸的第一刀。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: 安全 workspace 路径解析函数
- **轰炸目标**: 在 `src/interface/desktop/src/main.rs` 中新增统一 `PathIntent` / `resolve_workspace_path`，替换 `read_file`、`write_file`、`list_dir` 的旧 `validate_path_within_workspace` 路径
- **任务性质**: Bug 修复 + 安全加固
- **输入基线**: 完整技术背景见模块2
- **输出要求**: 安全 resolver + 单元测试 + `cargo check -p hajimi-desktop` 通过 + 债务状态不得直接清零
- **通用铁律**:
  1. 不允许对不存在目标 fallback 到未解析 symlink 的路径
  2. 新建路径必须 canonicalize 父目录
  3. existing path 必须 canonicalize 目标
  4. 任何越界、symlink 外跳、父目录缺失必须返回可读错误
  5. 不引入上层依赖，不破坏 Tauri command 签名

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 记录当前分支和 HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 债务来源 | `CS-HAJIMI-002` 仍为 `OPEN / P0` | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` 第 4.2 节 | 必须 |
| 目标文件 | Tauri desktop 入口 | `src/interface/desktop/src/main.rs:166-227` | 必须 |
| 当前缺陷 | `canonicalize().unwrap_or(resolved)` 对不存在目标存在 symlink 逃逸风险 | `rg -n "canonicalize\\(\\)\\.unwrap_or\\(resolved\\)" src/interface/desktop/src/main.rs` | 必须 |
| 现有命令 | `read_file`, `write_file`, `list_dir` 使用旧 resolver | `rg -n "fn read_file|fn write_file|fn list_dir|validate_path_within_workspace" src/interface/desktop/src/main.rs` | 必须 |
| 技术约束 | Interface 层可依赖下层，但本修复应局限 desktop，不改 Engine/Intelligence | `cargo check -p hajimi-desktop` | 必须 |
| 测试约束 | Windows symlink/junction 行为需要考虑；自动测试至少覆盖非平台敏感逻辑 | Rust `#[cfg(test)]` 单元测试或说明无法测 symlink 的原因 | 必须 |
| 文档同步 | 状态最多 `OPEN -> VERIFY` | 当前债务总表或 Day receipt | 按需 |

### 探索补充栏

本任务为已知安全缺陷修复，无需开放探索。唯一待确认点是测试放在 `main.rs` 内部 `#[cfg(test)]`，还是抽出小模块；选择应以最小改动和 cargo-discoverable 为准。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-02/15
- **角色**: Engineer
- **目标**: 统一安全路径解析并替换核心文件读写/list 命令
- **输入**: `main.rs:166-227`、债务总表 4.2、Day 01 audit receipt
- **依赖关系**: 依赖 Day 01 状态复核完成

### 2）输出交付物

- **变更文件**:
  - `src/interface/desktop/src/main.rs`
  - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`，仅状态或 receipt 需要同步时
- **核心修改点**:
  - 新增 `PathIntent`，至少包含 `ExistingFile`, `ExistingDir`, `NewFile`, `NewDir`, `AnyExisting`
  - 新增 `resolve_workspace_path(input, base_dir, intent) -> Result<PathBuf, String>`
  - 替换 `read_file`, `write_file`, `list_dir` 的旧 resolver 调用
  - 为 traversal、missing parent、new path parent symlink、existing outside path 增加测试
- **必须包含**:
  - base dir 先 canonicalize
  - new path canonicalize parent，再 join leaf name
  - existing path canonicalize target
  - `starts_with(canonical_base)` containment
  - 对 `..` 和绝对路径越界的拒绝测试
- **禁止包含**:
  - `canonicalize().unwrap_or(resolved)` 的旧 fallback
  - 硬编码 workspace 名称
  - 为通过测试而 mock 成功
  - 修改 `run_command` 白名单
- **交付证明**:
  - `cargo check -p hajimi-desktop`
  - resolver 单元测试命令，若 binary crate 测试受限需说明并提供替代证据
  - `rg -n "resolve_workspace_path|PathIntent|unwrap_or\\(resolved\\)" src/interface/desktop/src/main.rs`

### 3）规模与复杂度观察

- **推荐目标**: resolver 单函数保持清晰阶段，复杂路径判断可拆成小 helper
- **复杂度说明**: Windows 路径规范化可能需要额外分支，若 resolver 超过 80 行需声明 `DEBT-COMPLEXITY-B02-001`
- **禁止行为**: 为了压行数拆出无意义 wrapper，或把安全逻辑散落到每个 command

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | desktop crate 编译通过 | `cargo check -p hajimi-desktop` | 返工 |
| FMT | Rust 格式通过 | `cargo fmt -- --check` | 返工 |
| LINT | 不新增明显 warning | `cargo clippy -p hajimi-desktop -- -D warnings` 或 `N/A + 原因` | 返工或声明债务 |
| TEST | resolver 测试通过 | `cargo test -p hajimi-desktop resolve_workspace_path` 或实际测试名 | 返工 |
| ARCH | 不改分层依赖 | `cargo check --workspace` 或 `cargo check -p hajimi-desktop` | 返工 |
| REAL | 不保留旧 fallback | `rg -n "unwrap_or\\(resolved\\)" src/interface/desktop/src/main.rs` 应无命中 | 返工 |
| DOC | 状态最多转 VERIFY | 债务总表 diff 或 receipt | 返工或声明无文档改动 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `PathIntent` 已定义 | `rg -n "enum PathIntent" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-002 | `resolve_workspace_path` 已定义 | `rg -n "fn resolve_workspace_path" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-003 | `read_file` 走新 resolver | `rg -n "fn read_file|resolve_workspace_path" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-004 | `write_file` 对新文件使用 `NewFile` | `rg -n "fn write_file|PathIntent::NewFile" src/interface/desktop/src/main.rs` | [ ] |
| CONST | CONST-001 | `list_dir` 使用 `ExistingDir` | `rg -n "fn list_dir|PathIntent::ExistingDir" src/interface/desktop/src/main.rs` | [ ] |
| CONST | CONST-002 | base canonicalize 存在 | `rg -n "canonical_base|base_dir\\.canonicalize" src/interface/desktop/src/main.rs` | [ ] |
| CONST | CONST-003 | new path canonicalize parent | `rg -n "parent\\(\\)|PathIntent::NewFile|PathIntent::NewDir" src/interface/desktop/src/main.rs` | [ ] |
| CONST | CONST-004 | desktop 编译通过 | `cargo check -p hajimi-desktop` | [ ] |
| NEG | NEG-001 | 旧 fallback 被删除 | `rg -n "unwrap_or\\(resolved\\)" src/interface/desktop/src/main.rs` 无命中 | [ ] |
| NEG | NEG-002 | traversal 被拒绝 | 测试名或 receipt 包含 `..` case | [ ] |
| NEG | NEG-003 | missing parent 被拒绝 | 测试名或 receipt 包含 missing parent case | [ ] |
| NEG | NEG-004 | symlink 外跳被拒绝 | 测试名或手动 receipt 包含 symlink case | [ ] |
| UX | UX-001 | 错误信息可读 | 单测或手动调用输出包含明确越界/不存在说明 | [ ] |
| UX | UX-002 | 合法 workspace 文件仍可读写 | 单测 normal read/write case | [ ] |
| E2E | E2E-001 | workspace 编译不破坏 | `cargo check --workspace` 或说明非本次外部错误 | [ ] |
| High | HIGH-001 | 安全路径统一入口 | `rg -n "validate_path_within_workspace\\(" src/interface/desktop/src/main.rs` 只剩兼容 wrapper 或无命中 | [ ] |

---

## 【模块3-B】地狱红线

1. 保留 `canonicalize().unwrap_or(resolved)`，返工
2. 对新文件 canonicalize 目标而不是父目录且失败后放行，返工
3. 只修 `write_file`，漏掉 `read_file/list_dir`，返工
4. 把安全逻辑写在前端，返工
5. 修改 `run_command` 白名单以绕过问题，返工
6. 没有任何负面测试或手动 symlink 证据，返工
7. 错误信息泄露过多宿主路径且无说明，返工
8. 编译失败仍收卷，返工
9. 未声明无法跑的测试原因，返工
10. 未给 Day 3 复用入口，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | read/write/list 是否走新 resolver | [ ] | FUNC-003~004, CONST-001 | |
| RG | 旧 fallback 是否消失 | [ ] | NEG-001 | |
| NG | traversal/missing parent/symlink 是否覆盖 | [ ] | NEG-002~004 | |
| UX | 合法路径和错误提示是否可用 | [ ] | UX-001~002 | |
| E2E | desktop crate 是否能编译 | [ ] | CONST-004 | |
| High | 高风险 symlink 是否单独验证 | [ ] | HIGH-001 | |
| 字段完整性 | 测试是否写明前置和预期 | [ ] | 测试名/receipt | |
| 需求映射 | 是否映射 `CS-HAJIMI-002` | [ ] | 债务总表 4.2 | |
| 自测执行 | 是否跑过 cargo 命令 | [ ] | 质量闸门 | |
| 范围边界与债务 | 是否没有扩大到 Day 3/4 | [ ] | git diff | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-02/15 完成并提交

### 提交信息
- Commit: `security(interface/desktop): harden workspace path resolver`
- 分支: `<实际分支>`
- 变更文件:
  - `src/interface/desktop/src/main.rs`
  - `<文档文件，如有>`

### 本轮目标与实际结果
- 目标: 修复 workspace symlink / nonexistent path 逃逸
- 实际完成: `<列出 resolver、替换点、测试>`
- 未完成/不在范围: create_dir/rename/delete 属 Day 3；Shell 白名单属 Day 4

### 自动化质量检查报告
- `cargo check -p hajimi-desktop`: `<摘要>`
- `cargo fmt -- --check`: `<摘要>`
- `cargo test -p hajimi-desktop <resolver_test>`: `<摘要或 N/A 原因>`
- `rg -n "unwrap_or\\(resolved\\)" ...`: `<摘要>`

### 债务声明
- `DEBT-TEST-B02-001`: `<若 Windows symlink 自动化不可用，写手动 receipt 路径>`

### 风险与回滚点
- 主要风险: 过严路径判断误伤合法路径
- 回滚方式: `git restore src/interface/desktop/src/main.rs`
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | resolver 需要跨 crate 共享才可继续 | 暂停抽象，先保持 desktop 内最小实现 | 避免大重构 |
| QUALITY-001 | `cargo check -p hajimi-desktop` 连续失败 | 停止扩展 Day 3 功能，先修编译 | 返工 |
| TEST-001 | Windows symlink 创建需要权限 | 使用 tempdir + non-symlink 负面测试，并补手动 receipt | 有条件交付 |
| COMPLEXITY-001 | resolver 分支过多难以审查 | 声明 `DEBT-COMPLEXITY-B02-001` 并写后续拆分点 | 记录债务 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 02 安全 workspace 路径解析函数**。

### 关键约束
- 新文件只 canonicalize 父目录
- existing path 必须 canonicalize 目标
- 不再使用 `canonicalize().unwrap_or(resolved)`
- Day 3 的 `create_dir/rename_path/delete_path` 必须能复用本 resolver

### 验收铁律
- `cargo check -p hajimi-desktop` 通过
- 旧 fallback 无命中
- 负面路径有测试或手动 receipt
- 债务状态最多推进到 `VERIFY`

闭环启动，Day 02，执行。
