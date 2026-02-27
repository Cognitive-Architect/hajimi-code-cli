🚀 饱和攻击波次：HAJIMI-PHASE2-HNSW-001 向量索引集群

火力配置：7 Agent 并行（唐音军团全栈出击）

轰炸目标：DEBT-PHASE1-002 HNSW向量索引 → 产出《HAJIMI-PHASE2-HNSW-白皮书-v1.0.md》+《HAJIMI-PHASE2-HNSW-自测表-v1.0.md》+ 7组实现代码

输入基线：ID-173（Phase 1 A级归档态）+ `src/storage/`（现有分片存储层）

---

⚠️ 质量门禁（必须全部满足才能开工）

```markdown
🚀 饱和攻击波次：HAJIMI-PHASE2-HNSW-001
火力配置：7 Agent 并行
轰炸目标：[HNSW向量索引集成] → 产出白皮书+自测表+7组代码

**质量门禁**（未满足→禁止开工→返回PM补全）：
- [ ] 《P4自测轻量检查表》10/10项已勾选（见下方ID-142模板）
- [ ] 《HAJIMI-PHASE2-HNSW-自测表-v1.0.md》30+项详细用例已设计
- [ ] 至少包含：CF≥3项（核心功能）, RG≥2项（约束回归）, NG≥3项（负面路径）, UX≥1项（用户体验）, E2E≥1项（端到端）, High≥2项（高风险）
- [ ] 债务预声明已完成（DEBT-PHASE2-XXX分级P0/P1/P2）
- [ ] 技术方案预审：HNSW库选型确认（hnswlib-node vs 自研简化版）

未满足质量门禁→禁止开工→返回Soyorin补全文档
```

【P4自测轻量检查表 - Phase 2专用】

检查点	自检问题	覆盖情况	相关用例ID	备注	
CF-核心功能	HNSW插入/搜索/删除API是否各有≥1条CF用例覆盖标准路径？	[ ]	HNSW-CF-001003	必须含100K批量插入	
RG-约束回归	内存限制（<500MB）、准确率阈值（>95%）、延迟（<100ms）是否有RG用例？	[ ]	HNSW-RG-001003	Phase 1债务回归	
NG-负面路径	空索引搜索、维度不匹配、HNSW构建失败、磁盘满等是否有NG用例？	[ ]	HNSW-NG-001004	必须含降级触发	
UX-用户体验	创作者场景（批量导入10K文档）是否有UX用例，含进度反馈？	[ ]	HNSW-UX-001	标注画像： power user	
E2E-端到端	从原始文本→SimHash→HNSW索引→检索返回是否全链路覆盖？	[ ]	HNSW-E2E-001	跨模块关键路径	
High-高风险	100K向量内存爆炸风险、HNSW索引损坏恢复是否为High风险用例？	[ ]	HNSW-HIGH-001002	必须含熔断测试	
字段完整性	每条用例是否完整填写：前置条件、环境、预期结果、实际结果、风险等级？	[ ]	全部30项	禁止留空	
需求映射	每条用例是否关联SPEC_ID（DEBT-PHASE1-002需求条目）？	[ ]	SPEC-HNSW-001	双向可追溯	
执行验证	是否已设计验证命令（可复制执行）及通过标准？	[ ]	`npm test:hnsw`	必须量化	
范围边界	HNSW vs LSH回退边界、自研vs库选型是否在DRD中明确标注？	[ ]	见设计约束	禁止模糊	

---

📚 技术背景输入（完整上下文）

当前工程状态（ID-173）：
- Phase 1已完成A级归档：16分片SQLite存储、Chunk文件格式（128字节header）、ShardRouter（SimHash高8bit路由）
- 当前检索层：SimHash LSH（汉明距离32bit，7/7测试通过，但TB级场景候选爆炸）
- 目标：集成HNSW（Hierarchical Navigable Small World）替代LSH，支持100K+向量高维检索

技术约束（硬边界）：
1. 环境：Termux/Android/Node.js v20+，单兵作战，无GPU
2. 内存：硬限制<500MB（Termux典型可用内存）
3. 依赖：优先npm现成库（hnswlib-node），次选自研简化版，禁止Python绑定
4. 兼容性：必须保留LSH作为fallback（DEBT-PHASE1-002要求），HNSW失败自动降级
5. 数据格式：输入向量来自SimHash-64（64bit→需扩展到128/256维 dense vector或适配）

