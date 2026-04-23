# Week3-4-ACCEPTANCE-AUDIT-002 建设性审计报告

**审计对象**: Week 3-4 P1 基建与结构清理交付物  
**审计日期**: 2026-04-16  
**审计官**: 压力怪/审计喵  
**审计性质**: 建设性审计（非对抗，严格标准）  

---

## 审计结论

| 维度 | 结论 |
|:---|:---|
| **评级** | **C**（合格，需改进） |
| **状态** | **返工** — 2 项关键问题需修复后方可进入 Week 5 |
| **与自检报告一致性** | 部分偏离（自检声称 7/7 通过，实际 2 项未达标） |

**建设性评语**: 🟠 **"有明显问题，需改进"**（C 级）

> "不是，这 simhash64 怎么还有三份实现？`foundation/hash/simhash.rs` 新建了，但 `tantivy_index.rs` 和 `adr_index.rs` 里的旧实现根本没删，也没 `use` 新库！
>
> 还有 codex-twist，`crates/hajimi-codex-twist/src` 里还有 20 个文件，派单要求只剩 `lib.rs` 做 thin wrapper！
>
> **问题清单**:
> 1. **simhash64 统一不完整** — 三份实现并存，行为一致性风险
> 2. **codex-twist 清理不彻底** — 20 个文件残留，双轨制未解决
> 3. **Cargo.toml 版本锁定不足** — 7 个，要求 10+
>
> **返工要求**:
> - simhash: 删除 `tantivy_index.rs` 和 `adr_index.rs` 中的本地实现，改为 `use foundation::hash::{simhash64, get_shard_id};`
> - codex-twist: 删除 `crates/hajimi-codex-twist/src/` 下除 `lib.rs` 外所有文件，添加 `pub use intelligence::codex_twist::*;`
> - 重新执行 V7-V10 验证
>
> **好消息**: DEBT-P0-001 完整， cargo audit CI 到位，Scrypt 移除干净。修完这两处再进 Week 5。"

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 | 关键证据 |
|:---|:---:|:---|:---|
| **B-05 债务文档** | **A** | DEBT-P0-001 完整（Owner/Deadline/KMS方案/验证命令） | 41 行，5 要素全齐 |
| **B-06 P0 优化** | **A** | 3/3 完成（sha256sum前置检查、PSK≥16字符、Owner标签） | V2=1, V3=1, grep 'Owner' ✅ |
| **B-07 审计基建** | **B** | cargo audit CI ✅, Scrypt移除 ✅, 版本锁定7个<10 | V4=1, V6=0(注释only), V5=7<10 |
| **B-08 simhash 统一** | **C** | 新库创建 ✅, 但旧实现未删除、未引用新库 | V7=1, V8=2(两处残留), 无`use foundation::hash` |
| **B-09 codex 清理** | **C** | lib.rs存在 ✅, 但20个文件未删除、非thin wrapper | V9=20>0, 无`pub use intelligence::codex_twist::*;` |
| **债务诚实性** | **A** | DEBT-P0-001完整, DEBT-CRYPTO-004隐含(cloud.rs注释) | 文档齐全 |

**整体健康度**: C（2项A + 1项B + 2项C + 1项A，存在结构性问题）

---

## 关键疑问回答（Q1-Q4）

### Q1: B-08 simhash64 行为一致性验证

**审计结论**: 🔴 **不一致风险（C 级）**

**问题分析**:
```rust
// 当前状态：三份实现并存

// 1. foundation/hash/simhash.rs (新库，目标实现)
pub fn simhash64(text: &str) -> u64 { ... }

// 2. tantivy_index.rs:91-97 (旧实现，仍在使用)
pub fn simhash64(text: &str) -> u64 { ... }  // 相同算法，但独立维护

// 3. adr_index.rs:16-26 (旧实现，仍在使用)
pub fn simhash64(text: &str) -> u64 { ... }  // 相同算法，但独立维护
```

**验证结果**:
| 检查项 | 期望 | 实际 | 状态 |
|:---|:---:|:---:|:---:|
| 新库存在 | ✅ | `foundation/hash/simhash.rs` 存在 | ✅ |
| 旧实现删除 | ❌ | `tantivy_index.rs` 仍有实现 | ❌ |
| 旧实现删除 | ❌ | `adr_index.rs` 仍有实现 | ❌ |
| 引用新库 | `use foundation::hash::` | 未找到 | ❌ |

**风险**: 
- 三份相同算法独立维护，后续修改可能漂移
- 未使用新库，统一目标未达成
- 可能引入细微行为差异（如编译优化不同）

**修复要求**:
```rust
// tantivy_index.rs 和 adr_index.rs 修改：
// 删除本地 simhash64/get_shard_id 实现
// 添加：
use foundation::hash::{simhash64, get_shard_id};
```

---

### Q2: B-09 codex-twist 下游兼容性

**审计结论**: 🔴 **双轨制未解决（C 级）**

**问题分析**:
```powershell
# 实际状态
Get-ChildItem src/crates/hajimi-codex-twist/src -Recurse -Filter *.rs | Where-Object { $_.Name -ne 'lib.rs' }
# 结果: 20 个文件
```

