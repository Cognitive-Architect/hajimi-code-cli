/**
 * HNSW 内存管理器 - Memory Manager
 * 
 * 职责：
 * 1. 内存池管理（预分配 TypedArray）
 * 2. LRU-like 缓存策略
 * 3. 内存压力自动释放
 * 4. 内存使用监控与告警
 * 
 * 约束：Termux 环境 <500MB 硬限制
 */

const EventEmitter = require('events');

// 内存配置
const MEMORY_CONFIG = {
  maxMemoryMB: 400,           // 最大内存限制
  warningThresholdMB: 350,    // 警告阈值
  criticalThresholdMB: 420,   // 紧急释放阈值
  
  // LRU 配置
  lruMaxSize: 10000,          // 最大缓存条目
  lruTTL: 5 * 60 * 1000,      // 条目过期时间 5分钟
  
  // 对象池配置
  poolInitialSize: 1000,      // 初始池大小
  poolMaxSize: 10000,         // 最大池大小
  vectorDim: 128              // 向量维度
};

/**
 * 内存池 - 预分配 TypedArray 减少 GC
 */
class VectorPool {
  constructor(dim = 128, initialSize = 1000, maxSize = 10000) {
    this.dim = dim;
    this.maxSize = maxSize;
    this.available = [];
    this.inUse = new Set();
    
    // 预分配
    for (let i = 0; i < initialSize; i++) {
      this.available.push(new Float32Array(dim));
    }
    
    this.stats = {
      totalAllocated: initialSize,
      totalAcquired: 0,
      totalReleased: 0,
      poolHits: 0,
      poolMisses: 0
    };
  }
  
  /**
   * 获取向量
   */
  acquire() {
    this.stats.totalAcquired++;
    
    if (this.available.length > 0) {
      this.stats.poolHits++;
      const vec = this.available.pop();
      this.inUse.add(vec);
      return vec;
    }
    
    // 池耗尽，创建新的
    this.stats.poolMisses++;
    if (this.stats.totalAllocated < this.maxSize) {
      this.stats.totalAllocated++;
      const vec = new Float32Array(this.dim);
      this.inUse.add(vec);
      return vec;
    }
    
    // 超过最大限制，抛出错误
    throw new Error('VectorPool exhausted: max size reached');
  }
  
  /**
   * 释放向量回池
   */
  release(vec) {
    if (!this.inUse.has(vec)) return false;
    
    this.inUse.delete(vec);
    this.stats.totalReleased++;
    
    // 清零并回收
    vec.fill(0);
    if (this.available.length < this.maxSize) {
      this.available.push(vec);
    }
    
    return true;
  }
  
  /**
   * 批量释放
   */
  releaseBatch(vecs) {
    for (const vec of vecs) {
      this.release(vec);
    }
  }
  
  /**
   * 清空池
   */
  clear() {
    this.available = [];
    this.inUse.clear();
    this.stats.totalAllocated = 0;
  }
  
  /**
   * 获取统计
   */
  getStats() {
    return {
      ...this.stats,
      available: this.available.length,
      inUse: this.inUse.size,
      hitRate: this.stats.totalAcquired > 0 
        ? (this.stats.poolHits / this.stats.totalAcquired * 100).toFixed(2) + '%'
        : '0%'
    };
  }
}

/**
 * LRU Cache - 带TTL的最近最少使用缓存
 */
class LRUCache {
  constructor(maxSize = 10000, ttl = 5 * 60 * 1000) {
    this.maxSize = maxSize;
    this.ttl = ttl;
    this.cache = new Map();  // key -> { value, timestamp, size }
    this.accessOrder = [];   // 访问顺序（最近在最后）
    this.currentSize = 0;
    
    this.stats = {
      hits: 0,
      misses: 0,
      evictions: 0,
      expirations: 0
    };
    
    // 启动过期清理定时器
    this._startCleanupTimer();
  }
  
