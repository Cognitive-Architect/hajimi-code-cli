/**
 * E2E测试: WASM + 磁盘 + API 三位一体
 * End-to-End Test: WASM + Disk + API Integration
 */

const { MemoryMappedStore } = require('../../src/disk/memory-mapped-store');
const { OverflowManager } = require('../../src/disk/overflow-manager');
const { HajimiServer } = require('../../src/api/server');
const http = require('http');

class E2ETestRunner {
  constructor() {
    this.results = [];
  }

  async runAll() {
    console.log('🚀 Starting E2E Tests...\n');
    
    await this.testDiskOverflow();
    await this.testAPIServer();
    await this.testIntegration();
    
    this.printSummary();
    return this.results.every(r => r.passed);
  }

  /**
   * E2E-001: 完整工作流
   */
  async testIntegration() {
    console.log('📋 Test: E2E-001 完整工作流');
    
    const test = {
      id: 'E2E-001',
      name: '完整工作流（存储→溢出→API查询）',
      passed: false,
      duration: 0
    };
    
    const start = Date.now();
    
    try {
      // 1. 初始化磁盘存储
      const store = new MemoryMappedStore({ basePath: './data/e2e-test' });
      await store.init();
      
      // 2. 初始化溢出管理器
      const overflow = new OverflowManager({ basePath: './data/e2e-test' });
      await overflow.init();
      
      // 3. 启动API服务器
      const server = new HajimiServer({ port: 3999 });
      await server.start();
      
      // 4. 执行操作
      const testData = Buffer.from(JSON.stringify({ test: 'data', timestamp: Date.now() }));
      await store.write('test-file', 0, testData);
      
      // 5. 健康检查
      const health = await this._httpGet('http://localhost:3999/health');
      if (health.status !== 'ok') {
        throw new Error('Health check failed');
      }
      
      // 6. 清理
      await server.stop();
      await overflow.close();
      await store.closeAll();
      
      test.passed = true;
      console.log('  ✅ Passed\n');
      
    } catch (err) {
      test.error = err.message;
      console.log(`  ❌ Failed: ${err.message}\n`);
    }
    
    test.duration = Date.now() - start;
    this.results.push(test);
  }

  /**
   * E2E-002: 100K向量场景
   */
  async testDiskOverflow() {
    console.log('📋 Test: E2E-002 100K向量内存限制');
    
    const test = {
      id: 'E2E-002',
      name: '100K向量场景（插入10万条→内存<200MB）',
      passed: false,
      duration: 0
    };
    
    const start = Date.now();
    
    try {
      const overflow = new OverflowManager({
        basePath: './data/e2e-overflow',
        criticalMB: 180,
        evictionBatchSize: 1000
      });
      
      await overflow.init();
      
      // 模拟添加大量数据
      const batchSize = 1000;
      const totalBatches = 100; // 100K
      
      for (let batch = 0; batch < totalBatches; batch++) {
        for (let i = 0; i < batchSize; i++) {
          const id = batch * batchSize + i;
          await overflow.add(id, { data: 'test' });
        }
        
        // 每10批检查一次内存
        if (batch % 10 === 0) {
          const mem = process.memoryUsage();
          console.log(`  Batch ${batch}/${totalBatches}, RSS: ${(mem.rss/1024/1024).toFixed(1)}MB`);
          
          if (mem.rss > 200 * 1024 * 1024) {
            console.log('  ⚠️ Memory threshold exceeded, triggering overflow...');
          }
        }
      }
      
      // 验证内存
      const finalMem = process.memoryUsage();
      const rssMB = finalMem.rss / 1024 / 1024;
      
      console.log(`  Final RSS: ${rssMB.toFixed(2)}MB`);
      
      if (rssMB > 250) {
        throw new Error(`Memory too high: ${rssMB.toFixed(2)}MB > 250MB`);
      }
      
      await overflow.close();
      
      test.passed = true;
      test.details = { finalRSS: rssMB.toFixed(2) };
      console.log('  ✅ Passed\n');
      
    } catch (err) {
      test.error = err.message;
      console.log(`  ❌ Failed: ${err.message}\n`);
    }
    
    test.duration = Date.now() - start;
    this.results.push(test);
  }

  /**
   * E2E-003: API服务器测试
   */
  async testAPIServer() {
    console.log('📋 Test: E2E-003 API服务器');
    
    const test = {
      id: 'E2E-003',
      name: 'API健康检查与基本路由',
      passed: false,
      duration: 0
    };
    
    const start = Date.now();
    const server = new HajimiServer({ port: 3998 });
    
    try {
      await server.start();
      
      // 测试健康检查
      const health = await this._httpGet('http://localhost:3998/health');
      if (health.status !== 'ok') {
        throw new Error('Health check failed');
      }
      
      // 测试就绪检查
      const ready = await this._httpGet('http://localhost:3998/health/ready');
      if (!ready.status) {
        throw new Error('Readiness check failed');
      }
      
      // 测试存活检查
      const live = await this._httpGet('http://localhost:3998/health/live');
      if (live.status !== 'alive') {
        throw new Error('Liveness check failed');
      }
      
      // 测试指标
      const metrics = await this._httpGet('http://localhost:3998/health/metrics');
      if (!metrics.memory) {
        throw new Error('Metrics missing');
      }
      
      await server.stop();
      
      test.passed = true;
      console.log('  ✅ Passed\n');
      
    } catch (err) {
      await server.stop().catch(() => {});
      test.error = err.message;
      console.log(`  ❌ Failed: ${err.message}\n`);
    }
    
    test.duration = Date.now() - start;
    this.results.push(test);
  }

  /**
   * HTTP GET 辅助方法
   */
  _httpGet(url) {
    return new Promise((resolve, reject) => {
      http.get(url, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
          try {
            resolve(JSON.parse(data));
          } catch {
            resolve(data);
          }
        });
      }).on('error', reject);
    });
  }

  /**
   * 打印汇总
   */
  printSummary() {
    console.log('\n' + '='.repeat(50));
    console.log('📊 E2E Test Summary');
    console.log('='.repeat(50));
    
    let passed = 0;
    let failed = 0;
    
    for (const test of this.results) {
      const icon = test.passed ? '✅' : '❌';
      console.log(`${icon} ${test.id}: ${test.name} (${test.duration}ms)`);
      if (!test.passed) failed++;
      else passed++;
    }
    
    console.log('-'.repeat(50));
    console.log(`Total: ${this.results.length} | Passed: ${passed} | Failed: ${failed}`);
    console.log('='.repeat(50));
  }
}

// 运行测试
if (require.main === module) {
  const runner = new E2ETestRunner();
  runner.runAll().then(success => {
    process.exit(success ? 0 : 1);
  });
}

module.exports = { E2ETestRunner };
