/**
 * Worker阻塞测试
 * 验证构建时API响应延迟
 */

const { IndexBuilderBridge } = require('../../src/worker/index-builder-bridge');
const { HajimiServer } = require('../../src/api/server');
const http = require('http');

class WorkerBlockingBenchmark {
  constructor() {
    this.results = [];
  }

  async runAll() {
    console.log('🔥 Starting Worker Blocking Benchmark...\n');
    
    await this.testWorkerMode();
    await this.testMainThreadMode();
    this.compareResults();
    
    this.printSummary();
  }

  /**
   * 测试Worker模式
   */
  async testWorkerMode() {
    console.log('📊 Benchmark: Worker Mode (Non-blocking)');
    
    const bridge = new IndexBuilderBridge({ useWorker: true });
    const server = new HajimiServer({ port: 3997 });
    
    try {
      await bridge.init();
      await server.start();
      
      // 生成测试向量
      const vectors = Array.from({ length: 3000 }, (_, i) => ({
        id: i,
        vector: Array.from({ length: 128 }, () => Math.random())
      }));
      
      // 启动构建
      console.log('  Starting index build...');
      const buildPromise = bridge.buildIndex(vectors);
      
      // 在构建期间测试API延迟
      console.log('  Testing API latency during build...');
      const latencies = [];
      
      for (let i = 0; i < 20; i++) {
        const start = Date.now();
        try {
          await this.httpGet('http://localhost:3997/health');
        } catch (err) {
          // 忽略错误
        }
        latencies.push(Date.now() - start);
        await new Promise(r => setTimeout(r, 100));
      }
      
      await buildPromise;
      
      const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const maxLatency = Math.max(...latencies);
      
      this.results.push({
        mode: 'worker',
        avgLatency,
        maxLatency,
        latencies,
        passed: avgLatency < 10 && maxLatency < 100
      });
      
      console.log(`    Avg latency: ${avgLatency.toFixed(2)}ms`);
      console.log(`    Max latency: ${maxLatency}ms`);
      console.log(`    Target <10ms: ${avgLatency < 10 ? '✅ PASS' : '❌ FAIL'}\n`);
      
    } finally {
      await server.stop();
      await bridge.shutdown();
    }
  }

  /**
   * 测试主线程模式（对比）
   */
  async testMainThreadMode() {
    console.log('📊 Benchmark: Main Thread Mode (Blocking)');
    
    const bridge = new IndexBuilderBridge({ useWorker: false });
    const server = new HajimiServer({ port: 3996 });
    
    try {
      await bridge.init();
      await server.start();
      
      // 生成测试向量（较少，因为主线程会阻塞）
      const vectors = Array.from({ length: 1000 }, (_, i) => ({
        id: i,
        vector: Array.from({ length: 128 }, () => Math.random())
      }));
      
      // 启动构建
      console.log('  Starting index build...');
      const buildPromise = bridge.buildIndex(vectors);
      
      // 在构建期间测试API延迟
      console.log('  Testing API latency during build...');
      const latencies = [];
      
      for (let i = 0; i < 10; i++) {
        const start = Date.now();
        try {
          await this.httpGet('http://localhost:3996/health');
        } catch (err) {
          // 忽略错误
        }
        latencies.push(Date.now() - start);
        await new Promise(r => setTimeout(r, 100));
      }
      
      await buildPromise;
      
      const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const maxLatency = Math.max(...latencies);
      
      this.results.push({
        mode: 'main_thread',
        avgLatency,
        maxLatency,
        latencies
      });
      
      console.log(`    Avg latency: ${avgLatency.toFixed(2)}ms`);
      console.log(`    Max latency: ${maxLatency}ms\n`);
      
    } finally {
      await server.stop();
      await bridge.shutdown();
    }
  }

  /**
   * 对比结果
   */
  compareResults() {
    console.log('📊 Comparison');
    
    const worker = this.results.find(r => r.mode === 'worker');
    const main = this.results.find(r => r.mode === 'main_thread');
    
    if (worker && main) {
      const improvement = (main.avgLatency / worker.avgLatency).toFixed(2);
      console.log(`  Worker is ${improvement}x more responsive`);
      console.log(`  Worker avg: ${worker.avgLatency.toFixed(2)}ms`);
      console.log(`  Main thread avg: ${main.avgLatency.toFixed(2)}ms\n`);
    }
  }

  /**
   * HTTP GET 辅助
   */
  httpGet(url) {
    return new Promise((resolve, reject) => {
      http.get(url, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => resolve(data));
      }).on('error', reject);
    });
  }

  /**
   * 打印汇总
   */
  printSummary() {
    console.log('='.repeat(60));
    console.log('📊 Worker Blocking Benchmark Summary');
    console.log('='.repeat(60));
    
    for (const r of this.results) {
      const icon = r.passed !== undefined ? (r.passed ? '✅' : '❌') : '⏱️';
      console.log(`${icon} ${r.mode}:`);
      console.log(`  Avg latency: ${r.avgLatency.toFixed(2)}ms`);
      console.log(`  Max latency: ${r.maxLatency}ms`);
      if (r.passed !== undefined) {
        console.log(`  Target <10ms: ${r.passed ? 'PASS' : 'FAIL'}`);
      }
    }
    
    console.log('='.repeat(60));
  }
}

// 运行
if (require.main === module) {
  const bench = new WorkerBlockingBenchmark();
  bench.runAll();
}

module.exports = { WorkerBlockingBenchmark };
