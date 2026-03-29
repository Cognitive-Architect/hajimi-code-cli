/**
 * 写入队列 - Write Queue
 * 
 * 职责：
 * 1. 写入请求队列化，保证顺序执行
 * 2. 批量处理，提升吞吐量
 * 3. 队列溢出保护（优雅降级）
 * 4. 操作日志追踪
 * 
 * DEBT-PHASE2-007 清偿方案
 */

const { EventEmitter } = require('events');

// 队列配置
const QUEUE_CONFIG = {
  maxQueueDepth: 1000,        // 最大队列深度
  batchSize: 50,              // 批量处理大小
  flushInterval: 100,         // 自动刷盘间隔(ms)
  concurrency: 1,             // 并发数（单线程为1）
  overflowStrategy: 'reject'  // 'reject' | 'drop-old' | 'block'
};

/**
 * 操作类型
 */
const OperationType = {
  INSERT: 'INSERT',
  DELETE: 'DELETE',
  UPDATE: 'UPDATE'
};

/**
 * 队列状态
 */
const QueueState = {
  IDLE: 'IDLE',
  PROCESSING: 'PROCESSING',
  PAUSED: 'PAUSED',
  OVERFLOW: 'OVERFLOW'
};

/**
 * 写入队列
 */
class WriteQueue extends EventEmitter {
  /**
   * @param {Object} options
   * @param {Function} options.processor - 处理函数 (operations) => Promise
   * @param {Object} options.config - 配置
   */
  constructor(options = {}) {
    super();
    
    this.processor = options.processor;
    this.config = { ...QUEUE_CONFIG, ...options.config };
    
    this.queue = [];
    this.state = QueueState.IDLE;
    this.processing = false;
    
    // 统计
    this.stats = {
      totalEnqueued: 0,
      totalProcessed: 0,
      totalRejected: 0,
      totalFailed: 0,
      avgBatchSize: 0,
      avgProcessTime: 0
    };
    
    // 操作日志
    this.operationLog = [];
    this.maxLogSize = 10000;
    
    // 定时器
    this.flushTimer = null;
  }
  
  /**
   * 启动队列处理
   */
  start() {
    if (this.flushTimer) return;
    
    this.flushTimer = setInterval(() => {
      this._tryFlush();
    }, this.config.flushInterval);
    
    if (this.flushTimer.unref) {
      this.flushTimer.unref();
    }
    
    console.log('[WriteQueue] Started');
    this.emit('started');
  }
  
  /**
   * 停止队列处理
   */
  stop() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
    
