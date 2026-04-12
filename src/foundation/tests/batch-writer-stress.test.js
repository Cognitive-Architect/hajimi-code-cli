/**
 * BatchWriterOptimized 压力测试
 * 
 * 目标：
 * - 吞吐 > 1000 ops/s
 * - 崩溃零丢失
 */

const { BatchWriterOptimized } = require('../src/storage/batch-writer-optimized');
const fs = require('fs').promises;

async function cleanup() {
  try {
    await fs.unlink('./data/stress-test.db');
    await fs.unlink('./data/stress-test.wal');
  } catch (err) {
    // ignore
  }
}

async function runStressTest() {
  console.log('=== BatchWriterOptimized Stress Test ===\n');

  await cleanup();

  const writer = new BatchWriterOptimized({
    dbPath: './data/stress-test.db',
    walPath: './data/stress-test.wal',
    batchSize: 100,
    compress: true
  });

  await writer.init();

  // 压力测试：写入10000条
  const totalOps = 10000;
  const startTime = Date.now();

  const promises = [];
  for (let i = 0; i < totalOps; i++) {
    promises.push(writer.write(`key-${i}`, { data: i, timestamp: Date.now() }));
  }

  await Promise.all(promises);

  // 强制刷盘
  await writer.close();

  const elapsed = Date.now() - startTime;
  const opsPerSecond = (totalOps / elapsed * 1000).toFixed(2);

  console.log(`Total operations: ${totalOps}`);
  console.log(`Elapsed time: ${elapsed}ms`);
  console.log(`Throughput: ${opsPerSecond} ops/s`);

  // 验证数据完整性
  const writer2 = new BatchWriterOptimized({
    dbPath: './data/stress-test.db',
    walPath: './data/stress-test.wal'
  });
  await writer2.init();

  let verified = 0;
  for (let i = 0; i < totalOps; i++) {
    const value = await writer2.read(`key-${i}`);
    if (value && value.data === i) {
      verified++;
    }
  }

  await writer2.close();

  console.log(`Verified: ${verified}/${totalOps}`);
  console.log(`Lost: ${totalOps - verified}`);

  const passed = parseFloat(opsPerSecond) >= 1000 && (totalOps - verified) === 0;
  console.log(`\n${passed ? '✅ PASSED' : '❌ FAILED'}`);
  console.log(`Target: >= 1000 ops/s, lost: 0`);

  await cleanup();
  process.exit(passed ? 0 : 1);
}

runStressTest().catch(err => {
  console.error('Test failed:', err);
  process.exit(1);
});
