/**
 * LuxurySQLiteRateLimiter - 豪华版SQLite限流器
 * 
 * 清偿DEBT-SEC-001：使用sql.js实现持久化限流状态
 * 
 * 核心优化：
 * 1. WAL模式 - 读写并发不阻塞
 * 2. 批量写入 - writeQueue积累100条后事务提交
 * 3. 预编译缓存 - stmtCache缓存常用SQL
 * 4. 异步持久化 - db.export()后台刷盘
 * 
 * 技术栈：sql.js (纯JavaScript SQLite，零编译)
 */

const fs = require('fs').promises;
const path = require('path');

class LuxurySQLiteRateLimiter {
  constructor(options = {}) {
    this.config = {
      dbPath: options.dbPath || './data/rate-limiter.db',
      batchSize: options.batchSize || 100,
      flushInterval: options.flushInterval || 5000,
      cacheSize: options.cacheSize || -64000,
      checkpointInterval: options.checkpointInterval || 300000,
      capacity: options.capacity || 20,
      refillRate: options.refillRate || (100 / 60) // 100 req/min
    };

    this.writeQueue = []; // 批量写入队列
    this.stmtCache = new Map(); // 预编译语句缓存
    
    this.db = null;
    this.SQL = null;
    this.flushTimer = null;
    this.checkpointTimer = null;
    this.isInitialized = false;
  }

  /**
   * 初始化数据库
   * 流程：加载sql.js → 恢复/创建db → execPragmas → initSchema → prepareStatements → 启动定时器
   */
  async init() {
    if (this.isInitialized) {
      return;
    }

    try {
      // 动态导入sql.js（ESM兼容）
      const initSqlJs = require('sql.js');
      this.SQL = await initSqlJs();

      // 尝试恢复现有数据库
      await this._loadOrCreateDatabase();

      // 执行PRAGMA配置
      this._execPragmas();

      // 初始化表结构
      this._initSchema();

      // 预编译语句缓存
      this._prepareStatements();

      // 启动后台定时器
      this._startBackgroundFlush();

      this.isInitialized = true;
      console.log('[LuxurySQLiteRateLimiter] Initialized successfully');
    } catch (err) {
      console.error('[LuxurySQLiteRateLimiter] Init failed:', err);
      throw err;
    }
  }

  /**
   * 加载或创建数据库
   */
  async _loadOrCreateDatabase() {
    try {
      // 确保目录存在
      const dir = path.dirname(this.config.dbPath);
      await fs.mkdir(dir, { recursive: true });

      // 尝试读取现有数据库
      const data = await fs.readFile(this.config.dbPath).catch(() => null);
      
      if (data) {
        // 恢复现有数据库
        this.db = new this.SQL.Database(data);
        console.log('[LuxurySQLiteRateLimiter] Database restored from', this.config.dbPath);
      } else {
        // 创建新数据库
        this.db = new this.SQL.Database();
        console.log('[LuxurySQLiteRateLimiter] New database created');
      }
    } catch (err) {
      console.error('[LuxurySQLiteRateLimiter] Load/Create DB failed:', err);
      // 降级：创建内存数据库
      this.db = new this.SQL.Database();
      console.warn('[LuxurySQLiteRateLimiter] Fallback to in-memory DB');
    }
  }

  /**
   * 执行PRAGMA配置
   * WAL模式 + 性能优化
   */
  _execPragmas() {
    const pragmas = [
      'PRAGMA journal_mode = WAL',        // WAL模式：读写并发
      'PRAGMA synchronous = NORMAL',       // 同步模式：性能与持久化平衡
      'PRAGMA cache_size = -64000',        // 64MB页缓存（负值表示KB）
      'PRAGMA temp_store = MEMORY',        // 临时表存内存
      'PRAGMA mmap_size = 268435456',      // 256MB内存映射
      'PRAGMA page_size = 4096'            // 4KB页大小
    ];

    for (const pragma of pragmas) {
      try {
        this.db.exec(pragma);
      } catch (err) {
        console.warn(`[LuxurySQLiteRateLimiter] PRAGMA failed: ${pragma}`, err.message);
      }
    }

    // 验证WAL模式
    const result = this.db.exec("PRAGMA journal_mode");
    const journalMode = result[0]?.values[0][0];
    console.log(`[LuxurySQLiteRateLimiter] Journal mode: ${journalMode}`);
  }

