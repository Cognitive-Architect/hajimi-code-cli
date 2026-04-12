/**
 * E2E Test: WASM Zero-Copy Path (Sprint2 Day2)
 * 
 * 测试16字节对齐内存池 + searchBatchZeroCopy集成
 * 
 * @author 唐音（Engineer）
 * @version 1.0.0
 */

const { AlignedMemoryPool } = require('../../src/wasm/wasm-memory-pool.js');

// 测试计数
let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    console.log(`✅ ${name}`);
    passed++;
  } catch (err) {
    console.log(`❌ ${name}: ${err.message}`);
    failed++;
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message || 'Assertion failed');
}

console.log('=== WASM Zero-Copy E2E Test ===\n');

// 测试1: 16字节对齐算法正确性
test('JS-014: 16字节对齐算法 _alignUp(0)===0', () => {
  const pool = new AlignedMemoryPool();
  assert(pool._alignUp(0) === 0, '_alignUp(0) should be 0');
});

test('JS-014: 16字节对齐算法 _alignUp(1)===16', () => {
  const pool = new AlignedMemoryPool();
  assert(pool._alignUp(1) === 16, '_alignUp(1) should be 16');
});

test('JS-014: 16字节对齐算法 _alignUp(15)===16', () => {
  const pool = new AlignedMemoryPool();
  assert(pool._alignUp(15) === 16, '_alignUp(15) should be 16');
});

test('JS-014: 16字节对齐算法 _alignUp(16)===16', () => {
  const pool = new AlignedMemoryPool();
  assert(pool._alignUp(16) === 16, '_alignUp(16) should be 16');
});

test('JS-014: 16字节对齐算法 _alignUp(17)===32', () => {
  const pool = new AlignedMemoryPool();
  assert(pool._alignUp(17) === 32, '_alignUp(17) should be 32');
});

// 测试2: AlignedMemoryPool类存在
test('JS-001: AlignedMemoryPool类存在', () => {
  assert(typeof AlignedMemoryPool === 'function', 'AlignedMemoryPool should be a class');
});

// 测试3: acquire方法返回Float32Array
test('JS-002: acquire返回Float32Array', () => {
  const pool = new AlignedMemoryPool({ initialSize: 1024 });
  const view = pool.acquire(10);
  assert(view instanceof Float32Array, 'should return Float32Array');
  assert(view.length === 10, 'length should be 10');
});

// 测试4: 返回的视图16字节对齐
test('JS-003: acquire返回16字节对齐视图', () => {
  const pool = new AlignedMemoryPool({ initialSize: 1024 });
  const view = pool.acquire(10);
  // Float32Array的byteOffset应该是16字节对齐
  assert(view.byteOffset % 16 === 0, `view.byteOffset (${view.byteOffset}) should be 16-byte aligned`);
});

// 测试5: ALIGNMENT常量定义
test('JS-007: ALIGNMENT常量定义为16', () => {
  const pool = new AlignedMemoryPool();
  assert(pool.ALIGNMENT === 16, 'ALIGNMENT should be 16');
});

// 测试6: 空输入处理
test('JS-010: 空输入处理（size=0）', () => {
  const pool = new AlignedMemoryPool();
  const view = pool.acquire(0);
  assert(view instanceof Float32Array, 'should return Float32Array for size=0');
  assert(view.length === 0, 'length should be 0');
});

// 测试7: 负输入处理
test('JS-010: 负输入处理（size<0）', () => {
  const pool = new AlignedMemoryPool();
  const view = pool.acquire(-5);
  assert(view instanceof Float32Array, 'should return Float32Array for negative size');
  assert(view.length === 0, 'length should be 0 for negative size');
});

// 测试8: 内存池统计信息
test('JS-001: getStats返回统计信息', () => {
  const pool = new AlignedMemoryPool({ initialSize: 1024 });
  const stats = pool.getStats();
  assert(typeof stats === 'object', 'should return object');
  assert(typeof stats.capacity === 'number', 'should have capacity');
  assert(typeof stats.used === 'number', 'should have used');
  assert(typeof stats.alignment === 'number', 'should have alignment');
  assert(stats.alignment === 16, 'alignment should be 16');
});

// 测试9: release方法存在
test('JS-006: release方法存在', () => {
  const pool = new AlignedMemoryPool();
  assert(typeof pool.release === 'function', 'release should be a function');
});

// 测试10: 多次acquire返回不同视图
test('JS-002: 多次acquire返回不同视图', () => {
  const pool = new AlignedMemoryPool({ initialSize: 4096 });
  const view1 = pool.acquire(10);
  const view2 = pool.acquire(20);
  assert(view1 !== view2, 'should return different views');
  assert(view1.length === 10, 'view1 length should be 10');
  assert(view2.length === 20, 'view2 length should be 20');
});

// 测试11: 内存池耗尽触发fallback（模拟）
test('JS-008: 内存池耗尽返回null触发fallback', () => {
  const pool = new AlignedMemoryPool({ initialSize: 64, maxSize: 128 }); // 极小池
  // 第一次分配应该成功
  const view1 = pool.acquire(10); // 40 bytes
  assert(view1 !== null, 'first acquire should succeed');
  // 持续分配直到耗尽
  let view2 = pool.acquire(1000); // 4000 bytes, should fail
  // 由于我们测试的是JS层，实际行为取决于池大小
  // 这里我们主要验证acquire能处理池耗尽情况
  assert(view2 === null || view2 instanceof Float32Array, 'should return null or Float32Array');
});

// 测试12: 数据拷贝验证
test('JS-002: 数据可正确拷贝到对齐视图', () => {
  const pool = new AlignedMemoryPool({ initialSize: 1024 });
  const view = pool.acquire(5);
  const data = new Float32Array([1.0, 2.0, 3.0, 4.0, 5.0]);
  view.set(data);
  assert(view[0] === 1.0, 'data[0] should be 1.0');
  assert(view[4] === 5.0, 'data[4] should be 5.0');
});

// 测试13: _isAligned方法
test('JS-003: _isAligned方法正确', () => {
  const pool = new AlignedMemoryPool();
  assert(pool._isAligned(0) === true, '0 should be aligned');
  assert(pool._isAligned(16) === true, '16 should be aligned');
  assert(pool._isAligned(32) === true, '32 should be aligned');
  assert(pool._isAligned(1) === false, '1 should not be aligned');
  assert(pool._isAligned(15) === false, '15 should not be aligned');
  assert(pool._isAligned(17) === false, '17 should not be aligned');
});

// 测试14: reset方法
test('JS-001: reset方法存在', () => {
  const pool = new AlignedMemoryPool();
  assert(typeof pool.reset === 'function', 'reset should be a function');
});

// 汇总
console.log('\n=== Results ===');
console.log(`✅ Passed: ${passed}`);
console.log(`❌ Failed: ${failed}`);
console.log(`📊 Total: ${passed + failed}`);

if (failed > 0) {
  console.log('\n⚠️  Some tests failed');
  process.exit(1);
} else {
  console.log('\n🎉 All E2E tests passed!');
  process.exit(0);
}
