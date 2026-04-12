/**
 * 紧急模式管理
 * 当磁盘满时切换到纯内存运行模式
 */

const EventEmitter = require('events');

class EmergencyMode extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.maxMemoryItems = options.maxMemoryItems || 10000;
    this.maxMemoryMB = options.maxMemoryMB || 200;
    
    this.state = {
      isActive: false,
      activatedAt: null,
      deactivatedAt: null,
      reason: null,
      itemCount: 0,
      droppedCount: 0
    };
    
    // 内存存储
    this.memoryStore = new Map();
    
    // 统计
    this.stats = {
      reads: 0,
      writes: 0,
      hits: 0,
      misses: 0
    };
  }

  /**
   * 激活紧急模式
   */
  activate(reason = 'disk_full') {
    if (this.state.isActive) return;
    
    console.error('🚨 EMERGENCY MODE ACTIVATED');
    console.error(`   Reason: ${reason}`);
    console.error('   Operating in MEMORY-ONLY mode');
    
    this.state.isActive = true;
    this.state.activatedAt = Date.now();
    this.state.reason = reason;
    
    this.emit('activated', { 
      timestamp: this.state.activatedAt,
      reason 
    });
  }

  /**
   * 停用紧急模式
   */
  deactivate() {
    if (!this.state.isActive) return;
    
    console.log('✅ EMERGENCY MODE DEACTIVATED');
    console.log(`   Duration: ${(Date.now() - this.state.activatedAt) / 1000}s`);
    console.log(`   Items in memory: ${this.state.itemCount}`);
    
    this.state.isActive = false;
    this.state.deactivatedAt = Date.now();
    
    this.emit('deactivated', {
      timestamp: this.state.deactivatedAt,
      duration: this.state.deactivatedAt - this.state.activatedAt,
      itemCount: this.state.itemCount
    });
    
    // 可以选择清空内存或保留
    // this.clear();
  }

  /**
   * 检查是否在紧急模式
   */
  isActive() {
    return this.state.isActive;
  }

  /**
   * 写入数据（仅内存）
   */
  async write(key, data) {
    if (!this.state.isActive) {
      throw new Error('Emergency mode not active, use normal storage');
    }
    
    this.stats.writes++;
    
    // 检查内存限制
    const memUsage = process.memoryUsage();
    const rssMB = memUsage.rss / 1024 / 1024;
    
    if (rssMB > this.maxMemoryMB) {
      // 内存超限，移除最旧的数据
      this._evictOldest();
    }
    
    if (this.memoryStore.size >= this.maxMemoryItems) {
      this._evictOldest();
    }
    
    const item = {
      key,
      data,
      timestamp: Date.now(),
      accessCount: 0
    };
    
    this.memoryStore.set(key, item);
    this.state.itemCount = this.memoryStore.size;
    
    this.emit('write', { key, size: data.length });
    
    return { memoryOnly: true, key };
  }

  /**
   * 读取数据
   */
  async read(key) {
    this.stats.reads++;
    
    const item = this.memoryStore.get(key);
    
    if (item) {
      item.accessCount++;
      item.lastAccess = Date.now();
      this.stats.hits++;
      
      this.emit('hit', { key });
      return item.data;
    }
    
    this.stats.misses++;
    this.emit('miss', { key });
    return null;
  }

  /**
   * 移除最旧的数据
   */
  _evictOldest() {
    let oldestKey = null;
    let oldestTime = Infinity;
    
    for (const [key, item] of this.memoryStore) {
      if (item.timestamp < oldestTime) {
        oldestTime = item.timestamp;
        oldestKey = key;
      }
    }
    
    if (oldestKey) {
      this.memoryStore.delete(oldestKey);
      this.state.droppedCount++;
      this.emit('evict', { key: oldestKey });
    }
  }

  /**
   * 删除数据
   */
  async delete(key) {
    const existed = this.memoryStore.has(key);
    this.memoryStore.delete(key);
    this.state.itemCount = this.memoryStore.size;
    
    return existed;
  }

  /**
   * 清空内存存储
   */
  clear() {
    const count = this.memoryStore.size;
    this.memoryStore.clear();
    this.state.itemCount = 0;
    
    console.log(`🗑️ Cleared ${count} items from emergency memory store`);
    this.emit('clear', { count });
  }

  /**
   * 获取统计
   */
  getStats() {
    const hitRate = this.stats.reads > 0 
      ? (this.stats.hits / this.stats.reads * 100).toFixed(2)
      : 0;
    
    return {
      ...this.state,
      memorySize: this.memoryStore.size,
      stats: { ...this.stats, hitRate: `${hitRate}%` },
      memoryUsageMB: (process.memoryUsage().rss / 1024 / 1024).toFixed(2)
    };
  }

  /**
   * 获取健康状态（用于API返回）
   */
  getHealthStatus() {
    return {
      emergency: this.state.isActive,
      mode: this.state.isActive ? 'memory_only' : 'normal',
      activatedAt: this.state.activatedAt,
      reason: this.state.reason,
      itemsInMemory: this.state.itemCount,
      memoryUsageMB: (process.memoryUsage().rss / 1024 / 1024).toFixed(2)
    };
  }

  /**
   * 导出所有数据（用于恢复后持久化）
   */
  exportData() {
    return Array.from(this.memoryStore.entries()).map(([key, item]) => ({
      key,
      data: item.data,
      timestamp: item.timestamp
    }));
  }

  /**
   * 导入数据
   */
  importData(items) {
    for (const { key, data, timestamp } of items) {
      this.memoryStore.set(key, {
        key,
        data,
        timestamp: timestamp || Date.now(),
        accessCount: 0
      });
    }
    
    this.state.itemCount = this.memoryStore.size;
    console.log(`📥 Imported ${items.length} items into emergency store`);
  }
}

module.exports = { EmergencyMode };
