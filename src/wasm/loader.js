/**
 * WASM加载器
 * WASM Loader with Memory Management
 */

const fs = require('fs').promises;
const path = require('path');

class WASMLoader {
  constructor(options = {}) {
    this.wasmPath = options.wasmPath || './crates/hajimi-hnsw/pkg/hajimi_hnsw_bg.wasm';
    this.module = null;
    this.instance = null;
    this.memory = null;
    
    // 内存限制
    this.maxMemoryMB = options.maxMemoryMB || 400;
    this.initialMemoryPages = options.initialMemoryPages || 256; // 16MB
  }

  /**
   * 加载WASM模块
   */
  async load(customPath) {
    const wasmPath = customPath || this.wasmPath;
    
    try {
      // 读取WASM文件
      const wasmBuffer = await fs.readFile(wasmPath);
      
      // 编译WASM
      this.module = await WebAssembly.compile(wasmBuffer);
      
      // 创建内存
      const memory = new WebAssembly.Memory({
        initial: this.initialMemoryPages,
        maximum: (this.maxMemoryMB * 1024 * 1024) / (64 * 1024), // 400MB in 64KB pages
      });
      
      // 实例化
      const importObject = {
        env: {
          memory,
          __memory_base: 0,
          __table_base: 0,
        }
      };
      
      this.instance = await WebAssembly.instantiate(this.module, importObject);
      this.memory = memory;
      
      console.log('✅ WASM loaded successfully');
      return this.instance.exports;
      
    } catch (err) {
      console.error('❌ Failed to load WASM:', err.message);
      throw new Error(`WASM load failed: ${err.message}`);
    }
  }

  /**
   * 检查WASM是否已加载
   */
  isLoaded() {
    return this.instance !== null;
  }

  /**
   * 获取导出函数
   */
  exports() {
    if (!this.instance) {
      throw new Error('WASM not loaded');
    }
    return this.instance.exports;
  }

  /**
   * 获取内存使用情况
   */
  getMemoryStats() {
    if (!this.memory) {
      return { loaded: false };
    }
    
    const buffer = this.memory.buffer;
    return {
      loaded: true,
      byteLength: buffer.byteLength,
      maxMB: this.maxMemoryMB,
      usagePercent: ((buffer.byteLength / (this.maxMemoryMB * 1024 * 1024)) * 100).toFixed(2)
    };
  }

  /**
   * 释放WASM资源
   */
  unload() {
    this.instance = null;
    this.module = null;
    this.memory = null;
    console.log('👋 WASM unloaded');
  }
}

// 单例模式
let defaultLoader = null;

function getLoader(options) {
  if (!defaultLoader) {
    defaultLoader = new WASMLoader(options);
  }
  return defaultLoader;
}

module.exports = {
  WASMLoader,
  getLoader
};
