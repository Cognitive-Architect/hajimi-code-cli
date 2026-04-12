/**
 * 磁盘溢出管理器
 * Overflow Manager - 当内存超过阈值时自动溢出到磁盘
 * 
 * 目标: 内存占用恒定在200MB以内
 */

const EventEmitter = require('events');
const { MemoryMappedStore } = require('./memory-mapped-store');

class OverflowManager extends EventEmitter {
  constructor(options = {}) {
    super();
    
    // 内存阈值配置
    this.thresholds = {
      warning: options.warningMB || 150,    // 警告阈值 150MB
      critical: options.criticalMB || 180,  // 开始溢出 180MB
      emergency: options.emergencyMB || 220 // 紧急处理 220MB
    };
    
    this.store = new MemoryMappedStore({
      basePath: options.basePath || './data/overflow',
      blockSize: options.blockSize || 4096,
      maxCacheBlocks: options.maxCacheBlocks || 500
    });
    
    // 溢出状态跟踪
    this.state = {
      isOverflowing: false,
      overflowStartTime: null,
      overflowedCount: 0,
      inMemoryCount: 0,
      totalCount: 0
    };
    
    // 监控定时器
    this.monitorInterval = null;
    this.monitorPeriodMs = options.monitorPeriodMs || 5000; // 5秒检查一次
    
    // 溢出策略
    this.policy = {
      evictionBatchSize: options.evictionBatchSize || 100,
      preferRecent: options.preferRecent !== false // 优先保留最近访问的数据
    };
    
    // 数据访问跟踪 (用于LRU淘汰)
    this.accessLog = new Map(); // id -> lastAccessTime
  }

  /**
   * 初始化
   */
  async init() {
    await this.store.init();
    this._startMonitoring();
  }

  /**
   * 启动内存监控
   */
  _startMonitoring() {
    if (this.monitorInterval) return;
    
    this.monitorInterval = setInterval(() => {
      this._checkMemory();
    }, this.monitorPeriodMs);
  }

  /**
   * 停止内存监控
   */
  stopMonitoring() {
    if (this.monitorInterval) {
      clearInterval(this.monitorInterval);
      this.monitorInterval = null;
    }
  }

  /**
   * 检查内存使用情况
   */
  _checkMemory() {
    const usage = process.memoryUsage();
    const rssMB = usage.rss / 1024 / 1024;
    const heapMB = usage.heapUsed / 1024 / 1024;
    
    this.emit('memory:check', { rssMB: rssMB.toFixed(2), heapMB: heapMB.toFixed(2) });
    
    if (rssMB > this.thresholds.emergency) {
      this.emit('memory:emergency', { rssMB });
      this._handleEmergency();
    } else if (rssMB > this.thresholds.critical && !this.state.isOverflowing) {
      this.emit('memory:critical', { rssMB });
      this._startOverflow();
    } else if (rssMB > this.thresholds.warning) {
      this.emit('memory:warning', { rssMB });
    }
    
    return { rssMB, heapMB };
  }

  /**
   * 处理紧急情况
   */
  _handleEmergency() {
    // 强制同步，释放缓存
    this.store.sync();
    
    // 尝试触发GC (如果可用)
    if (global.gc) {
      global.gc();
    }
  }

  /**
   * 开始溢出处理
   */
  async _startOverflow() {
    if (this.state.isOverflowing) return;
    
    this.state.isOverflowing = true;
    this.state.overflowStartTime = Date.now();
    this.emit('overflow:start');
    
    try {
      await this._evictBatch();
    } finally {
      this.state.isOverflowing = false;
      this.emit('overflow:end', { 
        duration: Date.now() - this.state.overflowStartTime 
      });
    }
  }

  /**
   * 淘汰一批数据到磁盘
   */
  async _evictBatch() {
    // 按访问时间排序，淘汰最久未访问的
    const sorted = Array.from(this.accessLog.entries())
      .sort((a, b) => a[1] - b[1]);
    
    const toEvict = sorted.slice(0, this.policy.evictionBatchSize);
    
    for (const [id, lastAccess] of toEvict) {
      this.emit('overflow:evict', { id, lastAccess });
      this.state.overflowedCount++;
      this.accessLog.delete(id);
    }
    
    return toEvict.length;
  }

  /**
   * 记录数据访问
   */
  touch(id) {
    this.accessLog.set(id, Date.now());
  }

  /**
   * 添加数据 (可能触发溢出)
   */
  async add(id, data) {
    this.touch(id);
    this.state.totalCount++;
    this.state.inMemoryCount++;
    
    // 检查是否需要立即溢出
    const usage = process.memoryUsage();
    if (usage.rss / 1024 / 1024 > this.thresholds.critical) {
      await this._startOverflow();
    }
    
    return { id, inMemory: true };
  }

  /**
   * 溢出特定数据到磁盘
   */
  async overflowToDisk(id, serializer) {
    const fileId = `overflow-${Math.floor(id / 1000)}`;
    const data = await serializer(id);
    
    if (data) {
      const offset = await this.store.getSize(fileId);
      await this.store.write(fileId, offset, data);
      this.state.overflowedCount++;
      this.state.inMemoryCount--;
      return { fileId, offset, size: data.length };
    }
    
    return null;
  }

  /**
   * 从磁盘读取数据
   */
  async readFromDisk(fileId, offset, size) {
    return await this.store.read(fileId, offset, size);
  }

  /**
   * 获取溢出统计
   */
  getStats() {
    const memory = process.memoryUsage();
    return {
      thresholds: this.thresholds,
      state: { ...this.state },
      memory: {
        rssMB: (memory.rss / 1024 / 1024).toFixed(2),
        heapMB: (memory.heapUsed / 1024 / 1024).toFixed(2),
        externalMB: (memory.external / 1024 / 1024).toFixed(2)
      },
      store: this.store.getStats(),
      policy: this.policy
    };
  }

  /**
   * 检查是否处于溢出状态
   */
  isOverflowing() {
    return this.state.isOverflowing;
  }

  /**
   * 关闭并清理
   */
  async close() {
    this.stopMonitoring();
    await this.store.closeAll();
  }
}

module.exports = { OverflowManager };
