# W27-AUDIT-001 Week 27建设性审计报告

## 审计结论
- **评级**: 🟡 **B+级（良好，有轻微行数偏差）**
- **状态**: ✅ **Go**（DEBT-GIT-CLI-W11确认清偿，Week 28就绪）
- **与自测报告一致性**: **部分一致**（功能实现符合，行数统计有偏差）

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| git2-rs零依赖 | **A** | V1验证：Cargo.toml无git2，代码仅注释提及（L1替代说明）✅ |
| ExitCode解析 | **A** | V2验证：`exit_code: i32`+`stderr: String`完整解析（L13,L46）✅ |
| 原子写入实现 | **A** | V3验证：`NamedTempFile`（L8,75）+ `fs::rename`（L86）✅ |
| 延迟写入策略 | **A** | V4验证：`dirty: bool`（L44）+ 15处dirty标记使用 ✅ |
| JSONL格式合规 | **A** | 换行分隔JSON（`writeln!` L83），非数组格式 ✅ |
| 路径安全 | **A** | V5验证：`dirs::home_dir()`（L53），`~/.hajimi`拼接 ✅ |
| Session→Auto流 | **A** | V6验证：`sync_from_session`（L147-159）完整实现 ✅ |
| Auto→Dream预留 | **B** | 无显式`sync_to_dream`，但架构可扩展 |
| 债务状态更新 | **A** | DEBT-GIT-CLI-W11声称已清偿，实现验证通过 ✅ |
| 三栏申报诚实 | **B** | git_cli.rs 111 vs 94（+17未申报），auto.rs 248 vs 218（+30未申报） |

**整体健康度评级**: **B+级**（债务清偿真实，架构契约兑现，行数统计需规范）

---

## 行数偏差详细分析

| 文件 | 申报 | 实际 | 偏差 | 三栏数据 | 状态 |
|:---|:---:|:---:|:---:|:---|:---:|
| git_cli.rs | 94 | **111** | **+17** | 生产60/测试51/总计111 | ⚠️ 未申报 |
| auto.rs | 218 | **248** | **+30** | 生产167/测试81/总计248 | ⚠️ 未申报 |
| tools/lib.rs | 2 | 1 | -1 | 生产1/测试0/总计1 | ✅ 合规 |
| memory/mod.rs | 14 | 91 | +77 | 原14行可能指增量 | ⚠️ 口径不明 |

**偏差评估**：
- git_cli.rs +17行（+18%）：偏差>5但未申报，违反LINE-COUNT-STANDARD-v1.0
- auto.rs +30行（+14%）：偏差>5但未申报，违反LINE-COUNT-STANDARD-v1.0
- **根因**：可能未使用Week 26建立的三栏申报标准

**债务登记建议**：
- DEBT-LINES-27-01: git_cli.rs行数偏差+17行
- DEBT-LINES-27-02: auto.rs行数偏差+30行

---

## 关键疑问回答（Q1-Q3）

### Q1：DEBT-GIT-CLI-W11是否真正清偿？零git2-rs依赖是否彻底？
**审计结论**: ✅ **真正清偿，零依赖确认**

**双重验证**：
1. **Cargo.toml验证**（V1）：`[dependencies]`仅含`tempfile = "3"`，无`git2`
2. **源代码验证**（V1）：全文仅L1注释提及"替代git2-rs"，无`use git2::`或`extern crate git2`

**ExitCode解析完整性**（V2）：
```rust
// L13: 错误定义含exit_code+stderr
CommandFailed { exit_code: i32, stderr: String }

// L46: 运行时解析
exit_code: output.status.code().unwrap_or(-1),
stderr: String::from_utf8_lossy(&output.stderr).to_string(),
```

**结论**：git2-rs依赖已彻底清除，Command封装完备，错误处理完整。

### Q2：Auto层原子写入是否为真原子？rename操作是否覆盖完整？
**审计结论**: ✅ **真原子实现，覆盖完整**

**原子写入实现**（V3）：
```rust
// L75: 在同目录创建temp文件（确保同分区rename原子性）
let mut temp = NamedTempFile::new_in(&self.storage_dir)?;

// L76-84: 写入所有entries
for (k, v) in &self.entries { writeln!(temp, "...")?; }

// L85-86: flush保证落盘 + rename原子替换
temp.flush()?;
fs::rename(temp.path(), &persist_file)?;
```

**关键正确性验证**：
- ✅ `new_in(&self.storage_dir)`：temp文件与目标同目录，确保同分区（rename原子性前提）
- ✅ `temp.flush()`：数据落盘后再rename，避免半写
- ✅ `fs::rename`：POSIX原子操作，崩溃后无残留

**并发安全评估**：
- 当前实现：无显式文件锁，依赖rename原子性
- 风险：多进程同时persist可能产生竞态（最后rename者胜）
- 缓解：Single-process架构（Hajimi CLI），实际风险低
- **建议**：Week 28可补充文件锁（`fs2::FileExt::lock`）作为增强

### Q3：延迟写入策略是否会导致数据丢失？dirty标记何时触发？
**审计结论**: ⚠️ **数据丢失风险可控，但需显式persist调用**

**dirty标记机制**（V4）：
| 操作 | dirty设置 | persist时机 |
|:---|:---:|:---|
| `insert()` | `dirty = true`（L124） | 延迟 |
| `get_mut()` | `dirty = true`（L133） | 延迟 |
| `remove()` | `dirty = true`（L138） | 延迟 |
| `clear()` | `dirty = true`（L143） | 延迟 |
| `persist()` | `dirty = false`（L89） | 立即 |

