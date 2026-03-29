/**
 * StorageV3 集成测试
 * 
 * 自测点：
 * - API-001：put后get一致性
 * - API-002：delete后get返回null
 * - API-003：stats返回16分片统计
 * - API-004：批量put性能
 * - API-005：并发put不冲突
 */

const { StorageV3 } = require('../api/storage');
const assert = require('assert');

console.log('='.repeat(60));
console.log('StorageV3 集成测试');
console.log('='.repeat(60));

let passed = 0;
let failed = 0;

async function test(name, fn) {
  try {
    await fn();
    passed++;
    console.log(`✅ ${name}`);
  } catch (err) {
    failed++;
    console.log(`❌ ${name}: ${err.message}`);
  }
}

// 创建存储实例
const storage = new StorageV3();

// 运行所有测试
(async () => {
  // API-001: put后get一致性
  await test('API-001: put后get一致性', async () => {
    const content = 'Hello, StorageV3!';
    const metadata = { type: 'text', tags: ['test'] };

    // put
    const putResult = await storage.put(content, metadata);
    assert(putResult.simhash, '应返回simhash');
    assert.strictEqual(putResult.size, content.length, '大小应一致');

    // get
    const getResult = await storage.get(putResult.simhash);
    assert(getResult, '应读取到数据');
    assert.strictEqual(getResult.data.toString(), content, '内容应一致');
    assert.deepStrictEqual(getResult.metadata.tags, metadata.tags, '元数据应一致');
  });

  // API-002: delete后get返回null
  await test('API-002: delete后get返回null', async () => {
    const content = 'To be deleted';
    const putResult = await storage.put(content);

    // 删除前可读取
    const beforeDelete = await storage.get(putResult.simhash);
    assert(beforeDelete, '删除前应存在');

    // 删除
    const deleted = await storage.delete(putResult.simhash);
    assert.strictEqual(deleted, true, '应删除成功');

    // 删除后读取为null
    const afterDelete = await storage.get(putResult.simhash);
    assert.strictEqual(afterDelete, null, '删除后应为null');
  });

  // API-003: stats返回16分片统计
  await test('API-003: stats返回16分片统计', async () => {
    const stats = await storage.stats();
    
    assert(stats.shards, '应有shards统计');
    assert.strictEqual(stats.shards.count, 16, '应有16分片');
    assert(stats.chunks, '应有chunks统计');
    assert(stats.pool, '应有pool统计');
  });

  // API-004: 批量put性能
  await test('API-004: 批量put性能', async () => {
    const items = [];
    for (let i = 0; i < 100; i++) {
      items.push({
        content: `Batch item ${i}`,
        metadata: { index: i }
      });
    }

    const start = Date.now();
    const results = await storage.batchPut(items);
    const duration = Date.now() - start;

    const successCount = results.filter(r => r.success).length;
    assert.strictEqual(successCount, 100, '100个应全部成功');
    
    console.log(`   批量写入100条耗时: ${duration}ms`);
  });

  // API-005: 并发put不冲突
  await test('API-005: 并发put不冲突', async () => {
    const promises = [];
    
    for (let i = 0; i < 20; i++) {
      promises.push(storage.put(`Concurrent ${i}`, { index: i }));
    }

    const results = await Promise.all(promises);
    
    // 验证每个都成功且可读取
    for (const result of results) {
      assert(result.simhash, '应有simhash');
      const read = await storage.get(result.simhash);
      assert(read, '应可读取');
    }
  });

  // 额外测试：query功能
  await test('额外: query查询功能', async () => {
    // 写入一条数据
    const result = await storage.put('Query test', { searchable: true });
    
    // 查询（模拟）
    // 实际查询需要SimHash高64位
    const simhash = BigInt('0x' + result.simhash);
    const simhashHi = simhash >> 64n;
    
    const queryResults = await storage.query(simhashHi);
    assert(Array.isArray(queryResults), '应返回数组');
  });

  // 清理
  await storage.close();

  // 输出结果
  console.log('\n' + '='.repeat(60));
  console.log('测试结果摘要');
  console.log('='.repeat(60));
  console.log(`通过: ${passed}/6`);
  console.log(`失败: ${failed}/6`);

  if (failed === 0) {
    console.log('\n✅ 全部测试通过');
    process.exit(0);
  } else {
    console.log('\n❌ 有测试失败');
    process.exit(1);
  }
})();
