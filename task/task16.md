🚀 饱和攻击波次：HAJIMI-PHASE3-B-02-04-FULL（豪华版批量+集成+债务归档全量开发）

火力配置：3 Agent 并行（B-02/04 批量优化工程师、B-03/04 业务集成工程师、B-04/04 债务归档审计员）

轰炸目标：完成Phase 3剩余3项工单（批量写入系统优化→限流业务集成→债务归档验证），产出《HAJIMI-PHASE3-COMPLETE-白皮书-v1.0.md》+《HAJIMI-PHASE3-COMPLETE-自测表-v1.0.md》（66项全绿）

输入基线：B-01/04 A级归档产物（ID-180事实源）
- `src/security/rate-limiter-sqlite-luxury.js`（B-01/04最终版，含8行修复代码）
- `tests/luxury-base.test.js`（18/18全绿基线）
- `docs/audit report/16/16-AUDIT-FIX-001-修复验收审计报告.md`（A级审计结论）

---

⚠️ 质量门禁（开工前必须全部满足）

检查点	自检问题	覆盖情况([ ] 已覆盖)	相关用例ID	备注	
CF-核心功能	B-01/04的18/18测试是否全绿？	[ ]	B-01-001018	必须验证	
RG-约束回归	DEBT-SEC-001是否已清偿？	[ ]	DEBT-SEC-001	必须验证	
NG-负面路径	是否有批量写入失败/限流熔断的NG用例？	[ ]	NG-BATCH-001003	必须设计	
UX-用户体验	限流提示信息是否友好（非技术错误码）？	[ ]	UX-THROTTLE-001	必须设计	
E2E-端到端	是否有从请求→限流→存储的完整链路测试？	[ ]	E2E-PHASE3-001	必须设计	
High-高风险	批量写入数据一致性（崩溃恢复）是否覆盖？	[ ]	HIGH-CRASH-001	必须设计	
字段完整性	前置条件/测试环境/预期结果是否完整？	[ ]	ALL	必须检查	
需求映射	是否关联到SPEC_ID（债务清偿需求）？	[ ]	SPEC-DEBT-001003	必须映射	
自测执行	是否已执行一轮自测并处理Fail项？	[ ]	ALL	必须执行	
范围边界	是否明确标注"不覆盖WASM 5x加速"？	[ ]	SCOPE-EXCLUDE	必须声明	

门禁结论：10/10项勾选且全绿方可开工，任一[ ]视为D级前置失败。

---

📋 工单矩阵（3 Agent并行）

工单 B-02/04 [批量写入系统优化工程师]
目标：基于B-01/04豪华版架构，实现生产级批量写入优化（吞吐>1000 ops/s，崩溃零丢失）

输入（精确到行号）：

输入项	强制要求	验证命令	
B-01/04基线代码	`src/security/rate-limiter-sqlite-luxury.js`第1-400行（含WAL+队列）	`grep "journal_mode.*wal" src/security/rate-limiter-sqlite-luxury.js` 命中	
写入队列实现	第45-60行（writeQueue数组+batchSize=100）	`grep "batchSize.*100" src/security/rate-limiter-sqlite-luxury.js` 命中	
批量写入方法	第120-150行（batchWrite方法）	`grep "async batchWrite" src/security/rate-limiter-sqlite-luxury.js` 命中	

输出交付物（5件套）：
1. `src/storage/batch-writer-optimized.js`（批量写入优化器，含压缩+异步刷盘）
2. `tests/batch-writer-stress.test.js`（压力测试，验证>1000 ops/s）
3. `docs/task16-batch/B-02-04-批量优化-白皮书-v1.0.md`（4章：设计/实现/压测/债务）
4. `docs/task16-batch/B-02-04-批量优化-自测表-v1.0.md`（22项刀刃，全绿）
5. `docs/benchmarks/BATCH-PERF-v1.0.md`（性能基准报告，含崩溃恢复测试）

刀刃风险自测表（B-02/04专用，16项）：