**数据丢失风险评估**：
- **进程崩溃**：未persist的数据丢失（符合延迟写入语义）
- **显式调用点**：`sync_from_session()`后需调用`persist()`（由调用方控制）
- **自动策略缺失**：无定时/阈值自动persist（Phase 4 Week 28可补充）

**建议**：
- 短期：文档说明"Auto层不保证崩溃安全，关键操作后需显式persist"
- 中期：Week 28添加`Drop`实现自动persist，或定时持久化线程

---

## 验证结果（V1-V6）

| 验证ID | 验证项 | 结果 | 证据 |
|:---:|:---|:---:|:---|
| V1 | git2-rs清零 | ✅ | Cargo.toml无git2，代码仅L1注释提及 |
| V2 | ExitCode解析 | ✅ | L13定义含exit_code+stderr，L46完整解析 |
| V3 | 原子写入 | ✅ | L8,75 `NamedTempFile` + L86 `fs::rename` |
| V4 | dirty标记 | ✅ | L44定义 + 15处使用（生产11+测试4） |
| V5 | 路径安全 | ✅ | L53 `dirs::home_dir()`，L54 `~/.hajimi`拼接 |
| V6 | Session→Auto流 | ✅ | L147-159 `sync_from_session`完整实现 |

---

## Week 28就绪度评估

| 检查项 | 状态 | 说明 |
|:---|:---:|:---|
| Auto→Dream接口预留 | ⚠️ **部分** | 无显式`sync_to_dream()`，但`AutoEntry`结构可扩展 |
| ONNX集成预备 | ⚠️ **待规划** | `AutoEntry`无`embedding`字段，Week 28需添加 |
| Scheduler架构预备 | ⚠️ **待规划** | `persist()`由调用方触发，Week 28需定时调度 |

**架构扩展性评估**：
```rust
// 当前AutoEntry（L24-29）
pub struct AutoEntry {
    pub session_entry: SessionEntry,
    pub file_path: PathBuf,
    pub last_persisted: DateTime<Utc>,
    // Week 28可添加: pub embedding: Option<Vec<f32>>,
}
```

**结论**：架构可扩展，但Week 28需显式添加ONNX集成字段和调度器。

---

## 问题与建议

### 短期（立即处理）
1. **申报行数债务**（DEBT-LINES-27-01/02）
   - git_cli.rs +17行，auto.rs +30行未申报
   - 需在DEBT-REGISTER-PHASE4.md登记

### 中期（Week 28内）
2. **自动persist策略**（数据安全增强）
   - 实现`Drop for AutoMemory`自动persist
   - 或添加定时持久化线程（Scheduler集成）

3. **文件锁并发保护**（可选增强）
   - 添加`fs2` crate文件锁
   - 防止多进程竞态（虽然Hajimi单进程）

4. **ONNX集成字段预留**
   - `AutoEntry`添加`embedding: Option<Vec<f32>>`
   - 准备Dream层384维向量存储

### 长期（Phase 4后续）
5. **三栏申报标准执行**
   - 强制使用LINE-COUNT-STANDARD-v1.0
   - 审计时携带三栏验证块

---

## 债务确认

| 债务ID | 描述 | 状态 | 说明 |
|:---|:---|:---:|:---|
| DEBT-GIT-CLI-W11 | Git CLI工具CLI化（清偿git2-rs） | ✅ **已清偿** | 零git2依赖，Command封装完整 |
| DEBT-LINES-27-01 | git_cli.rs行数偏差+17 | 🆕 **新增** | 申报94实际111，未申报 |
| DEBT-LINES-27-02 | auto.rs行数偏差+30 | 🆕 **新增** | 申报218实际248，未申报 |
| DEBT-PERF-W25 | 性能观察 | ⏳ **Week 29** | 按计划时点验证 |

---

## 压力怪评语

> 🥁 **"无聊"**（B+级：债务清偿真实，架构契约兑现，行数统计需规范）
>
> DEBT-GIT-CLI-W11真正清偿：零git2-rs依赖，ExitCode完整解析，Command封装正确。
>
> Auto层架构契约兑现：NamedTempFile+rename真原子，dirty标记延迟写入完整，Session→Auto数据流畅通。
>
> **但是**：行数申报又飘了。git_cli.rs 111 vs 94（+17），auto.rs 248 vs 218（+30），都超5行阈值但未申报。Week 26刚立的LINE-COUNT-STANDARD-v1.0就忘？
>
> 数据持久性有优化空间（Drop自动persist），Week 28 ONNX集成需显式规划。
>
> **B+级通过**，Go至Week 28。记得申报行数债务，别再犯。
>
> ☝️🐍♾️⚖️🟡

---

## 衔尾蛇链

```
Week 26(A/口径修复) → Week 27(B+/债务清偿) → Week 28(Dream层ONNX)
```

---

## 归档建议

- **审计报告**: `audit report/phase4/week27/W27-AUDIT-001.md` ✅
- **新增债务**: DEBT-LINES-27-01/02（行数偏差）
- **Week 28准入**: **Granted**（需补充行数债务登记）

---

*审计官: 压力怪*  
*日期: 2026-04-02*  
*审计链: Week 26(A) → Week 27(B+) → Week 28(Dream层)*
