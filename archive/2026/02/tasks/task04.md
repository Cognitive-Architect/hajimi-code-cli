🚀 饱和攻击波次：HAJIMI-V3-PHASE1-SHARD 集群
火力配置：5 Agent 并行（唐音 ×5）

轰炸目标：Hash 分片存储系统 v3.0 → 产出《Phase1-白皮书》+《Phase1-自测表》+ 可运行代码

输入基线：
- `docs/SQLITE-SHARDING-方案对比.md`（方案 A 设计）
- `docs/DEBT-SQLITE-001-REPORT.md`（分片预研）
- `docs/V3-ROADMAP-v2-CORRECTED.md`（Phase 1 计划）
- 已归档的 `04-AUDIT-FIX-001-FINAL.md`（A级基线）

技术约束：
- 16 分片（hash_prefix % 16）
- SimHash-64 高 8bit 路由
- Termux/Node.js 环境优先
- 内存上限 400MB（HNSW 约束）

---

⚠️ 质量门禁（必须全部满足才能开工）

```markdown
- [ ] 《P4自测轻量检查表》10/10项已勾选（每工单）
- [ ] 债务预声明：至少3项DEBT-XXX（P0/P1/P2分级）
- [ ] 额度预算确认：预估Token消耗<剩余额度25%
- [ ] 验证命令可复制：每交付物附带即时验证（bash/node）
- [ ] 范围边界标注：明确"本轮不覆盖"项（如WebRTC传输层）
```

---

📋 工单矩阵（5路并行）

工单 P1-01/05 唐音 → ShardRouter 核心路由层
目标：实现 `src/storage/shard-router.js`，SimHash 高 8bit → 分片 00-15 路由

输入：SQLITE-SHARDING-方案对比.md 第4.1节（分片规则）

输出：
- `src/storage/shard-router.js`（核心路由类）
- `src/test/shard-router.test.js`（路由准确性测试）
- `docs/PHASE1-SHARD-ROUTER.md`（设计文档）

核心功能：

```javascript
class ShardRouter {
  getShardId(simhash_hi) { /* 高8bit % 16 */ }
  getShardPath(shardId) { /* meta/shard_XX.db */ }
  validateSimHash(hash) { /* 格式校验 */ }
}
```

自测点：
- SHARD-001：hash_prefix 00 → shard_00
- SHARD-002：hash_prefix FF → shard_15
- SHARD-003：边界 FF → 15（非16）
- SHARD-004：非法输入抛出错误
- SHARD-005：100K 记录均匀分布（标准差<5%）

P4检查：

检查点	覆盖	用例ID	
CF	[ ]	SHARD-001003（核心路由）	
RG	[ ]	SHARD-004（约束回归）	
NG	[ ]	SHARD-005（分布不均负面路径）	
UX	[ ]	错误提示可读性	
E2E	[ ]	SHARD-001+002+003（完整路由链路）	
High	[ ]	SHARD-003（边界正确性）	

即时验证：

```bash
node -e "
const {ShardRouter}=require('./src/storage/shard-router');
const r=new ShardRouter();
console.assert(r.getShardId(0x00FFn)===0,'00失败');
console.assert(r.getShardId(0xFF00n)===15,'FF失败');
console.log('✅ 路由测试通过');
"
```

---

工单 P1-02/05 唐音 → 分片连接池管理
目标：实现 `src/storage/connection-pool.js`，16 分片独立连接池

输入：方案对比.md 第4.3节（连接池设计）

输出：
- `src/storage/connection-pool.js`（连接池实现）
- `src/test/connection-pool.test.js`（并发测试）
- `docs/PHASE1-CONN-POOL.md`（设计文档）

核心功能：

```javascript
class ShardConnectionPool {
  async query(simhash_hi, sql, params) { /* 路由+执行 */ }
  async write(sql, params) { /* 广播或路由 */ }
  closeAll() { /* 关闭所有连接 */ }
}
```

自测点：
- POOL-001：单分片连接创建成功
- POOL-002：并发查询不冲突（16分片并行）
- POOL-003：连接泄漏检测（上限8连接/分片）
- POOL-004：错误分片自动重试
- POOL-005：关闭时全部释放

P4检查：

检查点	覆盖	用例ID	
CF	[ ]	POOL-001,002	
RG	[ ]	POOL-003（连接上限约束）	
NG	[ ]	POOL-004（错误恢复）	
UX	[ ]	POOL-005（优雅关闭）	
E2E	[ ]	POOL-002（并发端到端）	
High	[ ]	POOL-003（连接泄漏高危）	

即时验证：

```bash
node src/test/connection-pool.test.js
# 预期: 5/5 通过
```

---

工单 P1-03/05 唐音 → Chunk 文件格式实现
目标：实现 `src/storage/chunk.js`，.hctx v3 格式（含分片元数据）

输入：DEBT-CLEARANCE-001 白皮书（Chunk 格式设计）

输出：
- `src/storage/chunk.js`（Chunk 读写类）
- `src/test/chunk.test.js`（格式测试）
- `docs/PHASE1-CHUNK-FORMAT.md`（格式规范）

核心功能：

```javascript
class ChunkStorage {
  async writeChunk(simhash, data, metadata) { /* 写入 */ }
  async readChunk(simhash) { /* 读取 */ }
  async deleteChunk(simhash) { /* 删除 */ }
}
```

