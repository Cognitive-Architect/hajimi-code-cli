/**
 * HNSW 混合索引
 * 自动在WASM和JS模式间切换
 */

const { WASMRuntimeLoader } = require('../wasm/runtime-loader');
const { HNSWIndex: JSIndex } = require('./hnsw-core');
const EventEmitter = require('events');

class HybridHNSWIndex extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.dimension = options.dimension || 128;
    this.options = options;
    
    // WASM加载器
    this.wasmLoader = new WASMRuntimeLoader(options);
    
    // 内部索引实例
    this.index = null;
    this.mode = 'unknown'; // 'wasm' or 'javascript'
    
    // 统计
    this.stats = {
      insertions: 0,
      searches: 0,
      wasmMode: false
    };
  }

  /**
   * 初始化索引
   */
  async init() {
    try {
      // 尝试加载WASM
      await this.wasmLoader.loadWASM();
      
      if (this.wasmLoader.isWASMLoaded()) {
        // 使用WASM模式
        await this._initWASM();
      } else {
        // 使用JS模式
        await this._initJS();
      }
    } catch (err) {
      // WASM加载失败，降级到JS
      console.warn('WASM load failed, falling back to JS:', err.message);
      await this._initJS();
    }
  }

  /**
   * 初始化WASM索引
   */
  async _initWASM() {
    try {
      const wasmModule = this.wasmLoader.getModule();
      
      if (wasmModule.HNSWIndex) {
        this.index = new wasmModule.HNSWIndex(this.dimension);
        this.mode = 'wasm';
        this.stats.wasmMode = true;
        
        console.log(`✅ HybridHNSW initialized in WASM mode (dim=${this.dimension})`);
        this.emit('mode', { mode: 'wasm' });
      } else {
        throw new Error('WASM module does not export HNSWIndex');
      }
    } catch (err) {
      console.warn('WASM init failed, falling back to JS:', err.message);
      await this._initJS();
    }
  }

  /**
   * 初始化JS索引
   */
  async _initJS() {
    this.index = new JSIndex({
      dimension: this.dimension,
      M: this.options.M || 16,
      efConstruction: this.options.efConstruction || 200
    });
    
    this.mode = 'javascript';
    this.stats.wasmMode = false;
    
    console.log(`✅ HybridHNSW initialized in JavaScript mode (dim=${this.dimension})`);
    this.emit('mode', { mode: 'javascript' });
  }

  /**
   * 插入向量
   */
  insert(id, vector) {
    if (!this.index) {
      throw new Error('Index not initialized');
    }
    
    if (vector.length !== this.dimension) {
      throw new Error(`Dimension mismatch: expected ${this.dimension}, got ${vector.length}`);
    }
    
    // 根据模式调用不同实现
    if (this.mode === 'wasm') {
      // WASM模式：传递Float32Array
      const floatVector = new Float32Array(vector);
      this.index.insert(id, floatVector);
    } else {
      // JS模式
      this.index.insert(id, vector);
    }
    
    this.stats.insertions++;
    return { id, mode: this.mode };
  }

  /**
   * 批量插入
   */
  batchInsert(vectors) {
    const results = [];
    
    for (const { id, vector } of vectors) {
      try {
        this.insert(id, vector);
        results.push({ id, success: true });
      } catch (err) {
        results.push({ id, success: false, error: err.message });
      }
    }
    
    return results;
  }

  /**
   * 搜索最近邻
   */
  search(query, k = 10) {
    if (!this.index) {
      throw new Error('Index not initialized');
    }
    
    if (query.length !== this.dimension) {
      throw new Error(`Dimension mismatch: expected ${this.dimension}, got ${query.length}`);
    }
    
    const startTime = Date.now();
    
    let results;
    
    if (this.mode === 'wasm') {
      // WASM模式
      const floatQuery = new Float32Array(query);
      const rawResults = this.index.search(floatQuery, k);
      results = JSON.parse(rawResults);
    } else {
      // JS模式
      results = this.index.search(query, k);
    }
    
    this.stats.searches++;
    
    return {
      results,
      mode: this.mode,
      latency: Date.now() - startTime
    };
  }

  /**
   * 获取索引统计
   */
  getStats() {
    const indexStats = this.index ? {
      elementCount: this.index.elementCount || 0,
      maxLevel: this.index.maxLevel || 0
    } : {};
    
    return {
      ...this.stats,
      mode: this.mode,
      dimension: this.dimension,
      ...indexStats
    };
  }

  /**
   * 获取当前模式
   */
  getMode() {
    return this.mode;
  }

  /**
   * 是否使用WASM
   */
  isWASM() {
    return this.mode === 'wasm';
  }

  /**
   * 强制降级到JS模式
   */
  forceDowngrade() {
    if (this.mode === 'javascript') return;
    
    console.log('⬇️ Force downgrading to JavaScript mode...');
    
    // 保存当前数据
    const data = this._exportData();
    
    // 重新初始化为JS模式
    this._initJS();
    
    // 恢复数据
    this._importData(data);
    
    this.emit('downgrade', { from: 'wasm', to: 'javascript' });
  }

  /**
   * 导出数据
   */
  _exportData() {
    if (!this.index) return [];
    
    const nodes = [];
    if (this.index.nodes) {
      for (const [id, node] of this.index.nodes) {
        nodes.push({
          id,
          vector: node.vector,
          level: node.level,
          connections: node.connections
        });
      }
    }
    
    return nodes;
  }

  /**
   * 导入数据
   */
  _importData(nodes) {
    for (const node of nodes) {
      try {
        this.insert(node.id, node.vector);
      } catch (err) {
        console.warn(`Failed to import node ${node.id}:`, err.message);
      }
    }
  }

  /**
   * 序列化索引
   */
  serialize() {
    if (!this.index) return null;
    
    return {
      mode: this.mode,
      dimension: this.dimension,
      nodes: this._exportData(),
      entryPoint: this.index.entryPoint,
      maxLevel: this.index.maxLevel,
      elementCount: this.index.elementCount
    };
  }

  /**
   * 从序列化数据恢复
   */
  deserialize(data) {
    this.dimension = data.dimension;
    
    // 根据存储的模式尝试初始化
    if (data.mode === 'wasm') {
      this._initWASM().catch(() => this._initJS());
    } else {
      this._initJS();
    }
    
    // 恢复数据
    this._importData(data.nodes);
  }
}

module.exports = { HybridHNSWIndex };
