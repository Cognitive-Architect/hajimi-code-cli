/**
 * 懒加载模块 - Lazy Loader
 * 
 * 职责：
 * 1. 按需加载 HNSW 索引分片
 * 2. 分片内存管理（加载/卸载）
 * 3. 分片预加载策略
 * 4. 与现有 16 分片存储集成
 */

const fs = require('fs').promises;
const path = require('path');
const { HNSWIndex } = require('./hnsw-core');

// 懒加载配置
const LAZY_CONFIG = {
  maxLoadedShards: 4,        // 最多同时加载4个分片
  preloadThreshold: 2,       // 预加载阈值
  unloadDelay: 60000,        // 卸载延迟（1分钟）
  persistencePath: null      // 索引持久化路径
};

/**
 * 分片元数据
 */
class ShardMetadata {
  constructor(shardId) {
    this.shardId = shardId;
    this.loaded = false;
    this.lastAccessed = 0;
    this.accessCount = 0;
    this.vectorCount = 0;
    this.index = null;
    this.filePath = null;
  }
}

/**
 * HNSW 分片懒加载器
 */
class LazyShardLoader {
  /**
   * @param {Object} options
   * @param {number} options.totalShards - 总分片数（默认16）
   * @param {string} options.persistencePath - 索引存储路径
   * @param {Function} options.indexFactory - 创建空索引的工厂函数
   */
  constructor(options = {}) {
    this.config = { ...LAZY_CONFIG, ...options };
    this.totalShards = options.totalShards || 16;
    this.persistencePath = options.persistencePath;
    this.indexFactory = options.indexFactory || (() => new HNSWIndex());
    
    // 分片元数据
    this.shards = new Map();  // shardId -> ShardMetadata
    for (let i = 0; i < this.totalShards; i++) {
      this.shards.set(i, new ShardMetadata(i));
    }
    
    // 加载队列
    this.loadedShards = new Set();  // 当前已加载的分片ID
    this.accessQueue = [];          // 访问顺序
    
    // 统计
    this.stats = {
      totalLoads: 0,
      totalUnloads: 0,
      cacheHits: 0,
      cacheMisses: 0,
      preloadHits: 0
    };
    
    // 启动后台清理
    this._startCleanupTimer();
  }
  
  /**
   * 获取分片索引（懒加载）
   * @param {number} shardId 
   * @returns {Promise<HNSWIndex|null>}
   */
  async getShard(shardId) {
    if (shardId < 0 || shardId >= this.totalShards) {
      throw new Error(`Invalid shardId: ${shardId}`);
    }
    
    const meta = this.shards.get(shardId);
    
    // 已加载，直接返回
    if (meta.loaded && meta.index) {
      this._updateAccess(shardId);
      this.stats.cacheHits++;
      return meta.index;
    }
    
    // 需要加载
    this.stats.cacheMisses++;
    await this._loadShard(shardId);
    return meta.index;
  }
  
  /**
   * 加载分片
   */
  async _loadShard(shardId) {
    const meta = this.shards.get(shardId);
    
    // 检查是否需要卸载其他分片
    while (this.loadedShards.size >= this.config.maxLoadedShards) {
      await this._unloadLRUShard();
    }
    
    console.log(`[LazyLoader] Loading shard ${shardId}...`);
    
    // 创建新索引
    meta.index = this.indexFactory();
    
    // 尝试从磁盘恢复
    if (this.persistencePath) {
      const loaded = await this._loadFromDisk(shardId);
      if (!loaded) {
        console.log(`[LazyLoader] Shard ${shardId}: no persisted data`);
      }
    }
    
    meta.loaded = true;
    meta.lastAccessed = Date.now();
    meta.vectorCount = meta.index.elementCount;
    
    this.loadedShards.add(shardId);
    this._updateAccess(shardId);
    this.stats.totalLoads++;
    
    console.log(`[LazyLoader] Shard ${shardId} loaded (${meta.vectorCount} vectors)`);
    
    // 触发预加载
    this._preloadNeighbors(shardId);
  }
  
  /**
   * 从磁盘加载
   */
  async _loadFromDisk(shardId) {
    if (!this.persistencePath) return false;
    
    const filePath = path.join(this.persistencePath, `hnsw-shard-${shardId}.json`);
    
    try {
      const data = await fs.readFile(filePath, 'utf8');
      const json = JSON.parse(data);
      const meta = this.shards.get(shardId);
      meta.index = HNSWIndex.fromJSON(json);
      return true;
    } catch (err) {
      if (err.code !== 'ENOENT') {
        console.warn(`[LazyLoader] Failed to load shard ${shardId}:`, err.message);
      }
      return false;
    }
  }
  
