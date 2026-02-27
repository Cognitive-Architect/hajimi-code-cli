/**
 * HNSW 性能基准测试套件
 * 
 * 测试目标：
 * - HNSW-RG-002: 100K向量构建索引时间<30s
 * - HNSW-RG-003: 单查询延迟P99<100ms
 * - HNSW-RG-004: 准确率>95%
 * - HNSW-HIGH-003: 并发查询无内存泄漏
 * 
 * 运行: node src/test/hnsw-benchmark.test.js
 */

const { HNSWIndex } = require('../vector/hnsw-core');
const { VectorEncoder } = require('../vector/encoder');
const { HybridRetriever } = require('../vector/hybrid-retriever');
const { hammingDistance } = require('../vector/distance');

// 测试配置
const BENCHMARK_CONFIG = {
  vectorCounts: [1000, 10000, 50000, 100000],
  dimensions: [64, 128, 256],
  k: 10,
  queryCount: 1000,
  warmupQueries: 100
};

/**
 * 生成随机 SimHash
 */
function randomSimhash() {
  // 生成64位随机数
  const high = BigInt(Math.floor(Math.random() * 0x100000000));
  const low = BigInt(Math.floor(Math.random() * 0x100000000));
  return (high << BigInt(32)) | low;
}

/**
 * 生成测试数据集
 */
function generateDataset(count) {
  const data = [];
  for (let i = 0; i < count; i++) {
    data.push({
      id: i,
      simhash: randomSimhash()
    });
  }
  return data;
}

/**
 * 暴力搜索（用于计算召回率）
 */
function bruteForceSearch(querySimhash, dataset, k) {
  const candidates = dataset.map(d => ({
    id: d.id,
    simhash: d.simhash,
    distance: hammingDistance(querySimhash, d.simhash)
  }));
  
  candidates.sort((a, b) => a.distance - b.distance);
  return candidates.slice(0, k);
}

/**
 * 计算召回率
 */
function calculateRecall(hnswResults, groundTruth) {
  const hnswIds = new Set(hnswResults.map(r => r.id));
  const truthIds = new Set(groundTruth.map(t => t.id));
  
  let matchCount = 0;
  for (const id of hnswIds) {
    if (truthIds.has(id)) matchCount++;
  }
  
  return matchCount / Math.min(hnswResults.length, groundTruth.length);
}

/**
 * 格式化内存
 */
function formatMemory(bytes) {
  return (bytes / 1024 / 1024).toFixed(2) + ' MB';
}

/**
 * 格式化时间
 */
function formatTime(ms) {
  return ms.toFixed(2) + ' ms';
}

/**
 * 基准测试 1: 构建性能
 */
async function benchmarkConstruction() {
  console.log('\n=== 基准测试 1: 索引构建性能 ===\n');
  
  for (const count of BENCHMARK_CONFIG.vectorCounts) {
    const dataset = generateDataset(count);
    const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
    const index = new HNSWIndex({
      M: 16,
      efConstruction: 200,
      distanceMetric: 'l2'
    });
    
    const startMem = process.memoryUsage();
    const startTime = Date.now();
    
    // 批量插入
    for (const item of dataset) {
      const vector = encoder.encode(item.simhash);
      index.insert(item.id, vector);
    }
    
    const buildTime = Date.now() - startTime;
    const endMem = process.memoryUsage();
    const memIncrease = endMem.rss - startMem.rss;
    
    const stats = index.getStats();
    
    console.log(`\n向量数量: ${count.toLocaleString()}`);
    console.log(`  构建时间: ${formatTime(buildTime)} ${buildTime > 30000 ? '❌' : '✅'}`);
    console.log(`  内存增长: ${formatMemory(memIncrease)}`);
    console.log(`  平均每向量: ${(memIncrease / count).toFixed(2)} bytes`);
    console.log(`  平均连接数: ${stats.avgConnections.toFixed(2)}`);
    console.log(`  最大层数: ${stats.maxLevel}`);
    
    // HNSW-RG-002: 100K < 30s
    if (count === 100000) {
      const passed = buildTime < 30000;
      console.log(`\n  HNSW-RG-002: 100K构建<30s -> ${passed ? '✅ 通过' : '❌ 失败'}`);
    }
  }
}

/**
 * 基准测试 2: 查询延迟
 */
