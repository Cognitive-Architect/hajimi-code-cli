/**
 * JS ↔ WASM HNSW 桥接层
 * HNSW Bridge - JavaScript to WASM Bridge
 */

const { WASMLoader } = require('./loader');

class HNSWWASMBridge {
  constructor(options = {}) {
    this.loader = new WASMLoader(options);
    this.index = null;
    this.dimension = options.dimension || 128;
    this.isInitialized = false;
  }

  /**
   * 初始化WASM索引
   */
  async init() {
    try {
      const exports = await this.loader.load();
      
      // 检查必要的导出函数
      if (!exports.HNSWIndex) {
        throw new Error('WASM module does not export HNSWIndex');
      }
      
      // 创建索引实例
      this.index = new exports.HNSWIndex(this.dimension);
      this.isInitialized = true;
      
      console.log(`✅ HNSW WASM Bridge initialized (dim=${this.dimension})`);
      return true;
      
    } catch (err) {
      console.warn('⚠️ WASM initialization failed, falling back to JS:', err.message);
      this.isInitialized = false;
      return false;
    }
  }

  /**
   * 插入向量
   */
  async insert(id, vector) {
    this._checkInitialized();
    
    if (vector.length !== this.dimension) {
      throw new Error(`Dimension mismatch: expected ${this.dimension}, got ${vector.length}`);
    }
    
    // 转换为Float32Array
    const floatVector = new Float32Array(vector);
    
    try {
      this.index.insert(id, floatVector);
      return { success: true, id };
    } catch (err) {
      return { success: false, error: err.message };
    }
  }

  /**
   * 批量插入
   */
  async batchInsert(vectors) {
    const results = [];
    
    for (const { id, vector } of vectors) {
      results.push(await this.insert(id, vector));
    }
    
    return results;
  }

  /**
   * 搜索最近邻
   */
  async search(query, k = 10) {
    this._checkInitialized();
    
    if (query.length !== this.dimension) {
      throw new Error(`Dimension mismatch: expected ${this.dimension}, got ${query.length}`);
    }
    
    const floatQuery = new Float32Array(query);
    
    try {
      const results = this.index.search(floatQuery, k);
      return JSON.parse(results);
    } catch (err) {
      throw new Error(`Search failed: ${err.message}`);
    }
  }

  /**
   * 获取统计信息
   */
  async getStats() {
    this._checkInitialized();
    
    try {
      const stats = this.index.stats();
      return JSON.parse(stats);
    } catch (err) {
      return { error: err.message };
    }
  }

  /**
   * 检查是否使用WASM
   */
  isUsingWASM() {
    return this.isInitialized && this.loader.isLoaded();
  }

  /**
   * 获取内存统计
   */
  getMemoryStats() {
    return this.loader.getMemoryStats();
  }

  /**
   * 销毁索引
   */
  destroy() {
    this.index = null;
    this.loader.unload();
    this.isInitialized = false;
  }

  /**
   * 检查初始化状态
   */
  _checkInitialized() {
    if (!this.isInitialized) {
      throw new Error('HNSW bridge not initialized');
    }
  }
}

module.exports = {
  HNSWWASMBridge
};
