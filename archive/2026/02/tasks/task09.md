🚀 饱和攻击波次：HAJIMI-PHASE4-WASM-WORKER-ROBUST-001
火力配置：6 Agent 并行（WASM编译/Worker Thread/集成/鲁棒性/E2E/基准）
轰炸目标：Phase 4三大支柱（WASM编译+Worker Thread+磁盘鲁棒性）→ 产出《HAJIMI-PHASE4-白皮书-v1.0.md》+《HAJIMI-PHASE4-自测表-v1.0.md》
输入基线：ID-174（Phase 3 A级归档态，v2.6.2-PHASE3-A）+ crates/hajimi-hnsw/src/lib.rs（193行Rust代码）

---

⚠️ 质量门禁（必须全部满足才能开工）

检查项	验证命令	通过标准	
Phase 3已归档	`cat docs/audit report/08/08-AUDIT-PHASE3-FINAL.md \| grep "A/Go"`	命中"A/Go"	
Rust代码存在	`ls crates/hajimi-hnsw/src/lib.rs`	文件存在且>100行	
wasm-pack安装	`which wasm-pack` 或 `cargo install wasm-pack`	返回路径或安装中	
Node.js版本	`node --version`	v20+	
输入债务清单	`grep "DEBT-PHASE2-001\|DEBT-PHASE2-004" docs/task08-phase3-wasm-disk-api/HAJIMI-PHASE3-白皮书-v1.0.md`	命中2行	

未满足质量门禁→禁止开工→返回Orchestrator补全环境！

---

📋 工单矩阵（6 Agent并行）

工单 B-01/06 WASM编译工程师 → Rust→WASM编译与包生成
目标：完成wasm-pack编译，生成可加载的.wasm文件，清偿DEBT-PHASE2-001
输入：
- `crates/hajimi-hnsw/Cargo.toml`（现有配置）
- `crates/hajimi-hnsw/src/lib.rs`（193行现有代码）
输出：
- `crates/hajimi-hnsw/pkg/hajimi_hnsw.js`（WASM胶水代码）
- `crates/hajimi-hnsw/pkg/hajimi_hnsw_bg.wasm`（WASM二进制）
- `crates/hajimi-hnsw/pkg/package.json`（WASM包配置）
- `src/wasm/wasm-loader.js`（更新版，支持加载pkg）
自测点：
- WASM-COMP-001: `wasm-pack build`执行成功（Exit 0）
- WASM-COMP-002: pkg/目录生成（hajimi_hnsw.js存在）
- WASM-COMP-003: WASM文件大小<500KB（优化后）
- WASM-COMP-004: 胶水代码无语法错误（node -e "require('./pkg/hajimi_hnsw.js')"）
- WASM-COMP-005: 导出函数检查（grep "search\|insert" pkg/hajimi_hnsw.js）

工单 B-02/06 Worker Thread架构师 → 索引构建Worker化（清偿DEBT-PHASE2-004）
目标：将HNSW索引构建（大内存操作）移至Worker Thread，不阻塞主线程
输入：
- `src/vector/hnsw-index.ts`（现有索引实现）
- `src/api/server.js`（现有服务器，需Worker集成）
输出：
- `src/worker/hnsw-worker.js`（Worker Thread脚本，使用worker_threads）
- `src/worker/worker-pool.js`（Worker池管理器，支持多核）
- `src/worker/index-builder-bridge.js`（主线程→Worker桥接）
- `src/api/server-worker-integrated.js`（集成Worker的服务器）
自测点：
- WORKER-FUNC-001: Worker启动无错误（new Worker不抛异常）
- WORKER-FUNC-002: 索引构建在Worker中执行（console.log显示worker线程ID）
- WORKER-FUNC-003: 主线程不阻塞（构建时API仍可响应/health）
- WORKER-PERF-001: 构建期间API延迟<100ms（P99）
- WORKER-NEG-001: Worker崩溃自动重启（进程杀死后恢复）
- WORKER-NEG-002: Worker内存超限优雅降级（>300MB时回退主线程）

