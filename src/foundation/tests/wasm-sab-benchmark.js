/**
 * WASM SAB Benchmark - 5x加速比验证
 * 
 * 对比：
 * 1. 纯JS实现（hnsw-core.js）
 * 2. WASM v2（基础WASM）
 * 3. WASM v3（SAB优化）
 */

const { HNSWIndex: JSIndex } = require('../src/vector/hnsw-core.js');
const { HNSWIndexWASMV2 } = require('../src/vector/hnsw-index-wasm-v2.js');
const { HNSWIndexWASMV3 } = require('../src/vector/hnsw-index-wasm-v3.js');

console.log('=== WASM SAB 5x Acceleration Benchmark ===\n');

const CONFIG = {
  DIMENSION: 128,
  M: 16,
  EF_CONSTRUCTION: 200,
  EF_SEARCH: 64,
  TEST_SIZES: [1000, 5000, 10000],
  QUERY_ITERATIONS: 1000
};

// 生成随机向量
function randomVector(dim) {
  const vec = new Float32Array(dim);
  for (let i = 0; i < dim; i++) {
    vec[i] = Math.random() * 2 - 1;
  }
  // 归一化
  let norm = 0;
  for (let i = 0; i < dim; i++) norm += vec[i] * vec[i];
  norm = Math.sqrt(norm);
  if (norm > 0) for (let i = 0; i < dim; i++) vec[i] /= norm;
  return vec;
}