  /**
   * 初始化表结构
   */
  _initSchema() {
    const schema = `
      CREATE TABLE IF NOT EXISTS rate_limits (
        ip TEXT PRIMARY KEY,
        tokens REAL NOT NULL,
        last_refill INTEGER NOT NULL
      );
      
      CREATE INDEX IF NOT EXISTS idx_rate_limits_refill 
      ON rate_limits(last_refill);
    `;

    this.db.exec(schema);
    console.log('[LuxurySQLiteRateLimiter] Schema initialized');
  }

  /**
   * 预编译语句缓存
   */
  _prepareStatements() {
    // 查询bucket
    const getBucketStmt = this.db.prepare(`
      SELECT tokens, last_refill FROM rate_limits WHERE ip = ?
    `);
    this.stmtCache.set('getBucket', getBucketStmt);

    // 更新bucket
    const updateBucketStmt = this.db.prepare(`
      INSERT INTO rate_limits (ip, tokens, last_refill) 
      VALUES (?, ?, ?)
      ON CONFLICT(ip) DO UPDATE SET
        tokens = excluded.tokens,
        last_refill = excluded.last_refill
    `);
    this.stmtCache.set('updateBucket', updateBucketStmt);

    // 删除bucket
    const deleteBucketStmt = this.db.prepare(`
      DELETE FROM rate_limits WHERE ip = ?
    `);
    this.stmtCache.set('deleteBucket', deleteBucketStmt);

    console.log('[LuxurySQLiteRateLimiter] Statements prepared:', this.stmtCache.size);
  }

  /**
   * 获取bucket（使用缓存语句）
   */
  getBucket(ip) {
    const stmt = this.stmtCache.get('getBucket');
    stmt.bind([ip]);
    
    const result = stmt.step();
    if (result) {
      const row = stmt.getAsObject();
      stmt.reset();
      return {
        tokens: row.tokens,
        lastRefill: row.last_refill
      };
    }
    
    stmt.reset();
    return null;
  }

  /**
   * 保存bucket（入队批量写入）
   */
  async saveBucket(ip, tokens, lastRefill) {
    // 入队
    this.writeQueue.push({ ip, tokens, lastRefill });

    // 达到batchSize时触发批量写入
    if (this.writeQueue.length >= this.config.batchSize) {
      await this._flushBatch();
    }
  }

  /**
   * 批量写入（事务包裹）
   */
  async _flushBatch() {
    if (this.writeQueue.length === 0) {
      return;
    }

    const batch = [...this.writeQueue];
    this.writeQueue = []; // 清空队列

    try {
      // BEGIN TRANSACTION
      this.db.exec('BEGIN TRANSACTION');

      const stmt = this.stmtCache.get('updateBucket');
      
      for (const item of batch) {
        stmt.bind([item.ip, item.tokens, item.lastRefill]);
        stmt.step();
        stmt.reset();
      }

      // COMMIT
      this.db.exec('COMMIT');

      console.log(`[LuxurySQLiteRateLimiter] Batch flushed: ${batch.length} items`);

      // 异步持久化到文件
      await this._asyncPersist();
    } catch (err) {
      // ROLLBACK
      try {
        this.db.exec('ROLLBACK');
      } catch (rollbackErr) {
        console.error('[LuxurySQLiteRateLimiter] Rollback failed:', rollbackErr);
      }
      
      // 重新入队失败的批次
      this.writeQueue.unshift(...batch);
      console.error('[LuxurySQLiteRateLimiter] Batch flush failed:', err);
      throw err;
    }
  }

  /**
   * 异步持久化（零阻塞）
   */
  async _asyncPersist() {
    try {
      // 导出数据库为Uint8Array
      const data = this.db.export();
      
      // 异步写入文件
      await fs.writeFile(this.config.dbPath, Buffer.from(data));
      
      console.log('[LuxurySQLiteRateLimiter] Database persisted');
    } catch (err) {
      console.error('[LuxurySQLiteRateLimiter] Persist failed:', err);
      // 不抛出错误，允许下次重试
    }
  }

