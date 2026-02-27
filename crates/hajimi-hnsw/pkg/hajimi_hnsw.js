/**
 * WASM 胶水代码 - 简化版
 * 手动创建以支持 Termux 环境
 */

const fs = require('fs');
const path = require('path');

// 同步加载 WASM 字节码
const wasmPath = path.join(__dirname, 'hajimi_hnsw_bg.wasm');
const wasmBytes = fs.readFileSync(wasmPath);

// 简单的 WASM 运行时封装
let wasmModule = null;
let wasmInstance = null;

// 导入对象
const imports = {
  env: {
    memory: new WebAssembly.Memory({ initial: 256, maximum: 16384 }), // 16MB ~ 1GB
    __memory_base: 0,
    __table_base: 0,
    __indirect_function_table: new WebAssembly.Table({ initial: 0, element: 'anyfunc' }),
    // 数学函数
    Math_random: Math.random,
  },
  // js-sys 导入
  "./hajimi_hnsw_bg.js": {
    __wbindgen_json_parse: (ptr, len) => {
      const mem = new Uint8Array(wasmInstance.exports.memory.buffer);
      const str = new TextDecoder().decode(mem.subarray(ptr, ptr + len));
      return JSON.parse(str);
    },
    __wbindgen_json_serialize: (idx) => {
      const obj = wasmInstance.exports.__wbindgen_export_0(idx);
      const str = JSON.stringify(obj);
      const bytes = new TextEncoder().encode(str);
      const ptr = wasmInstance.exports.__wbindgen_malloc(bytes.length);
      const mem = new Uint8Array(wasmInstance.exports.memory.buffer);
      mem.set(bytes, ptr);
      return ptr;
    },
    __wbindgen_throw: (ptr, len) => {
      const mem = new Uint8Array(wasmInstance.exports.memory.buffer);
      const str = new TextDecoder().decode(mem.subarray(ptr, ptr + len));
      throw new Error(str);
    },
    __wbindgen_rethrow: (idx) => {
      throw wasmInstance.exports.__wbindgen_export_0(idx);
    }
  }
};

// 同步实例化 WASM
const wasmModuleObj = new WebAssembly.Module(wasmBytes);
wasmInstance = new WebAssembly.Instance(wasmModuleObj, imports);
wasmModule = wasmInstance.exports;

/**
 * HNSW 索引类
 */
class HNSWIndex {
  constructor(dimension, m = 16, ef_construction = 200) {
    this.ptr = wasmModule.hajimihnsw_hnswindex_new(dimension, m, ef_construction);
    this.dimension = dimension;
  }

  insert(id, vector) {
    if (vector.length !== this.dimension) {
      throw new Error(`Dimension mismatch: expected ${this.dimension}, got ${vector.length}`);
    }
    // 将向量转换为 Float32Array 并写入 WASM 内存
    const vecBytes = new Float32Array(vector).buffer;
    const ptr = wasmModule.__wbindgen_malloc(vecBytes.byteLength);
    const mem = new Uint8Array(wasmModule.memory.buffer);
    mem.set(new Uint8Array(vecBytes), ptr);
    
    try {
      wasmModule.hajimihnsw_hnswindex_insert(this.ptr, id, ptr, vector.length);
    } finally {
      wasmModule.__wbindgen_free(ptr, vecBytes.byteLength);
    }
  }

  search(query, k = 10) {
    if (query.length !== this.dimension) {
      throw new Error(`Dimension mismatch: expected ${this.dimension}, got ${query.length}`);
    }
    // 将查询向量写入 WASM 内存
    const vecBytes = new Float32Array(query).buffer;
    const ptr = wasmModule.__wbindgen_malloc(vecBytes.byteLength);
    const mem = new Uint8Array(wasmModule.memory.buffer);
    mem.set(new Uint8Array(vecBytes), ptr);
    
    try {
      const resultPtr = wasmModule.hajimihnsw_hnswindex_search(this.ptr, ptr, query.length, k);
      // 读取结果 JSON 字符串
      const resultMem = new Uint8Array(wasmModule.memory.buffer);
      let end = resultPtr;
      while (resultMem[end] !== 0) end++;
      const resultStr = new TextDecoder().decode(resultMem.subarray(resultPtr, end));
      wasmModule.__wbindgen_free(resultPtr, end - resultPtr + 1);
      return JSON.parse(resultStr);
    } finally {
      wasmModule.__wbindgen_free(ptr, vecBytes.byteLength);
    }
  }

  stats() {
    const resultPtr = wasmModule.hajimihnsw_hnswindex_stats(this.ptr);
    const mem = new Uint8Array(wasmModule.memory.buffer);
    let end = resultPtr;
    while (mem[end] !== 0) end++;
    const str = new TextDecoder().decode(mem.subarray(resultPtr, end));
    wasmModule.__wbindgen_free(resultPtr, end - resultPtr + 1);
    return JSON.parse(str);
  }

  free() {
    if (this.ptr) {
      wasmModule.hajimihnsw_hnswindex_free(this.ptr);
      this.ptr = 0;
    }
  }
}

/**
 * 内存管理器
 */
class MemoryManager {
  static memoryUsage() {
    return wasmModule.hajimihnsw_memorymanager_memory_usage();
  }

  static maxMemory() {
    return wasmModule.hajimihnsw_memorymanager_max_memory();
  }
}

module.exports = {
  HNSWIndex,
  MemoryManager,
  __wasm: wasmModule
};
