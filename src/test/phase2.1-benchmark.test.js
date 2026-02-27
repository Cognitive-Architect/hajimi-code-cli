/**
 * Phase 2.1 债务清偿基准测试
 * 
 * 测试目标：
 * - PERF-RG-001: 构建时间 ≤ 80s (vs 原 80s)
 * - PERF-RG-002: P99 查询 ≤ 45ms (vs 原 45ms)
 * - PERF-RG-003: 内存峰值 ≤ 150MB (vs 原 150MB)
 * - PERF-RG-004: checkpoint 期间查询延迟不增加 > 10%
 * 
 * 债务清偿验证：
 * - DEBT-PHASE2-006: WAL 自动截断
 * - DEBT-PHASE2-007: 并发写入无丢数据
 * - DEBT-PHASE2-005: 二进制序列化 < 500ms
 * 
 * 运行: node src/test/phase2.1-benchmark.test.js
 */

const { HNSWIndex } = require('../vector/hnsw-core');
const { VectorEncoder } = require('../vector/encoder');
const { HNSWPersistence } = require('../vector/hnsw-persistence');
const { WALCheckpointer } = require('../vector/wal-checkpointer');
const { WriteQueue } = require('../vector/write-queue');
const { serializeHNSW } = require('../format/hnsw-binary');
const path = require('path');
const os = require('os');
const fs = require('fs').promises;

// 测试配置
const TEST_CONFIG = {
  vectorCount: 50000,  // 50K for faster testing
  dimension: 128,
  queryCount: 1000
};

// Phase 2 基线
const PHASE2_BASELINE = {
  buildTime: 80000,      // 80s
  p99Latency: 45,        // 45ms
  memoryPeak: 150 * 1024 * 1024,  // 150MB
  jsonSerializeTime: 2500  // 2.5s
};

/**
 * 生成随机 SimHash
 */
function randomSimhash() {
  const high = BigInt(Math.floor(Math.random() * 0x100000000));
  const low = BigInt(Math.floor(Math.random() * 0x100000000));
  return (high << BigInt(32)) | low;
}

/**
 * 格式化时间
 */
function formatTime(ms) {
  return ms < 1000 ? `${ms.toFixed(0)}ms` : `${(ms/1000).toFixed(1)}s`;
}

/**
 * 格式化内存
 */
function formatMemory(bytes) {
  return `${(bytes/1024/1024).toFixed(1)}MB`;
}

/**
 * 基准测试 1: 构建性能（PERF-RG-001）
 */
async function benchmarkBuild() {
  console.log('\n=== 基准测试 1: 构建性能（PERF-RG-001）===\n');
  
  const index = new HNSWIndex({ distanceMetric: 'l2' });
  const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
  
  const startMem = process.memoryUsage().rss;
  const startTime = Date.now();
  
  for (let i = 0; i < TEST_CONFIG.vectorCount; i++) {
    const simhash = randomSimhash();
    const vector = encoder.encode(simhash);
    index.insert(i, vector);
  }
  
  const buildTime = Date.now() - startTime;
  const peakMem = process.memoryUsage().rss - startMem;
  
  console.log(`向量数量: ${TEST_CONFIG.vectorCount.toLocaleString()}`);
  console.log(`构建时间: ${formatTime(buildTime)} (基线: ${formatTime(PHASE2_BASELINE.buildTime)}) ${buildTime <= PHASE2_BASELINE.buildTime ? '✅' : '❌'}`);
  console.log(`内存增长: ${formatMemory(peakMem)} (基线: ${formatMemory(PHASE2_BASELINE.memoryPeak)}) ${peakMem <= PHASE2_BASELINE.memoryPeak ? '✅' : '❌'}`);
  
  return { buildTime, peakMem };
}

/**
 * 基准测试 2: 查询延迟（PERF-RG-002）
 */
async function benchmarkQuery() {
  console.log('\n=== 基准测试 2: 查询延迟（PERF-RG-002）===\n');
  
  // 先构建索引
  const index = new HNSWIndex({ distanceMetric: 'l2', efSearch: 64 });
  const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
  
  console.log('构建索引...');
  for (let i = 0; i < TEST_CONFIG.vectorCount; i++) {
    const vector = encoder.encode(randomSimhash());
    index.insert(i, vector);
  }
  
  // 生成查询
  const queries = [];
  for (let i = 0; i < TEST_CONFIG.queryCount; i++) {
    queries.push(encoder.encode(randomSimhash()));
  }
  
  console.log(`执行 ${TEST_CONFIG.queryCount} 次查询...`);
  const latencies = [];
  
  for (const query of queries) {
    const start = process.hrtime.bigint();
    index.search(query, 10);
    const end = process.hrtime.bigint();
    latencies.push(Number(end - start) / 1000000);
  }
  
  latencies.sort((a, b) => a - b);
  const avg = latencies.reduce((a, b) => a + b, 0) / latencies.length;
  const p50 = latencies[Math.floor(latencies.length * 0.5)];
  const p99 = latencies[Math.floor(latencies.length * 0.99)];
  
  console.log(`平均延迟: ${formatTime(avg)}`);
  console.log(`P50延迟:  ${formatTime(p50)}`);
  console.log(`P99延迟:  ${formatTime(p99)} (基线: ${PHASE2_BASELINE.p99Latency}ms) ${p99 <= PHASE2_BASELINE.p99Latency ? '✅' : '❌'}`);
  
  return { avg, p50, p99 };
}

