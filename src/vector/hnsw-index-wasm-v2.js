/**
 * HNSW Index WASM v2 - 运行时新标准
 * 
 * 特性：
 * 1. 自动检测WASM可用性
 * 2. 无缝降级到JS
 * 3. 性能统计接口
 * 4. 混合模式（构建用WASM快，查询按需选择）
 */

const { getWASMLoader } = require('./wasm-loader.js');

class HNSWIndexWASMV2 {
  /**
   * @param {Object} options
   * @param {number} options.dimension - 向量维度
   * @param {number} options.M - 每层最大连接数（默认16）
   * @param {number} options.efConstruction - 构建搜索深度（默认200）
   * @param {number} options.efSearch - 查询搜索深度（默认64）
   * @param {string} options.mode - 'auto', 'wasm', 'js'（默认auto）
   */
  constructor(options = {}) {
    this.config = {
      dimension: options.dimension || 128,
      M: options.M || 16,
      efConstruction: options.efConstruction || 200,
      efSearch: options.efSearch || 64,
      mode: options.mode || 'auto'
    };

    this._index = null;
    this._loader = null;
    this._actualMode = null;
    this.stats = {
      inserts: 0,
      searches: 0,
      totalInsertTime: 0,
      totalSearchTime: 0
    };
  }

  /**
   * 初始化索引
   */
  async init() {
    if (this._index) return;

    this._loader = await getWASMLoader();
    
    // 根据配置选择模式
    if (this.config.mode === 'wasm' && this._loader.getMode() !== 'wasm') {
      throw new Error('WASM模式请求但WASM不可用');
    }
    
    if (this.config.mode === 'js') {
      // 强制使用JS（即使WASM可用）
      const { HNSWIndex } = require('./hnsw-core.js');
      this._index = new HNSWIndex({
        M: this.config.M,
        efConstruction: this.config.efConstruction,
        efSearch: this.config.efSearch,
        distanceMetric: 'cosine'
      });
      this._actualMode = 'javascript';
    } else {
      // auto或wasm模式
      this._index = this._loader.createIndex(
        this.config.dimension,
        this.config.M,
        this.config.efConstruction
      );
      this._actualMode = this._index.getMode();
    }

    console.log(`[HNSWIndexV2] Initialized in ${this._actualMode} mode`);
  }

  /**
   * 插入向量
   * @param {number} id - 向量ID
   * @param {Float32Array|Array} vector - 向量数据
   */
  insert(id, vector) {
    if (!this._index) {
      throw new Error('Index not initialized. Call init() first.');
    }

    const start = Date.now();
    this._index.insert(id, vector);
    this.stats.inserts++;
    this.stats.totalInsertTime += Date.now() - start;
  }

  /**
   * 批量插入
   * @param {Array<{id: number, vector: Array}>} items
   */
  insertBatch(items) {
    const start = Date.now();
    for (const item of items) {
      this.insert(item.id, item.vector);
    }
    console.log(`[HNSWIndexV2] Batch insert: ${items.length} items in ${Date.now() - start}ms`);
  }

  /**
   * 搜索最近邻
   * @param {Float32Array|Array} query - 查询向量
   * @param {number} k - 返回数量
   * @returns {Array<{id: number, distance: number}>}
   */
  search(query, k = 10) {
    if (!this._index) {
      throw new Error('Index not initialized. Call init() first.');
    }

    const start = Date.now();
    const results = this._index.search(query, k);
    this.stats.searches++;
    this.stats.totalSearchTime += Date.now() - start;

    return results;
  }

  /**
   * 获取索引统计
   */
  getStats() {
    const indexStats = this._index ? this._index.stats() : {};
    
    return {
      config: this.config,
      actualMode: this._actualMode,
      indexStats,
      performance: {
        inserts: this.stats.inserts,
        searches: this.stats.searches,
        avgInsertTime: this.stats.inserts > 0 ? 
          (this.stats.totalInsertTime / this.stats.inserts).toFixed(3) : 0,
        avgSearchTime: this.stats.searches > 0 ? 
          (this.stats.totalSearchTime / this.stats.searches).toFixed(3) : 0
      }
    };
  }

  /**
   * 获取当前运行模式
   * @returns {string} - 'wasm', 'javascript', 'unknown'
   */
  getMode() {
    return this._actualMode;
  }

  /**
   * 获取降级信息
   */
  getFallbackInfo() {
    return {
      requestedMode: this.config.mode,
      actualMode: this._actualMode,
      wasFallback: this.config.mode === 'auto' && this._actualMode === 'javascript',
      loaderStats: this._loader?.getStats()
    };
  }

  /**
   * 清空索引
   */
  clear() {
    if (this._index && this._index.clear) {
      this._index.clear();
    }
    this.stats = { inserts: 0, searches: 0, totalInsertTime: 0, totalSearchTime: 0 };
  }
}

module.exports = { HNSWIndexWASMV2 };