  /**
   * 获取值
   */
  get(key) {
    const entry = this.cache.get(key);
    
    if (!entry) {
      this.stats.misses++;
      return undefined;
    }
    
    // 检查是否过期
    if (Date.now() - entry.timestamp > this.ttl) {
      this.delete(key);
      this.stats.expirations++;
      this.stats.misses++;
      return undefined;
    }
    
    // 更新访问顺序
    this._updateAccessOrder(key);
    entry.timestamp = Date.now();
    
    this.stats.hits++;
    return entry.value;
  }
  
  /**
   * 设置值
   */
  set(key, value, size = 1) {
    // 如果已存在，先删除旧值
    if (this.cache.has(key)) {
      this.delete(key);
    }
    
    // 如果超出容量，驱逐最旧的
    while (this.currentSize + size > this.maxSize && this.accessOrder.length > 0) {
      this._evictLRU();
    }
    
    this.cache.set(key, {
      value,
      timestamp: Date.now(),
      size
    });
    
    this.accessOrder.push(key);
    this.currentSize += size;
    
    return true;
  }
  
  /**
   * 删除
   */
  delete(key) {
    const entry = this.cache.get(key);
    if (!entry) return false;
    
    this.cache.delete(key);
    const index = this.accessOrder.indexOf(key);
    if (index > -1) {
      this.accessOrder.splice(index, 1);
    }
    this.currentSize -= entry.size;
    
    return true;
  }
  
  /**
   * 更新访问顺序
   */
  _updateAccessOrder(key) {
    const index = this.accessOrder.indexOf(key);
    if (index > -1) {
      this.accessOrder.splice(index, 1);
      this.accessOrder.push(key);
    }
  }
  
  /**
   * 驱逐最久未使用
   */
  _evictLRU() {
    if (this.accessOrder.length === 0) return;
    
    const oldestKey = this.accessOrder[0];
    this.delete(oldestKey);
    this.stats.evictions++;
  }
  
  /**
   * 启动清理定时器
   */
  _startCleanupTimer() {
    // 每30秒清理一次过期条目
    this.cleanupTimer = setInterval(() => {
      this._cleanupExpired();
    }, 30000);
    
    // Node.js 环境下确保定时器不阻止退出
    if (this.cleanupTimer.unref) {
      this.cleanupTimer.unref();
    }
  }
  
  /**
   * 清理过期条目
   */
  _cleanupExpired() {
    const now = Date.now();
    const keysToDelete = [];
    
    for (const [key, entry] of this.cache) {
      if (now - entry.timestamp > this.ttl) {
        keysToDelete.push(key);
      }
    }
    
    for (const key of keysToDelete) {
      this.delete(key);
      this.stats.expirations++;
    }
    
    if (keysToDelete.length > 0) {
      console.log(`[LRU] Cleaned up ${keysToDelete.length} expired entries`);
    }
  }
  
  /**
   * 清空缓存
   */
  clear() {
    this.cache.clear();
    this.accessOrder = [];
    this.currentSize = 0;
  }
  
  /**
   * 获取统计
   */
  getStats() {
    return {
      ...this.stats,
      size: this.cache.size,
      currentSize: this.currentSize,
      maxSize: this.maxSize,
      hitRate: (this.stats.hits + this.stats.misses) > 0
        ? (this.stats.hits / (this.stats.hits + this.stats.misses) * 100).toFixed(2) + '%'
        : '0%'
    };
  }
  
  /**
   * 停止定时器
   */
  destroy() {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = null;
    }
  }
}

/**
 * 内存管理器主类
 */
class MemoryManager extends EventEmitter {
  constructor(config = {}) {
    super();
    this.config = { ...MEMORY_CONFIG, ...config };
    
    // 向量池
    this.vectorPool = new VectorPool(
      this.config.vectorDim,
      this.config.poolInitialSize,
      this.config.poolMaxSize
    );
    
    // LRU缓存
    this.cache = new LRUCache(this.config.lruMaxSize, this.config.lruTTL);
    
    // 内存监控
    this.monitoring = false;
    this.monitorInterval = null;
    
    // 压力级别
    this.pressureLevel = 'normal'; // 'normal' | 'warning' | 'critical'
  }
  
