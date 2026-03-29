/**
 * WAL 自动 Checkpointer - Auto Checkpointer
 * 
 * 职责：
 * 1. 监控 WAL 文件大小，自动触发 checkpoint
 * 2. 定时 checkpoint（默认 5 分钟）
 * 3. checkpoint 过程不阻塞读写
 * 4. 支持强制 checkpoint 和优雅关闭
 * 
 * DEBT-PHASE2-006 清偿方案
 */

const fs = require('fs').promises;
const path = require('path');
const crypto = require('crypto');
const { EventEmitter } = require('events');

// Checkpoint 配置
const CHECKPOINTER_CONFIG = {
  walSizeThreshold: 100 * 1024 * 1024,  // 100MB 大小阈值
  timeInterval: 5 * 60 * 1000,           // 5 分钟时间间隔
  maxRetries: 3,                         // 最大重试次数
  retryDelay: 1000,                      // 重试延迟
  syncOnCheckpoint: false                // checkpoint 是否同步刷盘
};

/**
 * Checkpoint 状态
 */
const CheckpointState = {
  IDLE: 'IDLE',
  RUNNING: 'RUNNING',
  FAILED: 'FAILED'
};

/**
 * WAL Checkpointer 类
 */
class WALCheckpointer extends EventEmitter {
  /**
   * @param {Object} options
   * @param {HNSWPersistence} options.persistence - 持久化实例
   * @param {HNSWIndex} options.index - HNSW 索引实例
   * @param {Object} options.config - 配置
   */
  constructor(options = {}) {
    super();
    
    this.persistence = options.persistence;
    this.index = options.index;
    this.config = { ...CHECKPOINTER_CONFIG, ...options.config };
    
    this.state = CheckpointState.IDLE;
    this.lastCheckpointTime = 0;
    this.checkpointCount = 0;
    this.failedCount = 0;
    
    // 定时器
    this.timer = null;
    this.monitoring = false;
    
    // 统计
    this.stats = {
      totalCheckpoints: 0,
      totalBytesProcessed: 0,
      avgCheckpointTime: 0
    };
  }
  
  /**
   * 启动监控
   */
  start() {
    if (this.monitoring) return;
    
    this.monitoring = true;
    console.log('[Checkpointer] Started');
    
    // 启动定时检查
    this._scheduleNextCheck();
    
    this.emit('started');
  }
  
  /**
   * 停止监控
   */
  stop() {
    this.monitoring = false;
    
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
    
    console.log('[Checkpointer] Stopped');
    this.emit('stopped');
  }
  
  /**
   * 调度下一次检查
   */
  _scheduleNextCheck() {
    if (!this.monitoring) return;
    
    this.timer = setTimeout(async () => {
      await this._checkAndCheckpoint();
      this._scheduleNextCheck();
    }, this.config.timeInterval);
    
    if (this.timer.unref) {
      this.timer.unref();
    }
  }
  
  /**
   * 检查并执行 checkpoint
   */
  async _checkAndCheckpoint() {
    try {
      // 检查 WAL 文件大小
      const walSize = await this._getWALSize();
      
      if (walSize > this.config.walSizeThreshold) {
        console.log(`[Checkpointer] WAL size ${(walSize/1024/1024).toFixed(1)}MB exceeds threshold, triggering checkpoint`);
        await this.checkpoint();
      }
    } catch (err) {
      console.error('[Checkpointer] Check failed:', err.message);
    }
  }
  
  /**
   * 获取 WAL 文件大小
   */
  async _getWALSize() {
    try {
      const stats = await fs.stat(this.persistence.walPath);
      return stats.size;
    } catch (err) {
      if (err.code === 'ENOENT') return 0;
      throw err;
    }
  }
  
  /**
   * 执行 checkpoint
   * @param {boolean} force - 强制 checkpoint（忽略状态）
   * @returns {Promise<boolean>}
   */
  async checkpoint(force = false) {
    // 检查状态
    if (this.state === CheckpointState.RUNNING && !force) {
      console.log('[Checkpointer] Checkpoint already in progress');
      return false;
    }
    
    this.state = CheckpointState.RUNNING;
    this.emit('checkpointStart');
    
    const startTime = Date.now();
    let retries = 0;
    
    while (retries < this.config.maxRetries) {
      try {
        // 1. 保存索引（包含当前 WAL 数据）
        await this._doCheckpoint();
        
        // 2. 清空 WAL
        await this._truncateWAL();
        
        // 3. 更新统计
        const duration = Date.now() - startTime;
        this._updateStats(duration);
        
        this.state = CheckpointState.IDLE;
        this.lastCheckpointTime = Date.now();
        this.checkpointCount++;
        
        console.log(`[Checkpointer] Checkpoint completed in ${duration}ms`);
        this.emit('checkpointComplete', { duration });
        
        return true;
        
      } catch (err) {
        retries++;
        this.failedCount++;
        
        console.error(`[Checkpointer] Checkpoint failed (attempt ${retries}):`, err.message);
        
        if (retries < this.config.maxRetries) {
          await this._delay(this.config.retryDelay * retries);
        } else {
          this.state = CheckpointState.FAILED;
          this.emit('checkpointFailed', err);
          throw err;
        }
      }
    }
    
    return false;
  }
  
