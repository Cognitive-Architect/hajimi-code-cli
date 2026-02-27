🚀 饱和攻击波次：HAJIMI-PHASE3-LUXURY-001（sql.js豪华版基础架构）
火力配置：1 Agent 并行（Engineer - 唐音）
轰炸目标：Phase 3豪华版基础架构 → 产出《HAJIMI-B-01-04-LUXURY-BASE-白皮书-v1.0.md》+《HAJIMI-B-01-04-LUXURY-BASE-自测表-v1.0.md》+ 可运行代码框架

⚠️ 技术背景（必读，无记忆空间访问权限）：

债务目标：清偿DEBT-SEC-001（限流状态持久化）。当前Phase 2使用内存Map存储限流状态，进程重启后数据丢失。

选型理由：选用`sql.js`（纯JavaScript SQLite），零编译，Termux/Windows全平台兼容，通过WAL+批量+预编译优化可达原生80%性能。

核心优化三件套：
1. WAL模式：`PRAGMA journal_mode = WAL`实现读写并发不阻塞
2. 事务批量：`writeQueue`数组积累100条后一次性事务提交（50x性能提升）
3. 预编译语句：`stmtCache` Map缓存常用SQL（1.5x查询提升）
4. 异步持久化：`db.export()`生成Uint8Array + `fs.promises.writeFile`后台刷盘（零阻塞）

输入基线：Phase 2代码基线（TokenBucketRateLimiter A级实现），需保持API兼容（`checkLimit`方法），内部用SQLite替换`this.buckets = new Map()`。

---

工单 B-01/04 Engineer → DEBT-SEC-001清偿基础架构
目标：实现LuxurySQLiteRateLimiter类（WAL+批量+预编译+异步持久化）
输入：Phase 2 rate-limiter-security.js基线代码 + sql.js包（`npm install sql.js`）
输出：
- `src/security/rate-limiter-sqlite-luxury.js`（代码框架，>300行）
- `tests/luxury-base.test.js`（基础测试）
- `HAJIMI-B-01-04-LUXURY-BASE-白皮书-v1.0.md`（4章：技术背景/架构设计/实现细节/债务清偿路径）
- `HAJIMI-B-01-04-LUXURY-BASE-自测表-v1.0.md`（16项刀刃+10项P4）
自测点：[LUX-BASE-001][LUX-BASE-016], [P4-LUX-001][P4-LUX-010]

强制交付类结构（必须实现）：

```javascript
class LuxurySQLiteRateLimiter {
  constructor(options) {
    this.config = { batchSize: 100, flushInterval: 5000, cacheSize: -64000, checkpointInterval: 300000 };
    this.writeQueue = []; // 批量队列
    this.stmtCache = new Map(); // 预编译缓存
  }
  async init() { /* 加载sql.js→恢复/创建db→_execPragmas()→_initSchema()→_prepareStatements()→启动定时器 */ }
  _execPragmas() { /* WAL模式：journal_mode=WAL, synchronous=NORMAL, cache_size=-64000, temp_store=MEMORY */ }
  _initSchema() { /* 创建rate_limits表：ip(TEXT PRIMARY KEY), tokens(REAL), last_refill(INTEGER) */ }
  _prepareStatements() { /* 缓存getBucket/updateBucket语句 */ }
  getBucket(ip) { /* 使用stmtCache.get('getBucket')查询 */ }
  async saveBucket(ip, tokens, lastRefill) { /* 入队writeQueue，>=batchSize时触发_flushBatch() */ }
  async _flushBatch() { /* BEGIN TRANSACTION→批量执行updateBucket→COMMIT→_asyncPersist() */ }
  async _asyncPersist() { /* db.export()→fs.promises.writeFile(Buffer.from(data)) */ }
  _startBackgroundFlush() { /* flushTimer(5s)+checkpointTimer(5min)+SIGINT强制刷盘 */ }
  async checkLimit(ip) { /* 兼容Phase 2 API：Token Bucket算法，返回{allowed, remaining} */ }
  async close() { /* 清理定时器→刷盘→db.close() */ }
}
```

