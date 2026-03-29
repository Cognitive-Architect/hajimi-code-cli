/**
 * Phase 4 集成测试
 * WASM + Worker + 磁盘鲁棒性 三位一体验证
 */

const { WorkerPool } = require('../../src/worker/worker-pool');
const { IndexBuilderBridge } = require('../../src/worker/index-builder-bridge');
const { OverflowManagerV2 } = require('../../src/disk/overflow-manager-v2');
const { EmergencyMode } = require('../../src/disk/emergency-mode');
const { HybridHNSWIndex } = require('../../src/vector/hnsw-index-hybrid');
const http = require('http');

class Phase4IntegrationTest {
  constructor() {
    this.results = [];
  }

  async runAll() {
    console.log('🚀 Starting Phase 4 Integration Tests...\n');
    
    await this.testWorkerIntegration();
    await this.testDiskRobustness();
    await this.testHybridIndex();
    await this.testFullWorkflow();
    
    this.printSummary();
    return this.results.every(r => r.passed);
  }

  /**
   * E2E-PH4-002: Worker不阻塞验证
   */
  async testWorkerIntegration() {
    console.log('📋 Test: E2E-PH4-002 Worker不阻塞验证');
    
    const test = {
      id: 'E2E-PH4-002',
      name: 'Worker构建时API不阻塞',
      passed: false,
      duration: 0
    };
    
    const start = Date.now();
    
    try {
      const bridge = new IndexBuilderBridge({ useWorker: true });
      await bridge.init();
      
      // 生成测试向量
      const vectors = Array.from({ length: 5000 }, (_, i) => ({
        id: i,
        vector: Array.from({ length: 128 }, () => Math.random())
      }));
      
      // 启动异步构建
      let progressReceived = false;
      bridge.on('progress', () => {
        progressReceived = true;
      });
      
      const buildPromise = bridge.buildIndex(vectors);
      
      // 模拟API请求（立即执行）
      const apiStart = Date.now();
      await new Promise(resolve => setTimeout(resolve, 10));
      const apiLatency = Date.now() - apiStart;
      
      // 等待构建完成
      const result = await buildPromise;
      
      await bridge.shutdown();
      
      test.passed = apiLatency < 100 && result.duration > 0;
      test.details = { apiLatency, buildDuration: result.duration, progressReceived };
      
      console.log(test.passed ? '  ✅ Passed' : '  ❌ Failed');
      console.log(`    API latency: ${apiLatency}ms, Build: ${result.duration}ms\n`);
      
    } catch (err) {
      test.error = err.message;
      console.log(`  ❌ Failed: ${err.message}\n`);
    }
    
    test.duration = Date.now() - start;
    this.results.push(test);
  }

  /**
   * E2E-PH4-003: 磁盘满模拟
   */
  async testDiskRobustness() {
    console.log('📋 Test: E2E-PH4-003 磁盘鲁棒性');
    
    const test = {
      id: 'E2E-PH4-003',
      name: 'ENOSPC优雅降级',
      passed: false,
      duration: 0
    };
    
    const start = Date.now();
    
    try {
      const overflow = new OverflowManagerV2({
        basePath: './data/e2e-phase4',
        emergencyThreshold: 1000 // 设置高阈值，手动触发
      });
      
      await overflow.init();
      
      // 手动触发紧急模式
      await overflow.forceEmergencyMode();
      
      // 在紧急模式下添加数据
      const result = await overflow.add(1, { test: 'data' });
      
      const wasEmergency = result.emergency === true;
      
      // 退出紧急模式
      await overflow.forceExitEmergencyMode();
      
      await overflow.close();
      
      test.passed = wasEmergency;
      test.details = { wasEmergency, mode: result.mode };
      
      console.log(test.passed ? '  ✅ Passed' : '  ❌ Failed');
      console.log(`    Emergency mode handled correctly\n`);
      
    } catch (err) {
      test.error = err.message;
      console.log(`  ❌ Failed: ${err.message}\n`);
    }
    
    test.duration = Date.now() - start;
    this.results.push(test);
  }

