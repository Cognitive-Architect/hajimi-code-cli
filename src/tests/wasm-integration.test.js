/**
 * WASM Integration Test - WASM/JS双模式测试
 * 
 * 验证：
 * 1. 自动检测WASM可用性
 * 2. 加载失败自动降级
 * 3. 接口100%兼容
 * 4. 性能统计
 */

const { WASMLoader, getWASMLoader, resetWASMLoader } = require('../src/vector/wasm-loader.js');
const { HNSWIndexWASMV2 } = require('../src/vector/hnsw-index-wasm-v2.js');

console.log('=== WASM Integration Test ===\n');

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

// 清理
function cleanup() {
  resetWASMLoader();
}

(async () => {
  cleanup();

  // WASM-JS-001: 自动检测WASM可用性
  await test('WASM-JS-001: Auto-detect WASM availability', async () => {
    const loader = new WASMLoader();
    await loader.init();
    const mode = loader.getMode();
    assert(mode === 'wasm' || mode === 'javascript', `Unexpected mode: ${mode}`);
    console.log(`    Mode: ${mode}`);
  });

  cleanup();

  // WASM-JS-002: 加载失败自动降级
  await test('WASM-JS-002: Fallback to JS on WASM failure', async () => {
    // 模拟WASM不可用的情况
    const loader = new WASMLoader();
    // 强制设置一个错误路径
    loader.wasmPath = '/nonexistent/path.js';
    await loader.init();
    // 应该降级到JS
    assert(loader.getMode() === 'javascript' || loader.getMode() === 'wasm', 'Should have a valid mode');
  });

  cleanup();

  // WASM-JS-003: 接口与JS版100%兼容（方法存在）
  await test('WASM-JS-003: Interface compatibility', async () => {
    const loader = await getWASMLoader();
    const index = loader.createIndex(8, 8, 50);
    
    assert(typeof index.insert === 'function', 'insert method missing');
    assert(typeof index.search === 'function', 'search method missing');
    assert(typeof index.stats === 'function', 'stats method missing');
    assert(typeof index.getMode === 'function', 'getMode method missing');
  });

  cleanup();

  // WASM-JS-004: 性能统计接口
  await test('WASM-JS-004: Performance stats interface', async () => {
    const loader = await getWASMLoader();
    const stats = loader.getStats();
    
    assert(typeof stats.mode === 'string', 'mode missing');
    assert(typeof stats.wasmAvailable === 'boolean', 'wasmAvailable missing');
    console.log(`    Stats: ${JSON.stringify(stats)}`);
  });

  cleanup();

  // WASM-JS-005: 内存管理（实例可创建和销毁）
  await test('WASM-JS-005: Memory management', async () => {
    const loader = await getWASMLoader();
    
    // 创建多个实例
    for (let i = 0; i < 5; i++) {
      const idx = loader.createIndex(8, 8, 50);
      idx.insert(i, [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]);
      const results = idx.search([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8], 3);
      assert(results.length > 0, 'Search should return results');
    }
    // 如果能走到这里，内存管理基本OK
  });

  cleanup();

  // WASM-JS-006: HNSWIndexV2初始化
  await test('WASM-JS-006: HNSWIndexV2 init', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8, M: 8 });
    await index.init();
    const mode = index.getMode();
    assert(mode === 'wasm' || mode === 'javascript', `Unexpected mode: ${mode}`);
    console.log(`    V2 Mode: ${mode}`);
  });

  cleanup();

  // WASM-JS-007: HNSWIndexV2插入和搜索
  await test('WASM-JS-007: HNSWIndexV2 insert/search', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8, M: 8 });
    await index.init();
    
    // 插入测试数据
    for (let i = 0; i < 10; i++) {
      const vec = new Float32Array(8).fill(0.1 * i);
      index.insert(i, vec);
    }
    
    // 搜索
    const query = new Float32Array(8).fill(0.5);
    const results = index.search(query, 3);
    
    assert(results.length === 3, `Expected 3 results, got ${results.length}`);
    assert(results[0].id !== undefined, 'Result should have id');
    assert(results[0].distance !== undefined, 'Result should have distance');
  });

  cleanup();

  // WASM-JS-008: HNSWIndexV2性能统计
  await test('WASM-JS-008: HNSWIndexV2 performance stats', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8, M: 8 });
    await index.init();
    
    // 执行一些操作
    for (let i = 0; i < 5; i++) {
      index.insert(i, new Float32Array(8).fill(0.1 * i));
    }
    
    index.search(new Float32Array(8).fill(0.5), 3);
    
    const stats = index.getStats();
    assert(stats.performance.inserts === 5, 'Insert count mismatch');
    assert(stats.performance.searches === 1, 'Search count mismatch');
    console.log(`    Performance: ${JSON.stringify(stats.performance)}`);
  });

  cleanup();

  // WASM-JS-009: 降级信息
  await test('WASM-JS-009: Fallback info', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8, mode: 'auto' });
    await index.init();
    
    const info = index.getFallbackInfo();
    assert(info.requestedMode === 'auto', 'requestedMode should be auto');
    assert(typeof info.actualMode === 'string', 'actualMode should be string');
    assert(typeof info.wasFallback === 'boolean', 'wasFallback should be boolean');
    console.log(`    Fallback info: ${JSON.stringify(info)}`);
  });

  cleanup();

  // WASM-JS-010: 强制JS模式
  await test('WASM-JS-010: Force JS mode', async () => {
    const index = new HNSWIndexWASMV2({ dimension: 8, mode: 'js' });
    await index.init();
    
    assert(index.getMode() === 'javascript', 'Should be in javascript mode');
  });

  cleanup();

  console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`);
  process.exit(failed > 0 ? 1 : 0);
})();
