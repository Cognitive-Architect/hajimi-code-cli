🚀 饱和攻击波次：HAJIMI-PHASE1-FIX-001 硬钢修复集群

火力配置：2 Agent 并行（B-01/02 双线程硬钢）

轰炸目标：2项审计缺陷（C→A升级）→ 产出《FIX-REPORT-001.md》+《PHASE1-DEBT-v1.1.md》

输入基线：审计报告 PHASE1-AUDIT-001-REPORT.md（C级缺陷清单）

---

🔧 工单矩阵（2Agent并行）

工单 B-01/02 [唐音-修复专家] → FIX-001（P1-致命Chunk BUG）

目标：修复Chunk文件头解析不一致，7/7测试全绿

输入：`src/storage/chunk.js` 第162行（`_parseHeader`函数）

输出：`src/storage/chunk.js`（修复后）+ 测试通过日志

自测点：
- CHUNK-FIX-001: 文件头128字节对齐验证
- CHUNK-FIX-002: 元数据JSON解析一致性
- CHUNK-FIX-003: 读写端到端测试（put→get）

---

工单 B-02/02 [唐音-测试专家] → FIX-002（P2-连接池测试）

目标：修复连接池测试代码作用域错误，7/7测试全绿

输入：`src/test/connection-pool.test.js`（变量`pool`未定义处）

输出：`src/test/connection-pool.test.js`（修复后）+ 测试通过日志

自测点：
- POOL-FIX-001: 单分片连接创建（POOL-001）
- POOL-FIX-002: 16分片并发查询（POOL-002）
- POOL-FIX-003: 连接上限8/分片（POOL-003）
- POOL-FIX-004: 错误重试机制（POOL-004）

---

⚠️ 质量门禁（必须全部满足才能收卷）

- B-01交付：`node src/test/chunk.test.js` → 7/7 PASS（截图证明）
- B-02交付：`node src/test/connection-pool.test.js` → 7/7 PASS（截图证明）
- 集成验证：`node src/test/storage-integration.test.js` → 6/6 PASS
- 路由验证：`node src/test/shard-router.test.js` → 8/8 PASS（回归测试）
- 债务更新：`docs/PHASE1-DEBT-v1.1.md` 补充2项隐藏债务（DEBT-PHASE1-HIDDEN-001/002）
- P4检查：10/10项全部勾选（CF/RG/NG/UX/E2E/High/字段/需求/执行/范围）

---

📦 收卷强制交付物（2份）

1. 《FIX-REPORT-001.md》（修复报告）
   - 修复项清单（FIX-001/002）
   - 修改文件diff（前后对比）
   - 测试通过证明（命令+输出截图）
   - 工时记录（B-01: 5min, B-02: 30min）

2. 《PHASE1-DEBT-v1.1.md》（债务更新）
   - 原3项债务（DEBT-PHASE1-001003）
   - 新增2项隐藏债务（DEBT-PHASE1-HIDDEN-001/002）
   - 清偿状态更新（FIX-001/002已清偿）

---

🎯 修复后评级目标

```
当前: C级（需修复后通过）
目标: A级（全部通过，债务诚实）
路径: FIX-001(5min) + FIX-002(30min) + 重新审计(10min) = 45分钟C→A
```

---

📋 P4自测轻量检查表（2工单×10项）

检查点	B-01 Chunk修复	B-02 连接池修复	
CF核心功能	CHUNK-FIX-001/002	POOL-FIX-001/002	
RG约束回归	不破坏既有格式	不降低并发性能	
NG负面路径	非法header处理	连接超限处理	
UX用户体验	错误提示可读	测试输出清晰	
E2E端到端	put→get一致性	16分片全链路	
High高风险	headerSize对齐	连接泄漏检测	
字段完整性	3项自测全填	4项自测全填	
需求映射	FIX-001	FIX-002	
执行结果	7/7 PASS	7/7 PASS	
范围边界	仅chunk.js	仅connection-pool.test.js