工单 B-03/06 WASM-JS集成工程师 → WASM与JS运行时集成
目标：实现WASM模式与JS模式的无缝切换，WASM加速>5倍
输入：
- `src/wasm/hnsw-bridge.js`（现有桥接层）
- `crates/hajimi-hnsw/pkg/`（B-01产出）
- `src/vector/hnsw-index.ts`（JS索引实现，作为fallback）
输出：
- `src/wasm/runtime-loader.js`（运行时WASM加载器，自动检测pkg存在）
- `src/vector/hnsw-index-hybrid.js`（混合索引：WASM可用用WASM，否则JS）
- `src/api/routes/vector-wasm.js`（WASM优化的向量路由）
自测点：
- WASM-INT-001: WASM自动检测（pkg存在时加载WASM，否则JS）
- WASM-INT-002: 数据传递无拷贝（Float32Array直接传递测试）
- WASM-PERF-001: WASM加速比>5x（对比纯JS版本，见B-06）
- WASM-INT-003: 内存共享正确（WASM内存增长不崩溃）
- WASM-NEG-001: WASM加载失败自动降级（删除pkg测试fallback）

工单 B-04/06 磁盘鲁棒性工程师 → ENOSPC优雅降级与边界强化
目标：实现磁盘满时的优雅降级，强化溢出边界处理
输入：
- `src/disk/overflow-manager.js`（现有溢出管理器）
- `src/disk/memory-mapped-store.js`（现有存储层）
输出：
- `src/disk/enospc-handler.js`（ENOSPC错误处理器，优雅降级策略）
- `src/disk/overflow-manager-v2.js`（增强版，含磁盘满检测）
- `src/disk/emergency-mode.js`（紧急模式：纯内存运行，禁止溢出）
自测点：
- DISK-ROB-001: ENOSPC检测（模拟磁盘满抛出错误）
- DISK-ROB-002: 优雅降级（磁盘满时切换纯内存模式，不崩溃）
- DISK-ROB-003: 写入队列防积压（磁盘满时队列暂停写入）
- DISK-ROB-004: 紧急模式API（GET /health返回emergency: true）
- DISK-ROB-005: 磁盘恢复自动恢复（清理空间后自动恢复溢出）

工单 B-05/06 E2E集成测试师 → WASM+Worker+磁盘三位一体验证
目标：验证Phase 4完整工作流：WASM加速+Worker不阻塞+磁盘鲁棒性
输入：
- B-01B-04所有产出
- `tests/e2e/wasm-disk-api.test.js`（现有E2E，需扩展）
输出：
- `tests/e2e/phase4-integration.test.js`（Phase 4集成测试）
- `tests/e2e/worker-stress.test.js`（Worker压力测试）
- `tests/e2e/disk-full-simulation.test.js`（磁盘满模拟测试）
自测点：
- E2E-PH4-001: 完整工作流（WASM构建→Worker索引→磁盘溢出→API查询）
- E2E-PH4-002: Worker不阻塞验证（构建10K向量时API延迟<50ms）
- E2E-PH4-003: 磁盘满模拟（模拟ENOSPC，系统不崩溃）
- E2E-PH4-004: WASM降级验证（删除WASM文件，自动回退JS）
- E2E-PH4-005: 并发混合负载（WASM查询+Worker构建+磁盘写入同时进行）

工单 B-06/06 基准测试师 → WASM 5x加速比验证与全面基准
目标：验证WASM比JS快5倍，Worker不阻塞主线程，磁盘模式性能可接受
输入：
- `tests/benchmark/performance.bench.js`（Phase 3基准）
- B-01（WASM编译产出）
- B-03（WASM-JS集成）
输出：
- `tests/benchmark/wasm-vs-js.bench.js`（WASM vs JS对比）
- `tests/benchmark/worker-blocking.bench.js`（Worker阻塞测试）
- `docs/BENCHMARK-PHASE4-REPORT.md`（基准测试报告）
自测点：
- BENCH-WASM-001: WASM查询加速比>5x（对比纯JS，10K向量搜索）
- BENCH-WASM-002: WASM构建加速比>3x（对比纯JS）
- BENCH-WORKER-001: Worker模式主线程零阻塞（API响应延迟<10ms）
- BENCH-HYBRID-001: 混合模式性能（WASM查询+JS构建同时运行）
- BENCH-DEBT-001: DEBT-PHASE2-001清偿验证（WASM功能可用）
- BENCH-DEBT-002: DEBT-PHASE2-004清偿验证（Worker功能可用）

