/**
 * WASM Loader - 自动加载WASM并降级到JS
 * 
 * 功能：
 * 1. 自动检测WASM可用性
 * 2. 加载失败自动降级到纯JS实现
 * 3. 接口与JS版100%兼容
 */

const fs = require('fs');
const path = require('path');

// 引入AlignedMemoryPool（Sprint2 Day2：16字节对齐内存池）
const { AlignedMemoryPool } = require('../wasm/wasm-memory-pool.js');

// WASM模块路径
const WASM_PKG_PATH = path.join(__dirname, '../../crates/hajimi-hnsw/pkg/hajimi_hnsw.js');

class WASMLoader {
  constructor() {
    this.mode = 'unknown'; // 'wasm', 'javascript', 'unknown'
    this.wasmModule = null;
    this.jsModule = null;
    this.loadError = null;
  }

  /**
   * 初始化加载器
   * @returns {Promise<boolean>} - true表示WASM模式，false表示JS降级
   */
  async init() {
    // 尝试加载WASM
    if (this._isWASMAvailable()) {
      try {
        this.wasmModule = require(WASM_PKG_PATH);
        
        // 验证WASM模块功能
        const testIndex = new this.wasmModule.HNSWIndex(8, 8, 50);
        const stats = testIndex.stats();
        
        if (stats && stats.get('nodeCount') !== undefined) {
          this.mode = 'wasm';
          console.log('[WASMLoader] WASM模式已加载');
          return true;
        }
      } catch (err) {
        this.loadError = err;
        console.warn('[WASMLoader] WASM加载失败:', err.message);
      }
    } else {
      console.log('[WASMLoader] WASM产物不存在');
    }

    // 降级到JS
    try {
      const { HNSWIndex } = require('./hnsw-core.js');
      this.jsModule = { HNSWIndex };
      this.mode = 'javascript';
      console.log('[WASMLoader] JS降级模式已加载');
      return false;
    } catch (err) {
      this.loadError = err;
      console.error('[WASMLoader] JS降级也失败:', err.message);
      throw new Error('无法加载任何HNSW实现');
    }
  }

  /**
   * 检查WASM是否可用
   */
  _isWASMAvailable() {
    try {
      return fs.existsSync(WASM_PKG_PATH);
    } catch (err) {
      return false;
    }
  }

  /**
   * 创建HNSW索引实例
   * @param {number} dimension - 向量维度
   * @param {number} m - 每层最大连接数
   * @param {number} efConstruction - 构建时的搜索深度
   * @returns {Object} - HNSWIndex实例
   */
  createIndex(dimension, m = 16, efConstruction = 200) {
    if (this.mode === 'wasm') {
      const index = new this.wasmModule.HNSWIndex(dimension, m, efConstruction);
      
      // 包装以提供统一接口
      return new WASMIndexWrapper(index, this.mode);
    } else if (this.mode === 'javascript') {
      const index = new this.jsModule.HNSWIndex({
        M: m,
        efConstruction: efConstruction,
        distanceMetric: 'cosine'
      });
      
      return new JSIndexWrapper(index, this.mode);
    }
    
    throw new Error('WASMLoader未初始化');
  }

  /**
   * 获取当前模式
   * @returns {string} - 'wasm', 'javascript', 'unknown'
   */
  getMode() {
    return this.mode;
  }

  /**
   * 获取性能统计
   */
  getStats() {
    return {
      mode: this.mode,
      wasmAvailable: this.mode === 'wasm',
      loadError: this.loadError?.message
    };
  }
}

/**
 * WASM索引包装器 - 提供统一接口
 */
class WASMIndexWrapper {
  constructor(wasmIndex, mode) {
    this._index = wasmIndex;
    this._mode = mode;
    this._dimension = null; // 从stats推断
    
    // Sprint2 Day2: 初始化16字节对齐内存池
    this._memoryPool = new AlignedMemoryPool({
      initialSize: 128 * 1024,  // 128KB，约32K个f32
      maxSize: 1024 * 1024,     // 1MB
      growthFactor: 2.0
    });
  }

  insert(id, vector) {
    // 确保向量是Array（不是Float32Array）
    const arr = Array.from(vector);
    this._index.insert(id, arr);
  }

  search(query, k = 10) {
    const arr = Array.from(query);
    const results = this._index.search(arr, k);
    
    // 转换为JS格式
    return results.map(r => ({
      id: r.id,
      distance: r.distance
    }));
  }

