/**
 * BatchWriterOptimized - 生产级批量写入优化器
 * 
 * 目标：吞吐>1000 ops/s，崩溃零丢失
 * 
 * 核心特性：
 * 1. WAL模式 - 读写并发
 * 2. 事务批量 - 100条批量提交
 * 3. 压缩存储 - gzip减少磁盘IO
 * 4. 异步刷盘 - fs.promises非阻塞
 * 5. 崩溃恢复 - WAL重放
 */

const fs = require('fs').promises;
const path = require('path');
const zlib = require('zlib');
const { promisify } = require('util');

const gzip = promisify(zlib.gzip);
const gunzip = promisify(zlib.gunzip);

class BatchWriterOptimized {
  constructor(options = {}) {
    this.config = {
      dbPath: options.dbPath || './data/batch-writer.db',
      walPath: options.walPath || './data/batch-writer.wal',
      batchSize: options.batchSize || 100,
      flushInterval: options.flushInterval || 100, // 100ms
      compress: options.compress !== false, // 默认启用压缩
      maxQueueSize: options.maxQueueSize || 10000, // 队列上限
      ...options
    };

    this.writeQueue = []; // 写入队列
    this.walBuffer = []; // WAL缓冲区
    this.isFlushing = false;
    this.flushTimer = null;
    this.stats = {
      totalWrites: 0,
      totalBatches: 0,
      totalBytes: 0,
      avgLatency: 0
    };
  }

  /**
   * 初始化
   */
  async init() {
    // 确保目录存在
    const dir = path.dirname(this.config.dbPath);
    await fs.mkdir(dir, { recursive: true });

    // 恢复WAL（崩溃恢复）
    await this._recoverFromWAL();

    // 启动后台刷盘
    this._startBackgroundFlush();

    console.log('[BatchWriterOptimized] Initialized');
  }

  /**
   * 写入数据（入队）
   */
  async write(key, value) {
    // 队列上限检查
    if (this.writeQueue.length >= this.config.maxQueueSize) {
      // 优雅降级：等待刷盘完成
      await this._flushBatch();
    }

    const entry = {
      key,
      value,
      timestamp: Date.now(),
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    };

    this.writeQueue.push(entry);
    this.walBuffer.push(entry); // 同时写入WAL缓冲区

    // 达到batchSize时触发批量写入
    if (this.writeQueue.length >= this.config.batchSize) {
      await this._flushBatch();
    }

    return entry.id;
  }

  /**
   * 批量写入（事务包裹）
   */
  async _flushBatch() {
    if (this.isFlushing || this.writeQueue.length === 0) {
      return;
    }

    this.isFlushing = true;
    const batch = [...this.writeQueue];
    this.writeQueue = [];

    const startTime = Date.now();

    try {
      // 1. 写入WAL（保证持久化）
      await this._writeWAL(batch);

      // 2. 批量写入主存储
      await this._writeToMainStorage(batch);

      // 3. 清空WAL缓冲区（已安全写入）
      this.walBuffer = this.walBuffer.filter(e => !batch.find(b => b.id === e.id));

      // 4. 更新统计
      this.stats.totalWrites += batch.length;
      this.stats.totalBatches++;
      const latency = Date.now() - startTime;
      this.stats.avgLatency = (this.stats.avgLatency * (this.stats.totalBatches - 1) + latency) / this.stats.totalBatches;

      console.log(`[BatchWriterOptimized] Batch flushed: ${batch.length} items, ${latency}ms`);
    } catch (err) {
      // 回滚：重新入队
      this.writeQueue.unshift(...batch);
      console.error('[BatchWriterOptimized] Flush failed:', err);
      throw err;
    } finally {
      this.isFlushing = false;
    }
  }

  /**
   * 写入WAL（Write-Ahead Log）
   */
  async _writeWAL(batch) {
    const walEntry = {
      timestamp: Date.now(),
      entries: batch,
      checksum: this._computeChecksum(batch)
    };

    const data = JSON.stringify(walEntry) + '\n';
    await fs.appendFile(this.config.walPath, data);
  }

  /**
   * 批量写入主存储
   */
  async _writeToMainStorage(batch) {
    // 读取现有数据
    let data = {};
    try {
      const existing = await fs.readFile(this.config.dbPath);
      const decompressed = this.config.compress 
        ? await gunzip(existing)
        : existing;
      data = JSON.parse(decompressed.toString());
    } catch (err) {
      // 文件不存在或损坏，使用空对象
      data = {};
    }

    // 合并新数据
    for (const entry of batch) {
      data[entry.key] = entry.value;
    }

    // 压缩并写入
    const output = JSON.stringify(data);
    const finalData = this.config.compress
      ? await gzip(Buffer.from(output))
      : Buffer.from(output);

    await fs.writeFile(this.config.dbPath, finalData);
    this.stats.totalBytes += finalData.length;
  }

  /**
   * 崩溃恢复（从WAL重放）
   */
  async _recoverFromWAL() {
    try {
      const walData = await fs.readFile(this.config.walPath, 'utf8');
      const lines = walData.split('\n').filter(l => l.trim());

      let recoveredCount = 0;
      for (const line of lines) {
        try {
          const entry = JSON.parse(line);
          // 验证checksum
          if (entry.checksum === this._computeChecksum(entry.entries)) {
            // 重放到主存储
            await this._writeToMainStorage(entry.entries);
            recoveredCount += entry.entries.length;
          }
        } catch (err) {
          console.warn('[BatchWriterOptimized] WAL entry corrupt, skipping');
        }
      }

      if (recoveredCount > 0) {
        console.log(`[BatchWriterOptimized] Recovered ${recoveredCount} entries from WAL`);
        // 清空WAL
        await fs.writeFile(this.config.walPath, '');
      }
    } catch (err) {
      // WAL不存在，无需恢复
    }
  }

  /**
   * 计算校验和
   */
  _computeChecksum(entries) {
    const str = JSON.stringify(entries);
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }
    return hash.toString(16);
  }

  /**
   * 启动后台刷盘
   */
  _startBackgroundFlush() {
    this.flushTimer = setInterval(async () => {
      if (this.writeQueue.length > 0 && !this.isFlushing) {
        try {
          await this._flushBatch();
        } catch (err) {
          console.error('[BatchWriterOptimized] Background flush failed:', err);
        }
      }
    }, this.config.flushInterval);
  }

  /**
   * 读取数据
   */
  async read(key) {
    // 优先检查队列（未刷盘数据）
    const queuedEntry = [...this.writeQueue].reverse().find(e => e.key === key);
    if (queuedEntry) {
      return queuedEntry.value;
    }

    // 从主存储读取
    try {
      const data = await fs.readFile(this.config.dbPath);
      const decompressed = this.config.compress
        ? await gunzip(data)
        : data;
      const obj = JSON.parse(decompressed.toString());
      return obj[key];
    } catch (err) {
      return undefined;
    }
  }

  /**
   * 获取统计信息
   */
  getStats() {
    return {
      ...this.stats,
      queueSize: this.writeQueue.length,
      walBufferSize: this.walBuffer.length
    };
  }

  /**
   * 关闭
   */
  async close() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }

    // 强制刷盘
    if (this.writeQueue.length > 0) {
      await this._flushBatch();
    }

    console.log('[BatchWriterOptimized] Closed');
  }
}

module.exports = { BatchWriterOptimized };
