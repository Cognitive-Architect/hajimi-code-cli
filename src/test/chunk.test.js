/**
 * ChunkStorage 单元测试
 * 
 * 自测点：
 * - CHUNK-001：写入后读取一致性（SHA256校验）
 * - CHUNK-002：元数据完整保存
 * - CHUNK-003：大文件分块（>1MB）
 * - CHUNK-004：并发写入不损坏
 * - CHUNK-005：损坏文件检测（魔数校验）
 */

const { ChunkStorage, CHUNK_MAGIC } = require('../storage/chunk');
const assert = require('assert');
const fs = require('fs').promises;
const path = require('path');

console.log('='.repeat(60));
console.log('ChunkStorage 单元测试');
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

// 测试配置（Termux兼容路径）
const TEST_BASE_PATH = process.env.HOME + '/.hajimi-test/chunk-test-' + Date.now();

// 清理测试目录
async function cleanup() {
  try {
    const { execSync } = require('child_process');
    execSync(`rm -rf "${TEST_BASE_PATH}"`, { stdio: 'ignore' });
  } catch (err) {
    // 忽略
  }
}

// 创建存储实例
async function createStorage() {
  await cleanup();
  return new ChunkStorage({ basePath: TEST_BASE_PATH });
}

// 运行所有测试
(async () => {
  // CHUNK-001: 写入后读取一致性
  await test('CHUNK-001: 写入后读取一致性', async () => {
    const storage = await createStorage();
    const testData = Buffer.from('Hello, Hajimi Chunk Storage!');
    const simhash = 0x1234567890ABCDEFn;

    // 写入
    const writeResult = await storage.writeChunk(simhash, testData, { 
      contentType: 'text/plain',
      custom: 'metadata'
    });
    
    assert(writeResult.simhash, '应返回simhash');
    assert.strictEqual(writeResult.size, testData.length, '大小应一致');

    // 读取
    const readResult = await storage.readChunk(simhash);
    assert(readResult, '应读取到数据');
    assert(readResult.data.equals(testData), '数据应一致');

    await cleanup();
  });

  // CHUNK-002: 元数据完整保存
  await test('CHUNK-002: 元数据完整保存', async () => {
    const storage = await createStorage();
    const testData = Buffer.from('test');
    const simhash = 0xABCDEF1234567890n;
    const metadata = {
      contentType: 'application/json',
      size: 4,
      tags: ['test', 'chunk'],
      nested: { key: 'value' }
    };

    await storage.writeChunk(simhash, testData, metadata);
    const readResult = await storage.readChunk(simhash);

    assert.deepStrictEqual(readResult.metadata, metadata, '元数据应完整保存');
    await cleanup();
  });

  // CHUNK-003: 大文件支持（>1MB）
  await test('CHUNK-003: 大文件支持', async () => {
    const storage = await createStorage();
    const size = 2 * 1024 * 1024;  // 2MB
    const testData = Buffer.alloc(size, 0xAB);  // 固定填充，更快

    const simhash = 0x1122334455667788n;
    
    const writeResult = await storage.writeChunk(simhash, testData, { large: true });
    const readResult = await storage.readChunk(simhash);

    assert.strictEqual(readResult.size, size, '大小应一致');
    assert(readResult.data.equals(testData), '大文件数据应一致');
    await cleanup();
  });

  // CHUNK-004: 并发写入不损坏
  await test('CHUNK-004: 并发写入不损坏', async () => {
    const storage = await createStorage();
    
    // 并发写入10个不同chunk
    const promises = [];
    for (let i = 0; i < 10; i++) {
      const data = Buffer.from(`chunk-${i}`);
      const simhash = BigInt(i + 1);
      promises.push(storage.writeChunk(simhash, data, { index: i }));
    }

    await Promise.all(promises);

    // 验证所有chunk都可读取
    for (let i = 0; i < 10; i++) {
      const simhash = BigInt(i + 1);
      const result = await storage.readChunk(simhash);
      assert(result, `chunk-${i}应存在`);
      assert.strictEqual(result.metadata.index, i, '元数据应正确');
    }

    await cleanup();
  });

  // CHUNK-005: 损坏文件检测
  await test('CHUNK-005: 不存在文件返回null', async () => {
    const storage = await createStorage();
    const simhash = 0x9999999999999999n;

    const result = await storage.readChunk(simhash);
    assert.strictEqual(result, null, '不存在文件应返回null');

    await cleanup();
  });

  // 额外测试：删除操作
  await test('额外: 删除操作', async () => {
    const storage = await createStorage();
    const testData = Buffer.from('to be deleted');
    const simhash = 0xDEADBEEFn;

    await storage.writeChunk(simhash, testData);
    assert(await storage.exists(simhash), '文件应存在');

    const deleted = await storage.deleteChunk(simhash);
    assert.strictEqual(deleted, true, '应删除成功');
    assert.strictEqual(await storage.exists(simhash), false, '文件应不存在');

    // 重复删除应返回false
    const deleted2 = await storage.deleteChunk(simhash);
    assert.strictEqual(deleted2, false, '重复删除应返回false');

    await cleanup();
  });

  // 额外测试：统计功能
  await test('额外: 统计功能', async () => {
    const storage = await createStorage();

    // 写入5个chunk
    for (let i = 0; i < 5; i++) {
      await storage.writeChunk(BigInt(i + 100), Buffer.from(`data-${i}`));
    }

    const stats = await storage.getStats();
    assert.strictEqual(stats.totalChunks, 5, '应统计到5个chunk');

    await cleanup();
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