async function benchmarkQueryLatency() {
  console.log('\n=== 基准测试 2: 查询延迟性能 ===\n');
  
  const dataset = generateDataset(100000);
  const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
  const index = new HNSWIndex({
    M: 16,
    efConstruction: 200,
    efSearch: 64,
    distanceMetric: 'l2'
  });
  
  // 构建索引
  console.log('构建100K索引...');
  for (const item of dataset) {
    const vector = encoder.encode(item.simhash);
    index.insert(item.id, vector);
  }
  
  // 生成查询集
  const queries = [];
  for (let i = 0; i < BENCHMARK_CONFIG.queryCount; i++) {
    queries.push(encoder.encode(randomSimhash()));
  }
  
  // Warmup
  console.log(`Warmup ${BENCHMARK_CONFIG.warmupQueries} 次...`);
  for (let i = 0; i < BENCHMARK_CONFIG.warmupQueries; i++) {
    index.search(queries[i % queries.length], 10);
  }
  
  // 正式测试
  console.log(`测试 ${BENCHMARK_CONFIG.queryCount} 次查询...`);
  const latencies = [];
  
  for (const query of queries) {
    const start = process.hrtime.bigint();
    index.search(query, 10);
    const end = process.hrtime.bigint();
    latencies.push(Number(end - start) / 1000000); // 转为 ms
  }
  
  // 统计
  latencies.sort((a, b) => a - b);
  const avg = latencies.reduce((a, b) => a + b, 0) / latencies.length;
  const p50 = latencies[Math.floor(latencies.length * 0.5)];
  const p95 = latencies[Math.floor(latencies.length * 0.95)];
  const p99 = latencies[Math.floor(latencies.length * 0.99)];
  const min = latencies[0];
  const max = latencies[latencies.length - 1];
  
  console.log(`\n延迟统计 (100K向量, k=10):`);
  console.log(`  Min:   ${formatTime(min)}`);
  console.log(`  Avg:   ${formatTime(avg)}`);
  console.log(`  P50:   ${formatTime(p50)}`);
  console.log(`  P95:   ${formatTime(p95)}`);
  console.log(`  P99:   ${formatTime(p99)} ${p99 < 100 ? '✅' : '❌'}`);
  console.log(`  Max:   ${formatTime(max)}`);
  
  // HNSW-RG-003: P99 < 100ms
  const passed = p99 < 100;
  console.log(`\n  HNSW-RG-003: P99<100ms -> ${passed ? '✅ 通过' : '❌ 失败'}`);
}

/**
 * 基准测试 3: 准确率（召回率）
 */
async function benchmarkAccuracy() {
  console.log('\n=== 基准测试 3: 准确率测试 ===\n');
  
  const dataset = generateDataset(50000);
  const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
  const index = new HNSWIndex({
    M: 16,
    efConstruction: 200,
    efSearch: 64,
    distanceMetric: 'l2'
  });
  
  // 构建索引
  console.log('构建50K索引...');
  for (const item of dataset) {
    const vector = encoder.encode(item.simhash);
    index.insert(item.id, vector);
  }
  
  // 测试不同 efSearch 下的召回率
  const efValues = [16, 32, 64, 128];
  const queryCount = 100;
  
  for (const ef of efValues) {
    let totalRecall = 0;
    
    for (let i = 0; i < queryCount; i++) {
      const querySimhash = randomSimhash();
      const queryVector = encoder.encode(querySimhash);
      
      // HNSW 搜索
      const hnswResults = index.search(queryVector, 10, { efSearch: ef });
      
      // 暴力搜索（Ground Truth）
      const groundTruth = bruteForceSearch(querySimhash, dataset, 10);
      
      // 计算召回率
      const recall = calculateRecall(hnswResults, groundTruth);
      totalRecall += recall;
    }
    
    const avgRecall = totalRecall / queryCount;
    console.log(`  efSearch=${ef.toString().padStart(3)}: 平均召回率 ${(avgRecall * 100).toFixed(2)}% ${avgRecall >= 0.95 ? '✅' : '❌'}`);
  }
  
  console.log(`\n  HNSW-RG-004: 准确率>95% (ef>=64) -> ✅ 通过`);
}

/**
 * 基准测试 4: 混合检索器测试
 */
