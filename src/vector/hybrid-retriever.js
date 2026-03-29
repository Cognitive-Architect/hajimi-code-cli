/**
 * 混合检索层 - Hybrid Retriever
 * 
 * 统一检索接口：
 * - HNSW 作为主检索引擎（高性能，适合100K+向量）
 * - LSH 作为降级方案（低内存，适合小规模）
 * - 自动切换，无缝降级
 * 
 * 输入：原始文本 → SimHash → 检索
 * 输出：排序结果 [{id, score, source}]
 */

const { HNSWIndex } = require('./hnsw-core');
const { VectorEncoder } = require('./encoder');
const { CircuitBreaker, HealthChecker } = require('./fallback-switch');
const { hammingDistance } = require('./distance');

// 默认配置
const DEFAULT_RETRIEVER_CONFIG = {
  // HNSW 配置
  hnswM: 16,
  hnswEfConstruction: 200,
  hnswEfSearch: 64,
  
  // 编码配置
  encoderMethod: 'hadamard',
  encoderOutputDim: 128,
  
  // 检索配置
  defaultK: 10,
  minHammingThreshold: 20,  // 汉明距离小于此值算相似
  
  // LSH 配置（fallback）
  lshMaxCandidates: 100,    // LSH 最多返回候选
  
  // 降级配置
  enableFallback: true
};

/**
 * 混合检索器
 */
class HybridRetriever {
  /**
   * @param {Object} config 
   */
  constructor(config = {}) {
    this.config = { ...DEFAULT_RETRIEVER_CONFIG, ...config };
    
    // HNSW 索引
    this.hnsw = new HNSWIndex({
      M: this.config.hnswM,
      efConstruction: this.config.hnswEfConstruction,
      efSearch: this.config.hnswEfSearch,
      distanceMetric: 'l2'  // 编码后使用L2距离
    });
    
    // 向量编码器
    this.encoder = new VectorEncoder({
      method: this.config.encoderMethod,
      outputDim: this.config.encoderOutputDim,
      normalize: true
    });
    
    // LSH 索引（简单内存存储）
    this.lshIndex = new Map();  // simhash -> {id, simhash, data}
    
    // 降级控制
    this.circuitBreaker = new CircuitBreaker();
    this.healthChecker = new HealthChecker();
    
    // 统计
    this.stats = {
      totalQueries: 0,
      hnswQueries: 0,
      lshQueries: 0,
      avgHNSWLatency: 0,
      avgLSHLatency: 0
    };
    
    // ID 映射
    this.idCounter = 0;
    this.simhashToId = new Map();  // simhash -> internal id
  }
  
  /**
   * 添加文档到索引
   * @param {bigint} simhash - SimHash-64
   * @param {Object} data - 关联数据
   * @returns {number} - 内部ID
   */
  add(simhash, data = {}) {
    if (typeof simhash !== 'bigint') {
      throw new TypeError('simhash must be bigint');
    }
    
    // 检查是否已存在
    if (this.simhashToId.has(simhash.toString())) {
      return this.simhashToId.get(simhash.toString());
    }
    
    const id = this.idCounter++;
    
    // 1. 添加到 LSH 索引（始终）
    this.lshIndex.set(id, {
      id,
      simhash,
      data,
      timestamp: Date.now()
    });
    this.simhashToId.set(simhash.toString(), id);
    
    // 2. 尝试添加到 HNSW
    if (this.circuitBreaker.canUseHNSW()) {
      try {
        const vector = this.encoder.encode(simhash);
        this.hnsw.insert(id, vector);
        this.circuitBreaker.recordSuccess(0);
      } catch (err) {
        this.circuitBreaker.recordFailure('error');
        console.warn(`HNSW insert failed: ${err.message}`);
      }
    }
    
    return id;
  }
  
  /**
   * 批量添加
   * @param {Array<{simhash: bigint, data: Object}>} items 
   * @param {Function} progressCallback - (current, total) => void
   * @returns {Array<number>} - 内部ID列表
   */
  addBatch(items, progressCallback = null) {
    const ids = [];
    const total = items.length;
    
    for (let i = 0; i < items.length; i++) {
      const { simhash, data } = items[i];
      const id = this.add(simhash, data);
      ids.push(id);
      
      if (progressCallback && i % 100 === 0) {
        progressCallback(i + 1, total);
      }
    }
    
    if (progressCallback) {
      progressCallback(total, total);
    }
    
    return ids;
  }
  
