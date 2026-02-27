🚀 饱和攻击波次：HAJIMI-PHASE3-WASM-DISK-API-001
火力配置：6 Agent 并行（WASM/磁盘/API/迁移器/集成/基准）
轰炸目标：Phase 3三大支柱（WASM加速+磁盘溢出+HTTP API）→ 产出《HAJIMI-PHASE3-白皮书-v1.0.md》+《HAJIMI-PHASE3-自测表-v1.0.md》
输入基线：ID-174（Phase 2.1 A级归档态）+ v2.6.1-HARDENED代码库

---

⚠️ 质量门禁（必须全部满足才能开工）

检查项	验证命令	通过标准	
已阅读Phase 2.1残余债务	`grep -E "DEBT-PHASE2-001\|DEBT-PHASE2.1-001" docs/DEBT-GOVERNANCE-v1.0.md`	命中2行	
WASM工具链就绪	`rustc --version && wasm-pack --version`	返回版本号	
输入代码基线确认	`git checkout v2.6.1-HARDENED`	HEAD指向f68049b	
P4自测表理解	阅读下方《P4自测轻量检查表》10项	口头确认已理解	

未满足质量门禁→禁止开工→返回Orchestrator补全环境！

---

📋 工单矩阵（6 Agent并行）

工单 B-01/06 WASM架构师 → WASM基础与HNSW Rust核心
目标：将HNSW算法编译为WASM，实现比JS快5倍的向量检索
输入：
- `src/vector/hnsw-index.ts`（现有JS实现，400行，作为功能基准）
- `src/vector/hnsw-binary.ts`（二进制格式，需WASM兼容）
输出：
- `crates/hajimi-hnsw/Cargo.toml`（Rust项目配置）
- `crates/hajimi-hnsw/src/lib.rs`（HNSW核心算法Rust实现）
- `src/wasm/loader.ts`（WASM加载器，含内存管理）
- `src/wasm/hnsw-bridge.ts`（JS ↔ WASM胶水层）
自测点：
- WASM-001: Rust HNSW构建成功（`wasm-pack build` exit 0）
- WASM-002: 加载器异常处理（无效URL返回Error）
- WASM-003: JS→WASM数据传递（Float32Array无拷贝传递）
- WASM-004: WASM→JS结果返回（邻居ID数组正确解析）
- WASM-005: 内存限制遵守（WASM内存≤400MB）

工单 B-02/06 磁盘管理师 → 磁盘溢出管理器（Memory-Mapped）
目标：实现当向量>100K时自动溢出到磁盘，内存占用恒定在200MB以内
输入：
- `src/vector/hnsw-persistence.ts`（现有持久化层）
- `src/storage/lru-cache.ts`（缓存策略）
输出：
- `src/disk/memory-mapped-store.ts`（mmap文件管理器）
- `src/disk/overflow-manager.ts`（溢出决策器：内存>180MB时触发）
- `src/disk/block-cache.ts`（磁盘块缓存，LRU策略）
自测点：
- DISK-001: 100K向量内存<200MB（`process.memoryUsage().rss`）
- DISK-002: 磁盘文件创建（溢出时生成.hnsw.disk文件）
- DISK-003: 读写一致性（写入→读取→SHA256一致）
- DISK-004: 崩溃恢复（进程杀死后数据不丢失）
- DISK-005: 性能损失<50%（磁盘模式vs纯内存模式）

工单 B-03/06 API工程师 → HTTP RESTful服务器
目标：封装现有功能为HTTP API，支持跨进程/跨机器调用
输入：
- `src/index.ts`（现有入口）
- `src/vector/`（所有向量操作）
输出：
- `src/api/server.ts`（Fastify/Express服务器）
- `src/api/routes/vector.ts`（向量操作路由：POST /vector/add, POST /vector/search）
- `src/api/routes/health.ts`（健康检查：GET /health）
- `src/api/middleware/error-handler.ts`（统一错误处理）
自测点：
- API-001: 服务启动（`npm run server` 端口3000监听）
- API-002: 健康检查（GET /health 返回{status:"ok"}）
- API-003: 向量添加（POST /vector/add 返回201）
- API-004: 向量搜索（POST /vector/search 返回邻居数组）
- API-005: 错误处理（无效JSON返回400+错误码）
- API-006: 并发支持（100请求/秒不崩溃）