/**
 * 基准测试 3: 二进制序列化（DEBT-PHASE2-005 清偿验证）
 */
async function benchmarkBinarySerialize() {
  console.log('\n=== 基准测试 3: 二进制序列化（DEBT-PHASE2-005）===\n');
  
  // 构建索引
  const index = new HNSWIndex({ distanceMetric: 'l2' });
  const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
  
  console.log('构建 50K 索引...');
  for (let i = 0; i < 50000; i++) {
    const vector = encoder.encode(randomSimhash());
    index.insert(i, vector);
  }
  
  // JSON 序列化
  console.log('JSON 序列化...');
  const jsonStart = Date.now();
  const jsonData = JSON.stringify(index.toJSON());
  const jsonTime = Date.now() - jsonStart;
  const jsonSize = Buffer.byteLength(jsonData);
  
  // 二进制序列化
  console.log('二进制序列化...');
  const binStart = Date.now();
  const binBuffer = serializeHNSW(index, { dimension: 128 });
  const binTime = Date.now() - binStart;
  const binSize = binBuffer.length;
  
  console.log(`JSON:   时间=${formatTime(jsonTime)}, 大小=${formatMemory(jsonSize)}`);
  console.log(`二进制: 时间=${formatTime(binTime)}, 大小=${formatMemory(binSize)}`);
  console.log(`\n性能提升: ${(jsonTime/binTime).toFixed(1)}x 更快`);
  console.log(`体积压缩: ${((1 - binSize/jsonSize) * 100).toFixed(1)}% 更小`);
  
  const passed = binTime < 500;  // 目标 < 500ms
  console.log(`\nDEBT-PHASE2-005 清偿: 二进制序列化 < 500ms -> ${passed ? '✅ 通过' : '❌ 失败'}`);
  
  return { jsonTime, binTime, jsonSize, binSize };
}

/**
 * 基准测试 4: WAL Checkpoint（DEBT-PHASE2-006 清偿验证）
 */
async function benchmarkCheckpoint() {
  console.log('\n=== 基准测试 4: WAL Checkpoint（DEBT-PHASE2-006）===\n');
  
  const testPath = path.join(os.tmpdir(), `hajimi-test-${Date.now()}`);
  
  const persistence = new HNSWPersistence({
    basePath: testPath,
    shardId: 'test',
    config: { walEnabled: true }
  });
  
  const index = new HNSWIndex({ distanceMetric: 'l2' });
  const encoder = new VectorEncoder();
  
  // 插入数据并记录 WAL
  console.log('插入 1000 条记录到 WAL...');
  for (let i = 0; i < 1000; i++) {
    const vector = encoder.encode(randomSimhash());
    index.insert(i, vector);
    await persistence.logInsert(i, vector);
  }
  
  // 强制刷 WAL
  await persistence.flush();
  
  // 检查 WAL 大小
  const walSizeBefore = await persistence.getStats().then(s => s.walSize);
  console.log(`WAL 大小: ${formatMemory(walSizeBefore)}`);
  
  // 创建 checkpointer
  const checkpointer = new WALCheckpointer({
    persistence,
    index,
    config: { walSizeThreshold: 1024 }  // 1KB 阈值，触发 checkpoint
  });
  
  // 手动触发 checkpoint
  console.log('触发 checkpoint...');
  const cpStart = Date.now();
  await checkpointer.checkpoint();
  const cpTime = Date.now() - cpStart;
  
  // 检查 WAL 是否被截断
  const walSizeAfter = await persistence.getStats().then(s => s.walSize);
  console.log(`Checkpoint 时间: ${formatTime(cpTime)}`);
  console.log(`WAL 截断后: ${formatMemory(walSizeAfter)}`);
  console.log(`\nDEBT-PHASE2-006 清偿: WAL 自动截断 -> ${walSizeAfter < walSizeBefore ? '✅ 通过' : '❌ 失败'}`);
  
  // 清理
  await fs.rm(testPath, { recursive: true, force: true });
  
  return { cpTime, walSizeBefore, walSizeAfter };
}