// 基准测试主函数
async function runBenchmark() {
  const results = [];
  
  for (const size of CONFIG.TEST_SIZES) {
    console.log(`\n${'='.repeat(60)}`);
    console.log(`Dataset Size: ${size} vectors`);
    console.log('='.repeat(60));
    
    // 生成测试数据
    console.log(`Generating ${size} random vectors...`);
    const vectors = [];
    for (let i = 0; i < size; i++) {
      vectors.push({ id: i, vector: randomVector(CONFIG.DIMENSION) });
    }
    const query = randomVector(CONFIG.DIMENSION);
    const batchQueries = Array(10).fill(0).map(() => randomVector(CONFIG.DIMENSION));

    // ========== 测试1: 纯JS ==========
    console.log('\n[1/3] Testing Pure JavaScript...');
    const jsIndex = new JSIndex({
      M: CONFIG.M,
      efConstruction: CONFIG.EF_CONSTRUCTION,
      efSearch: CONFIG.EF_SEARCH,
      distanceMetric: 'cosine'
    });
    
    const jsBuildStart = Date.now();
    for (const v of vectors) jsIndex.insert(v.id, v.vector);
    const jsBuildTime = Date.now() - jsBuildStart;
    
    // 预热
    for (let i = 0; i < 100; i++) jsIndex.search(query, 10);
    
    // 单查询测试
    const jsQueryStart = Date.now();
    for (let i = 0; i < CONFIG.QUERY_ITERATIONS; i++) {
      jsIndex.search(query, 10);
    }
    const jsQueryTime = Date.now() - jsQueryStart;
    const jsAvgLatency = jsQueryTime / CONFIG.QUERY_ITERATIONS;
    
    console.log(`  Build: ${jsBuildTime}ms, Query: ${jsAvgLatency.toFixed(3)}ms/op`);

    // ========== 测试2: WASM v2 ==========
    console.log('\n[2/3] Testing WASM v2...');
    const wasmV2 = new HNSWIndexWASMV2({
      dimension: CONFIG.DIMENSION,
      M: CONFIG.M,
      efConstruction: CONFIG.EF_CONSTRUCTION,
      efSearch: CONFIG.EF_SEARCH
    });
    await wasmV2.init();
    
    const v2BuildStart = Date.now();
    for (const v of vectors) wasmV2.insert(v.id, v.vector);
    const v2BuildTime = Date.now() - v2BuildStart;
    
    // 预热
    for (let i = 0; i < 100; i++) wasmV2.search(query, 10);
    
    // 单查询测试
    const v2QueryStart = Date.now();
    for (let i = 0; i < CONFIG.QUERY_ITERATIONS; i++) {
      wasmV2.search(query, 10);
    }
    const v2QueryTime = Date.now() - v2QueryStart;
    const v2AvgLatency = v2QueryTime / CONFIG.QUERY_ITERATIONS;
    
    console.log(`  Build: ${v2BuildTime}ms, Query: ${v2AvgLatency.toFixed(3)}ms/op`);

    // ========== 测试3: WASM v3 (SAB) ==========
    console.log('\n[3/3] Testing WASM v3 (SAB)...');
    const wasmV3 = new HNSWIndexWASMV3({
      dimension: CONFIG.DIMENSION,
      M: CONFIG.M,
      efConstruction: CONFIG.EF_CONSTRUCTION,
      efSearch: CONFIG.EF_SEARCH,
      useSAB: true
    });
    await wasmV3.init();
    
    const v3BuildStart = Date.now();
    for (const v of vectors) wasmV3.insert(v.id, v.vector);
    const v3BuildTime = Date.now() - v3BuildStart;
    
    // 预热
    for (let i = 0; i < 100; i++) wasmV3.search(query, 10);
    
    // 单查询测试
    const v3QueryStart = Date.now();
    for (let i = 0; i < CONFIG.QUERY_ITERATIONS; i++) {
      wasmV3.search(query, 10);
    }
    const v3QueryTime = Date.now() - v3QueryStart;
    const v3AvgLatency = v3QueryTime / CONFIG.QUERY_ITERATIONS;
    
    // 批量查询测试
    const v3BatchStart = Date.now();
    for (let i = 0; i < 100; i++) {
      wasmV3.searchBatch(batchQueries, 10);
    }
    const v3BatchTime = Date.now() - v3BatchStart;
    const v3BatchAvg = v3BatchTime / (100 * 10); // 每查询的平均时间
    
    console.log(`  Build: ${v3BuildTime}ms, Query: ${v3AvgLatency.toFixed(3)}ms/op`);
    console.log(`  Batch: ${v3BatchAvg.toFixed(3)}ms/op (per query in batch)`);
    
    // SAB状态
    const sabStatus = wasmV3.getSABStatus();
    console.log(`  SAB: ${sabStatus.enabled ? 'enabled' : 'disabled'}, ${JSON.stringify(sabStatus.stats)}`);

    // ========== 计算加速比 ==========
    const speedupV2 = jsAvgLatency / v2AvgLatency;
    const speedupV3 = jsAvgLatency / v3AvgLatency;
    const speedupV3Batch = jsAvgLatency / v3BatchAvg;
    
    const buildSpeedupV2 = jsBuildTime / v2BuildTime;
    const buildSpeedupV3 = jsBuildTime / v3BuildTime;

    console.log('\n' + '-'.repeat(60));
    console.log('Speedup Results:');
    console.log(`  WASM v2 Query: ${speedupV2.toFixed(2)}x`);
    console.log(`  WASM v3 Query: ${speedupV3.toFixed(2)}x`);
    console.log(`  WASM v3 Batch: ${speedupV3Batch.toFixed(2)}x`);
    console.log(`  WASM v2 Build: ${buildSpeedupV2.toFixed(2)}x`);
    console.log(`  WASM v3 Build: ${buildSpeedupV3.toFixed(2)}x`);
    console.log('-'.repeat(60));

    // 目标检查
    const target5xMet = speedupV3 >= 5.0 || speedupV3Batch >= 5.0;
    console.log(`\n🎯 5x Target: ${target5xMet ? '✅ PASS' : '❌ FAIL'}`);
    if (!target5xMet) {
      console.log(`   (v3: ${speedupV3.toFixed(2)}x, batch: ${speedupV3Batch.toFixed(2)}x)`);
    }

    results.push({
      size,
      jsBuildTime,
      jsAvgLatency,
      v2BuildTime,
      v2AvgLatency,
      v3BuildTime,
      v3AvgLatency,
      v3BatchAvg,
      speedupV2,
      speedupV3,
      speedupV3Batch,
      target5xMet
    });
  }

  // ========== 汇总 ==========
  console.log('\n\n' + '='.repeat(60));
  console.log('BENCHMARK SUMMARY');
  console.log('='.repeat(60));

  const avgSpeedupV3 = results.reduce((sum, r) => sum + r.speedupV3, 0) / results.length;
  const avgSpeedupV3Batch = results.reduce((sum, r) => sum + r.speedupV3Batch, 0) / results.length;
  const avgBuildSpeedup = results.reduce((sum, r) => sum + (r.jsBuildTime / r.v3BuildTime), 0) / results.length;

  console.log(`\nAverage Query Speedup (WASM v3): ${avgSpeedupV3.toFixed(2)}x`);
  console.log(`Average Query Speedup (Batch): ${avgSpeedupV3Batch.toFixed(2)}x`);
  console.log(`Average Build Speedup: ${avgBuildSpeedup.toFixed(2)}x`);

  console.log('\n--- Target Check ---');
  const overall5x = avgSpeedupV3 >= 5.0 || avgSpeedupV3Batch >= 5.0;
  console.log(`5x Query Target: ${overall5x ? '✅ PASS' : '❌ FAIL'} (v3: ${avgSpeedupV3.toFixed(2)}x, batch: ${avgSpeedupV3Batch.toFixed(2)}x)`);

  // JSON报告
  const report = {
    timestamp: new Date().toISOString(),
    config: CONFIG,
    results,
    summary: {
      avgQuerySpeedup: avgSpeedupV3,
      avgQuerySpeedupBatch: avgSpeedupV3Batch,
      avgBuildSpeedup,
      target5xMet: overall5x
    }
  };

  console.log('\n--- JSON Report ---');
  console.log(JSON.stringify(report, null, 2));

  return report;
}

runBenchmark().catch(err => {
  console.error('Benchmark failed:', err);
  process.exit(1);
});
