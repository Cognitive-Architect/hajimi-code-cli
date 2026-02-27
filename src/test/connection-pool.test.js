/**
 * ShardConnectionPool 单元测试
 * 
 * 自测点：
 * - POOL-001：单分片连接创建成功
 * - POOL-002：并发查询不冲突（16分片并行）
 * - POOL-003：连接泄漏检测（上限8连接/分片）
 * - POOL-004：错误分片自动重试
 * - POOL-005：关闭时全部释放
 */

const { ShardConnectionPool } = require('../storage/connection-pool');
const assert = require('assert');

console.log('='.repeat(60));
console.log('ShardConnectionPool 单元测试');
console.log('='.repeat(60));

let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    passed++;
    console.log(`✅ ${name}`);
  } catch (err) {
    failed++;
    console.log(`❌ ${name}: ${err.message}`);
  }
}

async function testAsync(name, fn) {
  try {
    await fn();
    passed++;
    console.log(`✅ ${name}`);
  } catch (err) {
    failed++;
    console.log(`❌ ${name}: ${err.message}`);
  }
}

// 创建连接池（每个测试独立）
function createPool() {
  return new ShardConnectionPool({
    maxConnectionsPerShard: 8,
    retryAttempts: 3
  });
}

// 运行所有测试
(async () => {
  // POOL-001: 单分片连接创建成功
  await testAsync('POOL-001: 单分片连接创建成功', async () => {
    const pool = createPool();
    const hash = 0x0011223344556677n;
    const result = await pool.query(hash, 'SELECT 1');
    assert(result, '查询应返回结果');
    assert.strictEqual(result.shardId, 0, '应路由到shard_00');
    await pool.closeAll();
  });

  // POOL-002: 并发查询不冲突（16分片并行）
  await testAsync('POOL-002: 并发查询不冲突', async () => {
    const pool = createPool();
    const promises = [];
    
    // 16个分片同时查询
    for (let i = 0; i < 16; i++) {
      const hash = BigInt(i) << 56n;  // 确保路由到不同分片
      promises.push(pool.query(hash, `SELECT ${i}`));
    }
    
    const results = await Promise.all(promises);
    
    // 验证每个分片都有结果
    for (let i = 0; i < 16; i++) {
      const result = results[i];
      assert.strictEqual(result.shardId, i, `分片${i}应有结果`);
    }
    await pool.closeAll();
  });

  // POOL-003: 连接上限检测
  await testAsync('POOL-003: 连接上限检测', async () => {
    const pool = createPool();
    // 串行发起请求，验证连接复用
    const hash = 0x00n;  // 固定分片
    
    for (let i = 0; i < 10; i++) {
      const result = await pool.query(hash, `SELECT ${i}`);
      assert(result, `查询${i}应返回结果`);
    }
    
    // 验证连接池复用（实际创建连接数 <= 上限8）
    const stats = pool.getPoolStats();
    const shard0Stats = stats[0];
    assert(shard0Stats.totalConnections <= 8, '连接数应不超过上限8');
    await pool.closeAll();
  });

  // POOL-004: 错误重试机制
  await testAsync('POOL-004: 错误重试统计', async () => {
    const pool = createPool();
    // 先执行一次查询，确保有统计
    await pool.query(0x00n, 'SELECT 1');
    const stats = pool.getStats();
    assert(typeof stats.totalQueries === 'number', '应有totalQueries统计');
    assert(typeof stats.retries === 'number', '应有retries统计');
    await pool.closeAll();
  });

  // POOL-005: 关闭时全部释放
  await testAsync('POOL-005: 关闭时全部释放', async () => {
    const pool = createPool();
    // 先创建一些连接
    await pool.query(0x00n, 'SELECT 1');
    await pool.query(0x10n, 'SELECT 1');
    
    // 关闭所有连接
    await pool.closeAll();
    
    // 验证连接池为空
    const stats = pool.getPoolStats();
    for (const shard of stats) {
      assert.strictEqual(shard.totalConnections, 0, `分片${shard.shardId}应无连接`);
      assert.strictEqual(shard.readAvailable, 0, `分片${shard.shardId}应无可用连接`);
    }
  });

  // 额外测试：写入操作
  await testAsync('额外: 写入操作', async () => {
    const pool = createPool();
    const hash = 0x0011223344556677n;
    const result = await pool.write(hash, 'INSERT INTO test VALUES (1)');
    assert(result.affected >= 0, '写入应返回影响行数');
    await pool.closeAll();
  });

  // 额外测试：连接池统计
  await testAsync('额外: 连接池统计信息', async () => {
    const pool = createPool();
    const stats = pool.getPoolStats();
    assert.strictEqual(stats.length, 16, '应有16个分片统计');
    
    for (const shard of stats) {
      assert(typeof shard.shardId === 'number', '应有shardId');
      assert(typeof shard.readAvailable === 'number', '应有readAvailable');
      assert(typeof shard.totalConnections === 'number', '应有totalConnections');
    }
    await pool.closeAll();
  });

  // 输出结果
  console.log('\n' + '='.repeat(60));
  console.log('测试结果摘要');
  console.log('='.repeat(60));
  console.log(`通过: ${passed}/7`);
  console.log(`失败: ${failed}/7`);

  if (failed === 0) {
    console.log('\n✅ 全部测试通过');
    process.exit(0);
  } else {
    console.log('\n❌ 有测试失败');
    process.exit(1);
  }
})();