  /**
   * E2E-PH4-004: WASM降级验证
   */
  async testHybridIndex() {
    console.log('📋 Test: E2E-PH4-004 WASM/JS混合索引');
    
    const test = {
      id: 'E2E-PH4-004',
      name: 'WASM降级到JS',
      passed: false,
      duration: 0
    };
    
    const start = Date.now();
    
    try {
      const index = new HybridHNSWIndex({ dimension: 128 });
      await index.init();
      
      // 无论WASM是否加载，都应该能工作
      const mode = index.getMode();
      
      // 插入测试向量
      index.insert(1, Array.from({ length: 128 }, () => Math.random()));
      index.insert(2, Array.from({ length: 128 }, () => Math.random()));
      
      // 搜索
      const result = index.search(Array.from({ length: 128 }, () => Math.random()), 2);
      
      test.passed = result.results && result.results.length > 0;
      test.details = { mode, resultsCount: result.results.length };
      
      console.log(test.passed ? '  ✅ Passed' : '  ❌ Failed');
      console.log(`    Mode: ${mode}, Results: ${result.results.length}\n`);
      
    } catch (err) {
      test.error = err.message;
      console.log(`  ❌ Failed: ${err.message}\n`);
    }
    
    test.duration = Date.now() - start;
    this.results.push(test);
  }

  /**
   * E2E-PH4-001: 完整工作流
   */
  async testFullWorkflow() {
    console.log('📋 Test: E2E-PH4-001 完整工作流');
    
    const test = {
      id: 'E2E-PH4-001',
      name: 'WASM+Worker+磁盘三位一体',
      passed: false,
      duration: 0
    };
    
    const start = Date.now();
    
    try {
      // 1. 初始化所有组件
      const bridge = new IndexBuilderBridge({ useWorker: true });
      await bridge.init();
      
      const overflow = new OverflowManagerV2({ basePath: './data/e2e-workflow' });
      await overflow.init();
      
      const index = new HybridHNSWIndex({ dimension: 64 });
      await index.init();
      
      // 2. 生成测试数据
      const vectors = Array.from({ length: 1000 }, (_, i) => ({
        id: i,
        vector: Array.from({ length: 64 }, () => Math.random())
      }));
      
      // 3. Worker构建索引
      const buildResult = await bridge.buildIndex(vectors, { dimension: 64 });
      
      // 4. 使用HybridIndex进行搜索
      for (const v of vectors.slice(0, 10)) {
        index.insert(v.id, v.vector);
      }
      
      const searchResult = index.search(vectors[0].vector, 5);
      
      // 5. 磁盘溢出管理
      for (let i = 0; i < 100; i++) {
        await overflow.add(i, { vector: vectors[i % vectors.length].vector });
      }
      
      // 6. 验证所有组件正常工作
      const allWorking = buildResult.duration > 0 && 
                         searchResult.results.length > 0 && 
                         !overflow.state.isEmergencyMode;
      
      // 清理
      await bridge.shutdown();
      await overflow.close();
      
      test.passed = allWorking;
      test.details = { 
        buildTime: buildResult.duration,
        searchResults: searchResult.results.length,
        mode: index.getMode()
      };
      
      console.log(test.passed ? '  ✅ Passed' : '  ❌ Failed');
      console.log(`    Build: ${buildResult.duration}ms, Search: ${searchResult.results.length} results\n`);
      
    } catch (err) {
      test.error = err.message;
      console.log(`  ❌ Failed: ${err.message}\n`);
    }
    
    test.duration = Date.now() - start;
    this.results.push(test);
  }

  /**
   * 打印汇总
   */
  printSummary() {
    console.log('\n' + '='.repeat(60));
    console.log('📊 Phase 4 Integration Test Summary');
    console.log('='.repeat(60));
    
    let passed = 0;
    let failed = 0;
    
    for (const test of this.results) {
      const icon = test.passed ? '✅' : '❌';
      console.log(`${icon} ${test.id}: ${test.name} (${test.duration}ms)`);
      if (!test.passed) failed++;
      else passed++;
    }
    
    console.log('-'.repeat(60));
    console.log(`Total: ${this.results.length} | Passed: ${passed} | Failed: ${failed}`);
    console.log('='.repeat(60));
  }
}

// 运行测试
if (require.main === module) {
  const test = new Phase4IntegrationTest();
  test.runAll().then(success => {
    process.exit(success ? 0 : 1);
  });
}

module.exports = { Phase4IntegrationTest };