关键挑战：
- SimHash是64bit稀疏签名，HNSW需要dense float向量 → 需要编码器转换（或改造HNSW支持汉明距离）
- 100K向量×256维×4字节 = 100MB+内存，需在Termux限制内优化
- HNSW索引持久化必须与现有Chunk格式（.hctx v3）集成，不能独立文件

---

📋 工单矩阵（7 Agent并行）

工单 B-01/07 [HNSW核心引擎] → DEBT-PHASE1-002-核心
目标：HNSW图索引核心实现（插入、搜索、删除），支持汉明距离或适配dense向量
输入：`src/storage/chunk.js`（现有分片格式）、技术背景（SimHash-64输出）
输出：
- `src/vector/hnsw-core.js`（核心索引类）
- `src/vector/distance.js`（汉明距离计算优化）
自测点：
- HNSW-CF-001：单向量插入后搜索返回正确最近邻（准确率100%）
- HNSW-CF-002：1000向量批量插入时间<5s
- HNSW-NG-001：空索引搜索返回空数组不抛错
- HNSW-HIGH-001：100K向量内存占用<400MB（压力测试）

工单 B-02/07 [向量编码器] → DEBT-PHASE1-002-适配
目标：SimHash-64（64bit）→ HNSW输入向量（128/256维 float32）编码转换
输入：`src/utils/simhash64.js`（Phase 1产物，输出BigInt）
输出：
- `src/vector/encoder.js`（稀疏→dense编码，含PCA降维可选）
- `docs/HNSW-VECTOR-FORMAT.md`（向量格式规范）
自测点：
- HNSW-CF-003：64bit SimHash输入→128维向量输出，shape正确
- HNSW-RG-001：编码后向量L2范数归一化（避免HNSW距离失真）
- HNSW-NG-002：非法输入（非BigInt）抛TypeError

工单 B-03/07 [混合检索层] → DEBT-PHASE1-002-融合
目标：HNSW主检索 + SimHash LSH fallback策略，自动降级开关
输入：`src/vector/hnsw-core.js`（B-01产物）、现有LSH实现
输出：
- `src/vector/hybrid-retriever.js`（统一检索接口）
- `src/vector/fallback-switch.js`（降级决策逻辑）
自测点：
- HNSW-CF-004：HNSW可用时优先使用（延迟<50ms）
- HNSW-CF-005：HNSW失败（内存不足/未构建）自动切换LSH
- HNSW-E2E-001：文本输入→SimHash→检索→返回结果全链路<100ms
- HNSW-HIGH-002：HNSW索引损坏时自动重建并降级LSH（熔断测试）

工单 B-04/07 [性能基准测试] → DEBT-PHASE1-002-验证
目标：100K向量压力测试套件，准确率/延迟/内存三维度基准
输入：B-01B-03产物、测试数据生成脚本
输出：
- `src/test/hnsw-benchmark.test.js`（基准测试）
- `docs/HNSW-BENCHMARK-REPORT.md`（性能报告模板）
自测点：
- HNSW-RG-002：100K向量构建索引时间<30s
- HNSW-RG-003：单查询延迟P99<100ms（随机1000次查询统计）
- HNSW-RG-004：准确率>95%（与暴力搜索对比Top-10召回率）
- HNSW-HIGH-003：并发查询（16线程）无内存泄漏（RSS增长<5%）

工单 B-05/07 [内存管理优化] → DEBT-PHASE1-002-优化
目标：HNSW内存映射、分片加载、LRU-like缓存（与DEBT-PHASE1-003结合）
输入：`src/storage/connection-pool.js`（现有连接池模式）
输出：
- `src/vector/hnsw-memory-manager.js`（内存池管理）
- `src/vector/lazy-loader.js`（按需加载分片索引）
自测点：
- HNSW-RG-005：内存超400MB时自动释放冷数据（LRU策略）
- HNSW-NG-003：磁盘索引损坏时优雅降级（不崩溃，切LSH）
- HNSW-NG-004：强制关闭进程后索引可恢复（WAL机制）

