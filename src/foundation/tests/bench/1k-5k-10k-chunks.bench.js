/**
 * 1K/5K/10K Chunks Benchmark - 真实双进程测试
 * 约束: ≤140行 | 内存泄漏检测 | fork进程
 * Usage: node tests/bench/1k-5k-10k-chunks.bench.js [1000|5000|10000]
 */
const { fork } = require('child_process');
const { P2PSyncBenchmark } = require('../../src/bench/p2p-sync-benchmark.js');
const os = require('os');

const SCENARIOS = [1000, 5000, 10000];
const TARGET_CHUNK = parseInt(process.argv[2], 10) || 1000;

if (process.send) {
  // 子进程模式
  const config = { chunkCount: TARGET_CHUNK, batchSize: 100, timeout: 30000 };
  const bench = new P2PSyncBenchmark(config);
  
  bench.on('progress', ({ progress, chunks, total }) => {
    process.send({ type: 'progress', progress, chunks, total });
  });
  
  bench.run().then(result => {
    process.send({ type: 'result', result });
    process.exit(0);
  }).catch(err => {
    process.send({ type: 'error', error: err.message });
    process.exit(1);
  });
} else {
  // 主进程模式
  console.log(`🚀 HAJIMI-P2P-BENCHMARK | ${os.platform()} | Node ${process.version}`);
  console.log('='.repeat(60));
  
  async function runForked(count) {
    return new Promise((resolve, reject) => {
      console.log(`\n📦 Testing ${count} chunks (forked process)...`);
      const startMem = process.memoryUsage().rss;
      const startTime = Date.now();
      
      const child = fork(__filename, [count.toString()], { silent: true });
      let result = null;
      
      child.on('message', (msg) => {
        if (msg.type === 'progress') {
          console.log(`  ⏳ ${msg.progress}% (${msg.chunks}/${msg.total})`);
        }
        if (msg.type === 'result') result = msg.result;
      });
      
      child.on('exit', (code) => {
        const duration = Date.now() - startTime;
        const memAfter = process.memoryUsage().rss;
        
        if (code !== 0 || !result) {
          console.log(`  ❌ FAILED (exit ${code})`);
          resolve({ count, success: false, duration });
          return;
        }
        
        // 内存泄漏检测
        const leakDetected = result.memoryGrowthMB > 50;
        const memoryOK = result.maxMemoryMB < 500;
        const latencyOK = result.p95Latency < 5000;
        
        console.log(`  ✅ Duration: ${result.durationMs}ms`);
        console.log(`  ✅ Throughput: ${result.throughput} chunks/s`);
        console.log(`  ✅ P95 Latency: ${result.p95Latency}ms ${latencyOK ? '✓' : '✗'}`);
        console.log(`  ✅ Max Memory: ${result.maxMemoryMB}MB ${memoryOK ? '✓' : '✗'}`);
        console.log(`  📊 Memory Growth: ${result.memoryGrowthMB}MB ${leakDetected ? '⚠️ LEAK?' : '✓'}`);
        console.log(`  🔥 Process Memory Delta: ${Math.round((memAfter - startMem) / 1024 / 1024)}MB`);
        
        resolve({ count, success: result.success && memoryOK && latencyOK, result, leakDetected });
      });
      
      child.on('error', reject);
    });
  }
  
  (async () => {
    const results = [];
    for (const count of [TARGET_CHUNK]) {
      results.push(await runForked(count));
    }
    
    console.log('\n' + '='.repeat(60));
    console.log('📋 BENCHMARK SUMMARY');
    console.log('='.repeat(60));
    results.forEach(r => {
      console.log(`${r.count} chunks: ${r.success ? '✅ PASS' : '❌ FAIL'} | ${r.result?.durationMs || 'N/A'}ms | ${r.result?.maxMemoryMB || 'N/A'}MB`);
    });
    
    // 清理exit
    setTimeout(() => process.exit(results.every(r => r.success) ? 0 : 1), 100);
  })();
}
