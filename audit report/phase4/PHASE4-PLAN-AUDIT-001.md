# PHASE4-PLAN-AUDIT-001 Phase 4详细开发计划审计报告

## 审计结论
- **计划评级**: 🟢 **A级（优秀，16周地狱精度）**
- **状态**: ✅ **Go**（Phase 4开发启动授权）
- **16周规划**: 16/16周完整（无空白周）
- **技术约束**: 零unsafe/any明确继承

---

## 16周地狱精度开发计划（Week 26-41）

### Month 1: 5层记忆系统基础（Week 26-29）

| 周次 | 交付物 | 行数 | 技术约束 | 债务清偿 |
|:---:|:---|:---:|:---|:---:|
| **W26** | `src/memory/session.rs` + `mod.rs` | 320±32 | 零unsafe，LRU Cache O(1) | - |
| **W27** | `src/memory/auto.rs` + DEBT-GIT-CLI-W11 | 280±28 | 零unwrap，文件原子写入 | ✅ GIT-CLI |
| **W28** | `src/memory/dream.rs` + `scheduler.rs` | 350±35 | ONNX 384维<500ms，零any | - |
| **W29** | `src/memory/graph.rs` + DEBT-PERF-W25 | 300±30 | SQLite-based自研图，E2E测试 | ✅ PERF观察 |

**Month 1小计**: 1250行（目标1200±120）

**技术细节Q1（5层记忆）**: 
- **Session层**: 内存HashMap+LRU，4K tokens上限，自动驱逐
- **Auto层**: `~/.hajimi/memory/{project}/`目录，JSON Lines格式
- **Dream层**: SQLite存储，ONNX Runtime 384维向量，夜间Cron整理
- **Graph层**: 自研SQLite图数据库（节点/边表），避免SurrealDB重依赖
- **Cloud层**: libsodium XChaCha20-Poly1305 E2E加密，可选私有云同步

---

### Month 2: 上下文压缩+索引（Week 30-33）

| 周次 | 交付物 | 行数 | 技术约束 |
|:---:|:---|:---:|:---|
| **W30** | `src/compression/micro.rs` | 120±12 | 标记替换，每轮自动，强制开启 |
| **W31** | `src/compression/auto.rs` + `compact.rs` | 200±20 | Token 50k阈值，LLM API摘要 |
| **W32** | `src/index/semantic.rs`（HNSW集成） | 180±18 | rust-hnsw crate，零unsafe |
| **W33** | `src/index/fulltext.rs`（Tantivy） | 150±15 | tantivy crate，Rust原生 |

**Month 2小计**: 650行（目标600±60）

**技术细节Q2（压缩4层触发条件）**:
- **micro**: 强制开启，每轮自动执行，简单字符串替换（如`~/projects/`→`$PROJ`）
- **auto**: Token>50k触发，Claude API/本地模型（Ollama）自动摘要
- **compact**: 用户命令`/compact`触发，完整对话LLM摘要+归档
- **cascade**: P2观察项，可选CDC增强，手动开启（hajimi-cascade集成）

**技术细节Q3（索引与记忆关系）**:
- **HNSW语义索引**: Dream层向量复用（384维ONNX输出），独立HNSW构建
- **Tantivy全文索引**: Auto层文件内容索引，增量更新
- **统一查询接口**: `src/index/unified.rs`融合语义+全文结果（Week 37）

---

### Month 3: ADR+知识图谱（Week 34-37）

| 周次 | 交付物 | 行数 | 技术约束 |
|:---:|:---|:---:|:---|
| **W34** | `src/adr/mod.rs` + `template.rs` | 140±14 | ADR模板，Markdown格式 |
| **W35** | `src/knowledge/graph.rs` | 220±22 | 实体关系网络，SQLite存储 |
| **W36** | `src/experience/debug.rs` + `mod.rs` | 160±16 | panic hook捕获，调试日志 |
| **W37** | `src/index/unified.rs` + `sync.rs` | 130±13 | HNSW+Tantivy融合查询 |

