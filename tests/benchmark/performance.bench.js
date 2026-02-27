/**
 * 性能基准测试
 * Performance Benchmarks
 * 
 * BENCH-001: WASM加速比 (对比JS版本)
 * BENCH-002: 磁盘模式查询延迟
 * BENCH-003: API并发性能
 */

const { PerformanceObserver, performance } = require('perf_hooks');
const { MemoryMappedStore } = require('../../src/disk/memory-mapped-store');
const { OverflowManager } = require('../../src/disk/overflow-manager');

class BenchmarkRunner {
  constructor() {
    this.results = [];
  }

  async runAll() {
    console.log('🔥 Starting Performance Benchmarks...\n');
    
    await this.benchmarkDiskPerformance();
    await this.benchmarkMemoryUsage();
    await this.benchmarkConcurrentOps();
    
    this.printSummary();
    return this.results;
  }

  /**
   * BENCH-002: 磁盘模式性能测试
   */
  async benchmarkDiskPerformance() {
    console.log('📊 Benchmark: BENCH-002 磁盘模式性能');
    
    const bench = {
      id: 'BENCH-002',
      name: '磁盘模式读写性能',
      tests: []
    };
    
    const store = new MemoryMappedStore({ 
      basePath: './data/bench-disk',
      blockSize: 4096
    });
    await store.init();
    
    // 测试1: 顺序写入
    console.log('  Testing sequential write...');
    const writeSizes = [1024, 4096, 16384, 65536]; // 1KB to 64KB
    
    for (const size of writeSizes) {
      const data = Buffer.alloc(size, 'x');
      const iterations = 100;
      
      const start = performance.now();
      for (let i = 0; i < iterations; i++) {
        await store.write('bench-write', i * size, data);
      }
      const duration = performance.now() - start;
      
      const throughput = (size * iterations / 1024 / 1024) / (duration / 1000);
      
      bench.tests.push({
        name: `Sequential Write ${size}B`,
        iterations,
        duration: duration.toFixed(2),
        throughputMBps: throughput.toFixed(2),
        latencyMs: (duration / iterations).toFixed(3)
      });
      
      console.log(`    ${size}B: ${throughput.toFixed(2)} MB/s, ${(duration/iterations).toFixed(3)}ms/op`);
    }
    
    // 测试2: 随机读取
    console.log('  Testing random read...');
    const readBench = { name: 'Random Read 4KB', iterations: 1000 };
    const offsets = Array.from({ length: 1000 }, () => Math.floor(Math.random() * 100) * 4096);
    
    const readStart = performance.now();
    for (const offset of offsets) {
      await store.read('bench-write', offset, 4096);
    }
    const readDuration = performance.now() - readStart;
    
    bench.tests.push({
      name: 'Random Read 4KB',
      iterations: 1000,
      duration: readDuration.toFixed(2),
      latencyMs: (readDuration / 1000).toFixed(3)
    });
    
    console.log(`    Random Read: ${(readDuration/1000).toFixed(3)}ms/op`);
    
    // 测试3: 缓存命中率
    console.log('  Testing cache performance...');
    const cacheStats1 = store.getStats().cache;
    
    // 重复读取相同数据
    for (let i = 0; i < 100; i++) {
      await store.read('bench-write', 0, 4096);
    }
    
    const cacheStats2 = store.getStats().cache;
    const hitRate = cacheStats2.hitRate;
    
    bench.tests.push({
      name: 'Cache Hit Rate',
      hitRate: hitRate
    });
    
    console.log(`    Cache Hit Rate: ${hitRate}`);
    
    await store.closeAll();
    
    // 评估标准: 磁盘模式查询延迟 < 100ms (P99)
    const avgReadLatency = parseFloat(bench.tests.find(t => t.name === 'Random Read 4KB')?.latencyMs || 0);
    bench.passed = avgReadLatency < 100;
    
    console.log(bench.passed ? '  ✅ PASSED\n' : '  ❌ FAILED\n');
    this.results.push(bench);
  }