  /**
   * 搜索最近邻
   * @param {bigint} querySimhash - 查询SimHash
   * @param {number} k - 返回数量
   * @param {Object} options - { useLSH: boolean }
   * @returns {Array} - [{id, simhash, distance, data, source}]
   */
  search(querySimhash, k = 10, options = {}) {
    this.stats.totalQueries++;
    const startTime = Date.now();
    
    // 强制使用 LSH
    if (options.useLSH) {
      const results = this._searchLSH(querySimhash, k);
      this._updateLatency('lsh', Date.now() - startTime);
      return results;
    }
    
    // 检查 HNSW 是否可用
    if (this.circuitBreaker.canUseHNSW() && this.hnsw.elementCount > 0) {
      try {
        const results = this._searchHNSW(querySimhash, k);
        const latency = Date.now() - startTime;
        this._updateLatency('hnsw', latency);
        
        // 检查延迟是否超标
        if (latency > 100) {
          this.circuitBreaker.recordFailure('latency');
        } else {
          this.circuitBreaker.recordSuccess(latency);
        }
        
        return results;
      } catch (err) {
        this.circuitBreaker.recordFailure('error');
        console.warn(`HNSW search failed: ${err.message}, falling back to LSH`);
      }
    }
    
    // 降级到 LSH
    this.circuitBreaker.recordLSHCall();
    const results = this._searchLSH(querySimhash, k);
    this._updateLatency('lsh', Date.now() - startTime);
    return results;
  }
  
  /**
   * HNSW 搜索
   */
  _searchHNSW(querySimhash, k) {
    const queryVector = this.encoder.encode(querySimhash);
    const results = this.hnsw.search(queryVector, k);
    
    return results.map(r => {
      const node = this.lshIndex.get(r.id);
      return {
        id: r.id,
        simhash: node?.simhash,
        distance: r.distance,
        hammingDistance: node ? hammingDistance(querySimhash, node.simhash) : null,
        data: node?.data || {},
        source: 'hnsw'
      };
    });
  }
  
  /**
   * LSH 搜索（暴力汉明距离）
   */
  _searchLSH(querySimhash, k) {
    const candidates = [];
    
    // 计算所有候选的汉明距离
    for (const [id, node] of this.lshIndex) {
      const dist = hammingDistance(querySimhash, node.simhash);
      candidates.push({
        id,
        simhash: node.simhash,
        hammingDistance: dist,
        data: node.data,
        source: 'lsh'
      });
    }
    
    // 按汉明距离排序
    candidates.sort((a, b) => a.hammingDistance - b.hammingDistance);
    
    // 返回前k个
    return candidates.slice(0, k).map(c => ({
      ...c,
      distance: c.hammingDistance / 64  // 归一化到 [0, 1]
    }));
  }
  
  /**
   * 更新延迟统计
   */
  _updateLatency(type, latency) {
    if (type === 'hnsw') {
      this.stats.hnswQueries++;
      this.stats.avgHNSWLatency = 
        (this.stats.avgHNSWLatency * (this.stats.hnswQueries - 1) + latency) / 
        this.stats.hnswQueries;
    } else {
      this.stats.lshQueries++;
      this.stats.avgLSHLatency = 
        (this.stats.avgLSHLatency * (this.stats.lshQueries - 1) + latency) / 
        this.stats.lshQueries;
    }
  }
  
  /**
   * 删除文档
   * @param {number} id 
   * @returns {boolean}
   */
  remove(id) {
    const node = this.lshIndex.get(id);
    if (!node) return false;
    
    // 从 LSH 删除
    this.lshIndex.delete(id);
    this.simhashToId.delete(node.simhash.toString());
    
    // 从 HNSW 删除
    this.hnsw.delete(id);
    
    return true;
  }
  
  /**
   * 获取文档
   * @param {number} id 
   */
  get(id) {
    return this.lshIndex.get(id) || null;
  }
  
