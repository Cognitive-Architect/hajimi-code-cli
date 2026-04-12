/**
 * 向量检索 API - Vector API
 * 
 * 提供面向应用的向量检索接口：
 * - putVector: 存储向量
 * - getVector: 获取向量
 * - searchVector: 相似度搜索
 * - batchImport: 批量导入
 * 
 * 底层使用 HybridRetriever（HNSW + LSH fallback）
 */

const { HybridRetriever } = require('../vector/hybrid-retriever');
const { HNSWPersistence } = require('../vector/hnsw-persistence');
const { MemoryManager } = require('../vector/hnsw-memory-manager');

/**
 * VectorAPI 主类
 */
class VectorAPI {
  /**
   * @param {Object} options
   * @param {string} options.basePath - 数据存储路径
   * @param {string} options.shardId - 分片ID（默认'default'）
   * @param {Object} options.hnswConfig - HNSW 配置
   * @param {Object} options.encoderConfig - 编码器配置
   */
  constructor(options = {}) {
    this.options = {
      basePath: options.basePath || require('path').join(process.env.HOME || '.', '.hajimi/storage/v3'),
      shardId: options.shardId || 'default',
      ...options
    };
    
    // 混合检索器
    this.retriever = new HybridRetriever({
      encoderMethod: options.encoderConfig?.method || 'hadamard',
      encoderOutputDim: options.encoderConfig?.outputDim || 128,
      ...options.hnswConfig
    });
    
    // 持久化
    this.persistence = new HNSWPersistence({
      basePath: require('path').join(this.options.basePath, 'hnsw'),
      shardId: this.options.shardId,
      config: { walEnabled: true }
    });
    
    // 内存管理
    this.memoryManager = new MemoryManager();
    
    // 状态
    this.initialized = false;
    this.stats = {
      totalPuts: 0,
      totalGets: 0,
      totalSearches: 0,
      startTime: Date.now()
    };
  }
  
  /**
   * 初始化（加载已有索引）
   */
  async init() {
    if (this.initialized) return;
    
    console.log(`[VectorAPI] Initializing shard ${this.options.shardId}...`);
    
    // 尝试加载已有索引
    const result = await this.persistence.load();
    if (result) {
      console.log(`[VectorAPI] Loaded existing index (${result.metadata.vectorCount || 0} vectors)`);
      this.retriever.hnsw = result.index;
    } else {
      console.log('[VectorAPI] Starting with empty index');
    }
    
    // 启动内存监控
    this.memoryManager.startMonitoring();
    
    this.initialized = true;
    console.log('[VectorAPI] Initialized');
  }
  
  /**
   * 存储向量
   * @param {bigint} simhash - SimHash-64
   * @param {Object} metadata - 关联元数据
   * @returns {Promise<number>} - 文档ID
   * 
   * @example
   * const id = await api.putVector(0x1234567890abcdefn, { 
   *   text: 'document content',
   *   tags: ['tag1'] 
   * });
   */
  async putVector(simhash, metadata = {}) {
    if (!this.initialized) await this.init();
    
    if (typeof simhash !== 'bigint') {
      throw new TypeError('simhash must be bigint');
    }
    
    const id = this.retriever.add(simhash, metadata);
    
    // 记录 WAL
    const vector = this.retriever.encoder.encode(simhash);
    await this.persistence.logInsert(id, vector);
    
    this.stats.totalPuts++;
    
    return id;
  }
  
  /**
   * 批量存储向量
   * @param {Array<{simhash: bigint, metadata: Object}>} items 
   * @param {Function} progressCallback - (current, total) => void
   * @returns {Promise<Array<number>>} - 文档ID列表
   * 
   * @example
   * const ids = await api.putVectors([
   *   { simhash: 0x1234n, metadata: { text: 'doc1' } },
   *   { simhash: 0x5678n, metadata: { text: 'doc2' } }
   * ], (c, t) => console.log(`${c}/${t}`));
   */
  async putVectors(items, progressCallback = null) {
    if (!this.initialized) await this.init();
    
    const ids = [];
    const total = items.length;
    
    for (let i = 0; i < items.length; i++) {
      const { simhash, metadata } = items[i];
      const id = await this.putVector(simhash, metadata);
      ids.push(id);
      
      if (progressCallback && (i + 1) % 100 === 0) {
        progressCallback(i + 1, total);
      }
    }
    
    if (progressCallback) {
      progressCallback(total, total);
    }
    
    return ids;
  }
  