用例ID	类别	场景	验证命令（可复制）	通过标准	状态(Engineer填)	
BATCH-001	FUNC	批量写入触发	`grep "if (this.writeQueue.length >= this.batchSize)" src/storage/batch-writer-optimized.js`	命中条件判断	[ ]	
BATCH-002	FUNC	事务包装	`grep "BEGIN TRANSACTION" src/storage/batch-writer-optimized.js`	命中	[ ]	
BATCH-003	FUNC	压缩启用	`grep "zstd\|gzip\|compress" src/storage/batch-writer-optimized.js`	命中压缩逻辑	[ ]	
BATCH-004	FUNC	异步刷盘	`grep "fsync\|flush" src/storage/batch-writer-optimized.js`	命中刷盘调用	[ ]	
BATCH-005	FUNC	崩溃恢复	`grep "recover\|replay" src/storage/batch-writer-optimized.js`	命中恢复逻辑	[ ]	
BATCH-006	CONST	吞吐>1000 ops/s	`node tests/batch-writer-stress.test.js` 输出	`ops/s >= 1000`	[ ]	
BATCH-007	CONST	数据零丢失	测试脚本模拟崩溃后验证	`lostCount === 0`	[ ]	
BATCH-008	NEG	磁盘满处理	`grep "ENOSPC" src/storage/batch-writer-optimized.js`	命中错误处理	[ ]	
BATCH-009	NEG	并发写入冲突	测试脚本并发写入同一Key	无数据竞争	[ ]	
BATCH-010	NEG	队列溢出	写入速度>刷盘速度10	graceful degradation	[ ]	
BATCH-011	UX	进度反馈	`grep "progress\|emit.*progress" src/storage/batch-writer-optimized.js`	命中进度事件	[ ]	
BATCH-012	UX	错误重试提示	`grep "retry\|backoff" src/storage/batch-writer-optimized.js`	命中重试逻辑	[ ]	
BATCH-013	E2E	端到端批量流	从API→批量写入→存储完整链路	延迟<100ms	[ ]	
BATCH-014	HIGH	崩溃一致性	`kill -9`模拟后重启验证数据	100%一致	[ ]	
BATCH-015	HIGH	WAL完整性	`grep "wal\|journal" src/storage/batch-writer-optimized.js`	WAL模式启用	[ ]	
BATCH-016	DEBT	性能债务声明	`docs/benchmarks/BATCH-PERF-v1.0.md`含"当前限制"章节	诚实声明	[ ]	

---

工单 B-03/04 [限流业务集成工程师]
目标：将限流器深度集成到Hajimi核心业务流程（API层+存储层+WebSocket），实现全链路限流保护

输入（精确到行号）：

输入项	强制要求	验证命令	
B-01/04限流器	`src/security/rate-limiter-sqlite-luxury.js`第80-110行（consumeToken逻辑）	`grep "async consumeToken" src/security/rate-limiter-sqlite-luxury.js` 命中	
API服务器	`src/api/server.js`（Express/Fastify路由）	`ls src/api/server.js` 存在	
存储层入口	`src/storage/index.js`（统一存储接口）	`ls src/storage/index.js` 存在	

输出交付物（5件套）：
1. `src/middleware/rate-limit-middleware.js`（API层限流中间件）
2. `src/storage/rate-limited-storage.js`（存储层限流包装器）
3. `tests/integration/rate-limit-e2e.test.js`（端到端限流测试）
4. `docs/task17-integration/B-03-04-业务集成-白皮书-v1.0.md`（4章：架构/集成点/熔断策略/降级方案）
5. `docs/task17-integration/B-03-04-业务集成-自测表-v1.0.md`（22项刀刃，全绿）

刀刃风险自测表（B-03/04专用，16项）：

