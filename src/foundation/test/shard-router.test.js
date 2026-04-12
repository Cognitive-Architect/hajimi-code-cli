/**
 * ShardRouter 单元测试
 * 
 * 自测点：
 * - SHARD-001：hash_prefix 00 → shard_00
 * - SHARD-002：hash_prefix FF → shard_15
 * - SHARD-003：边界 FF → 15（非16）
 * - SHARD-004：非法输入抛出错误
 * - SHARD-005：100K 记录均匀分布（标准差<5%）
 */

const { ShardRouter, ShardDistributionTester } = require('../storage/shard-router');
const assert = require('assert');

console.log('='.repeat(60));
console.log('ShardRouter 单元测试');
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

// 创建路由器实例
const router = new ShardRouter({
  basePath: '/tmp/test-hajimi'
});

// SHARD-001: hash_prefix 00 → shard_00
test('SHARD-001: hash_prefix 00 → shard_00', () => {
  const hash00 = 0x00FFn;  // 高8bit = 0x00
  const shardId = router.getShardId(hash00);
  assert.strictEqual(shardId, 0, `Expected 0, got ${shardId}`);
});

// SHARD-002: hash_prefix FF → shard_15
test('SHARD-002: hash_prefix FF → shard_15', () => {
  const hashFF = 0xFF00000000000000n;  // 高8bit = 0xFF = 255, 255 % 16 = 15
  const shardId = router.getShardId(hashFF);
  assert.strictEqual(shardId, 15, `Expected 15, got ${shardId}`);
});

// SHARD-003: 边界值正确性
test('SHARD-003: 边界值正确性', () => {
  // 测试所有边界值（高8bit）
  const testCases = [
    { prefix: 0x00, expected: 0 },   // 0 % 16 = 0
    { prefix: 0x0F, expected: 15 },  // 15 % 16 = 15
    { prefix: 0x10, expected: 0 },   // 16 % 16 = 0
    { prefix: 0xF0, expected: 0 },   // 240 % 16 = 0
    { prefix: 0xFF, expected: 15 },  // 255 % 16 = 15
  ];
  
  for (const tc of testCases) {
    // 构造高8bit为prefix的simhash
    const hash = BigInt(tc.prefix) << 56n;
    const shardId = router.getShardId(hash);
    assert.strictEqual(shardId, tc.expected, 
      `Prefix 0x${tc.prefix.toString(16)}: expected ${tc.expected}, got ${shardId}`);
  }
});

// SHARD-004: 非法输入抛出错误
test('SHARD-004: 非法输入抛出错误', () => {
  // 非bigint
  assert.throws(() => router.getShardId(123), /Invalid SimHash type/);
  assert.throws(() => router.getShardId('abc'), /Invalid SimHash type/);
  
  // 负数
  assert.throws(() => router.getShardId(-1n), /SimHash must be non-negative/);
  
  // 超过64bit
  assert.throws(() => router.getShardId(0xFFFFFFFFFFFFFFFFFn), /exceeds 64 bits/);
});

// 路径生成测试
test('路径生成正确性', () => {
  const path00 = router.getShardPath(0);
  assert(path00.includes('shard_00.db'), `Path should include shard_00.db: ${path00}`);
  
  const path15 = router.getShardPath(15);
  assert(path15.includes('shard_0f.db'), `Path should include shard_0f.db: ${path15}`);
});

// 越界检测
test('分片ID越界检测', () => {
  assert.throws(() => router.getShardPath(-1), /out of range/);
  assert.throws(() => router.getShardPath(16), /out of range/);
});

// SHARD-005: 100K 记录均匀分布（标准差<5%）
test('SHARD-005: 100K记录分布均匀性', () => {
  const tester = new ShardDistributionTester(router);
  const stats = tester.testDistribution(100000);
  
  console.log(`   分布统计: 期望=${stats.expected}, 标准差=${stats.stdDev} (${stats.stdDevPercent})`);
  
  // 标准差应小于5%
  assert(stats.isUniform, `标准差过高: ${stats.stdDevPercent} (应<5%)`);
});

// 批量路由测试
test('批量路由一致性', () => {
  const hashes = [
    0x0011223344556677n,
    0x1122334455667788n,
    0x2233445566778899n,
    0x33445566778899AAn,
    0x445566778899AABBn,
  ];
  
  for (const hash of hashes) {
    const shard1 = router.getShardId(hash);
    const shard2 = router.getShardId(hash);
    assert.strictEqual(shard1, shard2, '同hash应路由到同分片');
  }
});

// 输出结果
console.log('\n' + '='.repeat(60));
console.log('测试结果摘要');
console.log('='.repeat(60));
console.log(`通过: ${passed}/8`);
console.log(`失败: ${failed}/8`);

if (failed === 0) {
  console.log('\n✅ 全部测试通过');
  process.exit(0);
} else {
  console.log('\n❌ 有测试失败');
  process.exit(1);
}
