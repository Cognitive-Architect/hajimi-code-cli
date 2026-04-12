/**
 * 磁盘块缓存 - LRU策略
 * Block Cache with LRU eviction policy
 * 
 * 目标: 缓存热点数据块，减少磁盘I/O
 */

const EventEmitter = require('events');

class BlockCache extends EventEmitter {
  constructor(options = {}) {
    super();
    this.blockSize = options.blockSize || 4096; // 4KB blocks
    this.maxBlocks = options.maxBlocks || 500;  // ~2MB cache
    this.currentBlocks = 0;
    
    // LRU cache: Map maintains insertion order
    this.cache = new Map(); // blockKey -> { data, lastAccess, hitCount }
    
    this.stats = {
      hits: 0,
      misses: 0,
      evictions: 0,
      totalRequests: 0
    };
  }

  /**
   * 生成缓存键
   */
  _makeKey(fileId, blockIndex) {
    return `${fileId}:${blockIndex}`;
  }

  /**
   * 获取块缓存
   */
  get(fileId, blockIndex) {
    const key = this._makeKey(fileId, blockIndex);
    this.stats.totalRequests++;
    
    const entry = this.cache.get(key);
    if (entry) {
      // Move to end (most recently used)
      this.cache.delete(key);
      this.cache.set(key, entry);
      entry.lastAccess = Date.now();
      entry.hitCount++;
      this.stats.hits++;
      return entry.data;
    }
    
    this.stats.misses++;
    return null;
  }

  /**
   * 设置块缓存
   */
  set(fileId, blockIndex, data) {
    const key = this._makeKey(fileId, blockIndex);
    
    // If already exists, update and move to end
    if (this.cache.has(key)) {
      this.cache.delete(key);
      this.cache.set(key, {
        data,
        lastAccess: Date.now(),
        hitCount: this.cache.get(key)?.hitCount || 0
      });
      return;
    }
    
    // Evict if necessary (LRU: remove oldest)
    while (this.currentBlocks >= this.maxBlocks && this.cache.size > 0) {
      const oldestKey = this.cache.keys().next().value;
      this.cache.delete(oldestKey);
      this.currentBlocks--;
      this.stats.evictions++;
    }
    
    // Add new entry
    this.cache.set(key, {
      data,
      lastAccess: Date.now(),
      hitCount: 0
    });
    this.currentBlocks++;
  }

  /**
   * 使缓存失效
   */
  invalidate(fileId, blockIndex) {
    const key = this._makeKey(fileId, blockIndex);
    if (this.cache.has(key)) {
      this.cache.delete(key);
      this.currentBlocks--;
    }
  }

  /**
   * 使整个文件的缓存失效
   */
  invalidateFile(fileId) {
    const prefix = `${fileId}:`;
    for (const key of this.cache.keys()) {
      if (key.startsWith(prefix)) {
        this.cache.delete(key);
        this.currentBlocks--;
      }
    }
  }

  /**
   * 清空缓存
   */
  clear() {
    this.cache.clear();
    this.currentBlocks = 0;
    this.stats = {
      hits: 0,
      misses: 0,
      evictions: 0,
      totalRequests: 0
    };
  }

  /**
   * 获取缓存统计
   */
  getStats() {
    const hitRate = this.stats.totalRequests > 0
      ? (this.stats.hits / this.stats.totalRequests * 100).toFixed(2)
      : 0;
    
    return {
      ...this.stats,
      hitRate: `${hitRate}%`,
      size: this.cache.size,
      maxBlocks: this.maxBlocks,
      blockSize: this.blockSize,
      memoryUsageMB: (this.cache.size * this.blockSize / 1024 / 1024).toFixed(2)
    };
  }

  /**
   * 预加载块到缓存
   */
  async preload(fileId, blockIndices, loader) {
    const promises = blockIndices.map(async (idx) => {
      const key = this._makeKey(fileId, idx);
      if (!this.cache.has(key)) {
        try {
          const data = await loader(idx);
          if (data) {
            this.set(fileId, idx, data);
          }
        } catch (err) {
          this.emit('error', { fileId, blockIndex: idx, error: err });
        }
      }
    });
    
    await Promise.all(promises);
  }
}

module.exports = { BlockCache };