  /**
   * 获取当前内存使用（MB）
   */
  getMemoryUsageMB() {
    if (typeof process !== 'undefined' && process.memoryUsage) {
      return process.memoryUsage().rss / 1024 / 1024;
    }
    return 0;
  }
  
  /**
   * 检查内存压力
   */
  checkPressure() {
    const memMB = this.getMemoryUsageMB();
    let newLevel = 'normal';
    
    if (memMB > this.config.criticalThresholdMB) {
      newLevel = 'critical';
    } else if (memMB > this.config.warningThresholdMB) {
      newLevel = 'warning';
    }
    
    if (newLevel !== this.pressureLevel) {
      this.pressureLevel = newLevel;
      this.emit('pressureChange', newLevel, memMB);
      
      if (newLevel === 'critical') {
        this._handleCriticalPressure();
      } else if (newLevel === 'warning') {
        this._handleWarningPressure();
      }
    }
    
    return {
      level: this.pressureLevel,
      memoryMB: memMB,
      maxMB: this.config.maxMemoryMB
    };
  }
  
  /**
   * 处理警告级别压力
   */
  _handleWarningPressure() {
    console.warn(`[MemoryManager] Warning: memory usage high (${this.getMemoryUsageMB().toFixed(1)}MB)`);
    
    // 清理过期缓存条目
    this.cache._cleanupExpired();
    
    // 尝试触发 GC
    if (global.gc) {
      global.gc();
    }
    
    this.emit('memoryWarning');
  }
  
  /**
   * 处理危急级别压力
   */
  _handleCriticalPressure() {
    console.error(`[MemoryManager] CRITICAL: memory usage critical (${this.getMemoryUsageMB().toFixed(1)}MB)`);
    
    // 清空缓存
    this.cache.clear();
    
    // 清空部分向量池
    const poolStats = this.vectorPool.getStats();
    if (poolStats.available > poolStats.inUse) {
      // 只保留使用中的向量
      this.vectorPool.clear();
    }
    
    // 强制 GC
    if (global.gc) {
      global.gc();
    }
    
    this.emit('memoryCritical');
  }
  
  /**
   * 启动内存监控
   */
  startMonitoring(intervalMs = 5000) {
    if (this.monitoring) return;
    
    this.monitoring = true;
    this.monitorInterval = setInterval(() => {
      this.checkPressure();
    }, intervalMs);
    
    console.log(`[MemoryManager] Monitoring started (${intervalMs}ms interval)`);
  }
  
  /**
   * 停止内存监控
   */
  stopMonitoring() {
    if (!this.monitoring) return;
    
    this.monitoring = false;
    if (this.monitorInterval) {
      clearInterval(this.monitorInterval);
      this.monitorInterval = null;
    }
    
    console.log('[MemoryManager] Monitoring stopped');
  }
  
  /**
   * 获取综合统计
   */
  getStats() {
    return {
      memory: {
        currentMB: this.getMemoryUsageMB(),
        maxMB: this.config.maxMemoryMB,
        pressureLevel: this.pressureLevel
      },
      pool: this.vectorPool.getStats(),
      cache: this.cache.getStats()
    };
  }
  
  /**
   * 手动释放内存
   */
  releaseMemory() {
    this.cache.clear();
    
    // 保留一部分池
    const stats = this.vectorPool.getStats();
    if (stats.available > 1000) {
      this.vectorPool.clear();
    }
    
    if (global.gc) {
      global.gc();
    }
    
    console.log('[MemoryManager] Memory manually released');
  }
  
  /**
   * 销毁
   */
  destroy() {
    this.stopMonitoring();
    this.cache.destroy();
    this.vectorPool.clear();
    this.removeAllListeners();
  }
}

module.exports = {
  MemoryManager,
  VectorPool,
  LRUCache,
  MEMORY_CONFIG
};