**Month 3小计**: 650行（目标650±65）

**技术细节Q4（ADR数据流）**:
- **ADR存储**: `.hajimi/adr/ADR-{NNNN}-{title}.md`，Markdown+Frontmatter
- **知识图谱实体提取**: LSP解析复用（Week 13 LSP工具集），AST遍历函数/模块
- **经验记录**: panic hook自动捕获，调试日志结构化存储（`debug_logs.jsonl`）

---

### Month 4: 智能化+收官（Week 38-41）

| 周次 | 交付物 | 行数 | 技术约束 |
|:---:|:---|:---:|:---|
| **W38** | `src/personalization/style.rs` | 140±14 | LoRA准备，代码风格统计 |
| **W39** | `src/personalization/habits.rs` | 130±13 | 时间模式分析，隐私本地 |
| **W40** | `src/personalization/predict.rs` | 150±15 | Top-5建议，概率模型 |
| **W41** | AUDIT-PHASE4-001 + 文档 | - | 全局扫描/评级/债务清零 |

**Month 4小计**: 420行（目标420±42）

---

## 总规划统计

| 维度 | 数值 | 偏差 |
|:---|:---:|:---:|
| **总周次** | 16/16周 | 0% |
| **总行数** | 2970行 | +4.2%（vs 2850目标） |
| **Rust代码** | ~2600行 | 87% |
| **TypeScript** | ~370行 | 13%（配置/类型） |
| **零unsafe承诺** | 100% | 0例外 |
| **零any承诺** | 100% | 0例外 |

---

## 关键疑问回答（Q1-Q4）

### Q1: 5层记忆每层技术实现细节？
**审计结论**: ✅ **全部明确**

| 层级 | 存储 | 容量 | 技术实现 | 延迟 |
|:---:|:---|:---:|:---|:---:|
| Session | 内存 | 4K tokens | HashMap+LRU | O(1) |
| Auto | 本地文件 | 32K tokens/文件 | JSON Lines | O(1) |
| Dream | SQLite | 1M tokens | ONNX 384维+HNSW | <500ms |
| Graph | SQLite | 无上限 | 自研图数据库 | O(log n) |
| Cloud | 可选云端 | 无上限 | libsodium E2E | 网络延迟 |

### Q2: 上下文压缩4层触发条件？
**审计结论**: ✅ **触发条件明确**

| 层级 | 触发条件 | 实现技术 | 默认状态 |
|:---:|:---|:---|:---:|
| micro | 每轮自动 | 字符串替换 | 强制开启 |
| auto | Token>50k | LLM API/本地模型 | 开启 |
| compact | 用户命令`/compact` | LLM摘要 | 开启 |
| cascade | 手动开启 | hajimi-cascade CDC | **可选** |

**审计官裁决**: Cascade明确为P2可选功能，不影响Phase 4 A级收官。

### Q3: 双引擎索引与5层记忆的关系？
**审计结论**: ✅ **关系清晰**

```
Dream层 (SQLite) 
    ↓ ONNX 384维向量
HNSW语义索引 ←──→ Unified查询接口 ←── 用户查询
    ↑                      ↑
Tantivy全文索引 ←── Auto层文件索引
```

- **HNSW**: Dream层向量复用，避免重复推理
- **Tantivy**: Auto层文件内容全文索引
- **Unified**: Week 37实现融合查询（语义+全文混合排序）

### Q4: ADR与知识图谱数据流？
**审计结论**: ✅ **数据流明确**

```
代码编辑/LSP解析
    ↓
AST遍历 → 实体提取（函数/模块/类型）
    ↓
┌─────────────────┬─────────────────┐
↓                 ↓                 ↓
ADR记录(.md)   知识图谱(SQLite)   经验记录(.jsonl)
架构决策        实体关系网络        调试日志
```