  /**
   * 实际执行 checkpoint
   */
  async _doCheckpoint() {
    // 先刷 WAL 缓冲区
    if (this.persistence.wal) {
      await this.persistence.wal.flush();
    }
    
    // 保存索引（自动包含 WAL 数据）
    await this.persistence.save(this.index, {
      checkpointTime: Date.now(),
      checkpointCount: this.checkpointCount + 1
    });
  }
  
  /**
   * 截断 WAL 文件
   */
  async _truncateWAL() {
    try {
      // 原子方式：先写空文件，再重命名
      const tempPath = `${this.persistence.walPath}.tmp`;
      await fs.writeFile(tempPath, '');
      await fs.rename(tempPath, this.persistence.walPath);
      
      console.log('[Checkpointer] WAL truncated');
    } catch (err) {
      if (err.code !== 'ENOENT') {
        throw err;
      }
    }
  }
  
  /**
   * 延迟
   */
  _delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  /**
   * 更新统计
   */
  _updateStats(duration) {
    this.stats.totalCheckpoints++;
    
    // 移动平均
    const n = this.stats.totalCheckpoints;
    this.stats.avgCheckpointTime = 
      (this.stats.avgCheckpointTime * (n - 1) + duration) / n;
  }
  
  /**
   * 获取状态
   */
  getStatus() {
    return {
      state: this.state,
      lastCheckpointTime: this.lastCheckpointTime,
      checkpointCount: this.checkpointCount,
      failedCount: this.failedCount,
      stats: { ...this.stats }
    };
  }
  
  /**
   * 获取 WAL 大小
   */
  async getWALSize() {
    return this._getWALSize();
  }
  
  /**
   * 强制 checkpoint（用于优雅关闭）
   */
  async forceCheckpoint() {
    console.log('[Checkpointer] Force checkpoint...');
    return this.checkpoint(true);
  }
  
  /**
   * 优雅关闭
   */
  async shutdown() {
    this.stop();
    
    // 最后一次 checkpoint
    try {
      await this.forceCheckpoint();
    } catch (err) {
      console.error('[Checkpointer] Shutdown checkpoint failed:', err.message);
    }
    
    this.removeAllListeners();
  }
}

/**
 * Checkpoint 调度器 - 支持多种触发策略
 */
class CheckpointScheduler {
  constructor(checkpointer, options = {}) {
    this.checkpointer = checkpointer;
    this.strategies = [];
    
    // 默认策略：大小阈值
    if (options.sizeThreshold) {
      this.addStrategy(new SizeBasedStrategy(options.sizeThreshold));
    }
    
    // 默认策略：时间间隔
    if (options.interval) {
      this.addStrategy(new TimeBasedStrategy(options.interval));
    }
  }
  
  addStrategy(strategy) {
    this.strategies.push(strategy);
    strategy.on('trigger', () => this.checkpointer.checkpoint());
  }
  
  start() {
    for (const strategy of this.strategies) {
      strategy.start();
    }
  }
  
  stop() {
    for (const strategy of this.strategies) {
      strategy.stop();
    }
  }
}

/**
 * 基于大小的策略
 */
class SizeBasedStrategy extends EventEmitter {
  constructor(threshold, checkInterval = 30000) {
    super();
    this.threshold = threshold;
    this.checkInterval = checkInterval;
    this.timer = null;
  }
  
  start() {
    this.timer = setInterval(() => this._check(), this.checkInterval);
    if (this.timer.unref) this.timer.unref();
  }
  
  stop() {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }
  
  async _check() {
    // 实际检查由 checkpointer 完成
    this.emit('check');
  }
}

/**
 * 基于时间的策略
 */
class TimeBasedStrategy extends EventEmitter {
  constructor(interval) {
    super();
    this.interval = interval;
    this.timer = null;
  }
  
  start() {
    this.timer = setInterval(() => {
      this.emit('trigger');
    }, this.interval);
    if (this.timer.unref) this.timer.unref();
  }
  
  stop() {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }
}

module.exports = {
  WALCheckpointer,
  CheckpointScheduler,
  CheckpointState,
  CHECKPOINTER_CONFIG
};
