/**
 * WASM vs JavaScript 性能基准测试 v2.0
 * 验证5x加速比目标
 */

const { HNSWIndex: WASMIndex } = require('../crates/hajimi-hnsw/pkg/hajimi_hnsw.js');
const { HNSWIndex: JSIndex } = require('../src/vector/hnsw-core.js');

console.log('=== WASM vs JavaScript HNSW Benchmark v2.0 ===\n');

// 配置
const DIMENSION = 128;
const M = 16;
const EF_CONSTRUCTION = 200;
const EF_SEARCH = 64;
const TEST_SIZES = [1000, 5000, 10000];

// 生成随机向量
function randomVector(dim) {
  const vec = new Float32Array(dim);
  for (let i = 0; i < dim; i++) {
    vec[i] = Math.random() * 2 - 1;
  }
  return vec;
}

// 归一化向量
function normalize(vec) {
  let norm = 0;
  for (let i = 0; i < vec.length; i++) {
    norm += vec[i] * vec[i];
  }
  norm = Math.sqrt(norm);
  if (norm > 0) {
    for (let i = 0; i < vec.length; i++) {
      vec[i] /= norm;
    }
  }
  return vec;
}

// 运行测试
async function runBenchmark() {
  const results = [];
  
  for (const size of TEST_SIZES) {
    console.log(`\n--- Test: ${size} vectors ---`);
    
    // 生成测试数据
    console.log(`Generating ${size} random vectors...`);
    const vectors = [];
    for (let i = 0; i < size; i++) {
      vectors.push(normalize(randomVector(DIMENSION)));
    }
    const query = normalize(randomVector(DIMENSION));
    
    // ========== WASM测试 ==========
    console.log('\n[WASM] Building index...');
    const wasmIndex = new WASMIndex(DIMENSION, M, EF_CONSTRUCTION);
    wasmIndex.set_ef_search(EF_SEARCH);
    
    const wasmBuildStart = Date.now();
    for (let i = 0; i < size; i++) {
      wasmIndex.insert(i, Array.from(vectors[i]));
    }
    const wasmBuildTime = Date.now() - wasmBuildStart;
    
    // WASM预热
    for (let i = 0; i < 100; i++) {
      wasmIndex.search(Array.from(query), 10);
    }
    
    console.log(`[WASM] Running queries...`);
    const wasmQueryStart = Date.now();
    const WASM_ITERATIONS = 1000;
    for (let i = 0; i < WASM_ITERATIONS; i++) {
      wasmIndex.search(Array.from(query), 10);
    }
    const wasmQueryTime = Date.now() - wasmQueryStart;
    const wasmAvgLatency = wasmQueryTime / WASM_ITERATIONS;
    
    // ========== JS测试 ==========
    console.log('\n[JS] Building index...');
    const jsIndex = new JSIndex({
      M: M,
      efConstruction: EF_CONSTRUCTION,
      efSearch: EF_SEARCH,
      distanceMetric: 'cosine'
    });
    
    const jsBuildStart = Date.now();
    for (let i = 0; i < size; i++) {
      jsIndex.insert(i, vectors[i]);
    }
    const jsBuildTime = Date.now() - jsBuildStart;
    
    // JS预热
    for (let i = 0; i < 100; i++) {
      jsIndex.search(query, 10);
    }
    
    console.log(`[JS] Running queries...`);
    const jsQueryStart = Date.now();
    const JS_ITERATIONS = Math.min(1000, Math.floor(30000 / (size / 1000))); // 动态调整
    for (let i = 0; i < JS_ITERATIONS; i++) {
      jsIndex.search(query, 10);
    }
    const jsQueryTime = Date.now() - jsQueryStart;
    const jsAvgLatency = jsQueryTime / JS_ITERATIONS;
    
    // ========== 计算加速比 ==========
    const speedup = jsAvgLatency / wasmAvgLatency;
    const buildSpeedup = jsBuildTime / wasmBuildTime;
    
    console.log('\n--- Results ---');
    console.log(`WASM Build: ${wasmBuildTime}ms, Query: ${wasmAvgLatency.toFixed(3)}ms/op`);
    console.log(`JS Build:   ${jsBuildTime}ms, Query: ${jsAvgLatency.toFixed(3)}ms/op`);
    console.log(`\n🚀 Query Speedup: ${speedup.toFixed(2)}x`);
    console.log(`🚀 Build Speedup: ${buildSpeedup.toFixed(2)}x`);
    
    results.push({
      size,
      wasmBuildTime,
      wasmAvgLatency,
      jsBuildTime,
      jsAvgLatency,
      querySpeedup: speedup,
      buildSpeedup
    });
    
    // 验证结果质量
    const wasmResults = wasmIndex.search(Array.from(query), 10);
    const jsResults = jsIndex.search(query, 10);
    console.log(`\nResult quality check:`);
    console.log(`WASM returned: ${wasmResults.length} items`);
    console.log(`JS returned: ${jsResults.length} items`);
  }
  
  // ========== 汇总 ==========
  console.log('\n\n' + '='.repeat(60));
  console.log('BENCHMARK SUMMARY');
  console.log('='.repeat(60));
  
  const avgSpeedup = results.reduce((sum, r) => sum + r.querySpeedup, 0) / results.length;
  const avgBuildSpeedup = results.reduce((sum, r) => sum + r.buildSpeedup, 0) / results.length;
  
  console.log('\nDataset sizes tested:', TEST_SIZES.join(', '));
  console.log('\nAverage Query Speedup:', avgSpeedup.toFixed(2) + 'x');
  console.log('Average Build Speedup:', avgBuildSpeedup.toFixed(2) + 'x');
  
  // 目标检查
  console.log('\n--- Target Check ---');
  console.log(`Query 5x target: ${avgSpeedup >= 5 ? '✅ PASS' : '❌ FAIL'} (${avgSpeedup.toFixed(2)}x)`);
  console.log(`Build 3x target: ${avgBuildSpeedup >= 3 ? '✅ PASS' : '❌ FAIL'} (${avgBuildSpeedup.toFixed(2)}x)`);
  
  // 输出JSON报告
  const report = {
    timestamp: new Date().toISOString(),
    config: { DIMENSION, M, EF_CONSTRUCTION, EF_SEARCH },
    results,
    summary: {
      avgQuerySpeedup: avgSpeedup,
      avgBuildSpeedup: avgBuildSpeedup,
      queryTargetMet: avgSpeedup >= 5,
      buildTargetMet: avgBuildSpeedup >= 3
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