**期望状态** (per 派单 B-09):
```powershell
# 期望状态
find src/crates/hajimi-codex-twist -name '*.rs' -not -name 'lib.rs' | wc -l
# 结果: 0
```

**当前 lib.rs 内容**:
```rust
// src/crates/hajimi-codex-twist/src/lib.rs
pub mod approval;   // 本地实现
pub mod lcr_adapter; // 本地实现
pub mod memory;      // 本地实现
// ... 其他本地模块

pub use approval::{...};  // 本地 re-export，非 intelligence::codex_twist
```

**缺失**:
- ❌ 未删除 20 个重复文件
- ❌ 未添加 `#![deprecated(...)]` 属性
- ❌ 未使用 `pub use intelligence::codex_twist::*;` thin wrapper 模式

**修复要求**:
```rust
// crates/hajimi-codex-twist/src/lib.rs 应改为：
#![deprecated(since = "3.8.0", note = "Use intelligence::codex_twist directly")]

pub use intelligence::codex_twist::*;
```

```bash
# 删除所有其他文件
rm -rf src/crates/hajimi-codex-twist/src/{approval,lcr_adapter,memory,storage,thread,tiered,turn,ffi}.rs
rm -rf src/crates/hajimi-codex-twist/src/{approval,lcr_adapter,memory,storage,thread,tiered,turn,ffi}/
```

---

### Q3: B-07 Scrypt 移除的兼容性边界

**审计结论**: ✅ **干净移除（A 级）**

**验证**:
```powershell
# V6: 检查 degraded_mode / derive_key_scrypt
Get-Content src/intelligence/memory/src/cloud.rs | Select-String 'degraded_mode|derive_key_scrypt'
# 结果: 1 处（仅注释说明已移除）
```

**实际代码** (cloud.rs:120):
```rust
/// Cloud记忆层E2EE实现 (degraded_mode and scrypt fallback removed per B-07 P1 redline - pure Argon2 only)
```

**验证**: 纯 Argon2id 实现，无 Scrypt 降级路径 ✅

**DEBT-CRYPTO-004 状态**:
- 代码注释已声明低内存环境可能影响
- 建议补写独立债务文档 `docs/debt/DEBT-CRYPTO-004.md`（非阻塞）

---

### Q4: DEBT-P0-001 的可执行性

**审计结论**: ✅ **完整可执行（A 级）**

**文档要素检查**:
| 要素 | 存在 | 内容 |
|:---|:---:|:---|
| Context | ✅ | Week 2 PSK 实现背景 |
| Recovery Plan | ✅ | KMS/Vault/轮换/30天周期 |
| Deadline | ✅ | Week 4 结束 |
| Owner | ✅ | @engineer-04 |
| 验证命令 | ✅ | 5 条 grep 命令 |
| 债务分类 | ✅ | SECURITY / P0 |

**验收命令验证**:
```bash
# V1 验证
grep -c 'PSK Long-term Management' docs/debt/DEBT-P0-001.md  # >=1 ✅
grep -c 'KMS\|Vault\|rotation' docs/debt/DEBT-P0-001.md      # >=2 ✅
grep -c 'Week 4' docs/debt/DEBT-P0-001.md                    # >=1 ✅
grep -c 'Owner: @engineer-04' docs/debt/DEBT-P0-001.md       # >=1 ✅
grep -c 'TODO\|FIXME' docs/debt/DEBT-P0-001.md               # ==0 ✅
```

---

## 验证结果（V1-V10）

| 验证 ID | 内容 | 结果 | 证据 | 标准 |
|:---|:---|:---:|:---|:---:|
| **V1** | DEBT-P0-001 存在 | ✅ | 文件存在，41行 | >=1 |
| **V2** | sha256sum 前置检查 | ✅ | `command -v sha256sum` 存在 | >=1 |
| **V3** | PSK 长度校验 | ✅ | `length < 16` + `exit 1` 存在 | >=1 |
| **V4** | cargo audit --deny | ✅ | CI 配置存在 | >=1 |
| **V5** | 版本锁定 | ⚠️ | 7 个 `version = "=` | >=10 |
| **V6** | Scrypt 移除 | ✅ | 仅注释提及，代码已移除 | 0 |
| **V7** | simhash 新库 | ✅ | `foundation/hash/simhash.rs` 存在 | >=1 |
| **V8** | simhash 旧实现删除 | ❌ | `tantivy_index.rs` 和 `adr_index.rs` 仍有实现 | 0 |
| **V9** | codex 文件清理 | ❌ | 20 个非 lib.rs 文件残留 | 0 |
| **V10** | 整体编译 | ⏸️ | 未执行（因 V8/V9 问题） | 0 错误 |

**关键失败**:
- **V8**: 两处旧 simhash 实现未删除
- **V9**: 20 个 codex-twist 文件未清理

---

## 问题与建议

