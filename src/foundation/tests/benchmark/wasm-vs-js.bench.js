/**
 * WASM vs JS 性能对比测试
 */

const { HybridHNSWIndex } = require('../../src/vector/hnsw-index-hybrid');
const { HNSWIndex: JSIndex } = require('../../src/vector/hnsw-core');
const { performance } = require('perf_hooks');

class WASMBenchmark {
  constructor() {
    this.results = [];
  }

  async runAll() {
    console.log('🔥 Starting WASM vs JS Benchmark...\n');
    
    await this.benchmarkJSMode();
    await this.benchmarkHybridMode();
    await this.compareResults();
    
    this.printSummary();
    return this.results;
  }

  /**
   * 生成测试向量
   */
  generateVectors(count, dimension) {
    return Array.from({ length: count }, (_, i) => ({
      id: i,
      vector: Array.from({ length: dimension }, () => Math.random())
    }));
  }

  /**
   * 测试纯JS模式
   */
  async benchmarkJSMode() {
    console.log('📊 Benchmark: Pure JavaScript Mode');
    
    const dimension = 128;
    const vectorCount = 5000;
    const searchCount = 100;
    
    const index = new JSIndex({ dimension, M: 16 });
    const vectors = this.generateVectors(vectorCount, dimension);
    
    // 构建测试
    console.log(`  Building index with ${vectorCount} vectors...`);
    const buildStart = performance.now();
    for (const v of vectors) {
      index.insert(v.id, v.vector);
    }
    const jsBuildTime = performance.now() - buildStart;
    
    // 搜索测试
    console.log(`  Searching ${searchCount} times...`);
    const searchStart = performance.now();
    for (let i = 0; i < searchCount; i++) {
      const query = vectors[i % vectors.length].vector;
      index.search(query, 10);
    }
    const jsSearchTime = performance.now() - searchStart;
    
    this.results.push({
      mode: 'javascript',
      buildTime: jsBuildTime,
      searchTime: jsSearchTime,
      searchAvg: (jsSearchTime / searchCount).toFixed(3),
      vectors: vectorCount
    });
    
    console.log(`    Build: ${jsBuildTime.toFixed(2)}ms`);
    console.log(`    Search avg: ${(jsSearchTime / searchCount).toFixed(3)}ms\n`);
  }

  /**
   * 测试混合模式（可能使用WASM）
   */
  async benchmarkHybridMode() {
    console.log('📊 Benchmark: Hybrid Mode (WASM/JS)');
    
    const dimension = 128;
    const vectorCount = 5000;
    const searchCount = 100;
    
    const index = new HybridHNSWIndex({ dimension });
    await index.init();
    
    const vectors = this.generateVectors(vectorCount, dimension);
    
    // 构建测试
    console.log(`  Building index with ${vectorCount} vectors...`);
    const buildStart = performance.now();
    for (const v of vectors) {
      index.insert(v.id, v.vector);
    }
    const hybridBuildTime = performance.now() - buildStart;
    
    // 搜索测试
    console.log(`  Searching ${searchCount} times...`);
    const searchStart = performance.now();
    for (let i = 0; i < searchCount; i++) {
      const query = vectors[i % vectors.length].vector;
      index.search(query, 10);
    }
    const hybridSearchTime = performance.now() - searchStart;
    
    const mode = index.getMode();
    
    this.results.push({
      mode: `hybrid-${mode}`,
      buildTime: hybridBuildTime,
      searchTime: hybridSearchTime,
      searchAvg: (hybridSearchTime / searchCount).toFixed(3),
      vectors: vectorCount,
      isWASM: mode === 'wasm'
    });
    
    console.log(`    Build: ${hybridBuildTime.toFixed(2)}ms`);
    console.log(`    Search avg: ${(hybridSearchTime / searchCount).toFixed(3)}ms`);
    console.log(`    Mode: ${mode}\n`);
  }

  /**
   * 对比结果
   */
  async compareResults() {
    console.log('📊 Comparison Results');
    
    const jsResult = this.results.find(r => r.mode === 'javascript');
    const hybridResult = this.results.find(r => r.mode.startsWith('hybrid'));
    
    if (!jsResult || !hybridResult) {
      console.log('  ❌ Missing results for comparison');
      return;
    }
    
    const buildSpeedup = (jsResult.buildTime / hybridResult.buildTime).toFixed(2);
    const searchSpeedup = (jsResult.searchTime / hybridResult.searchTime).toFixed(2);
    
    console.log(`  Build speedup: ${buildSpeedup}x`);
    console.log(`  Search speedup: ${searchSpeedup}x`);
    
    if (hybridResult.isWASM) {
      const wasmTarget = 5;
      const searchPassed = parseFloat(searchSpeedup) >= wasmTarget;
      console.log(`  WASM 5x target: ${searchPassed ? '✅ PASS' : '❌ FAIL'} (need ${wasmTarget}x, got ${searchSpeedup}x)`);
    } else {
      console.log('  WASM not available, using JS mode');
    }
    
    this.comparison = {
      buildSpeedup: parseFloat(buildSpeedup),
      searchSpeedup: parseFloat(searchSpeedup),
      wasmAvailable: hybridResult.isWASM
    };
    
    console.log('');
  }

  /**
   * 打印汇总
   */
  printSummary() {
    console.log('='.repeat(60));
    console.log('📊 WASM vs JS Benchmark Summary');
    console.log('='.repeat(60));
    
    for (const r of this.results) {
      console.log(`\n${r.mode}:`);
      console.log(`  Build: ${r.buildTime.toFixed(2)}ms`);
      console.log(`  Search: ${r.searchAvg}ms avg`);
    }
    
    if (this.comparison) {
      console.log('\nSpeedup:');
      console.log(`  Build: ${this.comparison.buildSpeedup}x`);
      console.log(`  Search: ${this.comparison.searchSpeedup}x`);
    }
    
    console.log('='.repeat(60));
  }
}

// 运行
if (require.main === module) {
  const bench = new WASMBenchmark();
  bench.runAll();
}

module.exports = { WASMBenchmark };