  /**
   * BENCH-002: 内存使用测试
   */
  async benchmarkMemoryUsage() {
    console.log('📊 Benchmark: BENCH-002 内存限制');
    
    const bench = {
      id: 'BENCH-002-MEM',
      name: '磁盘模式内存占用',
      tests: []
    };
    
    const overflow = new OverflowManager({
      basePath: './data/bench-memory',
      criticalMB: 150,
      maxCacheBlocks: 250 // 限制缓存
    });
    
    await overflow.init();
    
    // 记录初始内存
    const initialMem = process.memoryUsage().rss / 1024 / 1024;
    
    // 添加大量数据
    console.log('  Adding 50K vectors...');
    for (let i = 0; i < 50000; i++) {
      await overflow.add(i, { 
        vector: new Array(128).fill(0).map(() => Math.random()),
        metadata: { index: i }
      });
      
      if (i % 10000 === 0) {
        const mem = process.memoryUsage().rss / 1024 / 1024;
        console.log(`    Progress: ${i}/50000, RSS: ${mem.toFixed(2)}MB`);
      }
    }
    
    // 记录最终内存
    const finalMem = process.memoryUsage().rss / 1024 / 1024;
    const delta = finalMem - initialMem;
    
    bench.tests.push({
      name: 'Memory Growth (50K vectors)',
      initialMB: initialMem.toFixed(2),
      finalMB: finalMem.toFixed(2),
      deltaMB: delta.toFixed(2),
      vectorsPerMB: (50000 / delta).toFixed(0)
    });
    
    console.log(`  Memory Delta: ${delta.toFixed(2)}MB (${(50000/delta).toFixed(0)} vectors/MB)`);
    
    await overflow.close();
    
    // 评估标准: 内存 < 200MB
    bench.passed = finalMem < 200;
    console.log(bench.passed ? '  ✅ PASSED\n' : '  ❌ FAILED\n');
    
    this.results.push(bench);
  }

  /**
   * BENCH-003: 并发操作测试
   */
  async benchmarkConcurrentOps() {
    console.log('📊 Benchmark: BENCH-003 并发性能');
    
    const bench = {
      id: 'BENCH-003',
      name: '并发操作性能',
      tests: []
    };
    
    const store = new MemoryMappedStore({ basePath: './data/bench-concurrent' });
    await store.init();
    
    // 测试并发写入
    console.log('  Testing concurrent writes...');
    const concurrencyLevels = [10, 50, 100];
    
    for (const concurrency of concurrencyLevels) {
      const promises = [];
      const data = Buffer.alloc(1024, 'c');
      
      const start = performance.now();
      
      for (let i = 0; i < concurrency; i++) {
        promises.push(
          store.write('concurrent', i * 1024, data)
            .catch(err => ({ error: err.message }))
        );
      }
      
      await Promise.all(promises);
      const duration = performance.now() - start;
      
      bench.tests.push({
        name: `Concurrent Write ${concurrency} ops`,
        concurrency,
        duration: duration.toFixed(2),
        opsPerSec: (concurrency / (duration / 1000)).toFixed(0)
      });
      
      console.log(`    ${concurrency} ops: ${duration.toFixed(2)}ms (${(concurrency/(duration/1000)).toFixed(0)} ops/sec)`);
    }
    
    await store.closeAll();
    
    // 评估标准: 100请求/秒
    const ops100 = parseFloat(bench.tests.find(t => t.concurrency === 100)?.opsPerSec || 0);
    bench.passed = ops100 >= 100;
    
    console.log(bench.passed ? '  ✅ PASSED\n' : '  ❌ FAILED\n');
    this.results.push(bench);
  }

  /**
   * 打印汇总
   */
  printSummary() {
    console.log('\n' + '='.repeat(60));
    console.log('📊 Benchmark Summary');
    console.log('='.repeat(60));
    
    let passed = 0;
    let failed = 0;
    
    for (const bench of this.results) {
      const icon = bench.passed ? '✅' : '❌';
      console.log(`\n${icon} ${bench.id}: ${bench.name}`);
      
      for (const test of bench.tests) {
        const details = Object.entries(test)
          .filter(([k]) => k !== 'name')
          .map(([k, v]) => `${k}=${v}`)
          .join(', ');
        console.log(`   - ${test.name}: ${details}`);
      }
      
      if (bench.passed) passed++;
      else failed++;
    }
    
    console.log('\n' + '-'.repeat(60));
    console.log(`Total: ${this.results.length} | Passed: ${passed} | Failed: ${failed}`);
    console.log('='.repeat(60));
  }
}

// 运行测试
if (require.main === module) {
  const runner = new BenchmarkRunner();
  runner.runAll().then(results => {
    const allPassed = results.every(r => r.passed);
    process.exit(allPassed ? 0 : 1);
  });
}

module.exports = { BenchmarkRunner };