用例ID	类别	场景	验证命令（可复制）	通过标准	状态(Engineer填)	
INTEG-001	FUNC	API中间件挂载	`grep "app.use.*rateLimit" src/api/server.js`	命中	[ ]	
INTEG-002	FUNC	存储层包装	`grep "RateLimitedStorage" src/storage/rate-limited-storage.js`	类定义存在	[ ]	
INTEG-003	FUNC	429响应码	`grep "429\|Too Many Requests" src/middleware/rate-limit-middleware.js`	命中	[ ]	
INTEG-004	FUNC	限流头信息	`grep "X-RateLimit-Limit\|X-RateLimit-Remaining" src/middleware/rate-limit-middleware.js`	命中响应头	[ ]	
INTEG-005	FUNC	WebSocket限流	`grep "ws\|websocket" src/middleware/rate-limit-middleware.js`	命中WS处理	[ ]	
INTEG-006	CONST	不同IP独立限流	测试脚本多IP并发	各IP计数器独立	[ ]	
INTEG-007	CONST	熔断阈值配置	`grep "circuitBreaker\|fuse" src/middleware/rate-limit-middleware.js`	命中熔断配置	[ ]	
INTEG-008	NEG	绕过检测	直接调用存储层绕过中间件	仍触发限流（存储层保护）	[ ]	
INTEG-009	NEG	时钟回拨攻击	系统时间回拨测试	限流器不重置（防刷）	[ ]	
INTEG-010	NEG	大流量突发	1000 req/s突发测试	graceful queue或拒绝	[ ]	
INTEG-011	UX	友好错误提示	`grep "message.*请稍后再试\|retry after" src/middleware/rate-limit-middleware.js`	命中中文友好提示	[ ]	
INTEG-012	UX	重试时间提示	响应含`Retry-After`头	值正确计算	[ ]	
INTEG-013	E2E	API→限流→存储	完整请求链路	全流程<50ms	[ ]	
INTEG-014	HIGH	限流器故障降级	模拟SQLite故障	降级到内存限流（不停服）	[ ]	
INTEG-015	HIGH	雪崩保护	后端过载时限流收紧	自动降低阈值	[ ]	
INTEG-016	DEBT	集成债务声明	白皮书含"未覆盖场景"章节	诚实声明	[ ]

工单 B-04/04 [债务归档审计员]
目标：完成DEBT-SEC-001及Phase 3所有技术债务的最终验证、文档归档与清偿证明
输入（精确到文件）：

输入项	强制要求	验证命令	
DEBT-SEC-001修复	`src/security/rate-limiter-sqlite-luxury.js`第186-193行（队列优先修复）	`grep -A 8 "for (let i = this.writeQueue.length - 1" src/security/rate-limiter-sqlite-luxury.js` 命中	
16号审计报告	`docs/audit report/16/16-AUDIT-FIX-001-修复验收审计报告.md`	`ls docs/audit report/16/` 存在	
B-02/04产物	`docs/benchmarks/BATCH-PERF-v1.0.md`	待生成	
B-03/04产物	`docs/task17-integration/B-03-04-业务集成-白皮书-v1.0.md`	待生成

输出交付物（5件套）：
1. 
 docs/debt/DEBT-PHASE3-FINAL-CLEARANCE.md （Phase 3债务最终清偿证明）
2. 
 docs/audit report/17/17-AUDIT-PHASE3-FINAL-债务归档审计报告.md （17号审计报告，A/B/C/D评级）
3. 
 docs/PHASE3-COMPLETION-REPORT.md （Phase 3完成报告，含3工单汇总）
4. 
 docs/task18-debt/B-04-04-债务归档-白皮书-v1.0.md （4章：债务清单/清偿证据/验证过程/剩余债务）
5. 
 docs/task18-debt/B-04-04-债务归档-自测表-v1.0.md （22项刀刃，全绿）
 
 用例ID	类别	场景	验证命令（可复制）	通过标准	状态(Engineer填)	