    console.log('[WriteQueue] Stopped');
    this.emit('stopped');
  }
  
  /**
   * 入队操作
   * @param {string} type - OperationType
   * @param {*} data - 操作数据
   * @returns {Promise} - 操作完成时 resolve
   */
  enqueue(type, data) {
    return new Promise((resolve, reject) => {
      // 检查队列深度
      if (this.queue.length >= this.config.maxQueueDepth) {
        this.stats.totalRejected++;
        
        switch (this.config.overflowStrategy) {
          case 'reject':
            reject(new Error('Queue overflow: max depth reached'));
            return;
            
          case 'drop-old':
            // 移除最旧的操作
            const dropped = this.queue.shift();
            console.warn('[WriteQueue] Dropped old operation:', dropped?.type);
            break;
            
          case 'block':
            // 阻塞等待（简化为 reject）
            reject(new Error('Queue overflow: blocking not supported in single-thread'));
            return;
        }
      }
      
      // 创建操作项
      const operation = {
        id: ++this.stats.totalEnqueued,
        type,
        data,
        timestamp: Date.now(),
        resolve,
        reject
      };
      
      this.queue.push(operation);
      
      // 记录操作日志
      this._logOperation(operation);
      
      this.emit('enqueued', operation);
      
      // 立即尝试处理（如果队列未满批处理大小）
      if (this.queue.length >= this.config.batchSize) {
        this._tryFlush();
      }
    });
  }
  
  /**
   * 插入请求
   */
  insert(id, vector, metadata = {}) {
    return this.enqueue(OperationType.INSERT, { id, vector, metadata });
  }
  
  /**
   * 删除请求
   */
  delete(id) {
    return this.enqueue(OperationType.DELETE, { id });
  }
  
  /**
   * 尝试刷盘
   */
  async _tryFlush() {
    if (this.processing || this.queue.length === 0) return;
    if (this.state === QueueState.PAUSED) return;
    
    this.processing = true;
    this.state = QueueState.PROCESSING;
    
    try {
      await this._processBatch();
    } finally {
      this.processing = false;
      
      if (this.queue.length === 0) {
        this.state = QueueState.IDLE;
      }
    }
  }
  
  /**
   * 处理一批操作
   */
  async _processBatch() {
    // 取出一批操作
    const batchSize = Math.min(this.queue.length, this.config.batchSize);
    const batch = this.queue.splice(0, batchSize);
    
    const startTime = Date.now();
    
    try {
      // 调用处理器
      if (this.processor) {
        await this.processor(batch);
      }
      
      // 成功，resolve 所有
      for (const op of batch) {
        op.resolve({ 
          id: op.id, 
          type: op.type,
          timestamp: Date.now() 
        });
      }
      
      // 更新统计
      const duration = Date.now() - startTime;
      this._updateStats(batch.length, duration);
      
      this.emit('batchProcessed', { 
        count: batch.length, 
        duration 
      });
      
    } catch (err) {
      console.error('[WriteQueue] Batch processing failed:', err);
      
      this.stats.totalFailed += batch.length;
      
      // 失败，reject 所有
      for (const op of batch) {
        op.reject(err);
      }
      
      this.emit('batchFailed', err);
    }
  }
  
  /**
   * 更新统计
   */
  _updateStats(batchSize, duration) {
    this.stats.totalProcessed += batchSize;
    
    // 移动平均
    const n = Math.ceil(this.stats.totalProcessed / this.config.batchSize);
    this.stats.avgBatchSize = (this.stats.avgBatchSize * (n - 1) + batchSize) / n;
    this.stats.avgProcessTime = (this.stats.avgProcessTime * (n - 1) + duration) / n;
  }
  
  /**
   * 记录操作日志
   */
  _logOperation(operation) {
    const logEntry = {
      id: operation.id,
      type: operation.type,
      timestamp: operation.timestamp,
      dataSize: JSON.stringify(operation.data).length
    };
    
    this.operationLog.push(logEntry);
    
    // 限制日志大小
    if (this.operationLog.length > this.maxLogSize) {
      this.operationLog = this.operationLog.slice(-this.maxLogSize / 2);
    }
  }
  
  /**
   * 获取队列状态
   */
  getStatus() {
    return {
      state: this.state,
      queueDepth: this.queue.length,
      maxDepth: this.config.maxQueueDepth,
      stats: { ...this.stats }
    };
  }
  
  /**
   * 获取操作日志
   */
  getOperationLog(limit = 100) {
    return this.operationLog.slice(-limit);
  }
  
  /**
   * 暂停处理
   */
  pause() {
    this.state = QueueState.PAUSED;
    console.log('[WriteQueue] Paused');
  }
  
  /**
   * 恢复处理
   */
  resume() {
    this.state = QueueState.IDLE;
    console.log('[WriteQueue] Resumed');
    this._tryFlush();
  }
  
  /**
   * 清空队列
   */
  clear() {
    const count = this.queue.length;
    
    // reject 所有等待的操作
    for (const op of this.queue) {
      op.reject(new Error('Queue cleared'));
    }
    
    this.queue = [];
    console.log(`[WriteQueue] Cleared ${count} operations`);
  }
  
  /**
   * 优雅关闭（处理完队列）
   */
  async shutdown() {
    this.stop();
    
    console.log('[WriteQueue] Draining queue...');
    
    // 等待队列处理完
    while (this.queue.length > 0) {
      await this._tryFlush();
      if (this.queue.length > 0) {
        await this._delay(10);
      }
    }
    
    console.log('[WriteQueue] Shutdown complete');
  }
  
  /**
   * 延迟
   */
  _delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * 操作日志追踪器
 */
class OperationLogger {
  constructor(options = {}) {
    this.maxSize = options.maxSize || 10000;
    this.logs = [];
  }
  
  log(operation, result) {
    const entry = {
      id: operation.id,
      type: operation.type,
      startTime: operation.timestamp,
      endTime: Date.now(),
      duration: Date.now() - operation.timestamp,
      success: !result.error,
      error: result.error?.message
    };
    
    this.logs.push(entry);
    
    // 限制大小
    if (this.logs.length > this.maxSize) {
      this.logs = this.logs.slice(-this.maxSize / 2);
    }
    
    return entry;
  }
  
  getLogs(filter = {}) {
    let logs = this.logs;
    
    if (filter.type) {
      logs = logs.filter(l => l.type === filter.type);
    }
    
    if (filter.success !== undefined) {
      logs = logs.filter(l => l.success === filter.success);
    }
    
    return logs;
  }
  
  getStats() {
    const total = this.logs.length;
    const successful = this.logs.filter(l => l.success).length;
    const failed = total - successful;
    const avgDuration = total > 0 
      ? this.logs.reduce((sum, l) => sum + l.duration, 0) / total 
      : 0;
    
    return {
      total,
      successful,
      failed,
      avgDuration: Math.round(avgDuration)
    };
  }
}

module.exports = {
  WriteQueue,
  OperationLogger,
  OperationType,
  QueueState,
  QUEUE_CONFIG
};