---

🔪 刀刃风险自测表（36项，全部手动勾选）
用例ID	类别	场景	验证命令	通过标准	状态（Engineer填）	
WASM编译						
WASM-COMP-001	FUNC	wasm-pack编译	`cd crates/hajimi-hnsw && wasm-pack build`	Exit 0	[ ]	
WASM-COMP-002	FUNC	pkg目录生成	`ls crates/hajimi-hnsw/pkg/*.wasm`	文件存在	[ ]	
WASM-COMP-003	PERF	WASM文件大小	`ls -lh crates/hajimi-hnsw/pkg/*.wasm`	<500KB	[ ]	
WASM-COMP-004	FUNC	胶水代码加载	`node -e "require('./pkg/hajimi_hnsw.js')"`	无错误	[ ]	
WASM-COMP-005	FUNC	导出函数检查	`grep "search" crates/hajimi-hnsw/pkg/hajimi_hnsw.js`	命中	[ ]	
Worker Thread						
WORKER-FUNC-001	FUNC	Worker启动	`node -e "new (require('worker_threads').Worker)('./src/worker/hnsw-worker.js')"`	无错误	[ ]	
WORKER-FUNC-002	FUNC	构建在Worker中	日志显示worker线程ID	非主线程ID	[ ]	
WORKER-FUNC-003	FUNC	主线程不阻塞	构建时curl /health	返回<100ms	[ ]	
WORKER-PERF-001	PERF	构建时API延迟	压力测试	P99<100ms	[ ]	
WORKER-NEG-001	NEG	Worker崩溃重启	kill worker进程	自动重启	[ ]	
WORKER-NEG-002	NEG	内存超限降级	Worker RSS>300MB	回退主线程	[ ]	
WASM-JS集成						
WASM-INT-001	FUNC	WASM自动检测	检测pkg存在性	自动选择模式	[ ]	
WASM-INT-002	FUNC	无拷贝传递	Float32Array传递测试	引用相等	[ ]	
WASM-PERF-001	PERF	加速比>5x	对比测试	5x	[ ]	
WASM-INT-003	FUNC	内存共享	连续搜索不崩溃	无OOM	[ ]	
WASM-NEG-001	NEG	WASM降级	删除pkg目录	回退JS	[ ]	
磁盘鲁棒性						
DISK-ROB-001	FUNC	ENOSPC检测	模拟磁盘满	抛出特定错误	[ ]	
DISK-ROB-002	FUNC	优雅降级	磁盘满时运行	切换内存模式	[ ]	
DISK-ROB-003	FUNC	队列防积压	磁盘满时写入	队列暂停	[ ]	
DISK-ROB-004	FUNC	紧急模式API	GET /health	emergency:true	[ ]	
DISK-ROB-005	FUNC	自动恢复	清理空间后	恢复溢出	[ ]	
E2E集成						
E2E-PH4-001	E2E	完整工作流	`npm run test:e2e:phase4`	通过	[ ]	
E2E-PH4-002	E2E	Worker不阻塞	构建时API测试	延迟<50ms	[ ]	
E2E-PH4-003	E2E	磁盘满模拟	ENOSPC模拟	不崩溃	[ ]	
E2E-PH4-004	E2E	WASM降级	删除WASM后测试	回退JS	[ ]	
E2E-PH4-005	E2E	并发混合负载	混合压力测试	无死锁	[ ]	
基准测试						
BENCH-WASM-001	PERF	WASM查询>5x	10K向量搜索	5x	[ ]	
BENCH-WASM-002	PERF	WASM构建>3x	10K向量构建	3x	[ ]	
BENCH-WORKER-001	PERF	主线程零阻塞	API延迟测试	<10ms	[ ]	
BENCH-HYBRID-001	PERF	混合模式	WASM+JS并发	性能可接受	[ ]	
债务清偿						
BENCH-DEBT-001	RG	DEBT-PHASE2-001	WASM功能可用	已清偿	[ ]	
BENCH-DEBT-002	RG	DEBT-PHASE2-004	Worker功能可用	已清偿	[ ]	
回归测试						
REG-PH3-001	RG	Phase 3功能保持	原E2E测试	仍通过	[ ]	
REG-API-001	RG	API兼容性	原API调用	无break	[ ]