  /**
   * RISK-02 FIX: 批量搜索 - 真·零拷贝API
   * 直接调用Rust searchBatch接口，避免多次WASM边界跨越
   */
  searchBatch(queries, queryCount, k = 10) {
    // queries是Float32Array，需要转换为普通数组传递给WASM
    const arr = Array.from(queries);
    const results = this._index.searchBatch(arr, queryCount, k);
    
    // 转换为JS格式
    return results.map(queryResults => 
      queryResults.map(r => ({
        id: r.id,
        distance: r.distance
      }))
    );
  }

  stats() {
    const s = this._index.stats();
    return {
      nodeCount: s.get('nodeCount'),
      maxLevel: s.get('maxLevel'),
      dimension: s.get('dimension'),
      m: s.get('m'),
      mode: this._mode
    };
  }

  getMode() {
    return this._mode;
  }
  
  /**
   * Sprint2 Day2: 零拷贝批量搜索（高性能路径）
   * 
   * 使用16字节对齐内存池，避免序列化开销
   * 双路径策略：对齐失败自动降级到searchBatch
   * 
   * @param {Float32Array} queries - 查询向量（将被复制到对齐内存）
   * @param {number} k - 返回最近邻数量
   * @returns {Array<Array>} - 每组查询的结果
   */
  searchBatchZeroCopy(queries, k = 10) {
    // 检查Rust接口是否存在
    if (!this._index.searchBatchZeroCopy) {
      // 接口不存在，fallback到旧路径
      return this.searchBatch(queries, queries.length / this._getDimension(), k);
    }
    
    // 检查输入
    if (!queries || queries.length === 0) {
      return [];
    }
    
    const dim = this._getDimension();
    if (dim === 0) {
      throw new Error('Dimension not available');
    }
    
    // 尝试从内存池获取16字节对齐内存
    const alignedView = this._memoryPool.acquire(queries.length);
    
    if (!alignedView) {
      // 内存池耗尽，fallback到旧路径
      console.warn('[WasmMemoryPool] Pool exhausted, falling back to searchBatch');
      return this.searchBatch(queries, queries.length / dim, k);
    }
    
    try {
      // 拷贝数据到对齐内存
      alignedView.set(queries);
      
      // 调用Rust零拷贝接口
      const results = this._index.searchBatchZeroCopy(alignedView, dim, k);
      
      // 转换为JS格式
      return results.map(queryResults => 
        queryResults.map(r => ({
          id: r.id,
          distance: r.distance
        }))
      );
    } catch (err) {
      // Rust调用失败，fallback到旧路径
      if (err.message && err.message.includes('MisalignedPointer')) {
        console.warn('[WasmMemoryPool] Alignment error, falling back to searchBatch');
      } else {
        console.warn('[WasmMemoryPool] Rust error, falling back to searchBatch:', err.message);
      }
      return this.searchBatch(queries, queries.length / dim, k);
    } finally {
      // 释放内存回池
      this._memoryPool.release(alignedView);
    }
  }
  
  /**
   * 获取维度（从stats推断并缓存）
   * @private
   */
  _getDimension() {
    if (!this._dimension) {
      const s = this._index.stats();
      this._dimension = s.get('dimension') || 0;
    }
    return this._dimension;
  }
}

/**
 * JS索引包装器 - 提供统一接口
 */
class JSIndexWrapper {
  constructor(jsIndex, mode) {
    this._index = jsIndex;
    this._mode = mode;
  }

  insert(id, vector) {
    this._index.insert(id, vector);
  }

  search(query, k = 10) {
    return this._index.search(query, k);
  }

  stats() {
    const s = this._index.getStats();
    return {
      nodeCount: s.elementCount,
      maxLevel: s.maxLevel,
      dimension: s.config?.maxElements, // 近似
      m: s.config?.M,
      mode: this._mode
    };
  }

  getMode() {
    return this._mode;
  }
}

// 单例导出 - 使用Promise缓存防止并发创建多实例
let loaderPromise = null;

async function getWASMLoader() {
  if (!loaderPromise) {
    // 立即创建Promise，后续并发调用会等待同一个Promise
    loaderPromise = (async () => {
      const loader = new WASMLoader();
      await loader.init();
      return loader;
    })();
  }
  return loaderPromise;
}

function resetWASMLoader() {
  loaderPromise = null;
}

module.exports = {
  WASMLoader,
  getWASMLoader,
  resetWASMLoader,
  WASMIndexWrapper,
  JSIndexWrapper
};