### 短期（立即处理）— 返工项
| 优先级 | 问题 | 修复步骤 | 验证 |
|:---|:---|:---|:---|
| **P0** | simhash64 统一不完整 | 1. 删除 `tantivy_index.rs:91-101` 本地实现<br>2. 删除 `adr_index.rs:16-26` 本地实现<br>3. 添加 `use foundation::hash::{simhash64, get_shard_id};` | V8=0, `grep 'use foundation::hash'` |
| **P0** | codex-twist 清理不彻底 | 1. 删除 `crates/hajimi-codex-twist/src/` 下除 `lib.rs` 外所有文件<br>2. 修改 `lib.rs` 为 thin wrapper: `pub use intelligence::codex_twist::*;`<br>3. 添加 `#![deprecated(...)]` | V9=0, `cargo check -p hajimi-codex-twist` |
| **P1** | 版本锁定不足 | 添加 3+ 个 `version = "=x.y.z"` 到 `Cargo.toml` | V5>=10 |

### 中期（Week 5 内）— 建议项
| 优先级 | 问题 | 建议 |
|:---|:---|:---|
| P2 | DEBT-CRYPTO-004 文档化 | 补写 `docs/debt/DEBT-CRYPTO-004.md` 记录低内存环境风险 |
| P2 | simhash 回归测试 | 添加跨模块一致性测试，确保三处使用相同算法 |
| P3 | codex-twist 下游通知 | 通知所有依赖 `hajimi-codex-twist` 的 crate 迁移到 `intelligence::codex_twist` |

### 长期（Week 6+ 考虑）
- **统一哈希库扩展**: 考虑将其他重复算法（如有）统一纳入 `foundation::hash`
- **crate 清理策略**: 建立废弃 crate 清理流程（3 版本过渡期后删除）

---

## 熔断检查

| 熔断 ID | 触发条件 | 状态 | 说明 |
|:---|:---|:---:|:---|
| **HASH-001** | V8 失败（simhash 旧实现残留）或行为不一致 | ✅ **触发** | tantivy_index.rs 和 adr_index.rs 仍有本地实现 |
| **API-001** | V10 失败（codex-twist 清理后编译失败） | ⏸️ **未验证** | 因 V9 问题，V10 暂缓执行 |
| **CRYPTO-001** | V6 失败（Scrypt 残留） | ❌ 未触发 | 仅注释提及，代码已移除 |
| **DEBT-001** | V1 失败（DEBT-P0-001 缺失） | ❌ 未触发 | 文档完整 |

**熔断触发**: HASH-001（simhash 统一不完整）

---

## 执行许可

| 检查项 | 状态 | 说明 |
|:---|:---:|:---|
| DEBT-P0-001 完整 | ✅ | Week 4 交付物就绪 |
| P0 代码优化 | ✅ | sha256sum/PSK/Owner 全部到位 |
| cargo audit CI | ✅ | 阻塞式安全扫描 |
| Scrypt 移除 | ✅ | 纯 Argon2id |
| **simhash 统一** | ❌ | 旧实现未删除，未引用新库 |
| **codex-twist 清理** | ❌ | 20 文件残留，非 thin wrapper |
| **版本锁定** | ⚠️ | 7<10，建议补充 |

**最终裁决**: 🔴 **返工**

**返工范围**:
1. simhash64 统一（2 文件修改，删除本地实现，引用新库）
2. codex-twist 清理（删除 20 文件，lib.rs 改为 thin wrapper）
3. 可选：Cargo.toml 添加 3+ 版本锁定

**返工后验证**:
```bash
# 必须全部通过
grep -c 'fn simhash64' src/engine/search/src/tantivy_index.rs  # 0
grep -c 'fn simhash64' src/intelligence/knowledge/src/adr_index.rs  # 0
grep -c 'use foundation::hash' src/engine/search/src/tantivy_index.rs  # >=1
grep -c 'use foundation::hash' src/intelligence/knowledge/src/adr_index.rs  # >=1
find src/crates/hajimi-codex-twist -name '*.rs' -not -name 'lib.rs' | wc -l  # 0
cargo check --workspace  # 0 errors
```

---

## 归档建议

| 资产 | 路径 | 说明 |
|:---|:---|:---|
| 审计报告 | `audit report/week3-4/Week3-4-ACCEPTANCE-AUDIT-002.md` | 本文件 |
| 债务文档 | `docs/debt/DEBT-P0-001.md` | 完整 ✅ |
| 待修复 | `src/engine/search/src/tantivy_index.rs` | simhash 本地实现 |
| 待修复 | `src/intelligence/knowledge/src/adr_index.rs` | simhash 本地实现 |
| 待清理 | `src/crates/hajimi-codex-twist/src/*` (除 lib.rs) | 20 文件 |
| 待优化 | `Cargo.toml` | 版本锁定 7→10+ |

**审计链**: P0 验收 → Week 3-4 执行 → **返工** → Week 3-4 复验 → Week 5 启动

---

*本审计报告基于实际代码验证生成，所有验证命令可复现*  
*关键问题: simhash 统一不完整 (V8❌), codex-twist 清理不彻底 (V9❌)*  
*压力怪盖章: C 级，返工，修完两处再进 Week 5* ☝️🐍♾️⚖️🟠