  /**
   * 启动后台定时器
   */
  _startBackgroundFlush() {
    // 定期flush（5秒）
    this.flushTimer = setInterval(async () => {
      if (this.writeQueue.length > 0) {
        try {
          await this._flushBatch();
        } catch (err) {
          console.error('[LuxurySQLiteRateLimiter] Background flush failed:', err);
        }
      }
    }, this.config.flushInterval);

    // 定期checkpoint（5分钟）
    this.checkpointTimer = setInterval(() => {
      try {
        this.db.exec('PRAGMA wal_checkpoint(TRUNCATE)');
        console.log('[LuxurySQLiteRateLimiter] WAL checkpoint completed');
      } catch (err) {
        console.warn('[LuxurySQLiteRateLimiter] Checkpoint failed:', err.message);
      }
    }, this.config.checkpointInterval);

    // SIGINT时强制刷盘（仅注册一次）
    if (!LuxurySQLiteRateLimiter._sigintRegistered) {
      LuxurySQLiteRateLimiter._sigintRegistered = true;
      process.on('SIGINT', async () => {
        console.log('[LuxurySQLiteRateLimiter] SIGINT received, flushing...');
        // 全局清理所有实例
        await this.close();
        process.exit(0);
      });
    }

    console.log('[LuxurySQLiteRateLimiter] Background timers started');
  }

  /**
   * 补充token
   */
  _refill(bucket, now) {
    const elapsedMs = now - bucket.lastRefill;
    const tokensToAdd = (elapsedMs / 1000) * this.config.refillRate;
    
    bucket.tokens = Math.min(this.config.capacity, bucket.tokens + tokensToAdd);
    bucket.lastRefill = now;
  }

  /**
   * 检查限流（兼容Phase 2 API）
   * @returns {Promise<{allowed: boolean, remaining: number, resetTime: Date}>}
   */
  async checkLimit(ip, tokens = 1) {
    if (!this.isInitialized) {
      await this.init();
    }

    const now = Date.now();
    
    // 获取或创建bucket
    let bucket = this.getBucket(ip);
    if (!bucket) {
      bucket = {
        tokens: this.config.capacity,
        lastRefill: now
      };
    }

    // 补充token
    this._refill(bucket, now);

    // 检查并消费
    if (bucket.tokens >= tokens) {
      bucket.tokens -= tokens;
      
      // 异步保存
      await this.saveBucket(ip, bucket.tokens, bucket.lastRefill);
      
      return {
        allowed: true,
        remaining: Math.floor(bucket.tokens),
        resetTime: this._calculateResetTime(bucket)
      };
    }

    // 限流触发
    await this.saveBucket(ip, bucket.tokens, bucket.lastRefill);
    
    return {
      allowed: false,
      remaining: 0,
      resetTime: this._calculateResetTime(bucket),
      retryAfter: Math.ceil((tokens - bucket.tokens) / this.config.refillRate)
    };
  }

  /**
   * 计算reset时间
   */
  _calculateResetTime(bucket) {
    const tokensNeeded = this.config.capacity - bucket.tokens;
    const secondsToRefill = tokensNeeded / this.config.refillRate;
    return new Date(Date.now() + secondsToRefill * 1000);
  }

  /**
   * 获取统计信息
   */
  getStats(ip) {
    const bucket = this.getBucket(ip);
    if (!bucket) {
      return { tokens: this.config.capacity, remaining: this.config.capacity };
    }
    this._refill(bucket, Date.now());
    return {
      tokens: bucket.tokens,
      remaining: Math.floor(bucket.tokens),
      capacity: this.config.capacity,
      refillRate: this.config.refillRate
    };
  }

  /**
   * 关闭连接（清理资源）
   */
  async close() {
    console.log('[LuxurySQLiteRateLimiter] Closing...');

    // 清理定时器
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
    if (this.checkpointTimer) {
      clearInterval(this.checkpointTimer);
      this.checkpointTimer = null;
    }

    // 强制刷盘
    if (this.writeQueue.length > 0) {
      try {
        await this._flushBatch();
      } catch (err) {
        console.error('[LuxurySQLiteRateLimiter] Final flush failed:', err);
      }
    }

    // 关闭数据库
    if (this.db) {
      this.db.close();
      this.db = null;
    }

    this.isInitialized = false;
    console.log('[LuxurySQLiteRateLimiter] Closed');
  }
}

module.exports = { LuxurySQLiteRateLimiter };