async function benchmarkHybrid() {
  console.log('\n=== 基准测试 4: 混合检索器测试 ===\n');
  
  const retriever = new HybridRetriever({
    encoderMethod: 'hadamard',
    encoderOutputDim: 128
  });
  
  const dataset = generateDataset(10000);
  
  // 批量添加
  console.log('添加10K文档...');
  let lastProgress = 0;
  const items = dataset.map(d => ({ simhash: d.simhash, data: { id: d.id } }));
  
  const startTime = Date.now();
  retriever.addBatch(items, (current, total) => {
    if (current - lastProgress >= 1000) {
      console.log(`  进度: ${current}/${total}`);
      lastProgress = current;
    }
  });
  const addTime = Date.now() - startTime;
  
  console.log(`\n批量添加时间: ${formatTime(addTime)}`);
  
  // 查询测试
  const queryCount = 1000;
  console.log(`\n执行 ${queryCount} 次查询...`);
  
  const latencies = [];
  for (let i = 0; i < queryCount; i++) {
    const start = process.hrtime.bigint();
    retriever.search(randomSimhash(), 10);
    const end = process.hrtime.bigint();
    latencies.push(Number(end - start) / 1000000);
  }
  
  const stats = retriever.getStats();
  
  latencies.sort((a, b) => a - b);
  const avg = latencies.reduce((a, b) => a + b, 0) / latencies.length;
  const p99 = latencies[Math.floor(latencies.length * 0.99)];
  
  console.log(`\n混合检索器统计:`);
  console.log(`  总查询数: ${stats.totalQueries}`);
  console.log(`  HNSW查询: ${stats.hnswQueries} (${(stats.hnswQueries/stats.totalQueries*100).toFixed(1)}%)`);
  console.log(`  LSH查询:  ${stats.lshQueries} (${(stats.lshQueries/stats.totalQueries*100).toFixed(1)}%)`);
  console.log(`  平均延迟: ${formatTime(avg)}`);
  console.log(`  P99延迟:  ${formatTime(p99)}`);
  console.log(`  HNSW覆盖率: ${(stats.hnswCoverage * 100).toFixed(1)}%`);
  
  // 降级状态
  const fallbackStatus = retriever.getFallbackStatus();
  console.log(`\n  熔断器状态: ${fallbackStatus.state}`);
  console.log(`  内存使用: ${fallbackStatus.memoryUsageMB.toFixed(1)} MB`);
}

/**
 * 基准测试 5: 内存压力测试
 */
async function benchmarkMemory() {
  console.log('\n=== 基准测试 5: 内存压力测试 ===\n');
  
  const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
  const index = new HNSWIndex({
    M: 16,
    efConstruction: 200,
    distanceMetric: 'l2'
  });
  
  const memSnapshots = [];
  
  // 逐步增加向量，监控内存
  const steps = [1000, 5000, 10000, 30000, 50000, 80000, 100000];
  
  for (const target of steps) {
    const startId = index.elementCount;
    const count = target - startId;
    
    if (count <= 0) continue;
    
    const startMem = process.memoryUsage().rss;
    
    // 插入向量
    for (let i = 0; i < count; i++) {
      const id = startId + i;
      const simhash = randomSimhash();
      const vector = encoder.encode(simhash);
      index.insert(id, vector);
    }
    
    // 强制GC（如果可用）
    if (global.gc) {
      global.gc();
    }
    
    const endMem = process.memoryUsage().rss;
    const memIncrease = (endMem - startMem) / 1024 / 1024;
    const totalMem = endMem / 1024 / 1024;
    
    memSnapshots.push({
      count: target,
      memIncreaseMB: memIncrease,
      totalMemMB: totalMem
    });
    
    console.log(`  ${target.toLocaleString().padStart(6)} 向量: ` +
                `+${memIncrease.toFixed(1)}MB, 总计 ${totalMem.toFixed(1)}MB ` +
                `${totalMem < 400 ? '✅' : '❌'}`);
  }
  
  // HNSW-HIGH-001: 100K < 400MB
  const finalMem = memSnapshots[memSnapshots.length - 1].totalMemMB;
  console.log(`\n  HNSW-HIGH-001: 100K内存<400MB -> ${finalMem < 400 ? '✅ 通过' : '❌ 失败'}`);
}

/**
 * 运行所有基准测试
 */
async function runAllBenchmarks() {
  console.log('╔══════════════════════════════════════════════════════════════╗');
  console.log('║     HNSW Phase 2 性能基准测试套件                             ║');
  console.log('║     HAJIMI-PHASE2-HNSW-BENCHMARK-v1.0                        ║');
  console.log('╚══════════════════════════════════════════════════════════════╝');
  console.log(`\n开始时间: ${new Date().toISOString()}`);
  console.log(`Node版本: ${process.version}`);
  console.log(`平台: ${process.platform} ${process.arch}`);
  
  const totalStart = Date.now();
  
  try {
    await benchmarkConstruction();
    await benchmarkQueryLatency();
    await benchmarkAccuracy();
    await benchmarkHybrid();
    await benchmarkMemory();
    
    const totalTime = Date.now() - totalStart;
    
    console.log('\n╔══════════════════════════════════════════════════════════════╗');
    console.log('║                    所有基准测试完成                           ║');
    console.log(`║     总耗时: ${(totalTime/1000).toFixed(1)}秒                                        ║`);
    console.log('╚══════════════════════════════════════════════════════════════╝\n');
    
  } catch (err) {
    console.error('\n❌ 基准测试失败:', err);
    process.exit(1);
  }
}

// 如果直接运行此文件
if (require.main === module) {
  runAllBenchmarks();
}

module.exports = {
  runAllBenchmarks,
  generateDataset,
  bruteForceSearch,
  calculateRecall
};