DEBT-001	FUNC	DEBT-SEC-001标记清偿	`grep "DEBT-SEC-001.*✅.*已清偿" docs/debt/DEBT-PHASE3-FINAL-CLEARANCE.md`	命中	[ ]	
DEBT-002	FUNC	清偿证据链完整	文档含16号审计报告引用+测试证据	证据完整	[ ]	
DEBT-003	FUNC	新债务发现记录	含B-02/04、B-03/04产生的新债务	诚实记录	[ ]	
DEBT-004	FUNC	债务分级正确	P0/P1/P2分级符合规范	无P0遗留	[ ]	
DEBT-005	FUNC	17号审计报告生成	`ls docs/audit report/17/17-AUDIT-PHASE3-FINAL-债务归档审计报告.md`	文件存在	[ ]	
DEBT-006	CONST	债务清偿率	计算Phase 3债务清偿比例	≥90%	[ ]	
DEBT-007	CONST	文档完整性	白皮书4章齐全	章节完整	[ ]	
DEBT-008	NEG	无隐藏债务	`grep -i "隐藏\|未声明\|未记录" docs/debt/DEBT-PHASE3-FINAL-CLEARANCE.md`	无结果	[ ]	
DEBT-009	NEG	无虚假清偿	验证所有声称"已清偿"债务的测试存在	测试可运行	[ ]	
DEBT-010	NEG	无重复债务	债务ID无重复（去重检查）	ID唯一	[ ]	
DEBT-011	UX	债务可追溯	每个债务含Git commit引用	可溯源	[ ]	
DEBT-012	UX	清偿过程可读	含时间线+责任人+验证人	信息完整	[ ]	
DEBT-013	E2E	端到端债务验证	从债务声明→修复→验证→归档完整流程	闭环	[ ]	
DEBT-014	HIGH	P0债务清零	`grep "P0.*债务" docs/debt/DEBT-PHASE3-FINAL-CLEARANCE.md`	数量为0或均已清偿	[ ]	
DEBT-015	HIGH	审计链完整	09→10→12→13→14→15→16→17号审计连续	无断号	[ ]	
DEBT-016	DEBT	剩余债务声明	明确声明Phase 4需处理的债务	诚实声明	[ ]

📊 P4自测轻量检查表（全局，10项）

CHECK_ID	检查项（覆盖情况由执行者填写）	覆盖情况	
P4-GLOBAL-001	B-01/04基线18/18测试全绿已验证	[ ]	
P4-GLOBAL-002	B-02/04批量优化22项刀刃自测设计完成	[ ]	
P4-GLOBAL-003	B-03/04业务集成22项刀刃自测设计完成	[ ]	
P4-GLOBAL-004	B-04/04债务归档22项刀刃自测设计完成	[ ]	
P4-GLOBAL-005	3个工单输出交付物路径符合规范	[ ]	
P4-GLOBAL-006	66项刀刃自测（22×3）全部手动勾选	[ ]	
P4-GLOBAL-007	端到端测试覆盖（B-02/04B-04/04联动）	[ ]	
P4-GLOBAL-008	性能基准报告（B-02/04吞吐>1000 ops/s）	[ ]	
P4-GLOBAL-009	17号审计报告生成（B-04/04产出）	[ ]	
P4-GLOBAL-010	3个白皮书+3个自测表+1个完成报告落盘	[ ]

🚫 D级红线（地狱难度，触发即永久失败）
1. 
B-02/04吞吐<500 ops/s → 永久失败（性能不达标）
2. 
B-02/04崩溃测试数据丢失>0 → 永久失败（一致性崩溃）
3. 
B-03/04限流绕过测试失败 → 永久失败（安全漏洞）
4. 
B-04/04发现未声明债务 → 永久失败（隐瞒债务）
5. 
任何工单自测表未手动勾选（自动生成或留空）→ 永久失败
6. 
B-03/04未实现熔断降级 → 永久失败（高风险缺失）
7. 
17号审计报告缺失或评级<C → 永久失败（归档失败）
8. 
超时6小时 → 永久失败（工期违约）


🎯 数值化验收（可复制命令）

# B-02/04验收
node tests/batch-writer-stress.test.js | grep "ops/s"  # 预期：>= 1000
node tests/batch-writer-crash.test.js | grep "lost"    # 预期：lost: 0

# B-03/04验收  
node tests/integration/rate-limit-e2e.test.js | grep "passed"  # 预期：22 passed
curl -s http://localhost:3000/api/test -H "X-Forwarded-For: 1.2.3.4"  # 429测试

# B-04/04验收
ls docs/audit report/17/17-AUDIT-PHASE3-FINAL-债务归档审计报告.md  # 存在
grep "A/Go\|B/Go" docs/audit report/17/17-AUDIT-PHASE3-FINAL-债务归档审计报告.md  # 命中

⚡ 战术金句
"B-01/04的A级不是终点，而是Phase 3豪华版的起点！批量写入要跑到1000 ops/s，限流集成要焊死每一道门，债务归档要钉死每一笔账——66项刀刃自测全绿，17号审计A级归档，这才是完美交付！☝️🐍♾️⚖️💥"
开工！ 3 Agent并行，6小时倒计时，66项自测全绿即胜利！