  /**
   * 保存到磁盘
   */
  async _saveToDisk(shardId) {
    if (!this.persistencePath) return false;
    
    const meta = this.shards.get(shardId);
    if (!meta.loaded || !meta.index) return false;
    
    const filePath = path.join(this.persistencePath, `hnsw-shard-${shardId}.json`);
    
    try {
      await fs.mkdir(this.persistencePath, { recursive: true });
      const json = meta.index.toJSON();
      await fs.writeFile(filePath, JSON.stringify(json));
      return true;
    } catch (err) {
      console.error(`[LazyLoader] Failed to save shard ${shardId}:`, err.message);
      return false;
    }
  }
  
  /**
   * 卸载分片
   */
  async _unloadShard(shardId, save = true) {
    const meta = this.shards.get(shardId);
    if (!meta.loaded) return;
    
    console.log(`[LazyLoader] Unloading shard ${shardId}...`);
    
    // 保存到磁盘
    if (save) {
      await this._saveToDisk(shardId);
    }
    
    // 释放内存
    if (meta.index) {
      meta.index.clear();
      meta.index = null;
    }
    
    meta.loaded = false;
    meta.index = null;
    this.loadedShards.delete(shardId);
    
    // 从访问队列移除
    const idx = this.accessQueue.indexOf(shardId);
    if (idx > -1) {
      this.accessQueue.splice(idx, 1);
    }
    
    this.stats.totalUnloads++;
    
    console.log(`[LazyLoader] Shard ${shardId} unloaded`);
  }
  
  /**
   * 卸载最久未使用的分片
   */
  async _unloadLRUShard() {
    if (this.accessQueue.length === 0) return;
    
    // 找到最久未使用的
    const shardId = this.accessQueue[0];
    await this._unloadShard(shardId);
  }
  
  /**
   * 更新访问记录
   */
  _updateAccess(shardId) {
    const meta = this.shards.get(shardId);
    meta.lastAccessed = Date.now();
    meta.accessCount++;
    
    // 更新访问队列
    const idx = this.accessQueue.indexOf(shardId);
    if (idx > -1) {
      this.accessQueue.splice(idx, 1);
    }
    this.accessQueue.push(shardId);
  }
  
  /**
   * 预加载相邻分片
   */
  async _preloadNeighbors(shardId) {
    if (this.loadedShards.size >= this.config.maxLoadedShards) return;
    
    // 简单策略：预加载下一个分片
    const nextShard = (shardId + 1) % this.totalShards;
    const nextMeta = this.shards.get(nextShard);
    
    if (!nextMeta.loaded && this.loadedShards.size < this.config.maxLoadedShards) {
      // 延迟预加载，不阻塞当前操作
      setTimeout(() => {
        this._loadShard(nextShard).catch(() => {});
      }, 100);
      
      this.stats.preloadHits++;
    }
  }
  
  /**
   * 启动后台清理定时器
   */
  _startCleanupTimer() {
    this.cleanupTimer = setInterval(() => {
      this._cleanupIdleShards();
    }, this.config.unloadDelay);
    
    if (this.cleanupTimer.unref) {
      this.cleanupTimer.unref();
    }
  }
  
  /**
   * 清理空闲分片
   */
  async _cleanupIdleShards() {
    const now = Date.now();
    const toUnload = [];
    
    for (const shardId of this.loadedShards) {
      const meta = this.shards.get(shardId);
      if (now - meta.lastAccessed > this.config.unloadDelay) {
        toUnload.push(shardId);
      }
    }
    
    for (const shardId of toUnload) {
      await this._unloadShard(shardId);
    }
    
    if (toUnload.length > 0) {
      console.log(`[LazyLoader] Cleaned up ${toUnload.length} idle shards`);
    }
  }
  
  /**
   * 保存所有分片
   */
  async saveAll() {
    const promises = [];
    for (const shardId of this.loadedShards) {
      promises.push(this._saveToDisk(shardId));
    }
    await Promise.all(promises);
  }
  
  /**
   * 加载所有分片
   */
  async loadAll() {
    for (let i = 0; i < this.totalShards; i++) {
      await this.getShard(i);
    }
  }
  
  /**
   * 卸载所有分片
   */
  async unloadAll() {
    const promises = [];
    for (const shardId of Array.from(this.loadedShards)) {
      promises.push(this._unloadShard(shardId));
    }
    await Promise.all(promises);
  }
  
  /**
   * 获取统计
   */
  getStats() {
    return {
      ...this.stats,
      loadedShards: this.loadedShards.size,
      totalShards: this.totalShards,
      cacheHitRate: this._calculateHitRate()
    };
  }
  
  /**
   * 计算命中率
   */
  _calculateHitRate() {
    const total = this.stats.cacheHits + this.stats.cacheMisses;
    if (total === 0) return '0%';
    return ((this.stats.cacheHits / total) * 100).toFixed(2) + '%';
  }
  
  /**
   * 销毁
   */
  async destroy() {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = null;
    }
    
    await this.saveAll();
    await this.unloadAll();
  }
}

module.exports = {
  LazyShardLoader,
  ShardMetadata,
  LAZY_CONFIG
};
