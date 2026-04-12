/**
 * WASM 运行时加载器
 * 自动检测WASM包并加载，支持降级到JS模式
 */

const fs = require('fs').promises;
const path = require('path');
const EventEmitter = require('events');

class WASMRuntimeLoader extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.pkgPath = options.pkgPath || path.join(__dirname, '../../crates/hajimi-hnsw/pkg');
    this.wasmFile = options.wasmFile || 'hajimi_hnsw_bg.wasm';
    this.jsFile = options.jsFile || 'hajimi_hnsw.js';
    
    this.state = {
      wasmAvailable: false,
      wasmLoaded: false,
      wasmModule: null,
      jsFallback: false,
      loadError: null
    };
  }

  /**
   * 检查WASM包是否存在
   */
  async checkWASMAvailable() {
    try {
      const wasmPath = path.join(this.pkgPath, this.wasmFile);
      const jsPath = path.join(this.pkgPath, this.jsFile);
      
      await fs.access(wasmPath);
      await fs.access(jsPath);
      
      this.state.wasmAvailable = true;
      return true;
    } catch (err) {
      this.state.wasmAvailable = false;
      return false;
    }
  }

  /**
   * 加载WASM模块
   */
  async loadWASM() {
    // 检查WASM是否可用
    const available = await this.checkWASMAvailable();
    
    if (!available) {
      console.log('ℹ️ WASM package not found, will use JS fallback');
      this.state.jsFallback = true;
      this.emit('fallback', { reason: 'wasm_not_found' });
      return null;
    }

    try {
      console.log('🚀 Loading WASM module...');
      
      // 加载胶水代码
      const jsPath = path.join(this.pkgPath, this.jsFile);
      
      // 清除require缓存以确保重新加载
      delete require.cache[require.resolve(jsPath)];
      
      const wasmModule = require(jsPath);
      
      // 等待WASM初始化
      if (wasmModule.__wbindgen_start) {
        await wasmModule.__wbindgen_start();
      }
      
      this.state.wasmModule = wasmModule;
      this.state.wasmLoaded = true;
      this.state.jsFallback = false;
      
      console.log('✅ WASM module loaded successfully');
      
      this.emit('loaded', { 
        module: wasmModule,
        exports: Object.keys(wasmModule)
      });
      
      return wasmModule;
      
    } catch (err) {
      console.error('❌ Failed to load WASM:', err.message);
      
      this.state.wasmLoaded = false;
      this.state.loadError = err.message;
      this.state.jsFallback = true;
      
      this.emit('error', { error: err });
      this.emit('fallback', { reason: 'load_error', error: err.message });
      
      return null;
    }
  }

  /**
   * 获取WASM模块（如果已加载）
   */
  getModule() {
    return this.state.wasmModule;
  }

  /**
   * 检查WASM是否已加载
   */
  isWASMLoaded() {
    return this.state.wasmLoaded;
  }

  /**
   * 是否在使用JS降级
   */
  isJSFallback() {
    return this.state.jsFallback;
  }

  /**
   * 获取加载状态
   */
  getState() {
    return { ...this.state };
  }

  /**
   * 强制使用JS模式（用于测试降级）
   */
  forceJSMode() {
    this.state.jsFallback = true;
    this.state.wasmLoaded = false;
    console.log('⚠️ Forced JS fallback mode');
    this.emit('fallback', { reason: 'forced' });
  }

  /**
   * 创建HNSW索引（自动选择WASM或JS）
   */
  async createHNSWIndex(dimension, options = {}) {
    if (this.state.wasmLoaded && this.state.wasmModule) {
      // 使用WASM
      try {
        const { HNSWIndex } = this.state.wasmModule;
        return new HNSWIndex(dimension);
      } catch (err) {
        console.warn('WASM HNSWIndex creation failed, falling back to JS:', err.message);
      }
    }
    
    // 使用JS降级
    const { HNSWIndex } = require('../vector/hnsw-core');
    return new HNSWIndex({ dimension, ...options });
  }

  /**
   * 获取性能提示
   */
  getPerformanceHint() {
    if (this.state.wasmLoaded) {
      return {
        mode: 'wasm',
        expectedSpeedup: '5x',
        message: 'Running in WASM mode for optimal performance'
      };
    } else {
      return {
        mode: 'javascript',
        expectedSpeedup: '1x',
        message: 'Running in JavaScript mode (WASM not available)'
      };
    }
  }
}

// 单例
let defaultLoader = null;

function getLoader(options) {
  if (!defaultLoader) {
    defaultLoader = new WASMRuntimeLoader(options);
  }
  return defaultLoader;
}

module.exports = {
  WASMRuntimeLoader,
  getLoader
};