工单 B-04/06 迁移专家 → 二进制版本迁移器
目标：实现v1（JSON）→v2（Binary）自动迁移，支持未来v3扩展
输入：
- `src/format/hnsw-binary.ts`（当前version=1）
- `docs/HAJIMI-PHASE2.1-白皮书-v1.0.md`（格式规范）
输出：
- `src/migration/migrator.ts`（迁移协调器）
- `src/migration/v1-to-v2.ts`（JSON→Binary具体迁移逻辑）
- `src/migration/version-detector.ts`（魔数+版本号检测）
- `scripts/migrate.ts`（迁移CLI工具）
自测点：
- MIG-001: 版本检测（读取JSON返回version=0，读取Binary返回version=1）
- MIG-002: v1→v2迁移（旧文件读取→新文件写入→验证一致）
- MIG-003: 迁移原子性（失败时原文件不损坏）
- MIG-004: CLI工具（`npx hajimi-migrate ./data` 执行成功）
- MIG-005: 向后兼容（无迁移时直接读取v2不报错）

工单 B-05/06 集成测试师 → 端到端场景验证
目标：验证"WASM+磁盘+API"三位一体工作流
输入：
- 上述B-01B-04的所有产物
- `tests/e2e/`（目录需创建）
输出：
- `tests/e2e/wasm-disk-api.test.ts`（三位一体测试）
- `tests/e2e/stress.test.ts`（24小时稳定性测试脚本）
自测点：
- E2E-001: 完整工作流（WASM构建索引→磁盘溢出→API查询）
- E2E-002: 100K向量场景（插入10万条→内存<200MB→查询<100ms）
- E2E-003: 跨进程调用（API客户端在独立进程成功查询）
- E2E-004: 崩溃恢复（kill -9后重启数据完整）

工单 B-06/06 基准测试师 → 性能对比与债务清偿验证
目标：证明WASM比JS快5倍，磁盘模式可接受，所有P1债务清零
输入：
- `src/test/phase2.1-benchmark.test.ts`（Phase 2.1基线）
- `docs/DEBT-GOVERNANCE-v1.0.md`（债务清单）
输出：
- `tests/benchmark/wasm-vs-js.bench.ts`（对比测试）
- `tests/benchmark/disk-performance.bench.ts`（磁盘性能）
- `docs/DEBT-CLEARANCE-PHASE3.md`（债务清偿报告）
自测点：
- BENCH-001: WASM加速比>5x（对比Phase 2.1 JS版本）
- BENCH-002: 磁盘模式查询延迟<100ms（P99）
- BENCH-003: 债务DEBT-PHASE2-001清偿（WASM方案实现）
- BENCH-004: 债务DEBT-PHASE2.1-001清偿（迁移器实现）

---

🔪 刀刃风险自测表（24项，全部手动勾选）

用例ID	类别	场景	验证命令	通过标准	状态（Engineer填）	
WASM-FUNC-001	FUNC	Rust编译成功	`cd crates/hajimi-hnsw && wasm-pack build`	Exit 0	[ ]	
WASM-FUNC-002	FUNC	WASM加载成功	`node -e "require('./src/wasm/loader.ts').load()"`	返回WebAssembly.Instance	[ ]	
WASM-NEG-001	NEG	无效WASM文件	加载损坏.wasm文件	抛出Error	[ ]	
DISK-FUNC-001	FUNC	溢出触发	插入150K向量	生成.disk文件	[ ]	
DISK-FUNC-002	FUNC	内存恒定	监控`process.memoryUsage().rss`	<200MB	[ ]	
DISK-NEG-001	NEG	磁盘满	模拟磁盘满（dd if=/dev/zero）	优雅降级（不崩溃）	[ ]	
API-FUNC-001	FUNC	服务启动	`curl http://localhost:3000/health`	返回{status:"ok"}	[ ]	
API-FUNC-002	FUNC	向量添加	`curl -X POST -d '{"id":"a","vec":[0.1]}' /vector/add`	返回201	[ ]	
API-FUNC-003	FUNC	向量搜索	`curl -X POST -d '{"vec":[0.1],"k":10}' /vector/search`	返回邻居数组	[ ]	
API-NEG-001	NEG	无效JSON	`curl -X POST -d 'invalid' /vector/add`	返回400	[ ]	
API-NEG-002	NEG	超大向量	`curl -X POST -d '{"vec":[...10000维]}' /vector/add`	返回413	[ ]	
MIG-FUNC-001	FUNC	检测v1	读取Phase 2 JSON文件	返回version=0	[ ]	
MIG-FUNC-002	FUNC	检测v2	读取Phase 2.1 Binary文件	返回version=1	[ ]	
MIG-FUNC-003	FUNC	迁移执行	`npx ts-node scripts/migrate.ts ./data`	Exit 0	[ ]	
MIG-NEG-001	NEG	损坏文件	迁移损坏文件	失败但原文件保留	[ ]	
E2E-FUNC-001	E2E	三位一体	`npm run test:e2e`	全部通过	[ ]	
E2E-CONST-001	CONST	内存限制	100K向量E2E测试	RSS<200MB	[ ]	
E2E-CONST-002	CONST	延迟保证	E2E查询测试	P99<100ms	[ ]	
BENCH-FUNC-001	FUNC	WASM加速	`npm run bench:wasm`	比JS快5倍	[ ]	
BENCH-FUNC-002	FUNC	磁盘性能	`npm run bench:disk`	查询<100ms	[ ]	
DEBT-001	RG	DEBT-PHASE2-001	检查WASM实现	已清偿	[ ]	
DEBT-002	RG	DEBT-PHASE2.1-001	检查迁移器	已清偿	[ ]	
DEBT-003	RG	DEBT-PHASE2-004	检查Worker Thread	已清偿或延期	[ ]	
UX-001	UX	CLI迁移体验	`npx hajimi-migrate --help`	显示帮助	[ ]