刀刃风险自测表（16项，手动勾选）：

用例ID	类别	场景	验证命令	通过标准	状态	
LUX-BASE-001	FUNC	sql.js导入	`node -e "const initSqlJs=require('sql.js');console.log('OK')"`	输出OK	[ ]	
LUX-BASE-002	FUNC	类定义	`grep "class LuxurySQLiteRateLimiter"`	命中	[ ]	
LUX-BASE-003	FUNC	init()异步	`grep "async init()"`	命中	[ ]	
LUX-BASE-004	FUNC	WAL配置	`grep "journal_mode = WAL"`	命中	[ ]	
LUX-BASE-005	FUNC	批量队列	`grep "writeQueue = \[\]"`	命中	[ ]	
LUX-BASE-006	FUNC	预编译缓存	`grep "stmtCache = new Map"`	命中	[ ]	
LUX-BASE-007	FUNC	异步持久化	`grep "async _asyncPersist"`	命中	[ ]	
LUX-BASE-008	CONST	batchSize=100	`grep "batchSize.*100"`	命中	[ ]	
LUX-BASE-009	CONST	cacheSize=-64000	`grep "cacheSize.*-64000"`	命中	[ ]	
LUX-BASE-010	NEG	无同步fs	`grep "writeFileSync\|readFileSync"`	无结果	[ ]	
LUX-BASE-011	E2E	init成功	`node -e "const L=require('.');new L().init().then(()=>console.log('INIT_OK'))"`	INIT_OK	[ ]	
LUX-BASE-012	E2E	WAL验证	`node tests/wal-check.test.js`	输出wal	[ ]	
LUX-BASE-013	E2E	CRUD测试	`node tests/luxury-base.test.js`	Exit 0	[ ]	
LUX-BASE-014	PERF	初始化<100ms	`console.time/init/timeEnd`	<100ms	[ ]	
LUX-BASE-015	UX	close()方法	`grep "async close()"`	命中	[ ]	
LUX-BASE-016	FUNC	checkLimit兼容	`grep "async checkLimit"`	命中	[ ]	

P4自测轻量检查表（10项，手动勾选）：

CHECK_ID	检查项	状态	
P4-LUX-001	sql.js零编译安装成功	[ ]	
P4-LUX-002	LuxurySQLiteRateLimiter类完整（含constructor/config）	[ ]	
P4-LUX-003	execPragmas实现（含WAL+5项PRAGMA）	[ ]	
P4-LUX-004	initSchema实现（rate_limits表结构）	[ ]	
P4-LUX-005	prepareStatements实现（stmtCache缓存get/update）	[ ]	
P4-LUX-006	saveBucket批量队列逻辑（入队+阈值触发）	[ ]	
P4-LUX-007	flushBatch事务包裹（BEGIN/COMMIT+批量执行）	[ ]	
P4-LUX-008	asyncPersist异步持久化（db.export+fs.promises.writeFile）	[ ]	
P4-LUX-009	startBackgroundFlush定时器（flush+checkpoint+SIGINT）	[ ]	
P4-LUX-010	checkLimit兼容Phase 2 API（返回值含allowed/remaining）	[ ]	

质量门禁：
- `npm install sql.js`成功（零编译）
- 代码>300行，类结构完整
- 16项刀刃自测全部手动[x]
- 10项P4检查全部手动[x]
- 白皮书4章完整（含WAL原理说明）

D级红线（触发即永久失败）：
1. 使用非sql.js驱动 → D级
2. WAL模式未启用（非wal） → D级
3. 同步fs调用 → D级
4. 缺失批量队列writeQueue → D级
5. 缺失预编译缓存stmtCache → D级
6. 16/10项自测未全部勾选 → D级
7. 超时2小时 → D级

战术金句："WAL是地基，批量是引擎，预编译是变速箱——B-01/04打造豪华跑车底盘！无记忆空间？技术背景写进派单，代码写进灵魂！☝️🐍♾️⚡"

开工！☝️😋🐍♾️💥