📊 P4自测轻量检查表（10项，Engineer手动勾选）

检查点	自检问题	覆盖情况	相关用例ID	备注	
核心功能用例（CF）	WASM编译/Worker/集成/鲁棒性是否有≥1条CF用例？	[ ]	WASM-COMP, WORKER-FUNC, WASM-INT, DISK-ROB		
约束与回归用例（RG）	Phase 3功能保持、债务清偿是否有RG用例？	[ ]	REG-PH3, BENCH-DEBT		
负面路径/防炸用例（NG）	WASM加载失败、Worker崩溃、磁盘满是否有NG用例？	[ ]	WASM-NEG, WORKER-NEG, E2E-PH4-003		
用户体验用例（UX）	自动降级、紧急模式提示是否有UX用例？	[ ]	WASM-NEG-001, DISK-ROB-004		
端到端关键路径	WASM+Worker+磁盘三位一体E2E是否有？	[ ]	E2E-PH4-001		
高风险场景（High）	Worker内存超限、磁盘满、WASM内存增长是否有High用例？	[ ]	WORKER-NEG-002, DISK-ROB-001, WASM-INT-003		
关键字段完整性	所有36项刀刃用例是否填写完整（前置条件、预期结果、实际结果）？	[ ]	全部36项		
需求条目映射	是否映射到债务ID（DEBT-PHASE2-001/004）？	[ ]	BENCH-DEBT-001/002		
自测执行与结果	是否已执行36项自测？Fail项是否有处理路径？	[ ]	全部36项		
范围边界标注	未实现项是否明确标注（本轮不覆盖）？	[ ]	-

未10/10勾选→禁止收卷→返回补测！

✅ 验收标准（数值化，零容忍）

验收项	验收命令	通过标准	失败标准（触发D级）	
WASM编译	`wasm-pack build crates/hajimi-hnsw`	Exit 0 + pkg/存在	Exit非0或无pkg	
WASM加速比	`node tests/benchmark/wasm-vs-js.bench.js`	查询>5x	≤5x	
Worker不阻塞	`node tests/benchmark/worker-blocking.bench.js`	构建时API<10ms	≥100ms	
磁盘满降级	`node tests/e2e/disk-full-simulation.test.js`	ENOSPC时不崩溃	崩溃或死锁	
债务DEBT-PHASE2-001	`node -e "require('./src/wasm/runtime-loader').loadWASM()"`	返回WASM实例	报错或fallback	
债务DEBT-PHASE2-004	`grep "Worker" src/api/server-worker-integrated.js`	命中	未使用Worker	
36项刀刃自测	检查上表36项全部[x]	36/36	<36	
10项P4检查	检查上表10项全部[x]	10/10	<10	
E2E测试	`npm run test:e2e:phase4`	Exit 0	Exit非0

🚨 D级红线（地狱难度，触发即永久失败）
1. 
WASM编译失败（Exit非0）→ D级（DEBT-PHASE2-001无法清偿）
2. 
WASM加速比≤5x → D级（性能目标未达成）
3. 
Worker构建时API阻塞（延迟≥100ms）→ D级（核心目标失败）
4. 
磁盘满时系统崩溃 → D级（鲁棒性失败）
5. 
债务虚假清偿（声称WASM/Worker完成实际不可用）→ D级（诚信问题）
6. 
未提供36项刀刃自测 → D级（偷工减料）
7. 
Phase 3功能退化（原E2E失败）→ D级（回归失败）
8. 
超时6小时 → D级（工期失控）