自测点：
- CHUNK-001：写入后读取一致性（SHA256校验）
- CHUNK-002：元数据完整保存（size/ctime/simhash）
- CHUNK-003：大文件分块（>1MB自动分片）
- CHUNK-004：并发写入不损坏
- CHUNK-005：损坏文件检测（魔数校验）

P4检查：

检查点	覆盖	用例ID	
CF	[ ]	CHUNK-001,002（CRUD核心）	
RG	[ ]	CHUNK-005（完整性约束）	
NG	[ ]	CHUNK-003（大文件边界）	
UX	[ ]	CHUNK-004（并发体验）	
E2E	[ ]	CHUNK-001（写→读端到端）	
High	[ ]	CHUNK-001（数据一致性）	

即时验证：

```bash
node -e "
const {ChunkStorage}=require('./src/storage/chunk');
const c=new ChunkStorage();
const testData=Buffer.from('test');
const hash=0x1234n;
await c.writeChunk(hash,testData,{size:4});
const read=await c.readChunk(hash);
console.assert(read.data.equals(testData),'读写不一致');
console.log('✅ Chunk测试通过');
"
```

---

工单 P1-04/05 唐音 → MetaDB Schema & 迁移工具
目标：实现 `src/storage/schema.sql` + `src/storage/migrate.js`（16分片初始化）

输入：SQLITE-SHARDING-方案对比.md 第4.2节（Schema设计）

输出：
- `src/storage/schema.sql`（分片内Schema）
- `src/storage/migrate.js`（分片初始化/升级）
- `docs/PHASE1-MIGRATION.md`（迁移文档）

Schema 要求：

```sql
CREATE TABLE chunks (id INTEGER PRIMARY KEY, simhash_hi BIGINT, simhash_lo BIGINT, 
                     md5 BLOB, size INTEGER, storage_path TEXT, created_at INTEGER);
CREATE INDEX idx_simhash ON chunks(simhash_hi);
CREATE TABLE shard_meta (key TEXT PRIMARY KEY, value TEXT);
```

自测点：
- MIG-001：16个分片库文件创建成功
- MIG-002：Schema正确执行（表+索引存在）
- MIG-003：shard_meta初始化（chunk_count=0）
- MIG-004：重复初始化不报错（幂等）
- MIG-005：版本升级路径（v1→v2→v3）

P4检查：

检查点	覆盖	用例ID	
CF	[ ]	MIG-001,002（初始化核心）	
RG	[ ]	MIG-004（幂等约束）	
NG	[ ]	MIG-005（升级负面路径）	
UX	[ ]	MIG-003（元数据完整）	
E2E	[ ]	MIG-001+002（初始化端到端）	
High	[ ]	MIG-004（幂等性高危）	

即时验证：

```bash
node src/storage/migrate.js --init && ls -la ~/.hajimi/storage/v3/meta/shard_*.db | wc -l
# 预期: 16
```

工单 P1-05/05 唐音 → 基础 CRUD API & 集成测试
目标：实现 `src/api/storage.js`，整合Router+Pool+Chunk，提供统一API

输入：前4工单交付物

输出：
- `src/api/storage.js`（StorageV3 API类）
- `src/test/storage-integration.test.js`（集成测试）
- `docs/PHASE1-API.md`（API文档）
- `docs/PHASE1-白皮书-v1.0.md`（整合白皮书，含5章节）

API 规格：
class StorageV3 {
  async put(content, metadata) { /* 存储，返回simhash */ }
  async get(simhash) { /* 读取内容 */ }
  async delete(simhash) { /* 删除 */ }
  async query(simhash_hi) { /* 按高32bit查询候选 */ }
  async stats() { /* 返回16分片统计 */ }
}

自测点：
 
API-001：put后get一致性
 
API-002：delete后get返回null
 
API-003：stats返回16分片统计
 
API-004：100K记录put性能（>50MB/s）
 
API-005：并发put不冲突（16线程）

P4检查：
检查点	覆盖	用例ID	
CF	[ ]	API-001,002,003（核心API）	
RG	[ ]	API-001（一致性约束）	
NG	[ ]	API-002（删除后读取负面路径）	
UX	[ ]	API-003（统计可读性）	
E2E	[ ]	API-001+004+005（完整链路）	
High	[ ]	API-001（数据一致性）

即时验证：
node src/test/storage-integration.test.js
# 预期: 5/5 通过

🎯 集群收卷强制交付物（总计6件套）
#	交付物	来源工单	类型	
1	`src/storage/shard-router.js` + test	P1-01	代码	
2	`src/storage/connection-pool.js` + test	P1-02	代码	
3	`src/storage/chunk.js` + test	P1-03	代码	
4	`src/storage/migrate.js` + schema.sql	P1-04	代码/配置	
5	`src/api/storage.js` + integration test	P1-05	代码	
6	`docs/PHASE1-白皮书-v1.0.md` + `docs/PHASE1-自测表-v1.0.md`	P1-05	文档

债务预声明（必须）：
 
DEBT-PHASE1-001：WebRTC传输层未实现（P2，Phase 3处理）
 
DEBT-PHASE1-002：HNSW向量索引未集成（P1，Phase 2处理）
 
DEBT-PHASE1-003：LRU缓存未实现（P2，性能优化阶段）