📊 P4自测轻量检查表（10项，Engineer手动勾选）

检查点	自检问题	覆盖情况	相关用例ID	备注	
核心功能用例（CF）	每个核心功能（WASM/磁盘/API/迁移）是否有≥1条CF用例？	[ ]	WASM-FUNC, DISK-FUNC, API-FUNC, MIG-FUNC		
约束与回归用例（RG）	债务清偿（DEBT-001003）是否有RG用例验证？	[ ]	DEBT-001,002,003		
负面路径/防炸用例（NG）	是否覆盖无效WASM/磁盘满/无效JSON/损坏文件？	[ ]	WASM-NEG, DISK-NEG, API-NEG, MIG-NEG		
用户体验用例（UX）	CLI工具/help信息是否有UX用例？	[ ]	UX-001		
端到端关键路径	是否有"WASM+磁盘+API"三位一体E2E用例？	[ ]	E2E-FUNC-001		
高风险场景（High）	WASM内存限制/磁盘溢出/并发请求是否有High用例？	[ ]	WASM-005, DISK-FUNC-002, API-006		
关键字段完整性	所有24项刀刃用例是否填写：前置条件、预期结果、实际结果（Pass/Fail）、风险等级？	[ ]	全部24项		
需求条目映射	每项用例是否映射到债务ID（DEBT-PHASE2-001等）？	[ ]	DEBT列		
自测执行与结果	是否已执行24项自测？Fail项是否有处理路径？	[ ]	全部24项		
范围边界标注	未实现项（如Worker Thread）是否明确标注"本轮不覆盖"？	[ ]	DEBT-003

未10/10勾选→禁止收卷→返回补测！

✅ 验收标准（数值化，零容忍）

验收项	验收命令	通过标准	失败标准（触发D级）	
WASM编译	`wasm-pack build crates/hajimi-hnsw`	Exit 0	Exit非0	
WASM加速比	`npm run bench:wasm`	5x	≤5x	
磁盘内存限制	`node -e "const m=require('process').memoryUsage();console.log(m.rss/1024/1024)"`	<200MB	≥200MB	
API健康检查	`curl -s http://localhost:3000/health \| grep ok`	命中	超时或错误	
债务清偿	`grep "DEBT-PHASE2-001.*已清偿\|清零" docs/DEBT-CLEARANCE-PHASE3.md`	命中	未命中	
迁移器工作	`npx ts-node scripts/migrate.ts ./test-data && echo "OK"`	输出OK	Exit非0	
24项刀刃自测	检查上表24项全部[x]	24/24	<24	
10项P4检查	检查上表10项全部[x]	10/10	<10	
E2E测试	`npm run test:e2e`	Exit 0	Exit非0

🚨 D级红线（地狱难度，触发即永久失败）
1. 
WASM编译失败 → D级（Phase 3核心支柱崩塌）
2. 
磁盘模式内存>200MB → D级（无法解决Termux内存限制）
3. 
API并发<50req/s → D级（无实用价值）
4. 
迁移器损坏原文件 → D级（数据安全红线）
5. 
债务虚假清偿（声称清偿实际未实现）→ D级（诚信问题）
6. 
未提供24项刀刃自测 → D级（偷工减料）
7. 
性能提升伪造（修改benchmark数据）→ D级（欺诈）
8. 
超时4小时 → D级（工期管理失控）