  /**
   * 通过 SimHash 获取ID
   * @param {bigint} simhash 
   */
  getIdBySimhash(simhash) {
    return this.simhashToId.get(simhash.toString()) || null;
  }
  
  /**
   * 获取统计信息
   */
  getStats() {
    return {
      ...this.stats,
      hnswStats: this.hnsw.getStats(),
      fallbackStatus: this.circuitBreaker.getStatus(),
      totalDocuments: this.lshIndex.size,
      hnswCoverage: this.hnsw.elementCount / Math.max(this.lshIndex.size, 1)
    };
  }
  
  /**
   * 获取降级状态
   */
  getFallbackStatus() {
    return this.circuitBreaker.getStatus();
  }
  
  /**
   * 手动重建 HNSW 索引
   * @param {Array} documents - 可选，要重新加载的文档列表 [{simhash: bigint, data: Object}]
   * @param {Function} progressCallback - (current, total) => void
   */
  async rebuildHNSW(documents = null, progressCallback = null) {
    console.log('Rebuilding HNSW index...');
    
    // 保存现有文档（如果没有提供外部数据）
    const docsToReload = documents || this._exportDocuments();
    
    // 清空现有索引
    this.hnsw.clear();
    this.idCounter = 0;
    this.simhashToId.clear();
    this.lshIndex.clear();
    
    // 从数据源重新加载
    if (docsToReload && docsToReload.length > 0) {
      const total = docsToReload.length;
      let count = 0;
      
      for (const doc of docsToReload) {
        // 支持多种数据格式
        const simhash = typeof doc.simhash === 'bigint' ? doc.simhash : BigInt(doc.simhash);
        const data = doc.data || {};
        
        this.add(simhash, data);
        count++;
        
        if (progressCallback && count % 100 === 0) {
          progressCallback(count, total);
        }
      }
      
      if (progressCallback) {
        progressCallback(total, total);
      }
      
      console.log(`Rebuilt HNSW index with ${count} documents`);
    } else {
      console.log('No documents to reload, HNSW index is empty');
    }
    
    console.log('HNSW index rebuilt');
  }
  
  /**
   * 导出文档列表（内部使用）
   * @returns {Array} - [{simhash, data}]
   * @private
   */
  _exportDocuments() {
    const documents = [];
    for (const [id, node] of this.lshIndex) {
      documents.push({
        simhash: node.simhash,
        data: node.data
      });
    }
    return documents;
  }
  
  /**
   * 导出索引数据
   */
  export() {
    const documents = [];
    for (const [id, node] of this.lshIndex) {
      documents.push({
        id,
        simhash: node.simhash.toString(),
        data: node.data
      });
    }
    
    return {
      documents,
      hnsw: this.hnsw.toJSON(),
      config: this.config,
      stats: this.stats
    };
  }
  
  /**
   * 导入索引数据
   * @param {Object} data 
   */
  import(data) {
    // 导入配置
    if (data.config) {
      this.config = { ...this.config, ...data.config };
    }
    
    // 导入文档
    if (data.documents) {
      for (const doc of data.documents) {
        this.add(BigInt(doc.simhash), doc.data);
      }
    }
    
    // 导入 HNSW（如果存在）
    if (data.hnsw) {
      try {
        this.hnsw = HNSWIndex.fromJSON(data.hnsw);
      } catch (err) {
        console.warn('Failed to import HNSW, will rebuild:', err.message);
      }
    }
    
    // 导入统计
    if (data.stats) {
      this.stats = { ...this.stats, ...data.stats };
    }
  }
  
  /**
   * 清空所有索引
   */
  clear() {
    this.hnsw.clear();
    this.lshIndex.clear();
    this.simhashToId.clear();
    this.idCounter = 0;
    this.circuitBreaker.reset();
    this.stats = {
      totalQueries: 0,
      hnswQueries: 0,
      lshQueries: 0,
      avgHNSWLatency: 0,
      avgLSHLatency: 0
    };
  }
}

module.exports = {
  HybridRetriever,
  DEFAULT_RETRIEVER_CONFIG
};