  /**
   * 获取向量
   * @param {number} id - 文档ID
   * @returns {Object|null} - { id, simhash, metadata }
   * 
   * @example
   * const doc = await api.getVector(123);
   * if (doc) {
   *   console.log(doc.simhash.toString(16));
   *   console.log(doc.metadata.text);
   * }
   */
  getVector(id) {
    const node = this.retriever.get(id);
    if (!node) return null;
    
    this.stats.totalGets++;
    
    return {
      id: node.id,
      simhash: node.simhash,
      metadata: node.data
    };
  }
  
  /**
   * 通过 SimHash 获取文档
   * @param {bigint} simhash 
   * @returns {Object|null}
   */
  getBySimhash(simhash) {
    const id = this.retriever.getIdBySimhash(simhash);
    if (id === null) return null;
    return this.getVector(id);
  }
  
  /**
   * 相似度搜索
   * @param {bigint} querySimhash - 查询 SimHash
   * @param {number} k - 返回数量（默认10）
   * @param {Object} options - { useLSH: boolean }
   * @returns {Array} - [{ id, simhash, distance, hammingDistance, metadata, source }]
   * 
   * @example
   * const results = await api.searchVector(0x1234n, 5);
   * results.forEach(r => {
   *   console.log(`ID: ${r.id}, Distance: ${r.distance}`);
   * });
   */
  searchVector(querySimhash, k = 10, options = {}) {
    if (!this.initialized) {
      throw new Error('API not initialized. Call init() first.');
    }
    
    if (typeof querySimhash !== 'bigint') {
      throw new TypeError('querySimhash must be bigint');
    }
    
    this.stats.totalSearches++;
    
    return this.retriever.search(querySimhash, k, options);
  }
  
  /**
   * 批量搜索
   * @param {Array<bigint>} queries 
   * @param {number} k 
   * @returns {Array<Array>} - 每组查询的结果
   */
  searchVectors(queries, k = 10) {
    return queries.map(q => this.searchVector(q, k));
  }
  
  /**
   * 删除向量
   * @param {number} id 
   * @returns {boolean}
   */
  async deleteVector(id) {
    if (!this.initialized) await this.init();
    
    const result = this.retriever.remove(id);
    
    if (result) {
      await this.persistence.logDelete(id);
    }
    
    return result;
  }
  
  /**
   * 保存索引到磁盘
   */
  async save() {
    if (!this.initialized) return;
    
    await this.persistence.save(this.retriever.hnsw, {
      vectorCount: this.retriever.hnsw.elementCount,
      timestamp: Date.now()
    });
  }
  
  /**
   * 获取统计信息
   * @returns {Object}
   */
  getStats() {
    const retrieverStats = this.retriever.getStats();
    const memoryStats = this.memoryManager.getStats();
    
    return {
      ...this.stats,
      uptime: Date.now() - this.stats.startTime,
      retriever: retrieverStats,
      memory: memoryStats
    };
  }
  
  /**
   * 获取降级状态
   */
  getFallbackStatus() {
    return this.retriever.getFallbackStatus();
  }
  
  /**
   * 重建索引
   */
  async rebuild() {
    console.log('[VectorAPI] Rebuilding index...');
    // 重新从所有文档构建 HNSW
    this.retriever.rebuildHNSW();
    await this.save();
    console.log('[VectorAPI] Index rebuilt');
  }
  
  /**
   * 清空所有数据
   */
  async clear() {
    this.retriever.clear();
    await this.persistence.destroy();
    this.stats = {
      totalPuts: 0,
      totalGets: 0,
      totalSearches: 0,
      startTime: Date.now()
    };
  }
  
  /**
   * 关闭（保存并清理资源）
   */
  async close() {
    await this.save();
    this.memoryManager.destroy();
    this.initialized = false;
  }
}

// 单例导出
let defaultInstance = null;

/**
 * 获取默认 VectorAPI 实例
 */
function getVectorAPI(options = {}) {
  if (!defaultInstance) {
    defaultInstance = new VectorAPI(options);
  }
  return defaultInstance;
}

/**
 * 重置默认实例
 */
function resetVectorAPI() {
  defaultInstance = null;
}

module.exports = {
  VectorAPI,
  getVectorAPI,
  resetVectorAPI
};
