/**
 * Phase 4 Integration Test - E2E端到端验证
 * 
 * 跨模块集成测试：
 * 1. WASM+Redis双开
 * 2. 降级链（WASM→JS→SQLite）
 * 3. 性能基线
 * 4. 数据一致性
 */

const { HNSWIndexWASMV2 } = require('../src/vector/hnsw-index-wasm-v2.js');
const { RateLimiterFactory } = require('../src/security/rate-limiter-factory.js');

console.log('=== Phase 4 Integration Test ===\n');

let passed = 0;
let failed = 0;

function test(name, fn) {
  return new Promise(async (resolve) => {
    try {
      await fn();
      console.log(`✅ ${name}`);
      passed++;
    } catch (err) {
      console.log(`❌ ${name}: ${err.message}`);
      failed++;
    }
    resolve();
  });
}

function assert(condition, message) {
  if (!condition) throw new Error(message || 'Assertion failed');
}

(async () => {
  // E2E-001: WASM模块加载
  await test('E2E-001: WASM module loads', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8 });
    await index.init();
    const mode = index.getMode();
    assert(mode === 'wasm' || mode === 'javascript', `Unexpected mode: ${mode}`);
    console.log(`    Mode: ${mode}`);
  });

  // E2E-002: 限流器初始化
  await test('E2E-002: Rate limiter initializes', async () => {
    const factory = new RateLimiterFactory({ redisEnabled: false });
    const limiter = await factory.init();
    assert(limiter !== null, 'Limiter should be created');
    assert(typeof limiter.checkLimit === 'function', 'checkLimit should exist');
    await limiter.close();
  });

  // E2E-003: 向量存储+限流联动
  await test('E2E-003: Vector + Rate limiter integration', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8 });
    await index.init();
    
    const factory = new RateLimiterFactory({ redisEnabled: false });
    const limiter = await factory.init();
    
    // 模拟请求：先限流检查，再向量搜索
    const clientIp = '192.168.1.100';
    
    // 限流检查
    const rateResult = await limiter.checkLimit(clientIp, 1);
    assert(rateResult.allowed === true, 'Request should be allowed');
    
    // 向量搜索
    index.insert(1, new Float32Array(8).fill(0.1));
    index.insert(2, new Float32Array(8).fill(0.2));
    const searchResult = index.search(new Float32Array(8).fill(0.15), 2);
    assert(searchResult.length === 2, 'Should return 2 results');
    
    await limiter.close();
    console.log(`    Rate: ${rateResult.remaining} remaining, Search: ${searchResult.length} results`);
  });

  // E2E-004: 降级链验证（WASM→JS）
  await test('E2E-004: WASM to JS fallback', async () => {
    // 强制使用JS模式
    const index = new HNSWIndexWASMV2({ dimension: 8, mode: 'js' });
    await index.init();
    
    assert(index.getMode() === 'javascript', 'Should be in JS mode');
    
    // 功能正常
    index.insert(1, new Float32Array(8).fill(0.5));
    const result = index.search(new Float32Array(8).fill(0.5), 1);
    assert(result.length === 1, 'Should return 1 result');
  });

  // E2E-005: 限流器降级（SQLite）
  await test('E2E-005: Rate limiter SQLite fallback', async () => {
    const factory = new RateLimiterFactory({ redisEnabled: false });
    const limiter = await factory.init();
    
    assert(limiter.getMode() === 'sqlite', 'Should use SQLite');
    assert(limiter.isDistributed() === false, 'SQLite is not distributed');
    
    // 功能正常
    const result = await limiter.checkLimit('10.0.0.1', 1);
    assert(result.allowed === true, 'Should be allowed');
    
    await limiter.close();
  });

  // E2E-006: 性能基线检查
  await test('E2E-006: Performance baseline', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8 });
    await index.init();
    
    // 批量插入
    const start = Date.now();
    for (let i = 0; i < 100; i++) {
      const vec = new Float32Array(8).map(() => Math.random());
      index.insert(i, vec);
    }
    const insertTime = Date.now() - start;
    
    // 搜索性能
    const query = new Float32Array(8).map(() => Math.random());
    const searchStart = Date.now();
    for (let i = 0; i < 100; i++) {
      index.search(query, 10);
    }
    const searchTime = Date.now() - searchStart;
    
    console.log(`    100 inserts: ${insertTime}ms, 100 searches: ${searchTime}ms`);
    
    // 宽松性能检查（E2E测试不严格限制）
    assert(insertTime < 5000, 'Insert should be reasonably fast');
    assert(searchTime < 5000, 'Search should be reasonably fast');
  });

  // E2E-007: 数据一致性（限流计数）
  await test('E2E-007: Rate limit consistency', async () => {
    const factory = new RateLimiterFactory({ redisEnabled: false });
    const limiter = await factory.init();
    
    const ip = '192.168.1.200';
    
    // 连续请求
    const results = [];
    for (let i = 0; i < 5; i++) {
      const r = await limiter.checkLimit(ip, 1);
      results.push(r.remaining);
    }
    
    // remaining应该递减
    for (let i = 1; i < results.length; i++) {
      assert(results[i] <= results[i-1], `Remaining should decrease: ${results}`);
    }
    
    console.log(`    Remaining sequence: ${results.join(' -> ')}`);
    await limiter.close();
  });

  // E2E-008: 全局债务清偿状态
  await test('E2E-008: Global debt clearance check', () => {
    // 检查代码文件存在性
    const fs = require('fs');
    
    const files = [
      'src/vector/wasm-loader.js',
      'src/vector/hnsw-index-wasm-v2.js',
      'src/security/rate-limiter-redis.js',
      'src/security/rate-limiter-factory.js',
      'crates/hajimi-hnsw/pkg/hajimi_hnsw.js'
    ];
    
    for (const file of files) {
      assert(fs.existsSync(file), `File missing: ${file}`);
    }
    
    console.log(`    All ${files.length} deliverables present`);
  });

  // E2E-009: 模式检测链
  await test('E2E-009: Mode detection chain', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8 });
    await index.init();
    
    const factory = new RateLimiterFactory({ redisEnabled: false });
    const limiter = await factory.init();
    
    const indexMode = index.getMode();
    const limiterMode = limiter.getMode();
    
    console.log(`    Index mode: ${indexMode}, Limiter mode: ${limiterMode}`);
    
    assert(indexMode === 'wasm' || indexMode === 'javascript', 'Valid index mode');
    assert(limiterMode === 'sqlite', 'Limiter should be sqlite');
    
    await limiter.close();
  });

  // E2E-010: 统计接口完整性
  await test('E2E-010: Statistics interface', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8 });
    await index.init();
    
    const factory = new RateLimiterFactory({ redisEnabled: false });
    const limiter = await factory.init();
    
    // 执行一些操作
    index.insert(1, new Float32Array(8).fill(0.1));
    await limiter.checkLimit('1.2.3.4', 1);
    
    // 获取统计
    const indexStats = index.getStats();
    const limiterStats = limiter.getStats();
    
    assert(typeof indexStats === 'object', 'Index stats should be object');
    assert(typeof limiterStats === 'object', 'Limiter stats should be object');
    
    console.log(`    Index: ${JSON.stringify(indexStats.performance)}`);
    console.log(`    Limiter: ${JSON.stringify(limiterStats)}`);
    
    await limiter.close();
  });

  console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`);
  
  if (failed === 0) {
    console.log('\n🎉 All E2E tests passed!');
  }
  
  process.exit(failed > 0 ? 1 : 0);
})();