- **ADR**: `.hajimi/adr/ADR-{NNNN}-{title}.md`
- **知识图谱**: SQLite节点/边表，自研轻量实现
- **经验记录**: panic hook自动捕获，LSP错误记录

---

## 技术选型裁决（Make vs Buy）

| 模块 | 原计划 | 审计裁决 | 理由 |
|:---|:---:|:---:|:---|
| **HNSW向量索引** | Buy | ✅ **Buy** | rust-hnsw成熟，零unsafe |
| **Tantivy全文** | Buy | ✅ **Buy** | tantivy crate，Rust原生 |
| **向量嵌入** | Buy | ✅ **Buy** | ONNX Runtime，<500ms |
| **图数据库** | Make(SurrealDB冲突) | ✅ **Make SQLite** | README vs config-examples冲突裁决：自研SQLite轻量实现，避免SurrealDB重依赖 |
| **加密同步** | Make | ✅ **Make** | libsodium E2E，隐私主权卖点 |
| **上下文压缩** | Make | ✅ **Make** | Hajimi-Diff升级，micro/auto/compact |

**关键冲突解决**: 
- config-examples提到`"backend": "surrealdb"`
- README提到"可选：SurrealDB 或嵌入式"
- **裁决**: 采用**自研SQLite图数据库**（嵌入式），SurrealDB作为Phase 5可选升级路径

---

## 验证结果（V1-V4）

| 验证ID | 验证项 | 结果 | 证据 |
|:---:|:---|:---:|:---|
| V1 | 16周全部规划 | ✅ | Week 26-41全部有交付物 |
| V2 | 行数估算精确 | ✅ | 2970行，偏差+4.2%在±10%内 |
| V3 | 技术约束继承 | ✅ | 零unsafe/any明确标注 |
| V4 | 债务清偿时点 | ✅ | GIT-CLI-W11(W27)，PERF-W25(W29) |

---

## Phase 4债务清单（进入时）

| 债务ID | 描述 | 清偿时点 | 状态 |
|:---|:---|:---:|:---:|
| DEBT-GIT-CLI-W11 | Git4工具CLI非git2 | **Week 27** | 计划清偿 |
| DEBT-PERF-W25 | 性能观察 | **Week 29** | 计划验证 |

---

## 风险与缓解

| 风险 | 等级 | 缓解措施 |
|:---|:---:|:---|
| ONNX Runtime体积大 | P2 | 可选依赖，默认关闭Dream层 |
| 图数据库性能不足 | P2 | SQLite+索引优化，Phase 5可升级SurrealDB |
| Cascade可选功能延迟 | P2 | 明确为P2，不影响A级收官 |

---

## 压力怪评语

> 🥁 **"还行吧"**（A级：16周全规划，行数精确，技术约束明确，债务时点清晰）
>
> 16周全部规划完成，无空白周。行数估算2970行（偏差+4.2%）。零unsafe/any约束100%继承。
>
> **技术选型冲突裁决**: 图数据库采用自研SQLite（轻量），SurrealDB作为Phase 5可选。Cascade明确为P2可选。
>
> **债务清偿**: GIT-CLI-W11(W27)，PERF-W25(W29)时点明确。
>
> Phase 4详细开发计划**A级通过**，Week 26启动授权Granted。
>
> ☝️🐍♾️⚖️🟢

---

## 衔尾蛇链

```
Phase 3(A) → PHASE4-PLAN-AUDIT-001(A) → Week 26-41（执行）→ AUDIT-PHASE4-001 → Phase 5
```

---

## 归档建议

- **审计报告**: `audit report/phase4/PHASE4-PLAN-AUDIT-001.md` ✅
- **16周计划**: 已包含在本报告中
- **技术选型裁决**: SQLite图数据库，Cascade P2可选
- **Phase 4准入**: **Granted**（Week 26启动）

---

*审计官: 黄瓜睦Architect + 压力怪联合审计*  
*日期: 2026-04-02*  
*审计链: Phase 3(A) → Phase 4计划(A) → Week 26启动*
