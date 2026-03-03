/**
 * RISK-01 FIX: SAB环境检测测试
 * 
 * 测试场景：
 * 1. SAB存在时正常初始化
 * 2. SAB不存在时降级
 * 3. 降级日志输出
 * 4. 非SAB模式回归
 */

const { HNSWIndexWASMV3, SABMemoryPool, checkSABEnvironment, getSABFallbackMessage } = require('../src/vector/hnsw-index-wasm-v3.js');

// 保存原始SAB引用
const originalSAB = global.SharedArrayBuffer;

// 测试计数
let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    console.log(`✅ RSK01-${name}`);
    passed++;
  } catch (err) {
    console.log(`❌ RSK01-${name}: ${err.message}`);
    failed++;
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message || 'Assertion failed');
}

console.log('=== RISK-01 SAB Environment Detection Test ===\n');

// RSK01-001: SAB存在时正常初始化
test('001: SAB exists - checkSABEnvironment returns available', () => {
  const result = checkSABEnvironment();
  assert(result.available === true, 'SAB should be available in Node.js');
  assert(result.reason === null, 'Reason should be null when available');
});

// RSK01-002: SAB不存在时检测失败
test('002: SAB undefined - checkSABEnvironment returns unavailable', () => {
  // 模拟无SAB环境
  global.SharedArrayBuffer = undefined;
  
  const result = checkSABEnvironment();
  assert(result.available === false, 'SAB should be unavailable');
  assert(result.reason.includes('not defined'), 'Reason should mention not defined');
  
  // 恢复
  global.SharedArrayBuffer = originalSAB;
});

// RSK01-003: SAB创建失败时检测失败
test('003: SAB creation fails - checkSABEnvironment returns unavailable', () => {
  // 模拟SAB构造函数抛出错误
  global.SharedArrayBuffer = class MockSAB {
    constructor() {
      throw new Error('SecurityError');
    }
  };
  
  const result = checkSABEnvironment();
  assert(result.available === false, 'SAB should be unavailable when creation fails');
  assert(result.reason.includes('COOP/COEP'), 'Reason should mention COOP/COEP headers');
  
  // 恢复
  global.SharedArrayBuffer = originalSAB;
});

// RSK01-004: 降级消息包含解决方案提示
test('004: Fallback message contains COOP/COEP solution', () => {
  const msg = getSABFallbackMessage();
  assert(msg.includes('Cross-Origin-Opener-Policy'), 'Should mention COOP');
  assert(msg.includes('Cross-Origin-Embedder-Policy'), 'Should mention COEP');
  assert(msg.includes('same-origin'), 'Should mention same-origin');
  assert(msg.includes('require-corp'), 'Should mention require-corp');
});

// RSK01-005: SABMemoryPool在无SAB环境抛出友好错误
test('005: SABMemoryPool throws friendly error when SAB unavailable', () => {
  global.SharedArrayBuffer = undefined;
  
  try {
    new SABMemoryPool({ dimension: 128 });
    throw new Error('Should have thrown');
  } catch (err) {
    assert(err.message.includes('SABEnvironmentError'), 'Error should be SABEnvironmentError');
    assert(err.message.includes('not defined'), 'Error should mention not defined');
  }
  
  // 恢复
  global.SharedArrayBuffer = originalSAB;
});

// RSK01-006: 非SAB模式不受SAB检测影响
test('006: useSAB=false mode not affected by SAB detection', async () => {
  // 即使SAB不可用，非SAB模式也应该正常工作
  global.SharedArrayBuffer = undefined;
  
  const index = new HNSWIndexWASMV3({ 
    dimension: 128, 
    useSAB: false  // 禁用SAB
  });
  
  // 不应该抛出错误
  assert(index.config.useSAB === false, 'useSAB should be false');
  
  // 恢复
  global.SharedArrayBuffer = originalSAB;
});

// RSK01-007: SABMemoryPool正常创建时不抛出错误
test('007: SABMemoryPool creates successfully when SAB available', () => {
  // 确保SAB可用
  global.SharedArrayBuffer = originalSAB;
  
  const pool = new SABMemoryPool({ dimension: 128, initialSize: 1024 });
  assert(pool !== null, 'Pool should be created');
  assert(pool.buffer instanceof SharedArrayBuffer, 'Buffer should be SAB');
  assert(pool.config.dimension === 128, 'Dimension should match');
});

// RSK01-008: Electron环境模拟测试
test('008: Electron environment simulation - graceful fallback', () => {
  // 模拟Electron无SAB环境（typeof SharedArrayBuffer === 'undefined'）
  delete global.SharedArrayBuffer;
  
  const result = checkSABEnvironment();
  assert(result.available === false, 'Electron env should not have SAB');
  assert(result.reason.includes('not defined'), 'Reason should indicate undefined');
  
  // 恢复
  global.SharedArrayBuffer = originalSAB;
});

// RSK01-009: 降级日志级别为WARN
test('009: Fallback message appropriate for console.warn', () => {
  const msg = getSABFallbackMessage();
  // 消息应该适合作为警告输出
  assert(msg.length > 0, 'Message should not be empty');
  assert(msg.includes('falling back'), 'Should mention fallback');
});

// RSK01-010: 测试SAB视图创建失败场景
test('010: SAB view creation failure detection', () => {
  // 模拟SAB创建成功但视图创建失败
  global.SharedArrayBuffer = class MockSAB {
    constructor() {
      // 创建成功但返回无效对象
    }
  };
  // 模拟Float32Array抛出错误
  const originalFloat32Array = global.Float32Array;
  global.Float32Array = class MockFloat32Array {
    constructor() {
      throw new Error('Cannot create view');
    }
  };
  
  const result = checkSABEnvironment();
  assert(result.available === false, 'Should detect view creation failure');
  
  // 恢复
  global.SharedArrayBuffer = originalSAB;
  global.Float32Array = originalFloat32Array;
});

// 汇总结果
console.log('\n=== Results ===');
console.log(`✅ Passed: ${passed}`);
console.log(`❌ Failed: ${failed}`);
console.log(`📊 Total: ${passed + failed}`);

if (failed > 0) {
  console.log('\n⚠️  Some tests failed');
  process.exit(1);
} else {
  console.log('\n🎉 All RISK-01 tests passed!');
  process.exit(0);
}