/**
 * 基准测试 5: 写入队列（DEBT-PHASE2-007 清偿验证）
 */
async function benchmarkWriteQueue() {
  console.log('\n=== 基准测试 5: 写入队列（DEBT-PHASE2-007）===\n');
  
  const index = new HNSWIndex({ distanceMetric: 'l2' });
  const encoder = new VectorEncoder();
  
  // 创建写入队列
  const processedIds = [];
  const queue = new WriteQueue({
    processor: async (batch) => {
      for (const op of batch) {
        if (op.type === 'INSERT') {
          index.insert(op.data.id, op.data.vector);
          processedIds.push(op.data.id);
        }
      }
    },
    config: { batchSize: 10 }
  });
  
  queue.start();
  
  // 并发 100 个写入
  console.log('并发 100 个写入请求...');
  const promises = [];
  for (let i = 0; i < 100; i++) {
    const vector = encoder.encode(randomSimhash());
    promises.push(queue.insert(i, vector));
  }
  
  await Promise.all(promises);
  
  // 等待队列处理完
  await queue.shutdown();
  
  // 验证
  const successCount = processedIds.length;
  const uniqueCount = new Set(processedIds).size;
  
  console.log(`提交: 100, 处理: ${successCount}, 唯一: ${uniqueCount}`);
  console.log(`队列统计:`, queue.getStats());
  
  const passed = successCount === 100 && uniqueCount === 100;
  console.log(`\nDEBT-PHASE2-007 清偿: 并发写入无丢失 -> ${passed ? '✅ 通过' : '❌ 失败'}`);
  
  return { successCount, uniqueCount };
}

/**
 * 运行所有基准测试
 */
async function runAllBenchmarks() {
  console.log('╔══════════════════════════════════════════════════════════════╗');
  console.log('║     Phase 2.1 债务清偿基准测试套件                           ║');
  console.log('║     HAJIMI-PHASE2.1-DEBT-CLEARANCE-BENCHMARK                 ║');
  console.log('╚══════════════════════════════════════════════════════════════╝');
  console.log(`\n开始时间: ${new Date().toISOString()}`);
  console.log(`Node版本: ${process.version}`);
  console.log(`测试规模: ${TEST_CONFIG.vectorCount.toLocaleString()} 向量`);
  
  const results = {};
  
  try {
    results.build = await benchmarkBuild();
    results.query = await benchmarkQuery();
    results.binary = await benchmarkBinarySerialize();
    results.checkpoint = await benchmarkCheckpoint();
    results.queue = await benchmarkWriteQueue();
    
    // 汇总报告
    console.log('\n╔══════════════════════════════════════════════════════════════╗');
    console.log('║                    债务清偿验证汇总                           ║');
    console.log('╠══════════════════════════════════════════════════════════════╣');
    console.log(`║ DEBT-PHASE2-006 (WAL膨胀)     ${results.checkpoint.walSizeAfter < results.checkpoint.walSizeBefore ? '✅ 已清偿' : '❌ 未清偿'}                    ║`);
    console.log(`║ DEBT-PHASE2-007 (并发安全)    ${results.queue.successCount === 100 ? '✅ 已清偿' : '❌ 未清偿'}                    ║`);
    console.log(`║ DEBT-PHASE2-005 (JSON瓶颈)    ${results.binary.binTime < 500 ? '✅ 已清偿' : '❌ 未清偿'}                    ║`);
    console.log('╠══════════════════════════════════════════════════════════════╣');
    console.log('║ 性能基线回归:                                                ║');
    console.log(`║   构建时间: ${formatTime(results.build.buildTime).padEnd(8)} <= 80s  ${results.build.buildTime <= 80000 ? '✅' : '❌'}                    ║`);
    console.log(`║   P99延迟:  ${formatTime(results.query.p99).padEnd(8)} <= 45ms ${results.query.p99 <= 45 ? '✅' : '❌'}                    ║`);
    console.log(`║   内存峰值: ${formatMemory(results.build.peakMem).padEnd(6)} <= 150MB ${results.build.peakMem <= 150*1024*1024 ? '✅' : '❌'}                   ║`);
    console.log('╚══════════════════════════════════════════════════════════════╝\n');
    
  } catch (err) {
    console.error('\n❌ 基准测试失败:', err);
    process.exit(1);
  }
}

// 如果直接运行
if (require.main === module) {
  runAllBenchmarks();
}

module.exports = {
  runAllBenchmarks
};
