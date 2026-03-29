/**
 * Token Bucket 限流器单元测试
 * 运行：node src/security/rate-limiter.test.js
 */

const { TokenBucketRateLimiter } = require('./rate-limiter');

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
  if (!condition) {
    throw new Error(message || 'Assertion failed');
  }
}

console.log('=== Token Bucket Rate Limiter Tests ===\n');

// RATE-001: 单IP 100次请求内全部通过
test('RATE-001: 单IP 100次请求内全部通过', () => {
  const limiter = new TokenBucketRateLimiter({ capacity: 100, refillRate: 100 });
  const ip = '192.168.1.1';
  
  for (let i = 0; i < 100; i++) {
    const result = limiter.consume(ip);
    assert(result.allowed, `Request ${i + 1} should be allowed`);
  }
});

// RATE-002: 单IP 101次请求返回429（模拟）
test('RATE-002: 超过capacity后请求被拒绝', () => {
  const limiter = new TokenBucketRateLimiter({ capacity: 10, refillRate: 1 });
  const ip = '192.168.1.2';
  
  // 消耗完所有token
  for (let i = 0; i < 10; i++) {
    limiter.consume(ip);
  }
  
  // 第11次应该被拒绝
  const result = limiter.consume(ip);
  assert(!result.allowed, 'Request 11 should be rejected');
  assert(result.retryAfter > 0, 'Should have retryAfter');
});

// RATE-003: 等待补充后请求再次通过（模拟）
test('RATE-003: token补充逻辑正确', () => {
  const limiter = new TokenBucketRateLimiter({ capacity: 10, refillRate: 10 }); // 10/sec
  const ip = '192.168.1.3';
  
  // 消耗部分token
  limiter.consume(ip, 5);
  let stats = limiter.getStats(ip);
  assert(stats.remaining === 5, 'Should have 5 remaining');
  
  // 模拟时间流逝（手动修改lastRefill）
  const bucket = limiter.buckets.get(ip);
  bucket.lastRefill -= 1000; // 1秒前
  
  stats = limiter.getStats(ip);
  // 应该补充了约10个token，但不超过capacity
  assert(stats.remaining > 5, 'Tokens should be refilled');
});

// RATE-004: 不同IP独立计数
test('RATE-004: 不同IP独立计数', () => {
  const limiter = new TokenBucketRateLimiter({ capacity: 5, refillRate: 1 });
  
  // IP1 超限
  for (let i = 0; i < 5; i++) {
    limiter.consume('ip1');
  }
  const ip1Result = limiter.consume('ip1');
  assert(!ip1Result.allowed, 'IP1 should be rejected');
  
  // IP2 正常
  const ip2Result = limiter.consume('ip2');
  assert(ip2Result.allowed, 'IP2 should be allowed');
});

// RATE-005: 突发20请求立即通过
test('RATE-005: 突发capacity请求全部通过', () => {
  const limiter = new TokenBucketRateLimiter({ capacity: 20, refillRate: 1 });
  const ip = '192.168.1.5';
  
  for (let i = 0; i < 20; i++) {
    const result = limiter.consume(ip);
    assert(result.allowed, `Burst request ${i + 1} should be allowed`);
  }
});

// 响应头信息测试
test('RATE-006: 响应包含remaining和resetTime', () => {
  const limiter = new TokenBucketRateLimiter({ capacity: 10, refillRate: 1 });
  const result = limiter.consume('192.168.1.6');
  
  assert(typeof result.remaining === 'number', 'Should have remaining count');
  assert(result.resetTime instanceof Date, 'Should have resetTime');
});

// 清理测试
test('RATE-007: 过期bucket清理', () => {
  const limiter = new TokenBucketRateLimiter();
  limiter.consume('old-ip');
  
  // 模拟2小时前
  const bucket = limiter.buckets.get('old-ip');
  bucket.lastRefill -= 2 * 60 * 60 * 1000;
  
  limiter.cleanup(60 * 60 * 1000); // 清理1小时前的
  
  assert(!limiter.buckets.has('old-ip'), 'Old bucket should be cleaned');
});

console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`);
process.exit(failed > 0 ? 1 : 0);
