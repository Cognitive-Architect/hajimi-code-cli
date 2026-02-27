/**
 * LuxurySQLiteRateLimiter 基础测试
 * 
 * 运行：node tests/luxury-base.test.js
 */

const { LuxurySQLiteRateLimiter } = require('../src/security/rate-limiter-sqlite-luxury');
const fs = require('fs').promises;
const path = require('path');

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
  if (!condition) {
    throw new Error(message || 'Assertion failed');
  }
}

async function cleanup() {
  try {
    await fs.unlink('./data/rate-limiter.db').catch(() => {});
  } catch (err) {
    // ignore
  }
}

console.log('=== LuxurySQLiteRateLimiter Base Tests ===\n');

(async () => {
  // 清理环境
  await cleanup();

  // LUX-BASE-001: sql.js导入
  await test('LUX-BASE-001: sql.js can be imported', async () => {
    const initSqlJs = require('sql.js');
    assert(typeof initSqlJs === 'function', 'sql.js should export a function');
  });

  // LUX-BASE-002: 类定义
  await test('LUX-BASE-002: LuxurySQLiteRateLimiter class exists', async () => {
    assert(typeof LuxurySQLiteRateLimiter === 'function', 'Class should be defined');
    const limiter = new LuxurySQLiteRateLimiter();
    assert(limiter instanceof LuxurySQLiteRateLimiter, 'Should be instance');
  });

  // LUX-BASE-003: init()异步
  await test('LUX-BASE-003: init() is async', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-003.db' });
    const result = limiter.init();
    assert(result instanceof Promise, 'init() should return Promise');
    await result;
    await limiter.close();
  });

  // LUX-BASE-004: WAL配置
  await test('LUX-BASE-004: WAL mode is configured', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-004.db' });
    await limiter.init();
    
    // 检查PRAGMA
    const result = limiter.db.exec("PRAGMA journal_mode");
    const mode = result[0]?.values[0][0];
    assert(mode === 'wal' || mode === 'WAL', `WAL mode expected, got: ${mode}`);
    
    await limiter.close();
  });

  // LUX-BASE-005: 批量队列
  await test('LUX-BASE-005: writeQueue exists', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-005.db' });
    assert(Array.isArray(limiter.writeQueue), 'writeQueue should be Array');
    assert(limiter.writeQueue.length === 0, 'writeQueue should be empty initially');
  });

  // LUX-BASE-006: 预编译缓存
  await test('LUX-BASE-006: stmtCache exists', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-006.db' });
    await limiter.init();
    assert(limiter.stmtCache instanceof Map, 'stmtCache should be Map');
    assert(limiter.stmtCache.size >= 2, 'Should have at least 2 prepared statements');
    await limiter.close();
  });

  // LUX-BASE-007: 异步持久化
  await test('LUX-BASE-007: _asyncPersist method exists', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-007.db' });
    await limiter.init();
    assert(typeof limiter._asyncPersist === 'function', '_asyncPersist should exist');
    await limiter.close();
  });

  // LUX-BASE-008: batchSize=100
  await test('LUX-BASE-008: batchSize defaults to 100', async () => {
    const limiter = new LuxurySQLiteRateLimiter();
    assert(limiter.config.batchSize === 100, 'batchSize should be 100');
  });

  // LUX-BASE-009: cacheSize=-64000
  await test('LUX-BASE-009: cacheSize defaults to -64000', async () => {
    const limiter = new LuxurySQLiteRateLimiter();
    assert(limiter.config.cacheSize === -64000, 'cacheSize should be -64000');
  });

  // LUX-BASE-010: 无同步fs
  await test('LUX-BASE-010: no sync fs calls in code', async () => {
    const code = await fs.readFile('./src/security/rate-limiter-sqlite-luxury.js', 'utf8');
    assert(!code.includes('writeFileSync'), 'Should not use writeFileSync');
    assert(!code.includes('readFileSync'), 'Should not use readFileSync');
    assert(code.includes('promises'), 'Should use async fs');
  });

  // LUX-BASE-011: init成功
  await test('LUX-BASE-011: init() succeeds', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-011.db' });
    await limiter.init();
    assert(limiter.isInitialized === true, 'Should be initialized');
    assert(limiter.db !== null, 'DB should be set');
    await limiter.close();
  });

  // LUX-BASE-012: WAL验证
  await test('LUX-BASE-012: WAL journal mode active', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-012.db' });
    await limiter.init();
    
    const result = limiter.db.exec("PRAGMA journal_mode");
    const mode = result[0]?.values[0][0];
    console.log(`    Journal mode: ${mode}`);
    assert(mode.toLowerCase() === 'wal', 'Should be WAL mode');
    
    await limiter.close();
  });

  // LUX-BASE-013: CRUD测试
  await test('LUX-BASE-013: CRUD operations work', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-013.db' });
    await limiter.init();

    // Create
    await limiter.saveBucket('192.168.1.1', 10.5, Date.now());
    await limiter._flushBatch(); // 强制刷盘
    
    // Read
    const bucket = limiter.getBucket('192.168.1.1');
    assert(bucket !== null, 'Bucket should exist');
    assert(bucket.tokens === 10.5, 'Tokens should match');
    
    // Update
    await limiter.saveBucket('192.168.1.1', 8.0, Date.now());
    await limiter._flushBatch(); // 强制刷盘
    const updated = limiter.getBucket('192.168.1.1');
    assert(updated.tokens === 8.0, 'Tokens should be updated');
    
    await limiter.close();
  });

  // LUX-BASE-014: 初始化<100ms
  await test('LUX-BASE-014: init completes in <100ms', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-014.db' });
    const start = Date.now();
    await limiter.init();
    const elapsed = Date.now() - start;
    console.log(`    Init time: ${elapsed}ms`);
    assert(elapsed < 100, `Init should be <100ms, took ${elapsed}ms`);
    await limiter.close();
  });

  // LUX-BASE-015: close()方法
  await test('LUX-BASE-015: close() method works', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ dbPath: './data/test-015.db' });
    await limiter.init();
    assert(typeof limiter.close === 'function', 'close() should exist');
    await limiter.close();
    assert(limiter.isInitialized === false, 'Should not be initialized after close');
  });

  // LUX-BASE-016: checkLimit兼容
  await test('LUX-BASE-016: checkLimit compatible with Phase 2 API', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ 
      dbPath: './data/test-016.db',
      capacity: 5,  // 减少容量便于测试
      refillRate: 0.1  // 降低补充速率
    });
    await limiter.init();

    // 首次请求应通过
    const result1 = await limiter.checkLimit('192.168.1.100');
    assert(typeof result1.allowed === 'boolean', 'Should have allowed boolean');
    assert(typeof result1.remaining === 'number', 'Should have remaining number');
    assert(result1.resetTime instanceof Date, 'Should have resetTime Date');
    assert(result1.allowed === true, 'First request should be allowed');

    // 耗尽所有token (capacity=5，第6次应被拒绝)
    for (let i = 0; i < 5; i++) {
      const r = await limiter.checkLimit('192.168.1.100');
    }

    // 下一次应被拒绝
    const result2 = await limiter.checkLimit('192.168.1.100');
    assert(result2.allowed === false, 'Should be rejected after capacity exhausted');
    assert(typeof result2.retryAfter === 'number', 'Should have retryAfter');

    await limiter.close();
  });

  // 批量写入测试
  await test('BONUS: Batch write works', async () => {
    const limiter = new LuxurySQLiteRateLimiter({ 
      dbPath: './data/test-batch.db',
      batchSize: 3  // 减小batchSize以便测试
    });
    await limiter.init();

    // 写入3条（触发批量）
    for (let i = 0; i < 3; i++) {
      await limiter.saveBucket(`ip-${i}`, 10, Date.now());
    }

    // 等待批量写入完成（定时器触发）
    await new Promise(r => setTimeout(r, 200));

    // 验证数据存在
    const bucket = limiter.getBucket('ip-0');
    assert(bucket !== null, 'Batch written data should exist');
    assert(bucket.tokens === 10, 'Tokens should match');

    await limiter.close();
  });

  // 持久化测试
  await test('BONUS: Persistence works', async () => {
    const dbPath = './data/test-persist.db';
    
    // 第一次实例：写入数据
    const limiter1 = new LuxurySQLiteRateLimiter({ dbPath });
    await limiter1.init();
    await limiter1.saveBucket('persist-test', 5.5, Date.now());
    await limiter1._flushBatch(); // 强制刷盘
    await limiter1.close();

    // 第二次实例：读取数据
    const limiter2 = new LuxurySQLiteRateLimiter({ dbPath });
    await limiter2.init();
    const bucket = limiter2.getBucket('persist-test');
    assert(bucket !== null, 'Persisted data should exist');
    assert(bucket.tokens === 5.5, 'Tokens should persist');
    await limiter2.close();
  });

  // 清理
  await cleanup();

  console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`);
  process.exit(failed > 0 ? 1 : 0);
})();