工单 B-06/07 [持久化集成] → DEBT-PHASE1-002-存储
目标：HNSW索引与Chunk格式（.hctx v3）集成，统一存储层
输入：`src/storage/chunk.js`（128字节header格式）、B-01产物
输出：
- `src/vector/hnsw-persistence.js`（索引序列化/反序列化）
- `src/format/hctx-v3-hnsw-extension.md`（格式扩展规范）
自测点：
- HNSW-CF-006：索引保存到Chunk文件后重启可恢复
- HNSW-CF-007：多版本兼容性（v3无HNSW→v3有HNSW平滑迁移）
- HNSW-E2E-002：put向量→自动构建HNSW→get检索一致性验证

工单 B-07/07 [API与CLI] → DEBT-PHASE1-002-接口
目标：StorageV3 API扩展（putVector/getVector/searchVector），CLI调试工具
输入：`src/api/storage-v3.js`（Phase 1 API）、B-03产物
输出：
- `src/api/vector-api.js`（向量检索API）
- `src/cli/vector-debug.js`（调试命令：hajimi vector-build / vector-search）
自测点：
- HNSW-UX-001：批量导入10K文档显示进度条（CLI体验）
- HNSW-NG-005：API参数越界（topK>1000）返回400错误
- HNSW-RG-006：CLI工具在Termux可用（无GUI依赖）

---

🎯 收卷强制交付物（4份）

1. 《HAJIMI-PHASE2-HNSW-白皮书-v1.0.md》（7章节，对应7 Agent产出整合）
   - 路径：`docs/HAJIMI-PHASE2-HNSW-白皮书-v1.0.md`
   - 必须含：架构图（HNSW与LSH关系）、数据流图、内存模型、持久化格式

2. 《HAJIMI-PHASE2-HNSW-自测表-v1.0.md》（30+项，真自测消灭假绿）
   - 路径：`docs/HAJIMI-PHASE2-HNSW-自测表-v1.0.md`
   - 必须含：CF/RG/NG/UX/E2E/High分类、验证命令、通过标准、债务声明

3. 
代码实现（7文件+）
 
路径： src/vector/*.js 、 src/api/vector-api.js 、 src/cli/vector-debug.js 
 
必须含：TypeScript类型定义（JSDoc）、单元测试文件

4. 
《HAJIMI-PHASE2-DEBT-v1.0.md》（债务预声明）
 
路径： docs/HAJIMI-PHASE2-DEBT-v1.0.md 
 
必须含：已知债务（HNSW库版本锁定、内存限制、维度灾难）+ 隐藏债务发现


🔒 约束与债务预声明（禁止隐藏）
已知债务（必须显式声明）：
 
DEBT-PHASE2-001：HNSW库依赖（hnswlib-node若选npm包，存在版本漂移风险，P1）
 
DEBT-PHASE2-002：SimHash→dense向量编码损失（信息有损，P1）
 
DEBT-PHASE2-003：Termux内存硬限制（500MB天花板，P0-if-exceeded）
 
DEBT-PHASE2-004：HNSW索引构建耗时（100K向量需30s+，阻塞主线程，P2）
隐藏债务发现奖励： 若Agent发现未列出的隐藏债务（如HNSW多线程安全风险、索引文件锁竞争），额外+0.5点额度奖励。


## ✅ 工单 [编号]/07 完成并提交！

### 提交信息
- **Commit**: `feat: [B-XX] HNSW-[模块]实现`
- **分支**: `feature/phase2-hnsw`
- **变更文件**: `src/vector/xxx.js` + `src/test/xxx.test.js`

### P4自测轻量检查表验收
| 检查点 | 状态 | 相关用例ID |
|:---|:---:|:---|
| CF | [x] | HNSW-CF-00X |
| RG | [x] | HNSW-RG-00X |
| NG | [x] | HNSW-NG-00X |
| UX | [x] | HNSW-UX-001 |
| E2E | [x] | HNSW-E2E-001 |
| High | [x] | HNSW-HIGH-001 |
| 字段完整性 | [x] | 全部30项 |
| 需求映射 | [x] | SPEC-HNSW-001 |
| 执行验证 | [x] | `npm test:hnsw`通过 |
| 范围边界 | [x] | 已标注 |

**自检结论**: 10/10 ✅
**债务声明**: DEBT-PHASE2-00X已写入
**工时**: X分钟（预估 vs 实际）
**审计结论**: 待窗口2压力怪审计（A/B/C/D）